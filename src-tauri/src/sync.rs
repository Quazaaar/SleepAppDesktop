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

        let response = self
            .client
            .post(format!("{}/api/sync", self.base_url))
            .header("Authorization", format!("Bearer {}", self.access_token))
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
