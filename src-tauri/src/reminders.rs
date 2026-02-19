use std::collections::HashMap;

use chrono::Local;
use tauri::AppHandle;

use crate::models::ReminderRule;

pub struct ReminderEngine {
    pub rules: Vec<ReminderRule>,
    last_triggered: HashMap<i64, chrono::DateTime<chrono::Local>>,
    cooldown_minutes: i64,
}

impl ReminderEngine {
    pub fn new() -> Self {
        Self {
            rules: Vec::new(),
            last_triggered: HashMap::new(),
            cooldown_minutes: 10,
        }
    }

    pub fn update_rules(&mut self, rules: Vec<ReminderRule>) {
        self.rules = rules;
    }

    pub fn check_rules(
        &mut self,
        current_app: &str,
        duration_on_app_secs: i64,
        total_continuous_secs: i64,
        app_handle: &AppHandle,
    ) {
        let now = Local::now();

        for rule in &self.rules {
            if !rule.enabled {
                continue;
            }

            let rule_id = match rule.id {
                Some(id) => id,
                None => continue,
            };

            let threshold_secs = rule.threshold_minutes * 60;

            let should_trigger = match rule.rule_type.as_str() {
                "app_limit" => {
                    rule.app_name.as_deref() == Some(current_app)
                        && duration_on_app_secs >= threshold_secs
                }
                "break_reminder" => total_continuous_secs >= threshold_secs,
                _ => false,
            };

            if should_trigger && !self.recently_triggered(rule_id, &now) {
                self.send_notification(app_handle, &rule.message);
                self.last_triggered.insert(rule_id, now);
            }
        }
    }

    fn recently_triggered(
        &self,
        rule_id: i64,
        now: &chrono::DateTime<chrono::Local>,
    ) -> bool {
        if let Some(last) = self.last_triggered.get(&rule_id) {
            let elapsed = now.signed_duration_since(*last);
            elapsed.num_minutes() < self.cooldown_minutes
        } else {
            false
        }
    }

    fn send_notification(&self, app_handle: &AppHandle, message: &str) {
        use tauri_plugin_notification::NotificationExt;
        let _ = app_handle
            .notification()
            .builder()
            .title("Sleep App Reminder")
            .body(message)
            .show();
    }
}
