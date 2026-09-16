use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::week::{MeetingKind, MeetingWeek};
use crate::error::AppError;

/// SQLite persistence for normalised `MeetingWeek` rows.
pub struct SqliteWeekStore<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteWeekStore<'a> {
    /// Borrow the process connection.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Inserts or replaces the week for profile + Monday + kind.
    pub fn upsert(&self, profile_id: &str, week: &MeetingWeek) -> Result<(), AppError> {
        let kind = kind_sql(week.kind);
        let payload = serde_json::to_string(week)
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        if let Some(id) = self.id_of(profile_id, &week.monday, kind)? {
            self.conn.execute(
                "UPDATE meeting_programs SET payload_json = ?1 WHERE id = ?2",
                params![payload, id],
            )?;
        } else {
            let id = uuid::Uuid::new_v4().to_string();
            self.conn.execute(
                "INSERT INTO meeting_programs (id, profile_id, date, kind, payload_json)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![id, profile_id, week.monday, kind, payload],
            )?;
        }
        Ok(())
    }

    /// Loads a stored week, if any.
    pub fn get(
        &self,
        profile_id: &str,
        monday: &str,
        kind: MeetingKind,
    ) -> Result<Option<MeetingWeek>, AppError> {
        let payload: Option<String> = self
            .conn
            .query_row(
                "SELECT payload_json FROM meeting_programs
                 WHERE profile_id = ?1 AND date = ?2 AND kind = ?3",
                params![profile_id, monday, kind_sql(kind)],
                |row| row.get(0),
            )
            .optional()?;
        match payload {
            Some(json) => Ok(Some(
                serde_json::from_str(&json).map_err(|e| AppError::Invariant(e.to_string()))?,
            )),
            None => Ok(None),
        }
    }

    fn id_of(&self, profile_id: &str, monday: &str, kind: &str) -> Result<Option<String>, AppError> {
        self.conn
            .query_row(
                "SELECT id FROM meeting_programs WHERE profile_id = ?1 AND date = ?2 AND kind = ?3",
                params![profile_id, monday, kind],
                |row| row.get(0),
            )
            .optional()
            .map_err(AppError::from)
    }
}

fn kind_sql(kind: MeetingKind) -> &'static str {
    match kind {
        MeetingKind::Midweek => "midweek",
        MeetingKind::Weekend => "weekend",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{configure_connection, migrate, SqliteProfileStore};
    use crate::domain::profile::{CreateProfileDto, ProfileStore};
    use crate::domain::week::{CivilDate, MeetingKind, MeetingWeek};

    fn memory() -> (Connection, String) {
        let conn = Connection::open_in_memory().expect("memory");
        configure_connection(&conn).expect("pragma");
        migrate(&conn).expect("migrate");
        let profile = SqliteProfileStore::new(&conn)
            .create(CreateProfileDto {
                name: "Cong".into(),
            })
            .expect("create");
        (conn, profile.id.0)
    }

    #[test]
    fn upsert_roundtrip() {
        let (conn, profile_id) = memory();
        let store = SqliteWeekStore::new(&conn);
        let monday = CivilDate {
            year: 2026,
            month: 9,
            day: 7,
        };
        let mut week = MeetingWeek::empty(monday, MeetingKind::Midweek, "S");
        week.title = "Tesoro".into();
        store.upsert(&profile_id, &week).expect("save");
        week.title = "Updated".into();
        store.upsert(&profile_id, &week).expect("update");
        let loaded = store
            .get(&profile_id, "2026-09-07", MeetingKind::Midweek)
            .expect("get")
            .expect("row");
        assert_eq!(loaded.title, "Updated");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM meeting_programs WHERE profile_id = ?1",
                rusqlite::params![profile_id],
                |row| row.get(0),
            )
            .expect("count");
        assert_eq!(count, 1);
    }
}
