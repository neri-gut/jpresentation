use rusqlite::Connection;

use crate::error::AppError;

const V1: &str = r#"
CREATE TABLE profiles (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    ui_locale TEXT NOT NULL DEFAULT 'en',
    content_locale TEXT NOT NULL DEFAULT 'E',
    media_provider TEXT NOT NULL DEFAULT 'jw-org',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE settings (
    profile_id TEXT NOT NULL,
    key TEXT NOT NULL,
    value_json TEXT NOT NULL,
    PRIMARY KEY (profile_id, key),
    FOREIGN KEY (profile_id) REFERENCES profiles(id)
);

CREATE TABLE meeting_programs (
    id TEXT PRIMARY KEY,
    profile_id TEXT NOT NULL,
    date TEXT NOT NULL,
    kind TEXT NOT NULL,
    payload_json TEXT NOT NULL,
    FOREIGN KEY (profile_id) REFERENCES profiles(id)
);
"#;

/// WAL + foreign keys. Called before applying versioned SQL.
pub fn configure_connection(conn: &Connection) -> Result<(), AppError> {
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.pragma_update(None, "foreign_keys", "ON")?;
    Ok(())
}

/// Applies pending schema versions. Version 1 is the cimentación tables.
pub fn migrate(conn: &Connection) -> Result<(), AppError> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at TEXT NOT NULL
        )",
        [],
    )?;

    let current: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current < 1 {
        conn.execute_batch(V1)?;
        conn.execute(
            "INSERT INTO schema_migrations (version, applied_at) VALUES (1, datetime('now'))",
            [],
        )?;
    }

    seed_default_profile(conn)?;
    Ok(())
}

fn seed_default_profile(conn: &Connection) -> Result<(), AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM profiles", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }
    let id = uuid::Uuid::new_v4().to_string();
    let stamp = crate::db::now_stamp();
    conn.execute(
        "INSERT INTO profiles (id, name, ui_locale, content_locale, media_provider, created_at, updated_at)
         VALUES (?1, 'Default', 'en', 'E', 'jw-org', ?2, ?2)",
        rusqlite::params![id, stamp],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn v1_creates_tables_and_english_default_profile() {
        let dir = tempdir().expect("tempdir");
        let path = dir.path().join("jpresentation.db");
        let conn = Connection::open(path).expect("open");
        configure_connection(&conn).expect("pragma");
        migrate(&conn).expect("migrate");

        let tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name IN ('profiles','settings','meeting_programs')",
                [],
                |row| row.get(0),
            )
            .expect("count tables");
        assert_eq!(tables, 3);

        let locale: String = conn
            .query_row("SELECT ui_locale FROM profiles LIMIT 1", [], |row| {
                row.get(0)
            })
            .expect("locale");
        assert_eq!(locale, "en");
    }
}
