use serde::Deserialize;

use crate::db;
use crate::models::{ActivitySession, WrapUpNote};

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
}

pub struct SyncResult {
    pub count: usize,
    pub tokens_refreshed: bool,
    pub new_access_token: Option<String>,
    pub new_refresh_token: Option<String>,
}

/// Base URL for the lucidshift-api. Baked into the Rust binary at compile time
/// so the URL is never present in the JS bundle. Override with `LUCIDSHIFT_API_URL`
/// at build time (e.g. `LUCIDSHIFT_API_URL=http://localhost:3000 cargo tauri dev`)
/// to point the desktop client at a local API.
pub fn api_base_url() -> String {
    option_env!("LUCIDSHIFT_API_URL")
        .unwrap_or("https://lucidshift-api-production.up.railway.app")
        .to_string()
}

/// User-Agent the desktop client sends on every API request. Surfaced server-side
/// in the `devices.user_agent` column so users can recognize their sessions.
pub fn client_user_agent() -> String {
    let ver = env!("CARGO_PKG_VERSION");
    let os = std::env::consts::OS;
    format!("lucidshift-app/{ver} ({os})")
}

/// Best-effort device label (hostname). Falls back to a generic string. The user
/// can rename the device server-side later if we add that affordance.
pub fn current_device_name() -> String {
    hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .filter(|s| !s.trim().is_empty())
        .unwrap_or_else(|| format!("LucidShift on {}", std::env::consts::OS))
}

/// Outcome of an authed request that may have rotated the refresh token along the way.
pub struct AuthedJson<T> {
    pub body: T,
    pub rotated_tokens: Option<(String, String)>,
}

/// Perform an authed JSON request with one-shot refresh-on-401 retry. Mirrors the
/// pattern in `SyncClient` but works for arbitrary endpoints (settings, devices, …).
/// Caller is responsible for persisting `rotated_tokens` if Some.
pub async fn authed_json<T: serde::de::DeserializeOwned>(
    base_url: &str,
    method: reqwest::Method,
    path: &str,
    access_token: &str,
    refresh_token: &str,
    body: Option<&serde_json::Value>,
) -> Result<AuthedJson<T>, String> {
    let client = reqwest::Client::new();
    let user_agent = client_user_agent();
    let url = format!("{}{}", base_url, path);

    let send = |access: &str, body: Option<&serde_json::Value>| {
        let mut req = client
            .request(method.clone(), &url)
            .header("Authorization", format!("Bearer {}", access))
            .header("User-Agent", &user_agent);
        if let Some(b) = body {
            req = req.json(b);
        }
        req.send()
    };

    // First attempt with the existing access token.
    let resp = send(access_token, body).await.map_err(|e| e.to_string())?;

    if resp.status() != reqwest::StatusCode::UNAUTHORIZED {
        return finalize::<T>(resp, None).await;
    }

    // 401: refresh once, retry once.
    let refresh_resp = client
        .post(format!("{}/api/auth/refresh", base_url))
        .header("User-Agent", &user_agent)
        .json(&serde_json::json!({ "refresh_token": refresh_token }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !refresh_resp.status().is_success() {
        return Err(format!("Refresh failed with status: {}", refresh_resp.status()));
    }
    let auth: AuthResponse = refresh_resp.json().await.map_err(|e| e.to_string())?;
    let rotated = (auth.access_token.clone(), auth.refresh_token.clone());

    let retry_resp = send(&auth.access_token, body)
        .await
        .map_err(|e| e.to_string())?;

    finalize::<T>(retry_resp, Some(rotated)).await
}

async fn finalize<T: serde::de::DeserializeOwned>(
    resp: reqwest::Response,
    rotated: Option<(String, String)>,
) -> Result<AuthedJson<T>, String> {
    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("Request failed ({}): {}", status, text));
    }
    let body: T = resp.json().await.map_err(|e| e.to_string())?;
    Ok(AuthedJson { body, rotated_tokens: rotated })
}

