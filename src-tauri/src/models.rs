use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityLog {
    pub id: Option<i64>,
    pub window_title: String,
    pub app_name: String,
    pub process_name: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySession {
    pub id: Option<i64>,
    pub app_name: String,
    pub window_title: String,
    pub start_time: String,
    pub end_time: String,
    pub duration_secs: i64,
    pub date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppUsageStat {
    pub app_name: String,
    pub total_duration_secs: i64,
    pub percentage: f64,
    pub session_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderRule {
    pub id: Option<i64>,
    pub rule_type: String,
    pub app_name: Option<String>,
    pub threshold_minutes: i64,
    pub message: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentAppInfo {
    pub app_name: String,
    pub window_title: String,
    pub duration_secs: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStats {
    pub date: String,
    pub total_tracked_secs: i64,
    pub app_usage: Vec<AppUsageStat>,
    pub most_used_app: String,
}
