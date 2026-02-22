use std::path::Path;

use rusqlite::{Connection, Result, params};

use crate::models::{ActivitySession, AppCategoryEntry, AppUsageStat, DailyStats, EscalationSettings, ReminderRule, TitleKeywordRule};

pub fn open_db(db_path: &str) -> Result<Connection> {
    Connection::open(db_path)
}

pub fn init_db(db_path: &Path) -> Result<()> {
    let conn = Connection::open(db_path)?;

    conn.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS activity_sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app_name TEXT NOT NULL,
            window_title TEXT NOT NULL,
            start_time TEXT NOT NULL,
            end_time TEXT NOT NULL,
            duration_secs INTEGER NOT NULL,
            date TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS reminder_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            rule_type TEXT NOT NULL,
            app_name TEXT,
            threshold_minutes INTEGER NOT NULL,
            message TEXT NOT NULL,
            enabled INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS sync_log (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            synced_at TEXT NOT NULL,
            records_synced INTEGER NOT NULL,
            status TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_sessions_date ON activity_sessions(date);
        CREATE INDEX IF NOT EXISTS idx_sessions_app ON activity_sessions(app_name);

        CREATE TABLE IF NOT EXISTS escalation_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            green_end_hour INTEGER NOT NULL DEFAULT 22,
            yellow_end_hour INTEGER NOT NULL DEFAULT 23,
            sensitivity REAL NOT NULL DEFAULT 0.5,
            enabled INTEGER NOT NULL DEFAULT 1,
            paused_until TEXT
        );
        INSERT OR IGNORE INTO escalation_settings (id) VALUES (1);

        CREATE TABLE IF NOT EXISTS ignored_apps (
            app_name TEXT PRIMARY KEY NOT NULL
        );

        CREATE TABLE IF NOT EXISTS app_categories (
            app_name TEXT PRIMARY KEY NOT NULL,
            category TEXT NOT NULL DEFAULT 'uncategorized',
            last_seen TEXT,
            is_default INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS title_keyword_rules (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            app_name TEXT NOT NULL,
            keyword TEXT NOT NULL,
            category TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_keyword_rules_app ON title_keyword_rules(app_name);
        ",
    )?;

    // Add multiplier columns to escalation_settings — silently ignore duplicate column errors
    let _ = conn.execute("ALTER TABLE escalation_settings ADD COLUMN productive_multiplier REAL NOT NULL DEFAULT 0.5", []);
    let _ = conn.execute("ALTER TABLE escalation_settings ADD COLUMN distracting_multiplier REAL NOT NULL DEFAULT 1.5", []);

    // Seed default reminder rules if table is empty
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM reminder_rules",
        [],
        |row| row.get(0),
    )?;

    if count == 0 {
        conn.execute(
            "INSERT INTO reminder_rules (rule_type, app_name, threshold_minutes, message, enabled) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["break_reminder", None::<String>, 60, "You've been at the computer for an hour. Take a 5-minute break!", 1],
        )?;
        conn.execute(
            "INSERT INTO reminder_rules (rule_type, app_name, threshold_minutes, message, enabled) VALUES (?1, ?2, ?3, ?4, ?5)",
            params!["break_reminder", None::<String>, 25, "Pomodoro: 25 minutes passed. Time for a short break!", 0],
        )?;
    }

    // Seed default app categories if table is empty
    let cat_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM app_categories",
        [],
        |row| row.get(0),
    )?;

    if cat_count == 0 {
        // Productive apps
        for app_name in &["code", "rider", "idea64", "pycharm64", "vim", "nvim", "windowsterminal", "winword", "excel"] {
            conn.execute(
                "INSERT INTO app_categories (app_name, category, last_seen, is_default) VALUES (?1, 'productive', NULL, 1)",
                params![app_name],
            )?;
        }
        // Neutral apps
        for app_name in &["slack", "teams", "discord", "outlook", "explorer", "finder"] {
            conn.execute(
                "INSERT INTO app_categories (app_name, category, last_seen, is_default) VALUES (?1, 'neutral', NULL, 1)",
                params![app_name],
            )?;
        }
        // Distracting apps
        for app_name in &["netflix", "steam"] {
            conn.execute(
                "INSERT INTO app_categories (app_name, category, last_seen, is_default) VALUES (?1, 'distracting', NULL, 1)",
                params![app_name],
            )?;
        }
    }

    // Seed default title keyword rules if table is empty
    let rule_count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM title_keyword_rules",
        [],
        |row| row.get(0),
    )?;

    if rule_count == 0 {
        // Distracting keyword rules
        let distracting_rules = [
            ("chrome", "youtube"), ("chrome", "netflix"), ("chrome", "reddit"),
            ("chrome", "twitter"), ("chrome", "tiktok"), ("chrome", "instagram"),
            ("chrome", "facebook"), ("firefox", "youtube"), ("firefox", "netflix"),
            ("firefox", "reddit"), ("msedge", "youtube"), ("msedge", "netflix"),
        ];
        for (app, keyword) in &distracting_rules {
            conn.execute(
                "INSERT INTO title_keyword_rules (app_name, keyword, category) VALUES (?1, ?2, 'distracting')",
                params![app, keyword],
            )?;
        }
        // Productive keyword rules
        let productive_rules = [
            ("chrome", "github"), ("chrome", "gitlab"), ("chrome", "stackoverflow"),
            ("chrome", "localhost"), ("firefox", "github"), ("msedge", "github"),
        ];
        for (app, keyword) in &productive_rules {
            conn.execute(
                "INSERT INTO title_keyword_rules (app_name, keyword, category) VALUES (?1, ?2, 'productive')",
                params![app, keyword],
            )?;
        }
    }

    Ok(())
}