pub struct SyncClient {
    client: reqwest::Client,
    base_url: String,
    access_token: String,
    refresh_token: String,
}

impl SyncClient {
    pub fn new(base_url: String, access_token: String, refresh_token: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            access_token,
            refresh_token,
        }
    }

    async fn refresh_tokens(&self) -> Result<(String, String), String> {
        let resp = self
            .client
            .post(format!("{}/api/auth/refresh", self.base_url))
            .header("User-Agent", client_user_agent())
            .json(&serde_json::json!({ "refresh_token": self.refresh_token }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !resp.status().is_success() {
            return Err(format!("Refresh failed with status: {}", resp.status()));
        }

        let body: AuthResponse = resp.json().await.map_err(|e| e.to_string())?;
        Ok((body.access_token, body.refresh_token))
    }

    pub async fn sync_daily_data(&mut self, db_path: &str) -> Result<SyncResult, String> {
        let conn = db::open_db(db_path).map_err(|e| e.to_string())?;

        let last_sync = db::get_last_sync_time(&conn).map_err(|e| e.to_string())?;
        let sessions: Vec<ActivitySession> =
            db::get_sessions_since(&conn, &last_sync).map_err(|e| e.to_string())?;

        if sessions.is_empty() {
            return Ok(SyncResult {
                count: 0,
                tokens_refreshed: false,
                new_access_token: None,
                new_refresh_token: None,
            });
        }

        let count = sessions.len();
        let user_agent = client_user_agent();

        let response = self
            .client
            .post(format!("{}/api/sync", self.base_url))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", &user_agent)
            .json(&sessions)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status() == reqwest::StatusCode::UNAUTHORIZED {
            // Try refreshing tokens and retry once
            let (new_access, new_refresh) = self.refresh_tokens().await?;
            self.access_token = new_access.clone();
            self.refresh_token = new_refresh.clone();

            let retry = self
                .client
                .post(format!("{}/api/sync", self.base_url))
                .header("Authorization", format!("Bearer {}", self.access_token))
                .header("User-Agent", &user_agent)
                .json(&sessions)
                .send()
                .await
                .map_err(|e| e.to_string())?;

            if retry.status().is_success() {
                db::log_sync(&conn, count as i64, "success").map_err(|e| e.to_string())?;
                return Ok(SyncResult {
                    count,
                    tokens_refreshed: true,
                    new_access_token: Some(new_access),
                    new_refresh_token: Some(new_refresh),
                });
            } else {
                let status = retry.status().to_string();
                db::log_sync(&conn, 0, "failed").map_err(|e| e.to_string())?;
                return Err(format!("Sync failed after refresh with status: {}", status));
            }
        }

        if response.status().is_success() {
            db::log_sync(&conn, count as i64, "success").map_err(|e| e.to_string())?;
            Ok(SyncResult {
                count,
                tokens_refreshed: false,
                new_access_token: None,
                new_refresh_token: None,
            })
        } else {
            let status = response.status().to_string();
            db::log_sync(&conn, 0, "failed").map_err(|e| e.to_string())?;
            Err(format!("Sync failed with status: {}", status))
        }
    }

    pub async fn sync_notes(&self, db_path: &str) -> Result<usize, String> {
        let conn = db::open_db(db_path).map_err(|e| e.to_string())?;
        let notes: Vec<WrapUpNote> = db::get_all_wrap_up_notes(&conn).map_err(|e| e.to_string())?;
        if notes.is_empty() {
            return Ok(0);
        }
        let count = notes.len();
        let resp = self
            .client
            .post(format!("{}/api/notes/sync", self.base_url))
            .header("Authorization", format!("Bearer {}", self.access_token))
            .header("User-Agent", client_user_agent())
            .json(&notes)
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.status().is_success() {
            Ok(count)
        } else {
            Err(format!("Notes sync failed: {}", resp.status()))
        }
    }
}
