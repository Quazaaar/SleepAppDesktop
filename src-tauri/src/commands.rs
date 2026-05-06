use chrono::Local;
use serde::{Deserialize, Serialize};
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
pub fn set_ignored_apps(
    state: State<SharedTrackerState>,
    app: tauri::AppHandle,
    apps: Vec<String>,
) -> Result<(), String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::save_ignored_apps(&conn, &apps).map_err(|e| e.to_string())?;
    autosave_active_profile_from_db(&app, &conn)?;
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
    app: tauri::AppHandle,
    rule: ReminderRule,
) -> Result<(), String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::save_reminder_rule(&conn, &rule).map_err(|e| e.to_string())?;
    autosave_active_profile_from_db(&app, &conn)
}

#[tauri::command]
pub fn delete_reminder_rule(
    state: State<SharedTrackerState>,
    app: tauri::AppHandle,
    rule_id: i64,
) -> Result<(), String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::delete_reminder_rule(&conn, rule_id).map_err(|e| e.to_string())?;
    autosave_active_profile_from_db(&app, &conn)
}

#[tauri::command]
pub fn toggle_reminder_rule(
    state: State<SharedTrackerState>,
    app: tauri::AppHandle,
    rule_id: i64,
    enabled: bool,
) -> Result<(), String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::toggle_reminder_rule(&conn, rule_id, enabled).map_err(|e| e.to_string())?;
    autosave_active_profile_from_db(&app, &conn)
}

#[derive(Deserialize)]
struct LoginResponse {
    access_token: String,
    refresh_token: String,
}

