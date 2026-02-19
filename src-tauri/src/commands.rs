use tauri::State;

use crate::db;
use crate::models::*;
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
