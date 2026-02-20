mod commands;
mod db;
mod escalation;
mod models;
mod reminders;
mod sync;
mod tracker;

use std::sync::{Arc, Mutex};

use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri_plugin_store::StoreExt;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let tracker_state = Arc::new(Mutex::new(tracker::TrackerState::new()));
    let tracker_for_bg = tracker_state.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_global_shortcut::Builder::default().build())
        .manage(tracker_state)
        .setup(|app| {
            // Initialize database
            let app_data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&app_data_dir)?;
            let db_path = app_data_dir.join("sleep_app.db");
            db::init_db(&db_path).expect("Failed to initialize database");

            // Configure tracker state
            {
                let mut state = tracker_for_bg.lock().unwrap();
                state.db_path = db_path.to_string_lossy().to_string();
                state.is_tracking = true;

                // Load sync config from store
                if let Ok(store) = app.store("settings.json") {
                    if let Some(url) = store.get("sync_url").and_then(|v| v.as_str().map(String::from)) {
                        state.sync_url = url;
                    }
                    if let Some(key) = store.get("api_key").and_then(|v| v.as_str().map(String::from)) {
                        state.api_key = key;
                    }
                }

                // Load escalation settings from SQLite
                if let Ok(conn) = db::open_db(&db_path.to_string_lossy()) {
                    if let Ok(esc_settings) = db::get_escalation_settings(&conn) {
                        state.escalation_engine.settings = esc_settings;
                    }
                }
            }

            // Start background tracking
            tracker::start_tracking(tracker_for_bg);

            // Inject app handle so the escalation engine can emit events.
            // The Arc<Mutex<TrackerState>> is shared between this setup closure
            // and the spawned loop, so writing app_handle here is visible to
            // the background task.
            {
                let app_handle = app.handle().clone();
                if let Ok(mut state) = app
                    .state::<Arc<Mutex<tracker::TrackerState>>>()
                    .lock()
                {
                    state.app_handle = Some(app_handle);
                }
            }

            // Build system tray
            let show_item =
                MenuItem::with_id(app, "show", "Show Dashboard", true, None::<&str>)?;
            let pause_item =
                MenuItem::with_id(app, "pause", "Pause Tracking", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &pause_item, &quit_item])?;

            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                    "pause" => {
                        if let Some(state) = app.try_state::<Arc<Mutex<tracker::TrackerState>>>() {
                            if let Ok(mut t) = state.lock() {
                                t.is_tracking = !t.is_tracking;
                            }
                        }
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_current_app,
            commands::get_daily_stats,
            commands::get_activity_timeline,
            commands::toggle_tracking,
            commands::get_ignored_apps,
            commands::set_ignored_apps,
            commands::get_reminder_rules,
            commands::save_reminder_rule,
            commands::delete_reminder_rule,
            commands::toggle_reminder_rule,
            commands::sync_now,
            commands::set_sync_config,
            commands::get_sync_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
