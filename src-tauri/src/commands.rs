use chrono::Local;
use tauri::{Manager, State};
use tauri::WebviewUrl;
use tauri::webview::WebviewWindowBuilder;
use tauri_plugin_store::StoreExt;

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
pub fn toggle_tracking(state: State<SharedTrackerState>, app: tauri::AppHandle) -> Result<bool, String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    tracker.is_tracking = !tracker.is_tracking;
    if let Some(tray_items) = app.try_state::<crate::TrayMenuItems>() {
        crate::update_tray_tracking_item(&tray_items, tracker.is_tracking);
    }
    Ok(tracker.is_tracking)
}

#[tauri::command]
pub fn get_tracking(state: State<SharedTrackerState>) -> Result<bool, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
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

#[derive(serde::Deserialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
}

#[tauri::command]
pub async fn login(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
    sync_url: String,
    email: String,
    password: String,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/auth/login", sync_url))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Login failed ({}): {}", status, body));
    }

    let body: LoginResponse = resp.json().await.map_err(|e| e.to_string())?;

    // Persist to TrackerState
    {
        let mut tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.sync_url = sync_url.clone();
        tracker.access_token = body.access_token.clone();
        tracker.refresh_token = body.refresh_token.clone();
    }

    // Persist to Tauri Store
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("sync_url", serde_json::json!(sync_url));
    store.set("access_token", serde_json::json!(body.access_token));
    store.set("refresh_token", serde_json::json!(body.refresh_token));
    store.save().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn register(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
    sync_url: String,
    email: String,
    password: String,
) -> Result<(), String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}/api/auth/register", sync_url))
        .json(&serde_json::json!({ "email": email, "password": password }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Registration failed ({}): {}", status, body));
    }

    let body: LoginResponse = resp.json().await.map_err(|e| e.to_string())?;

    // Persist to TrackerState
    {
        let mut tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.sync_url = sync_url.clone();
        tracker.access_token = body.access_token.clone();
        tracker.refresh_token = body.refresh_token.clone();
    }

    // Persist to Tauri Store
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("sync_url", serde_json::json!(sync_url));
    store.set("access_token", serde_json::json!(body.access_token));
    store.set("refresh_token", serde_json::json!(body.refresh_token));
    store.save().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub async fn logout(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    {
        let mut tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.access_token.clear();
        tracker.refresh_token.clear();
    }

    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("access_token", serde_json::json!(""));
    store.set("refresh_token", serde_json::json!(""));
    store.save().map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
pub fn get_auth_status(state: State<SharedTrackerState>) -> Result<bool, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    Ok(!tracker.access_token.is_empty())
}

#[tauri::command]
pub async fn sync_now(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
) -> Result<usize, String> {
    let (db_path, sync_url, access_token, refresh_token) = {
        let tracker = state.lock().map_err(|e| e.to_string())?;
        (
            tracker.db_path.clone(),
            tracker.sync_url.clone(),
            tracker.access_token.clone(),
            tracker.refresh_token.clone(),
        )
    };
    if sync_url.is_empty() || access_token.is_empty() {
        return Err("Not logged in".into());
    }
    let mut client = sync::SyncClient::new(sync_url, access_token, refresh_token);
    let result = client.sync_daily_data(&db_path).await?;

    // Persist refreshed tokens if they were rotated
    if result.tokens_refreshed {
        if let (Some(new_access), Some(new_refresh)) =
            (result.new_access_token, result.new_refresh_token)
        {
            {
                let mut tracker = state.lock().map_err(|e| e.to_string())?;
                tracker.access_token = new_access.clone();
                tracker.refresh_token = new_refresh.clone();
            }
            let store = app.store("settings.json").map_err(|e| e.to_string())?;
            store.set("access_token", serde_json::json!(new_access));
            store.set("refresh_token", serde_json::json!(new_refresh));
            store.save().map_err(|e| e.to_string())?;
        }
    }

    Ok(result.count)
}

#[tauri::command]
pub fn get_sync_status(state: State<SharedTrackerState>) -> Result<SyncStatus, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    let last_sync = db::get_last_sync_time(&conn).map_err(|e| e.to_string())?;
    Ok(SyncStatus {
        configured: !tracker.sync_url.is_empty() && !tracker.access_token.is_empty(),
        last_sync_time: last_sync,
    })
}