async fn auth_request(
    path: &str,
    email: &str,
    password: &str,
    device_name: &str,
) -> Result<LoginResponse, String> {
    let client = reqwest::Client::new();
    let resp = client
        .post(format!("{}{}", sync::api_base_url(), path))
        .header("User-Agent", sync::client_user_agent())
        .json(&serde_json::json!({
            "email": email,
            "password": password,
            "device_name": device_name,
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(format!("Auth request failed ({}): {}", status, body));
    }

    resp.json::<LoginResponse>().await.map_err(|e| e.to_string())
}

fn persist_auth(
    state: &State<'_, SharedTrackerState>,
    app: &tauri::AppHandle,
    body: &LoginResponse,
) -> Result<(), String> {
    {
        let mut tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.access_token = body.access_token.clone();
        tracker.refresh_token = body.refresh_token.clone();
    }

    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("access_token", serde_json::json!(body.access_token));
    store.set("refresh_token", serde_json::json!(body.refresh_token));
    // Old key — purge so nothing in settings.json reveals the API URL.
    let _ = store.delete("sync_url");
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn login(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
    email: String,
    password: String,
) -> Result<(), String> {
    let device_name = sync::current_device_name();
    let body = auth_request("/api/auth/login", &email, &password, &device_name).await
        .map_err(|e| e.replace("Auth request failed", "Login failed"))?;
    persist_auth(&state, &app, &body)
}

#[tauri::command]
pub async fn register(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
    email: String,
    password: String,
) -> Result<(), String> {
    let device_name = sync::current_device_name();
    let body = auth_request("/api/auth/register", &email, &password, &device_name).await
        .map_err(|e| e.replace("Auth request failed", "Registration failed"))?;
    persist_auth(&state, &app, &body)
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
    let (db_path, access_token, refresh_token) = {
        let tracker = state.lock().map_err(|e| e.to_string())?;
        (
            tracker.db_path.clone(),
            tracker.access_token.clone(),
            tracker.refresh_token.clone(),
        )
    };
    if access_token.is_empty() {
        return Err("Not logged in".into());
    }
    let mut client = sync::SyncClient::new(sync::api_base_url(), access_token, refresh_token);
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

    // Sync wrap-up notes after session sync (tokens already fresh if refresh was needed)
    let notes_count = client.sync_notes(&db_path).await.unwrap_or(0);

    // Profile snapshots are stored in the generic cloud settings document. Keep this
    // best-effort so activity sync does not fail just because profile sync is offline.
    if let Ok(payload) = read_local_profile_payload(&app) {
        if !payload.profiles.is_empty() {
            if let Ok(final_payload) =
                put_profiles_to_cloud(&state, &app, &payload, true, None).await
            {
                let _ = write_local_profile_payload(&app, &final_payload);
            }
        }
    }

    Ok(result.count + notes_count)
}

#[tauri::command]
pub fn get_sync_status(state: State<SharedTrackerState>) -> Result<SyncStatus, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    let last_sync = db::get_last_sync_time(&conn).map_err(|e| e.to_string())?;
    Ok(SyncStatus {
        configured: !tracker.access_token.is_empty(),
        last_sync_time: last_sync,
    })
}

/// Returns the primary monitor's size in **logical pixels**, normalizing for the
/// monitor's scale factor.
///
/// `Monitor::size()` returns physical pixels (e.g. 1920x1080 on a 1.0-scale display,
/// or 2400x1350 reported as 1920x1080 physical on a 125%-scaled 1080p panel — varies
/// per device). `WebviewWindowBuilder::inner_size` and `position` consume *logical*
/// pixels. Mixing the two on fractional-DPI displays yields windows that are
/// `scale_factor` times too large and positioned off-screen.
///
/// Falls back to (1920.0, 1080.0) if the primary monitor cannot be queried.
fn primary_monitor_logical_size(app: &tauri::AppHandle) -> (f64, f64) {
    app.primary_monitor()
        .ok()
        .flatten()
        .map(|m| {
            let size = m.size();
            let scale = m.scale_factor();
            // Guard against a zero scale factor (shouldn't happen, but avoid div-by-zero).
            let scale = if scale > 0.0 { scale } else { 1.0 };
            (size.width as f64 / scale, size.height as f64 / scale)
        })
        .unwrap_or((1920.0, 1080.0))
}

#[tauri::command]
pub async fn show_escalation_window(
    app: tauri::AppHandle,
    state: State<'_, SharedTrackerState>,
    level: String,
) -> Result<(), String> {
    // Close any existing escalation windows before creating a new one.
    // This prevents window accumulation when the level advances (Pitfall 4).
    // For the popup, persist its position/size before closing so we can restore it.
    if let Some(w) = app.get_webview_window("escalation-popup") {
        // outer_position/inner_size return PHYSICAL pixels; the builder consumes
        // LOGICAL pixels. Convert before persisting so save→restore is unit-stable
        // on fractional-DPI displays (otherwise the popup grows by scale_factor
        // on every open).
        if let (Ok(pos), Ok(size), Ok(scale)) =
            (w.outer_position(), w.inner_size(), w.scale_factor())
        {
            let pos_l = pos.to_logical::<f64>(scale);
            let size_l = size.to_logical::<f64>(scale);
            if let Ok(store) = app.store("settings.json") {
                let _ = store.set("popup_x", serde_json::json!(pos_l.x));
                let _ = store.set("popup_y", serde_json::json!(pos_l.y));
                let _ = store.set("popup_w", serde_json::json!(size_l.width));
                let _ = store.set("popup_h", serde_json::json!(size_l.height));
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
            // All geometry below is in LOGICAL pixels (what the builder expects).
            // The save path normalizes via to_logical(); the upper clamp also
            // recovers any pre-fix users whose stored values are still physical.
            let (mon_w, mon_h) = primary_monitor_logical_size(&app);
            let default_w = 320.0_f64;
            let default_h = 140.0_f64;
            let margin = 24.0_f64;

            let (mut saved_x, mut saved_y, mut w, mut h) =
                (None, None, default_w, default_h);
            if let Ok(store) = app.store("settings.json") {
                if let (Some(sx), Some(sy)) = (
                    store.get("popup_x").and_then(|v| v.as_f64()),
                    store.get("popup_y").and_then(|v| v.as_f64()),
                ) {
                    saved_x = Some(sx);
                    saved_y = Some(sy);
                }
                if let (Some(sw), Some(sh)) = (
                    store.get("popup_w").and_then(|v| v.as_f64()),
                    store.get("popup_h").and_then(|v| v.as_f64()),
                ) {
                    w = sw.clamp(200.0, mon_w * 0.5);
                    h = sh.clamp(80.0, mon_h * 0.5);
                }
            }

            // Default to bottom-right; clamp restored position so the window
            // stays on-screen even if the user changed monitors since last save.
            let (px, py) = match (saved_x, saved_y) {
                (Some(sx), Some(sy)) => (
                    sx.clamp(0.0, (mon_w - w).max(0.0)),
                    sy.clamp(0.0, (mon_h - h).max(0.0)),
                ),
                _ => (
                    (mon_w - w - margin).max(0.0),
                    (mon_h - h - margin).max(0.0),
                ),
            };

            WebviewWindowBuilder::new(
                &app,
                "escalation-popup",
                WebviewUrl::App("/#/overlay/popup".into()),
            )
            .title("LucidShift")
            .inner_size(w, h)
            .position(px, py)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(true)
            .build()
            .map_err(|e| e.to_string())?;
        }
        "Level3" => {
            // Generate session key on first L3 appearance for this escalation cycle
            {
                let mut tracker = state.lock().map_err(|e| e.to_string())?;
                if tracker.current_session_key.is_none() {
                    tracker.current_session_key = Some(chrono::Local::now().to_rfc3339());
                }
            }

            // inner_size + position consume logical pixels — see helper docs above.
            // Previously this used `monitor.size()` (physical) directly, which produced
            // a panel `scale_factor` times too wide and positioned off-screen on
            // 125%/150%-scaled Windows displays.
            let (logical_width, logical_height) = primary_monitor_logical_size(&app);

            WebviewWindowBuilder::new(
                &app,
                "escalation-panel",
                WebviewUrl::App("/#/overlay/panel".into()),
            )
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .inner_size(logical_width * 0.3, logical_height)
            .position(logical_width * 0.7, 0.0)
            .build()
            .map_err(|e| e.to_string())?;
        }
        "Level4" => {
            // Cover the full primary monitor with explicit logical-pixel geometry.
            //
            // We previously used `.maximized(true) + .transparent(true) + .decorations(false)`,
            // which on Windows + fractional DPI is unreliable: the OS maximizes to the
            // unscaled work area while WebView2 receives a logical-pixel client size,
            // leaving the overlay clipped on the bottom and/or right.
            //
            // The overlay markup itself paints rgba(0,0,0,0.88) so `transparent(true)`
            // is unnecessary — drop it to avoid the maximized-transparent quirk.
            // We deliberately do NOT use `fullscreen(true)` (Tauri bug #7328 on Windows).
            let (logical_width, logical_height) = primary_monitor_logical_size(&app);

            WebviewWindowBuilder::new(
                &app,
                "escalation-fullscreen",
                WebviewUrl::App("/#/overlay/fullscreen".into()),
            )
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .resizable(false)
            .position(0.0, 0.0)
            .inner_size(logical_width, logical_height)
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
    // Set engine to Done and emit the event. Clear session key for next escalation cycle.
    {
        let mut tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.escalation_engine.dismiss(&app);
        tracker.current_session_key = None;
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
    app: tauri::AppHandle,
    settings: EscalationSettings,
) -> Result<(), String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::save_escalation_settings(&conn, &settings).map_err(|e| e.to_string())?;
    autosave_active_profile_from_db(&app, &conn)?;
    tracker.escalation_engine.settings = settings;
    Ok(())
}

#[tauri::command]
pub fn get_app_categories(state: State<SharedTrackerState>) -> Result<Vec<AppCategoryEntry>, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::get_app_categories(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_app_category(
    state: State<SharedTrackerState>,
    app: tauri::AppHandle,
    app_name: String,
    category: String,
) -> Result<(), String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::set_app_category(&conn, &app_name, &category).map_err(|e| e.to_string())?;
    autosave_active_profile_from_db(&app, &conn)?;
    tracker.app_categories.insert(app_name.to_lowercase(), category);
    Ok(())
}

#[tauri::command]
pub fn get_title_keyword_rules(state: State<SharedTrackerState>) -> Result<Vec<TitleKeywordRule>, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::get_title_keyword_rules(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_title_keyword_rule(
    state: State<SharedTrackerState>,
    app: tauri::AppHandle,
    app_name: String,
    keyword: String,
    category: String,
) -> Result<i64, String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    let new_id = db::add_title_keyword_rule(&conn, &app_name, &keyword, &category)
        .map_err(|e| e.to_string())?;
    autosave_active_profile_from_db(&app, &conn)?;
    // Reload title_keyword_rules cache from DB
    let rules = db::get_title_keyword_rules(&conn).map_err(|e| e.to_string())?;
    tracker.title_keyword_rules = rules
        .iter()
        .map(|r| (r.app_name.clone(), r.keyword.clone(), r.category.clone()))
        .collect();
    Ok(new_id)
}

#[tauri::command]
pub fn delete_title_keyword_rule(
    state: State<SharedTrackerState>,
    app: tauri::AppHandle,
    id: i64,
) -> Result<(), String> {
    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::delete_title_keyword_rule(&conn, id).map_err(|e| e.to_string())?;
    autosave_active_profile_from_db(&app, &conn)?;
    // Reload title_keyword_rules cache from DB
    let rules = db::get_title_keyword_rules(&conn).map_err(|e| e.to_string())?;
    tracker.title_keyword_rules = rules
        .iter()
        .map(|r| (r.app_name.clone(), r.keyword.clone(), r.category.clone()))
        .collect();
    Ok(())
}

#[tauri::command]
pub fn get_uncategorized_count(state: State<SharedTrackerState>) -> Result<i64, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::get_uncategorized_count(&conn).map_err(|e| e.to_string())
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
pub fn save_wrap_up_note(
    state: State<SharedTrackerState>,
    working_on: String,
    next_steps: String,
) -> Result<(), String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let session_key = tracker
        .current_session_key
        .clone()
        .ok_or_else(|| "No active session key — L3 escalation not started".to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::save_wrap_up_note(&conn, &session_key, &working_on, &next_steps)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_latest_wrap_up_note(
    state: State<SharedTrackerState>,
) -> Result<Option<crate::models::WrapUpNote>, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    let conn = db::open_db(&tracker.db_path).map_err(|e| e.to_string())?;
    db::get_latest_wrap_up_note(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_current_session_key(state: State<SharedTrackerState>) -> Result<Option<String>, String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    Ok(tracker.current_session_key.clone())
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

// --- Device profiles, settings + devices cloud sync ---------------------------

const DEVICE_PROFILES_KEY: &str = "device_profiles";
const ACTIVE_DEVICE_PROFILE_ID_KEY: &str = "active_device_profile_id";
const DEVICE_PROFILES_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloudDeviceProfilesPayload {
    #[serde(default = "default_device_profiles_version")]
    version: u32,
    #[serde(default)]
    profiles: Vec<DeviceProfile>,
}

fn default_device_profiles_version() -> u32 {
    DEVICE_PROFILES_VERSION
}

impl Default for CloudDeviceProfilesPayload {
    fn default() -> Self {
        Self {
            version: DEVICE_PROFILES_VERSION,
            profiles: Vec::new(),
        }
    }
}

fn now_rfc3339() -> String {
    chrono::Local::now().to_rfc3339()
}

fn new_profile_id() -> String {
    format!("profile-{}", chrono::Utc::now().timestamp_millis())
}

fn cleaned_profile_name(name: &str) -> Result<String, String> {
    let cleaned = name.trim();
    if cleaned.is_empty() {
        return Err("Profile name is required".into());
    }
    Ok(cleaned.chars().take(64).collect())
}

fn read_local_profile_payload(
    app: &tauri::AppHandle,
) -> Result<CloudDeviceProfilesPayload, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let payload = store
        .get(DEVICE_PROFILES_KEY)
        .and_then(|v| serde_json::from_value::<CloudDeviceProfilesPayload>(v).ok())
        .unwrap_or_default();
    Ok(CloudDeviceProfilesPayload {
        version: DEVICE_PROFILES_VERSION,
        profiles: payload.profiles,
    })
}

fn write_local_profile_payload(
    app: &tauri::AppHandle,
    payload: &CloudDeviceProfilesPayload,
) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set(
        DEVICE_PROFILES_KEY,
        serde_json::to_value(payload).map_err(|e| e.to_string())?,
    );
    store.save().map_err(|e| e.to_string())
}

fn read_active_profile_id(app: &tauri::AppHandle) -> Result<Option<String>, String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    Ok(store
        .get(ACTIVE_DEVICE_PROFILE_ID_KEY)
        .and_then(|v| v.as_str().map(String::from))
        .filter(|s| !s.trim().is_empty()))
}

fn write_active_profile_id(
    app: &tauri::AppHandle,
    profile_id: Option<&str>,
) -> Result<(), String> {
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    if let Some(id) = profile_id {
        store.set(ACTIVE_DEVICE_PROFILE_ID_KEY, serde_json::json!(id));
    } else {
        let _ = store.delete(ACTIVE_DEVICE_PROFILE_ID_KEY);
    }
    store.save().map_err(|e| e.to_string())
}

fn device_profiles_state(app: &tauri::AppHandle) -> Result<DeviceProfilesState, String> {
    let payload = read_local_profile_payload(app)?;
    let active_profile_id = read_active_profile_id(app)?.filter(|active_id| {
        payload
            .profiles
            .iter()
            .any(|profile| profile.id == *active_id)
    });
    Ok(DeviceProfilesState {
        profiles: payload.profiles,
        active_profile_id,
    })
}

fn ensure_local_profiles(
    app: &tauri::AppHandle,
    conn: &rusqlite::Connection,
) -> Result<DeviceProfilesState, String> {
    let mut payload = read_local_profile_payload(app)?;
    let mut active_profile_id = read_active_profile_id(app)?;

    if payload.profiles.is_empty() {
        let default_id = "default".to_string();
        payload.profiles.push(DeviceProfile {
            id: default_id.clone(),
            name: "Default".into(),
            settings: db::export_device_profile_settings(conn).map_err(|e| e.to_string())?,
            updated_at: now_rfc3339(),
        });
        active_profile_id = Some(default_id);
        write_local_profile_payload(app, &payload)?;
    }

    if active_profile_id
        .as_ref()
        .map(|active_id| {
            !payload
                .profiles
                .iter()
                .any(|profile| profile.id == *active_id)
        })
        .unwrap_or(true)
    {
        active_profile_id = payload.profiles.first().map(|profile| profile.id.clone());
    }

    write_active_profile_id(app, active_profile_id.as_deref())?;

    Ok(DeviceProfilesState {
        profiles: payload.profiles,
        active_profile_id,
    })
}

fn autosave_active_profile_from_db(
    app: &tauri::AppHandle,
    conn: &rusqlite::Connection,
) -> Result<(), String> {
    let Some(active_profile_id) = read_active_profile_id(app)? else {
        return Ok(());
    };
    let mut payload = read_local_profile_payload(app)?;
    let Some(profile) = payload
        .profiles
        .iter_mut()
        .find(|profile| profile.id == active_profile_id)
    else {
        return Ok(());
    };

    profile.settings = db::export_device_profile_settings(conn).map_err(|e| e.to_string())?;
    profile.updated_at = now_rfc3339();
    write_local_profile_payload(app, &payload)
}

fn reload_profile_runtime_state(
    tracker: &mut crate::tracker::TrackerState,
    conn: &rusqlite::Connection,
) -> Result<(), String> {
    tracker.escalation_engine.settings =
        db::get_escalation_settings(conn).map_err(|e| e.to_string())?;
    tracker.ignored_apps = db::get_ignored_apps(conn).map_err(|e| e.to_string())?;
    tracker.app_categories.clear();
    for (name, category) in db::get_all_app_categories_for_cache(conn).map_err(|e| e.to_string())?
    {
        tracker.app_categories.insert(name, category);
    }
    tracker.title_keyword_rules = db::get_title_keyword_rules(conn)
        .map_err(|e| e.to_string())?
        .iter()
        .map(|rule| {
            (
                rule.app_name.clone(),
                rule.keyword.clone(),
                rule.category.clone(),
            )
        })
        .collect();
    Ok(())
}

fn apply_profile_settings_to_device(
    state: &State<'_, SharedTrackerState>,
    settings: &DeviceProfileSettings,
) -> Result<(), String> {
    let db_path = {
        let tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.db_path.clone()
    };
    let conn = db::open_db(&db_path).map_err(|e| e.to_string())?;
    db::apply_device_profile_settings(&conn, settings).map_err(|e| e.to_string())?;

    let mut tracker = state.lock().map_err(|e| e.to_string())?;
    reload_profile_runtime_state(&mut tracker, &conn)
}

fn profile_is_newer(candidate: &DeviceProfile, current: &DeviceProfile) -> bool {
    match (
        chrono::DateTime::parse_from_rfc3339(&candidate.updated_at),
        chrono::DateTime::parse_from_rfc3339(&current.updated_at),
    ) {
        (Ok(candidate_time), Ok(current_time)) => candidate_time > current_time,
        _ => candidate.updated_at > current.updated_at,
    }
}

fn merge_profile_payloads(
    local: &CloudDeviceProfilesPayload,
    remote: &CloudDeviceProfilesPayload,
    deleted_profile_id: Option<&str>,
) -> CloudDeviceProfilesPayload {
    let mut merged = local.clone();

    for remote_profile in &remote.profiles {
        if let Some(local_profile) = merged
            .profiles
            .iter_mut()
            .find(|profile| profile.id == remote_profile.id)
        {
            if profile_is_newer(remote_profile, local_profile) {
                *local_profile = remote_profile.clone();
            }
        } else {
            merged.profiles.push(remote_profile.clone());
        }
    }

    if let Some(deleted_id) = deleted_profile_id {
        merged.profiles.retain(|profile| profile.id != deleted_id);
    }

    merged.version = DEVICE_PROFILES_VERSION;
    merged
}

fn profiles_from_cloud_settings(settings: &serde_json::Value) -> CloudDeviceProfilesPayload {
    settings
        .get(DEVICE_PROFILES_KEY)
        .and_then(|value| serde_json::from_value::<CloudDeviceProfilesPayload>(value.clone()).ok())
        .unwrap_or_default()
}

fn settings_object(value: Option<&serde_json::Value>) -> serde_json::Map<String, serde_json::Value> {
    value
        .and_then(|settings| settings.as_object().cloned())
        .unwrap_or_default()
}

async fn put_profiles_to_cloud(
    state: &State<'_, SharedTrackerState>,
    app: &tauri::AppHandle,
    local_payload: &CloudDeviceProfilesPayload,
    merge_remote: bool,
    deleted_profile_id: Option<&str>,
) -> Result<CloudDeviceProfilesPayload, String> {
    let (access, refresh) = read_auth(state)?;
    let sync_url = sync::api_base_url();

    let pulled = sync::authed_json::<serde_json::Value>(
        &sync_url,
        reqwest::Method::GET,
        "/api/settings",
        &access,
        &refresh,
        None,
    )
    .await?;
    store_rotated(state, app, pulled.rotated_tokens)?;

    let remote_settings_map = settings_object(pulled.body.get("settings"));
    let remote_settings_value = serde_json::Value::Object(remote_settings_map.clone());
    let remote_payload = profiles_from_cloud_settings(&remote_settings_value);
    let final_payload = if merge_remote {
        merge_profile_payloads(local_payload, &remote_payload, deleted_profile_id)
    } else {
        let mut payload = local_payload.clone();
        if let Some(deleted_id) = deleted_profile_id {
            payload.profiles.retain(|profile| profile.id != deleted_id);
        }
        payload
    };

    let mut next_settings = remote_settings_map;
    next_settings.insert(
        DEVICE_PROFILES_KEY.into(),
        serde_json::to_value(&final_payload).map_err(|e| e.to_string())?,
    );

    let body = serde_json::json!({
        "settings": serde_json::Value::Object(next_settings),
    });

    let (access, refresh) = read_auth(state)?;
    let pushed = sync::authed_json::<serde_json::Value>(
        &sync_url,
        reqwest::Method::PUT,
        "/api/settings",
        &access,
        &refresh,
        Some(&body),
    )
    .await?;
    store_rotated(state, app, pushed.rotated_tokens)?;

    Ok(final_payload)
}

async fn push_profiles_best_effort(
    state: &State<'_, SharedTrackerState>,
    app: &tauri::AppHandle,
    payload: &CloudDeviceProfilesPayload,
    merge_remote: bool,
    deleted_profile_id: Option<&str>,
) {
    if read_auth(state).is_ok() {
        if let Ok(final_payload) =
            put_profiles_to_cloud(state, app, payload, merge_remote, deleted_profile_id).await
        {
            let _ = write_local_profile_payload(app, &final_payload);
        }
    }
}

fn read_auth(
    state: &State<'_, SharedTrackerState>,
) -> Result<(String, String), String> {
    let tracker = state.lock().map_err(|e| e.to_string())?;
    if tracker.access_token.is_empty() {
        return Err("Not logged in".into());
    }
    Ok((
        tracker.access_token.clone(),
        tracker.refresh_token.clone(),
    ))
}

fn store_rotated(
    state: &State<'_, SharedTrackerState>,
    app: &tauri::AppHandle,
    rotated: Option<(String, String)>,
) -> Result<(), String> {
    let Some((access, refresh)) = rotated else { return Ok(()); };
    {
        let mut tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.access_token = access.clone();
        tracker.refresh_token = refresh.clone();
    }
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set("access_token", serde_json::json!(access));
    store.set("refresh_token", serde_json::json!(refresh));
    store.save().map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn list_device_profiles(
    state: State<SharedTrackerState>,
    app: tauri::AppHandle,
) -> Result<DeviceProfilesState, String> {
    let db_path = {
        let tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.db_path.clone()
    };
    let conn = db::open_db(&db_path).map_err(|e| e.to_string())?;
    ensure_local_profiles(&app, &conn)
}

#[tauri::command]
pub async fn create_device_profile(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
    name: String,
) -> Result<DeviceProfilesState, String> {
    let name = cleaned_profile_name(&name)?;
    let db_path = {
        let tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.db_path.clone()
    };
    let conn = db::open_db(&db_path).map_err(|e| e.to_string())?;
    ensure_local_profiles(&app, &conn)?;

    let mut payload = read_local_profile_payload(&app)?;
    let profile_id = new_profile_id();
    payload.profiles.push(DeviceProfile {
        id: profile_id.clone(),
        name,
        settings: db::export_device_profile_settings(&conn).map_err(|e| e.to_string())?,
        updated_at: now_rfc3339(),
    });
    write_local_profile_payload(&app, &payload)?;
    write_active_profile_id(&app, Some(&profile_id))?;

    push_profiles_best_effort(&state, &app, &payload, true, None).await;
    device_profiles_state(&app)
}

#[tauri::command]
pub async fn rename_device_profile(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
    profile_id: String,
    name: String,
) -> Result<DeviceProfilesState, String> {
    let name = cleaned_profile_name(&name)?;
    let mut payload = read_local_profile_payload(&app)?;
    let Some(profile) = payload
        .profiles
        .iter_mut()
        .find(|profile| profile.id == profile_id)
    else {
        return Err("Profile not found".into());
    };
    profile.name = name;
    profile.updated_at = now_rfc3339();
    write_local_profile_payload(&app, &payload)?;

    push_profiles_best_effort(&state, &app, &payload, true, None).await;
    device_profiles_state(&app)
}

#[tauri::command]
pub async fn delete_device_profile(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
    profile_id: String,
) -> Result<DeviceProfilesState, String> {
    let mut payload = read_local_profile_payload(&app)?;
    if payload.profiles.len() <= 1 {
        return Err("Keep at least one profile".into());
    }

    let original_len = payload.profiles.len();
    payload.profiles.retain(|profile| profile.id != profile_id);
    if payload.profiles.len() == original_len {
        return Err("Profile not found".into());
    }
    write_local_profile_payload(&app, &payload)?;

    let deleted_active = read_active_profile_id(&app)?
        .as_ref()
        .map(|active_id| active_id == &profile_id)
        .unwrap_or(false);
    if deleted_active {
        let next_profile = payload
            .profiles
            .first()
            .cloned()
            .ok_or_else(|| "No remaining profiles".to_string())?;
        write_active_profile_id(&app, Some(&next_profile.id))?;
        apply_profile_settings_to_device(&state, &next_profile.settings)?;
    }

    push_profiles_best_effort(&state, &app, &payload, true, Some(&profile_id)).await;
    device_profiles_state(&app)
}

#[tauri::command]
pub async fn select_device_profile(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
    profile_id: String,
) -> Result<DeviceProfilesState, String> {
    let db_path = {
        let tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.db_path.clone()
    };
    let conn = db::open_db(&db_path).map_err(|e| e.to_string())?;
    ensure_local_profiles(&app, &conn)?;
    autosave_active_profile_from_db(&app, &conn)?;

    let payload = read_local_profile_payload(&app)?;
    let profile = payload
        .profiles
        .iter()
        .find(|profile| profile.id == profile_id)
        .cloned()
        .ok_or_else(|| "Profile not found".to_string())?;

    apply_profile_settings_to_device(&state, &profile.settings)?;
    write_active_profile_id(&app, Some(&profile.id))?;

    push_profiles_best_effort(&state, &app, &payload, true, None).await;
    device_profiles_state(&app)
}

#[tauri::command]
pub async fn save_active_device_profile(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
) -> Result<DeviceProfilesState, String> {
    let db_path = {
        let tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.db_path.clone()
    };
    let conn = db::open_db(&db_path).map_err(|e| e.to_string())?;
    ensure_local_profiles(&app, &conn)?;
    autosave_active_profile_from_db(&app, &conn)?;

    let payload = read_local_profile_payload(&app)?;
    push_profiles_best_effort(&state, &app, &payload, true, None).await;
    device_profiles_state(&app)
}

#[tauri::command]
pub async fn sync_device_profiles(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
) -> Result<DeviceProfilesState, String> {
    let db_path = {
        let tracker = state.lock().map_err(|e| e.to_string())?;
        tracker.db_path.clone()
    };
    let conn = db::open_db(&db_path).map_err(|e| e.to_string())?;
    ensure_local_profiles(&app, &conn)?;

    let local_payload = read_local_profile_payload(&app)?;
    let synced_payload = put_profiles_to_cloud(&state, &app, &local_payload, true, None).await?;
    write_local_profile_payload(&app, &synced_payload)?;

    let mut active_profile_id = read_active_profile_id(&app)?;
    if active_profile_id
        .as_ref()
        .map(|active_id| {
            !synced_payload
                .profiles
                .iter()
                .any(|profile| profile.id == *active_id)
        })
        .unwrap_or(true)
    {
        active_profile_id = synced_payload
            .profiles
            .first()
            .map(|profile| profile.id.clone());
        write_active_profile_id(&app, active_profile_id.as_deref())?;
    }

    if let Some(active_id) = active_profile_id {
        if let Some(profile) = synced_payload
            .profiles
            .iter()
            .find(|profile| profile.id == active_id)
        {
            apply_profile_settings_to_device(&state, &profile.settings)?;
        }
    }

    device_profiles_state(&app)
}

#[tauri::command]
pub async fn pull_settings(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let (access, refresh) = read_auth(&state)?;
    let sync_url = sync::api_base_url();
    let result = sync::authed_json::<serde_json::Value>(
        &sync_url,
        reqwest::Method::GET,
        "/api/settings",
        &access,
        &refresh,
        None,
    )
    .await?;
    store_rotated(&state, &app, result.rotated_tokens)?;
    let mut body = result.body;
    if let Some(object) = body.as_object_mut() {
        object.remove("theme");
    }
    Ok(body)
}

#[tauri::command]
pub async fn push_settings(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
    settings: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let (access, refresh) = read_auth(&state)?;
    let sync_url = sync::api_base_url();

    let incoming_settings = settings_object(Some(&settings));
    let mut next_settings =
        match sync::authed_json::<serde_json::Value>(
            &sync_url,
            reqwest::Method::GET,
            "/api/settings",
            &access,
            &refresh,
            None,
        )
        .await
        {
            Ok(pulled) => {
                store_rotated(&state, &app, pulled.rotated_tokens)?;
                settings_object(pulled.body.get("settings"))
            }
            Err(e) => {
                let local_profiles = read_local_profile_payload(&app).unwrap_or_default();
                if incoming_settings.is_empty() && local_profiles.profiles.is_empty() {
                    return Err(e);
                }
                serde_json::Map::new()
            }
        };

    for (key, value) in incoming_settings {
        next_settings.insert(key, value);
    }

    if !next_settings.contains_key(DEVICE_PROFILES_KEY) {
        let local_profiles = read_local_profile_payload(&app).unwrap_or_default();
        if !local_profiles.profiles.is_empty() {
            next_settings.insert(
                DEVICE_PROFILES_KEY.into(),
                serde_json::to_value(local_profiles).map_err(|e| e.to_string())?,
            );
        }
    }

    let body = serde_json::json!({
        "settings": serde_json::Value::Object(next_settings),
    });
    let (access, refresh) = read_auth(&state)?;
    let result = sync::authed_json::<serde_json::Value>(
        &sync_url,
        reqwest::Method::PUT,
        "/api/settings",
        &access,
        &refresh,
        Some(&body),
    )
    .await?;
    store_rotated(&state, &app, result.rotated_tokens)?;
    Ok(result.body)
}

#[tauri::command]
pub async fn list_devices(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let (access, refresh) = read_auth(&state)?;
    let sync_url = sync::api_base_url();
    let result = sync::authed_json::<serde_json::Value>(
        &sync_url,
        reqwest::Method::GET,
        "/api/devices",
        &access,
        &refresh,
        None,
    )
    .await?;
    store_rotated(&state, &app, result.rotated_tokens)?;
    Ok(result.body)
}

#[tauri::command]
pub async fn revoke_device(
    state: State<'_, SharedTrackerState>,
    app: tauri::AppHandle,
    device_id: i64,
) -> Result<serde_json::Value, String> {
    let (access, refresh) = read_auth(&state)?;
    let sync_url = sync::api_base_url();
    let path = format!("/api/devices/{device_id}");
    let result = sync::authed_json::<serde_json::Value>(
        &sync_url,
        reqwest::Method::DELETE,
        &path,
        &access,
        &refresh,
        None,
    )
    .await?;
    store_rotated(&state, &app, result.rotated_tokens)?;
    Ok(result.body)
}
