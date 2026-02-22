use chrono::{Local, Timelike};
use tauri::{AppHandle, Emitter};

use crate::models::{EscalationLevel, EscalationSettings, EscalationStatePayload};

#[derive(PartialEq)]
enum TimeZone {
    Green,
    Yellow,
    Red,
}

pub struct EscalationEngine {
    pub settings: EscalationSettings,
    pub current_level: EscalationLevel,
    last_level_change: Option<chrono::DateTime<chrono::Local>>,
    /// Timestamps of popup (Level 2) dismissals in the current session.
    pub popup_dismissals: Vec<String>,
}

impl EscalationEngine {
    /// Create a new engine with the given settings.
    ///
    /// `last_level_change` is set to `Some(Local::now())` on construction so
    /// the full gap must elapse before the first escalation fires (Pitfall 3 in
    /// research — prevents immediate escalation burst if the app starts inside
    /// the yellow/red zone).
    pub fn new(settings: EscalationSettings) -> Self {
        Self {
            settings,
            current_level: EscalationLevel::None,
            last_level_change: Some(Local::now()),
            popup_dismissals: Vec::new(),
        }
    }

    /// Called every 2 seconds from the tracker loop.
    ///
    /// `is_idle` — true when the OS reports no keyboard/mouse activity for
    /// the configured idle threshold.
    /// `current_category` — resolved category for the active app ("productive", "distracting",
    /// "neutral", "uncategorized"). Adjusts escalation gap via multiplier.
    pub fn tick(&mut self, app_handle: &AppHandle, is_idle: bool, current_category: &str) {
        // Terminal state — nothing more to do tonight.
        if self.current_level == EscalationLevel::Done {
            return;
        }

        // Honour pause-until timestamp.
        if let Some(ref until) = self.settings.paused_until.clone() {
            if let Ok(until_dt) = chrono::DateTime::parse_from_rfc3339(until) {
                if Local::now() < until_dt.with_timezone(&Local) {
                    return;
                }
            }
            // Pause has expired — clear it.
            self.settings.paused_until = None;
        }

        if !self.settings.enabled {
            return;
        }

        let now = Local::now();
        let current_hour = now.hour();
        let zone = self.time_zone(current_hour);

        // Green zone: silence and de-escalate back to None if we drifted in.
        if zone == TimeZone::Green {
            if self.current_level != EscalationLevel::None {
                self.set_level(EscalationLevel::None, app_handle);
            }
            return;
        }

        // Idle: hold current level, no advancement.
        if is_idle {
            return;
        }

        // Gap between levels.
        // sensitivity 0.0 → 20 min; sensitivity 1.0 → 10 min.
        let gap_minutes = 20.0 - (self.settings.sensitivity * 10.0);

        // Apply category multiplier: higher multiplier = shorter gap = faster escalation
        let multiplier: f32 = match current_category {
            "distracting" => self.settings.distracting_multiplier.max(0.01),
            "productive"  => self.settings.productive_multiplier.max(0.01),
            _             => 1.0, // neutral, uncategorized
        };
        let gap_secs = ((gap_minutes * 60.0) / multiplier) as i64;

        let elapsed = self
            .last_level_change
            .map(|t| now.signed_duration_since(t).num_seconds())
            .unwrap_or(i64::MAX);

        let should_advance = if zone == TimeZone::Red {
            // Red zone: halve the required gap for faster escalation.
            elapsed >= gap_secs / 2
        } else {
            elapsed >= gap_secs
        };

        if should_advance {
            let next = self.next_level();
            self.set_level(next, app_handle);
        }
    }

    /// Drop one escalation level (called when user switches to a productive
    /// app — Phase 2 will wire the category signal).
    pub fn de_escalate(&mut self, app_handle: &AppHandle) {
        let next = match self.current_level {
            EscalationLevel::Level2 => EscalationLevel::Level1,
            EscalationLevel::Level3 => EscalationLevel::Level2,
            EscalationLevel::Level4 => EscalationLevel::Level3,
            ref other => other.clone(),
        };
        self.set_level(next, app_handle);
    }

    /// Mark escalation as Done — called when the Level 4 wrap-up form is
    /// submitted.  No further escalation occurs this session.
    pub fn dismiss(&mut self, app_handle: &AppHandle) {
        self.set_level(EscalationLevel::Done, app_handle);
    }

    // --- Private helpers ---

    fn set_level(&mut self, level: EscalationLevel, app_handle: &AppHandle) {
        if self.current_level == level {
            return;
        }
        self.current_level = level.clone();
        self.last_level_change = Some(Local::now());

        let payload = EscalationStatePayload {
            message: self.message_for_level(&level),
            level,
        };
        // Broadcast to all listening windows.
        // Source: https://v2.tauri.app/develop/calling-frontend/
        let _ = app_handle.emit("escalation-state-changed", &payload);
    }

    fn next_level(&self) -> EscalationLevel {
        match self.current_level {
            EscalationLevel::None => EscalationLevel::Level1,
            EscalationLevel::Level1 => EscalationLevel::Level2,
            EscalationLevel::Level2 => EscalationLevel::Level3,
            EscalationLevel::Level3 => EscalationLevel::Level4,
            EscalationLevel::Level4 | EscalationLevel::Done => EscalationLevel::Done,
        }
    }

    fn time_zone(&self, hour: u32) -> TimeZone {
        if hour < self.settings.green_end_hour {
            TimeZone::Green
        } else if hour < self.settings.yellow_end_hour {
            TimeZone::Yellow
        } else {
            TimeZone::Red
        }
    }

    fn message_for_level(&self, level: &EscalationLevel) -> String {
        match level {
            EscalationLevel::Level1 => {
                "It's getting late — time to start wrapping up.".into()
            }
            EscalationLevel::Level2 => "Still going? Consider finishing up soon.".into(),
            EscalationLevel::Level3 => {
                "You've been working late. Start your wrap-up.".into()
            }
            EscalationLevel::Level4 => {
                "Time to stop. Write your notes and call it a night.".into()
            }
            _ => String::new(),
        }
    }
}
