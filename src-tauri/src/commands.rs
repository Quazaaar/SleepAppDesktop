use tauri::{Manager, State};
use tauri::WebviewUrl;
use tauri::webview::WebviewWindowBuilder;

use crate::db;
use crate::models::*;
use crate::sync;
use crate::tracker::SharedTrackerState;

#[tauri::command]
pub fn get_current_app(state: State<SharedTrackerState>) -> Result<CurrentAppInfo, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    tracker
        .current_app
        .clone()
        .ok_or_else(|| "No active tracking".into())
}

#[tauri::command]
pub fn get_daily_stats(state: State<SharedTrackerState>, date: String) -> Result<DailyStats, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::get_daily_stats(&conn, &date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_activity_timeline(
    state: State<SharedTrackerState>,
    date: String,
) -> Result<Vec<ActivitySession>, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::get_daily_sessions(&conn, &date).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_tracking(state: State<SharedTrackerState>) -> Result<bool, String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    tracker.is_tracking = !tracker.is_tracking;
    Ok(tracker.is_tracking)
}

#[tauri::command]
pub fn get_ignored_apps(state: State<SharedTrackerState>) -> Result<Vec<String>, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    Ok(tracker.ignored_apps.clone())
}

#[tauri::command]
pub fn set_ignored_apps(state: State<SharedTrackerState>, apps: Vec<String>) -> Result<(), String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::save_ignored_apps(&conn, &apps).map_err(|e| e.to_string())?;
    tracker.ignored_apps = apps;
    Ok(())
}

#[tauri::command]
pub fn get_reminder_rules(state: State<SharedTrackerState>) -> Result<Vec<ReminderRule>, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::get_reminder_rules(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_reminder_rule(
    state: State<SharedTrackerState>,
    rule: ReminderRule,
) -> Result<(), String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::save_reminder_rule(&conn, &rule).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_reminder_rule(state: State<SharedTrackerState>, rule_id: i64) -> Result<(), String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::delete_reminder_rule(&conn, rule_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn toggle_reminder_rule(
    state: State<SharedTrackerState>,
    rule_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::toggle_reminder_rule(&conn, rule_id, enabled).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn sync_now(state: State<'_, SharedTrackerState>) -> Result<usize, String> {
    let (db_path, sync_url, api_key) = {
        let tracker = state.lock().map_err(|e| e.to_string())?;
        (
            tracker.db_path.clone(),
            tracker.sync_url.clone(),
            tracker.api_key.clone(),
        )
    };
    if sync_url.is_empty() || api_key.is_empty() {
        return Err("Sync not configured".into());
    }
    let client = sync::SyncClient::new(sync_url, api_key);
    client.sync_daily_data(&db_path).await
}

#[tauri::command]
pub fn set_sync_config(
    state: State<SharedTrackerState>,
    sync_url: String,
    api_key: String,
) -> Result<(), String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    tracker.sync_url = sync_url;
    tracker.api_key = api_key;
    Ok(())
}

#[tauri::command]
pub fn get_sync_status(state: State<SharedTrackerState>) -> Result<SyncStatus, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    let last_sync = db::get_last_sync_time(&conn).map_err(|e| e.to_string())?;
    Ok(SyncStatus {
        configured: !tracker.sync_url.is_empty() && !tracker.api_key.is_empty(),
        last_sync_time: last_sync,
    })
}

#[tauri::command]
pub async fn show_escalation_window(app: tauri::AppHandle, level: String) -> Result<(), String> {
    // Close any existing escalation windows before creating a new one.
    // This prevents window accumulation when the level advances (Pitfall 4).
    for label in ["escalation-popup", "escalation-panel", "escalation-fullscreen"] {
        if let Some(w) = app.get_webview_window(label) {
            let _ = w.close();
        }
    }

    match level.as_str() {
        "Level2" => {
            WebviewWindowBuilder::new(
                &app,
                "escalation-popup",
                WebviewUrl::App("/#/overlay/popup".into()),
            )
            .title("LucidShift")
            .inner_size(320.0, 140.0)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .build()
            .map_err(|e| e.to_string())?;
        }
        "Level3" => {
            let (width, height) = app
                .primary_monitor()
                .ok()
                .flatten()
                .map(|m| {
                    let size = m.size();
                    (size.width as f64, size.height as f64)
                })
                .unwrap_or((1920.0, 1080.0));

            WebviewWindowBuilder::new(
                &app,
                "escalation-panel",
                WebviewUrl::App("/#/overlay/panel".into()),
            )
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .inner_size(width * 0.3, height)
            .position(width * 0.7, 0.0)
            .build()
            .map_err(|e| e.to_string())?;
        }
        "Level4" => {
            WebviewWindowBuilder::new(
                &app,
                "escalation-fullscreen",
                WebviewUrl::App("/#/overlay/fullscreen".into()),
            )
            .decorations(false)
            .transparent(true)
            .maximized(true) // NOT fullscreen(true) — Tauri bug #7328 on Windows
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .build()
            .map_err(|e| e.to_string())?;
        }
        "Level1" | "None" | "Done" => {
            // Windows already closed above; nothing more to do.
        }
        _ => {}
    }

    Ok(())
}

#[tauri::command]
pub async fn dismiss_escalation(
    app: tauri::AppHandle,
    state: State<'_, SharedTrackerState>,
) -> Result<(), String> {
    // Set engine to Done and emit the event.
    {
        let mut tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.escalation_engine.dismiss(&app);
    }

    // Close all escalation overlay windows.
    for label in ["escalation-popup", "escalation-panel", "escalation-fullscreen"] {
        if let Some(w) = app.get_webview_window(label) {
            let _ = w.close();
        }
    }

    Ok(())
}

#[tauri::command]
pub fn get_escalation_settings(state: State<SharedTrackerState>) -> Result<EscalationSettings, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    Ok(tracker.escalation_engine.settings.clone())
}

#[tauri::command]
pub fn set_escalation_settings(
    state: State<SharedTrackerState>,
    settings: EscalationSettings,
) -> Result<(), String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::save_escalation_settings(&conn, &settings).map_err(|e| e.to_string())?;
    tracker.escalation_engine.settings = settings;
    Ok(())
}

#[tauri::command]
pub fn test_reminder_notification(app: tauri::AppHandle, message: String) -> Result<(), String> {
    use tauri_plugin_notification::NotificationExt;
    app.notification()
        .builder()
        .title("Sleep App Reminder")
        .body(&message)
        .show()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pause_escalation(
    state: State<SharedTrackerState>,
    hours: Option<i64>,
) -> Result<(), String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    let until = match hours {
        Some(h) => Some((chrono::Local::now() + chrono::Duration::hours(h)).to_rfc3339()),
        None => None, // unpause
    };
    tracker.escalation_engine.settings.paused_until = until;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::save_escalation_settings(&conn, &tracker.escalation_engine.settings).map_err(|e| e.to_string())?;
    Ok(())
}
