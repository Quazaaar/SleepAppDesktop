use crate::db;
use crate::models::ActivitySession;

pub struct SyncClient {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
}

impl SyncClient {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            api_key,
        }
    }

    pub async fn sync_daily_data(&self, db_path: &str) -> Result<usize, String> {
        let conn = db::open_db(db_path).map_err(|e| e.to_string())?;

        let last_sync = db::get_last_sync_time(&conn).map_err(|e| e.to_string())?;
        let sessions: Vec<ActivitySession> =
            db::get_sessions_since(&conn, &last_sync).map_err(|e| e.to_string())?;

        if sessions.is_empty() {
            return Ok(0);
        }

        let count = sessions.len();

        let response = self
            .client
            .post(format!("{}/api/sync", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&sessions)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if response.status().is_success() {
            db::log_sync(&conn, count as i64, "success").map_err(|e| e.to_string())?;
            Ok(count)
        } else {
            let status = response.status().to_string();
            db::log_sync(&conn, 0, "failed").map_err(|e| e.to_string())?;
            Err(format!("Sync failed with status: {}", status))
        }
    }
}
