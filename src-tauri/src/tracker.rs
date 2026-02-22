use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use active_win_pos_rs::get_active_window;
use tauri::Manager;
use chrono::Local;
use tokio::time::{Duration, interval};
use user_idle::UserIdle;

use crate::db;
use crate::escalation::EscalationEngine;
use crate::models::{ActivitySession, CurrentAppInfo, EscalationSettings};

pub struct TrackerState {
    pub is_tracking: bool,
    pub current_app: Option<CurrentAppInfo>,
    pub current_session_start: Option<String>,
    pub current_session_app: Option<String>,
    pub current_session_title: Option<String>,
    pub db_path: String,
    pub ignored_apps: Vec<String>,
    pub total_continuous_secs: i64,
    pub sync_url: String,
    pub access_token: String,
    pub refresh_token: String,
    pub escalation_engine: EscalationEngine,
    pub app_handle: Option<tauri::AppHandle>,
    pub idle_threshold_secs: u64,
    pub app_categories: HashMap<String, String>,
    pub title_keyword_rules: Vec<(String, String, String)>,
}

impl TrackerState {
    pub fn new() -> Self {
        Self {
            is_tracking: false,
            current_app: None,
            current_session_start: None,
            current_session_app: None,
            current_session_title: None,
            db_path: String::new(),
            ignored_apps: Vec::new(),
            total_continuous_secs: 0,
            sync_url: String::new(),
            access_token: String::new(),
            refresh_token: String::new(),
            escalation_engine: EscalationEngine::new(EscalationSettings::default()),
            app_handle: None,
            idle_threshold_secs: 120, // 2 minutes
            app_categories: HashMap::new(),
            title_keyword_rules: Vec::new(),
        }
    }
}

/// Resolve the current app's category using keyword rules first, then app-level lookup.
/// Pure function — no DB calls. Returns "uncategorized" if no match.
pub fn resolve_category(
    app_name: &str,
    window_title: &str,
    app_categories: &HashMap<String, String>,
    keyword_rules: &[(String, String, String)],
) -> String {
    let app_lower = app_name.to_lowercase();
    let title_lower = window_title.to_lowercase();
    // 1. Keyword rules win (browser title matching)
    for (rule_app, keyword, category) in keyword_rules {
        if rule_app == &app_lower && title_lower.contains(keyword.as_str()) {
            return category.clone();
        }
    }
    // 2. App-level category
    if let Some(cat) = app_categories.get(&app_lower) {
        return cat.clone();
    }
    // 3. Unknown
    "uncategorized".into()
}

pub type SharedTrackerState = Arc<Mutex<TrackerState>>;

pub fn start_tracking(state: SharedTrackerState) {
    tauri::async_runtime::spawn(async move {
        let mut ticker = interval(Duration::from_secs(2));
        loop {
            ticker.tick().await;

            let mut tracker = match state.lock() {
                Ok(t) => t,
                Err(_) => continue,
            };

            if !tracker.is_tracking || tracker.db_path.is_empty() {
                continue;
            }

            let window = match get_active_window() {
                Ok(w) => w,
                Err(_) => continue,
            };

            let app_name = window.app_name.clone();
            let window_title = window.title.clone();

            // Skip ignored apps
            if tracker
                .ignored_apps
                .iter()
                .any(|a| a.eq_ignore_ascii_case(&app_name))
            {
                continue;
            }

            let now = Local::now();
            let now_str = now.to_rfc3339();
            let today = now.format("%Y-%m-%d").to_string();

            // Update total continuous screen time
            tracker.total_continuous_secs += 2;

            let app_changed = tracker
                .current_session_app
                .as_ref()
                .map(|a| a != &app_name)
                .unwrap_or(true);

            if app_changed {
                // Finalize previous session if one exists
                if let (Some(prev_app), Some(prev_title), Some(start)) = (
                    tracker.current_session_app.take(),
                    tracker.current_session_title.take(),
                    tracker.current_session_start.take(),
                ) {
                    let start_time =
                        chrono::DateTime::parse_from_rfc3339(&start).unwrap_or(now.into());
                    let duration = now.signed_duration_since(start_time);
                    let duration_secs = duration.num_seconds();

                    if duration_secs >= 5 {
                        let session = ActivitySession {
                            id: None,
                            app_name: prev_app.clone(),
                            window_title: prev_title,
                            start_time: start,
                            end_time: now_str.clone(),
                            duration_secs,
                            date: today.clone(),
                        };

                        if let Ok(conn) = db::open_db(&tracker.db_path) {
                            let _ = db::insert_session(&conn, &session);
                        }
                        // Update in-memory cache if app not already present
                        tracker.app_categories.entry(prev_app.to_lowercase()).or_insert_with(|| "uncategorized".into());
                    }
                }

                // Start new session
                tracker.current_session_app = Some(app_name.clone());
                tracker.current_session_title = Some(window_title.clone());
                tracker.current_session_start = Some(now_str);
                tracker.current_app = Some(CurrentAppInfo {
                    app_name,
                    window_title,
                    duration_secs: 0,
                });
            } else {
                // Same app — update duration
                if let Some(ref mut info) = tracker.current_app {
                    info.duration_secs += 2;
                    info.window_title = window_title;
                }
            }

            // Idle detection — treat errors as "not idle" (Pitfall 7 in research)
            let is_idle = UserIdle::get_time()
                .map(|t| t.as_seconds() >= tracker.idle_threshold_secs)
                .unwrap_or(false);

            // Resolve current category from in-memory cache (no DB call)
            let current_app_name = tracker
                .current_session_app
                .as_deref()
                .unwrap_or("");
            let current_title = tracker
                .current_app
                .as_ref()
                .map(|a| a.window_title.as_str())
                .unwrap_or("");
            let current_category = resolve_category(
                current_app_name,
                current_title,
                &tracker.app_categories,
                &tracker.title_keyword_rules,
            );

            // Tick escalation engine if we have an app handle to emit events.
            // Clone the handle so we release the immutable borrow on tracker
            // before calling tick() which requires a mutable borrow.
            if let Some(handle) = tracker.app_handle.clone() {
                tracker.escalation_engine.tick(&handle, is_idle, &current_category);
                // Sync tray menu when pause expires during tick
                if let Some(tray_items) = handle.try_state::<crate::TrayMenuItems>() {
                    let is_paused = tracker.escalation_engine.settings.paused_until.is_some();
                    crate::update_tray_pause_items(&tray_items, is_paused);
                }
            }
        }
    });
}
