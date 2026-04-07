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

pub struct TrayMenuItems {
    pub tracking_toggle: MenuItem<tauri::Wry>,
    pub pause_1h: MenuItem<tauri::Wry>,
    pub pause_2h: MenuItem<tauri::Wry>,
    pub pause_tonight: MenuItem<tauri::Wry>,
    pub resume: MenuItem<tauri::Wry>,
}

pub fn update_tray_pause_items(items: &TrayMenuItems, is_paused: bool) {
    let _ = items.pause_1h.set_enabled(!is_paused);
    let _ = items.pause_2h.set_enabled(!is_paused);
    let _ = items.pause_tonight.set_enabled(!is_paused);
    let _ = items.resume.set_enabled(is_paused);
}

pub fn update_tray_tracking_item(items: &TrayMenuItems, is_tracking: bool) {
    let text = if is_tracking { "Pause Tracking" } else { "Resume Tracking" };
    let _ = items.tracking_toggle.set_text(text);
}

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
                    if let Some(token) = store.get("access_token").and_then(|v| v.as_str().map(String::from)) {
                        state.access_token = token;
                    }
                    if let Some(token) = store.get("refresh_token").and_then(|v| v.as_str().map(String::from)) {
                        state.refresh_token = token;
                    }
                }

                // Load settings from SQLite
                if let Ok(conn) = db::open_db(&db_path.to_string_lossy()) {
                    if let Ok(esc_settings) = db::get_escalation_settings(&conn) {
                        state.escalation_engine.settings = esc_settings;
                    }
                    if let Ok(apps) = db::get_ignored_apps(&conn) {
                        state.ignored_apps = apps;
                    }
                    if let Ok(cats) = db::get_all_app_categories_for_cache(&conn) {
                        for (name, cat) in cats {
                            state.app_categories.insert(name, cat);
                        }
                    }
                    if let Ok(rules) = db::get_title_keyword_rules(&conn) {
                        state.title_keyword_rules = rules
                            .iter()
                            .map(|r| (r.app_name.clone(), r.keyword.clone(), r.category.clone()))
                            .collect();
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

            // Show resume-notes popup if a recent wrap-up note exists
            {
                let db_path = db_path.clone();
                let app_handle = app.handle().clone();
                if let Ok(conn) = db::open_db(&db_path.to_string_lossy()) {
                    if let Ok(Some(_)) = db::get_latest_wrap_up_note(&conn) {
                        std::thread::spawn(move || {
                            std::thread::sleep(std::time::Duration::from_millis(500));
                            let handle = app_handle.clone();
                            let _ = app_handle.run_on_main_thread(move || {
                                use tauri::WebviewUrl;
                                use tauri::webview::WebviewWindowBuilder;
                                let _ = WebviewWindowBuilder::new(
                                    &handle,
                                    "resume-notes",
                                    WebviewUrl::App("/#/overlay/resume".into()),
                                )
                                .title("LucidShift — Resume Notes")
                                .inner_size(340.0, 220.0)
                                .decorations(false)
                                .always_on_top(true)
                                .skip_taskbar(true)
                                .resizable(true)
                                .build();
                            });
                        });
                    }
                }
            }

            // Build system tray
            let show_item =
                MenuItem::with_id(app, "show", "Show Dashboard", true, None::<&str>)?;
            let pause_item =
                MenuItem::with_id(app, "pause", "Pause Tracking", true, None::<&str>)?;
            let pause_1h =
                MenuItem::with_id(app, "pause_1h", "Pause Escalation 1 hour", true, None::<&str>)?;
            let pause_2h =
                MenuItem::with_id(app, "pause_2h", "Pause Escalation 2 hours", true, None::<&str>)?;
            let pause_tonight =
                MenuItem::with_id(app, "pause_tonight", "Pause Escalation until tomorrow", true, None::<&str>)?;
            let resume_esc =
                MenuItem::with_id(app, "resume_esc", "Resume Escalation", false, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &pause_item, &pause_1h, &pause_2h, &pause_tonight, &resume_esc, &quit_item])?;

            // Manage tray menu item handles for dynamic enable/disable
            let tray_items = TrayMenuItems {
                tracking_toggle: pause_item.clone(),
                pause_1h: pause_1h.clone(),
                pause_2h: pause_2h.clone(),
                pause_tonight: pause_tonight.clone(),
                resume: resume_esc.clone(),
            };
            // Check initial pause state and set tray items accordingly
            if let Some(state) = app.try_state::<Arc<Mutex<tracker::TrackerState>>>() {
                if let Ok(t) = state.lock() {
                    let is_paused = t.escalation_engine.settings.paused_until.as_ref().map_or(false, |until| {
                        chrono::DateTime::parse_from_rfc3339(until)
                            .map(|dt| chrono::Local::now() < dt.with_timezone(&chrono::Local))
                            .unwrap_or(false)
                    });
                    update_tray_pause_items(&tray_items, is_paused);
                }
            }
            app.manage(tray_items);

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
                                if let Some(tray_items) = app.try_state::<TrayMenuItems>() {
                                    update_tray_tracking_item(&tray_items, t.is_tracking);
                                }
                            }
                        }
                    }
                    "pause_1h" | "pause_2h" | "pause_tonight" => {
                        let hours: i64 = match event.id().as_ref() {
                            "pause_1h" => 1,
                            "pause_2h" => 2,
                            "pause_tonight" => {
                                let now = chrono::Local::now();
                                let tomorrow_6am = (now + chrono::Duration::days(1))
                                    .date_naive()
                                    .and_hms_opt(6, 0, 0)
                                    .unwrap();
                                let tomorrow_6am = tomorrow_6am
                                    .and_local_timezone(chrono::Local)
                                    .unwrap();
                                let diff = tomorrow_6am.signed_duration_since(now);
                                diff.num_hours().max(1)
                            }
                            _ => 1,
                        };
                        let until = (chrono::Local::now() + chrono::Duration::hours(hours))
                            .to_rfc3339();
                        if let Some(state) = app.try_state::<std::sync::Arc<std::sync::Mutex<crate::tracker::TrackerState>>>() {
                            if let Ok(mut t) = state.lock() {
                                t.escalation_engine.settings.paused_until = Some(until.clone());
                                if let Ok(conn) = crate::db::open_db(&t.db_path) {
                                    let _ = crate::db::save_escalation_settings(&conn, &t.escalation_engine.settings);
                                }
                            }
                        }
                        if let Some(tray_items) = app.try_state::<TrayMenuItems>() {
                            update_tray_pause_items(&tray_items, true);
                        }
                    }
                    "resume_esc" => {
                        if let Some(state) = app.try_state::<std::sync::Arc<std::sync::Mutex<crate::tracker::TrackerState>>>() {
                            if let Ok(mut t) = state.lock() {
                                t.escalation_engine.settings.paused_until = None;
                                if let Ok(conn) = crate::db::open_db(&t.db_path) {
                                    let _ = crate::db::save_escalation_settings(&conn, &t.escalation_engine.settings);
                                }
                            }
                        }
                        if let Some(tray_items) = app.try_state::<TrayMenuItems>() {
                            update_tray_pause_items(&tray_items, false);
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
                if window.label() == "main" {
                    let _ = window.hide();
                    api.prevent_close();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_current_app,
            commands::get_daily_stats,
            commands::get_activity_timeline,
            commands::toggle_tracking,
            commands::get_tracking,
            commands::get_ignored_apps,
            commands::set_ignored_apps,
            commands::get_reminder_rules,
            commands::save_reminder_rule,
            commands::delete_reminder_rule,
            commands::toggle_reminder_rule,
            commands::sync_now,
            commands::get_sync_status,
            commands::login,
            commands::register,
            commands::logout,
            commands::get_auth_status,
            commands::show_escalation_window,
            commands::dismiss_escalation,
            commands::acknowledge_popup,
            commands::get_popup_dismissals,
            commands::get_escalation_settings,
            commands::set_escalation_settings,
            commands::pause_escalation,
            commands::test_reminder_notification,
            commands::get_app_categories,
            commands::set_app_category,
            commands::get_title_keyword_rules,
            commands::add_title_keyword_rule,
            commands::delete_title_keyword_rule,
            commands::get_uncategorized_count,
            commands::save_wrap_up_note,
            commands::get_latest_wrap_up_note,
            commands::get_current_session_key,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