pub fn insert_session(conn: &Connection, session: &ActivitySession) -> Result<()> {
    conn.execute(
        "INSERT INTO activity_sessions (app_name, window_title, start_time, end_time, duration_secs, date) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            session.app_name,
            session.window_title,
            session.start_time,
            session.end_time,
            session.duration_secs,
            session.date,
        ],
    )?;
    conn.execute(
        "INSERT INTO app_categories (app_name, last_seen)
         VALUES (?1, ?2)
         ON CONFLICT(app_name) DO UPDATE SET last_seen = ?2",
        params![session.app_name.to_lowercase(), session.end_time],
    )?;
    Ok(())
}

pub fn get_daily_sessions(conn: &Connection, date: &str) -> Result<Vec<ActivitySession>> {
    let mut stmt = conn.prepare(
        "SELECT id, app_name, window_title, start_time, end_time, duration_secs, date FROM activity_sessions WHERE date = ?1 ORDER BY start_time ASC",
    )?;

    let sessions = stmt.query_map(params![date], |row| {
        Ok(ActivitySession {
            id: Some(row.get(0)?),
            app_name: row.get(1)?,
            window_title: row.get(2)?,
            start_time: row.get(3)?,
            end_time: row.get(4)?,
            duration_secs: row.get(5)?,
            date: row.get(6)?,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(sessions)
}

pub fn get_daily_stats(conn: &Connection, date: &str) -> Result<DailyStats> {
    let mut stmt = conn.prepare(
        "SELECT app_name, SUM(duration_secs) as total, COUNT(*) as cnt FROM activity_sessions WHERE date = ?1 GROUP BY app_name ORDER BY total DESC",
    )?;

    let rows: Vec<(String, i64, i64)> = stmt
        .query_map(params![date], |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?
        .collect::<Result<Vec<_>>>()?;

    let total_secs: i64 = rows.iter().map(|(_, t, _)| t).sum();
    let most_used = rows.first().map(|(name, _, _)| name.clone()).unwrap_or_default();

    let app_usage: Vec<AppUsageStat> = rows
        .into_iter()
        .map(|(app_name, total_duration_secs, session_count)| {
            let percentage = if total_secs > 0 {
                (total_duration_secs as f64 / total_secs as f64) * 100.0
            } else {
                0.0
            };
            AppUsageStat {
                app_name,
                total_duration_secs,
                percentage,
                session_count,
            }
        })
        .collect();

    Ok(DailyStats {
        date: date.to_string(),
        total_tracked_secs: total_secs,
        app_usage,
        most_used_app: most_used,
    })
}

// Reminder rule CRUD
pub fn get_reminder_rules(conn: &Connection) -> Result<Vec<ReminderRule>> {
    let mut stmt = conn.prepare(
        "SELECT id, rule_type, app_name, threshold_minutes, message, enabled FROM reminder_rules ORDER BY id ASC",
    )?;

    let rules = stmt.query_map([], |row| {
        let enabled_int: i64 = row.get(5)?;
        Ok(ReminderRule {
            id: Some(row.get(0)?),
            rule_type: row.get(1)?,
            app_name: row.get(2)?,
            threshold_minutes: row.get(3)?,
            message: row.get(4)?,
            enabled: enabled_int != 0,
        })
    })?
    .collect::<Result<Vec<_>>>()?;

    Ok(rules)
}

pub fn save_reminder_rule(conn: &Connection, rule: &ReminderRule) -> Result<()> {
    if let Some(id) = rule.id {
        conn.execute(
            "UPDATE reminder_rules SET rule_type = ?1, app_name = ?2, threshold_minutes = ?3, message = ?4, enabled = ?5 WHERE id = ?6",
            params![rule.rule_type, rule.app_name, rule.threshold_minutes, rule.message, rule.enabled as i64, id],
        )?;
    } else {
        conn.execute(
            "INSERT INTO reminder_rules (rule_type, app_name, threshold_minutes, message, enabled) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![rule.rule_type, rule.app_name, rule.threshold_minutes, rule.message, rule.enabled as i64],
        )?;
    }
    Ok(())
}

pub fn delete_reminder_rule(conn: &Connection, rule_id: i64) -> Result<()> {
    conn.execute("DELETE FROM reminder_rules WHERE id = ?1", params![rule_id])?;
    Ok(())
}

pub fn toggle_reminder_rule(conn: &Connection, rule_id: i64, enabled: bool) -> Result<()> {
    conn.execute(
        "UPDATE reminder_rules SET enabled = ?1 WHERE id = ?2",
        params![enabled as i64, rule_id],
    )?;
    Ok(())
}

// Sync helpers
pub fn get_last_sync_time(conn: &Connection) -> Result<Option<String>> {
    let result = conn.query_row(
        "SELECT synced_at FROM sync_log WHERE status = 'success' ORDER BY id DESC LIMIT 1",
        [],
        |row| row.get(0),
    );
    match result {
        Ok(time) => Ok(Some(time)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e),
    }
}

pub fn get_sessions_since(conn: &Connection, since: &Option<String>) -> Result<Vec<ActivitySession>> {
    let (sql, param): (&str, Option<String>) = match since {
        Some(time) => (
            "SELECT id, app_name, window_title, start_time, end_time, duration_secs, date FROM activity_sessions WHERE start_time > ?1 ORDER BY start_time ASC",
            Some(time.clone()),
        ),
        None => (
            "SELECT id, app_name, window_title, start_time, end_time, duration_secs, date FROM activity_sessions ORDER BY start_time ASC",
            None,
        ),
    };

    let mut stmt = conn.prepare(sql)?;
    let sessions = if let Some(ref p) = param {
        stmt.query_map(params![p], |row| {
            Ok(ActivitySession {
                id: Some(row.get(0)?),
                app_name: row.get(1)?,
                window_title: row.get(2)?,
                start_time: row.get(3)?,
                end_time: row.get(4)?,
                duration_secs: row.get(5)?,
                date: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?
    } else {
        stmt.query_map([], |row| {
            Ok(ActivitySession {
                id: Some(row.get(0)?),
                app_name: row.get(1)?,
                window_title: row.get(2)?,
                start_time: row.get(3)?,
                end_time: row.get(4)?,
                duration_secs: row.get(5)?,
                date: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?
    };

    Ok(sessions)
}

pub fn log_sync(conn: &Connection, records_synced: i64, status: &str) -> Result<()> {
    let now = chrono::Local::now().to_rfc3339();
    conn.execute(
        "INSERT INTO sync_log (synced_at, records_synced, status) VALUES (?1, ?2, ?3)",
        params![now, records_synced, status],
    )?;
    Ok(())
}

// Ignored apps CRUD

pub fn get_ignored_apps(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT app_name FROM ignored_apps ORDER BY app_name ASC")?;
    let apps = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>>>()?;
    Ok(apps)
}

pub fn save_ignored_apps(conn: &Connection, apps: &[String]) -> Result<()> {
    conn.execute("DELETE FROM ignored_apps", [])?;
    let mut stmt = conn.prepare("INSERT INTO ignored_apps (app_name) VALUES (?1)")?;
    for app in apps {
        stmt.execute(params![app])?;
    }
    Ok(())
}

// Escalation settings CRUD

pub fn get_escalation_settings(conn: &Connection) -> Result<EscalationSettings> {
    conn.query_row(
        "SELECT green_end_hour, yellow_end_hour, sensitivity, enabled, paused_until,
                productive_multiplier, distracting_multiplier
         FROM escalation_settings WHERE id = 1",
        [],
        |row| {
            Ok(EscalationSettings {
                green_end_hour: row.get(0)?,
                yellow_end_hour: row.get(1)?,
                sensitivity: row.get(2)?,
                enabled: row.get::<_, i64>(3)? != 0,
                paused_until: row.get(4)?,
                productive_multiplier: row.get(5)?,
                distracting_multiplier: row.get(6)?,
            })
        },
    )
}

pub fn save_escalation_settings(conn: &Connection, settings: &EscalationSettings) -> Result<()> {
    conn.execute(
        "UPDATE escalation_settings SET green_end_hour = ?1, yellow_end_hour = ?2,
         sensitivity = ?3, enabled = ?4, paused_until = ?5,
         productive_multiplier = ?6, distracting_multiplier = ?7 WHERE id = 1",
        params![
            settings.green_end_hour,
            settings.yellow_end_hour,
            settings.sensitivity,
            settings.enabled as i64,
            settings.paused_until,
            settings.productive_multiplier,
            settings.distracting_multiplier,
        ],
    )?;
    Ok(())
}

// App category CRUD

pub fn get_app_categories(conn: &Connection) -> Result<Vec<AppCategoryEntry>> {
    let mut stmt = conn.prepare(
        "SELECT app_name, category, last_seen FROM app_categories
         WHERE last_seen IS NOT NULL
         ORDER BY CASE WHEN category = 'uncategorized' THEN 0 ELSE 1 END ASC,
                  category ASC, app_name ASC",
    )?;
    let entries = stmt
        .query_map([], |row| {
            Ok(AppCategoryEntry {
                app_name: row.get(0)?,
                category: row.get(1)?,
                last_seen: row.get(2)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(entries)
}

pub fn set_app_category(conn: &Connection, app_name: &str, category: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO app_categories (app_name, category, is_default)
         VALUES (?1, ?2, 0)
         ON CONFLICT(app_name) DO UPDATE SET category = ?2, is_default = 0",
        params![app_name.to_lowercase(), category],
    )?;
    Ok(())
}

pub fn get_all_app_categories_for_cache(conn: &Connection) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare("SELECT app_name, category FROM app_categories")?;
    let pairs = stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>>>()?;
    Ok(pairs)
}

pub fn get_title_keyword_rules(conn: &Connection) -> Result<Vec<TitleKeywordRule>> {
    let mut stmt = conn.prepare(
        "SELECT id, app_name, keyword, category FROM title_keyword_rules ORDER BY app_name ASC, keyword ASC",
    )?;
    let rules = stmt
        .query_map([], |row| {
            Ok(TitleKeywordRule {
                id: Some(row.get(0)?),
                app_name: row.get(1)?,
                keyword: row.get(2)?,
                category: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>>>()?;
    Ok(rules)
}

pub fn add_title_keyword_rule(
    conn: &Connection,
    app_name: &str,
    keyword: &str,
    category: &str,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO title_keyword_rules (app_name, keyword, category) VALUES (?1, ?2, ?3)",
        params![app_name.to_lowercase(), keyword.to_lowercase(), category],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn delete_title_keyword_rule(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM title_keyword_rules WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn get_uncategorized_count(conn: &Connection) -> Result<i64> {
    conn.query_row(
        "SELECT COUNT(*) FROM app_categories WHERE category = 'uncategorized' AND last_seen IS NOT NULL",
        [],
        |row| row.get(0),
    )
}
