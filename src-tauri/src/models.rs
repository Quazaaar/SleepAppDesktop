use serde::{Deserialize, Serialize};

// --- Escalation types ---

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EscalationLevel {
    None,
    Level1, // toast
    Level2, // popup window
    Level3, // side panel
    Level4, // fullscreen overlay
    Done,   // dismissed for the night
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationSettings {
    pub green_end_hour: u32,        // hour at which yellow zone starts (e.g. 22 = 10 PM)
    pub yellow_end_hour: u32,       // hour at which red zone starts (e.g. 23 = 11 PM)
    pub sensitivity: f32,           // 0.0 (gentle/slow) to 1.0 (aggressive/fast)
    pub enabled: bool,
    pub paused_until: Option<String>, // RFC3339 timestamp or None
    pub productive_multiplier: f32,   // <1.0 = slower escalation when productive
    pub distracting_multiplier: f32,  // >1.0 = faster escalation when distracted
}

impl Default for EscalationSettings {
    fn default() -> Self {
        Self {
            green_end_hour: 22,
            yellow_end_hour: 23,
            sensitivity: 0.5,
            enabled: true,
            paused_until: None,
            productive_multiplier: 0.5,
            distracting_multiplier: 1.5,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EscalationStatePayload {
    pub level: EscalationLevel,
    pub message: String,
}

// --- End escalation types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppCategoryEntry {
    pub app_name: String,
    pub category: String,
    pub last_seen: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleKeywordRule {
    pub id: Option<i64>,
    pub app_name: String,
    pub keyword: String,
    pub category: String,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub configured: bool,
    pub last_sync_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WrapUpNote {
    pub session_key: String,
    pub working_on: String,
    pub next_steps: String,
    pub created_at: String, // RFC3339
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileAppCategory {
    pub app_name: String,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfileSettings {
    pub escalation: EscalationSettings,
    pub ignored_apps: Vec<String>,
    pub reminder_rules: Vec<ReminderRule>,
    pub app_categories: Vec<ProfileAppCategory>,
    pub title_keyword_rules: Vec<TitleKeywordRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfile {
    pub id: String,
    pub name: String,
    pub settings: DeviceProfileSettings,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceProfilesState {
    pub profiles: Vec<DeviceProfile>,
    pub active_profile_id: Option<String>,
}