#[tauri::command]
pub async fn show_escalation_window(app: tauri::AppHandle, level: String) -> Result<(), String> {
    // Close any existing escalation windows before creating a new one.
    // This prevents window accumulation when the level advances (Pitfall 4).
    // For the popup, persist its position/size before closing so we can restore it.
    if let Some(w) = app.get_webview_window("escalation-popup") {
        if let (Ok(pos), Ok(size)) = (w.outer_position(), w.inner_size()) {
            if let Ok(store) = app.store("settings.json") {
                let _ = store.set("popup_x", serde_json::json!(pos.x));
                let _ = store.set("popup_y", serde_json::json!(pos.y));
                let _ = store.set("popup_w", serde_json::json!(size.width));
                let _ = store.set("popup_h", serde_json::json!(size.height));
                let _ = store.save();
            }
        }
        let _ = w.close();
    }
    for label in ["escalation-panel", "escalation-fullscreen"] {
        if let Some(w) = app.get_webview_window(label) {
            let _ = w.close();
        }
    }

    match level.as_str() {
        "Level2" => {
            // Restore saved geometry or use defaults.
            let (mut x, mut y, mut w, mut h) = (None, None, 320.0_f64, 140.0_f64);
            if let Ok(store) = app.store("settings.json") {
                if let (Some(sx), Some(sy)) = (
                    store.get("popup_x").and_then(|v| v.as_f64()),
                    store.get("popup_y").and_then(|v| v.as_f64()),
                ) {
                    x = Some(sx);
                    y = Some(sy);
                }
                if let (Some(sw), Some(sh)) = (
                    store.get("popup_w").and_then(|v| v.as_f64()),
                    store.get("popup_h").and_then(|v| v.as_f64()),
                ) {
                    w = sw.max(200.0);
                    h = sh.max(80.0);
                }
            }

            let mut builder = WebviewWindowBuilder::new(
                &app,
                "escalation-popup",
                WebviewUrl::App("/#/overlay/popup".into()),
            )
            .title("LucidShift")
            .inner_size(w, h)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(true);

            if let (Some(px), Some(py)) = (x, y) {
                builder = builder.position(px, py);
            }

            builder.build().map_err(|e| e.to_string())?;
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
pub async fn acknowledge_popup(
    app: tauri::AppHandle,
    state: State<'_, SharedTrackerState>,
) -> Result<u32, String> {
    // Record the dismissal timestamp.
    let count = {
        let mut tracker = state.lock().map_err(|e| e.to_string())?;
        tracker
            .escalation_engine
            .popup_dismissals
            .push(Local::now().to_rfc3339());
        tracker.escalation_engine.popup_dismissals.len() as u32
    };

    // Close the popup window (but don't change escalation level).
    if let Some(w) = app.get_webview_window("escalation-popup") {
        let _ = w.close();
    }

    Ok(count)
}

#[tauri::command]
pub fn get_popup_dismissals(state: State<SharedTrackerState>) -> Result<Vec<String>, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    Ok(tracker.escalation_engine.popup_dismissals.clone())
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
    app: tauri::AppHandle,
    hours: Option<i64>,
) -> Result<(), String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    let until = match hours {
        Some(h) => Some((chrono::Local::now() + chrono::Duration::hours(h)).to_rfc3339()),
        None => None, // unpause
    };
    let is_paused = until.is_some();
    tracker.escalation_engine.settings.paused_until = until;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::save_escalation_settings(&conn, &tracker.escalation_engine.settings).map_err(|e| e.to_string())?;
    if let Some(tray_items) = app.try_state::<crate::TrayMenuItems>() {
        crate::update_tray_pause_items(&tray_items, is_paused);
    }
    Ok(())
}
