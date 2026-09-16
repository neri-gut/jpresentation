use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::profile::{
    validate_profile_name, CreateProfileDto, ProfileDto, ProfileId, ProfileStore, UpdateProfileDto,
};
use crate::error::AppError;

/// rusqlite implementation of [`ProfileStore`].
pub struct SqliteProfileStore<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteProfileStore<'a> {
    /// Borrow an open connection. The caller holds the `AppState` mutex.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    fn row_to_profile(row: &rusqlite::Row<'_>) -> rusqlite::Result<ProfileDto> {
        Ok(ProfileDto {
            id: ProfileId(row.get(0)?),
            name: row.get(1)?,
            ui_locale: row.get(2)?,
            content_locale: row.get(3)?,
            media_provider: row.get(4)?,
            created_at: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }
}

impl ProfileStore for SqliteProfileStore<'_> {
    fn list(&self) -> Result<Vec<ProfileDto>, AppError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, ui_locale, content_locale, media_provider, created_at, updated_at
             FROM profiles ORDER BY name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], Self::row_to_profile)?;
        let mut out = Vec::new();
        for row in rows {
            out.push(row?);
        }
        Ok(out)
    }

    fn create(&self, input: CreateProfileDto) -> Result<ProfileDto, AppError> {
        let name = validate_profile_name(&input.name)?;
        let id = uuid::Uuid::new_v4().to_string();
        let stamp = crate::db::now_stamp();
        self.conn.execute(
            "INSERT INTO profiles (id, name, ui_locale, content_locale, media_provider, created_at, updated_at)
             VALUES (?1, ?2, 'en', 'E', 'jw-org', ?3, ?3)",
            params![id, name, stamp],
        )?;
        self.get(&ProfileId(id))
    }

    fn select(&self, id: &ProfileId) -> Result<ProfileDto, AppError> {
        let n = self.conn.execute(
            "UPDATE profiles SET updated_at = ?1 WHERE id = ?2",
            params![crate::db::now_stamp(), id.as_str()],
        )?;
        if n == 0 {
            return Err(AppError::NotFound);
        }
        self.get(id)
    }

    fn selected(&self) -> Result<ProfileDto, AppError> {
        self.conn
            .query_row(
                "SELECT id, name, ui_locale, content_locale, media_provider, created_at, updated_at
                 FROM profiles ORDER BY updated_at DESC, created_at DESC LIMIT 1",
                [],
                Self::row_to_profile,
            )
            .optional()?
            .ok_or(AppError::NotFound)
    }

    fn update(&self, input: UpdateProfileDto) -> Result<ProfileDto, AppError> {
        let current = self.get(&ProfileId(input.id.clone()))?;
        let name = match input.name {
            Some(raw) => validate_profile_name(&raw)?,
            None => current.name,
        };
        let ui_locale = match input.ui_locale {
            Some(raw) => {
                let trimmed = raw.trim();
                if trimmed.is_empty() {
                    return Err(AppError::Invariant("ui_locale is required".into()));
                }
                trimmed.to_string()
            }
            None => current.ui_locale,
        };
        self.conn.execute(
            "UPDATE profiles SET name = ?1, ui_locale = ?2, updated_at = ?3 WHERE id = ?4",
            params![name, ui_locale, crate::db::now_stamp(), input.id],
        )?;
        self.get(&ProfileId(input.id))
    }

    fn get(&self, id: &ProfileId) -> Result<ProfileDto, AppError> {
        self.conn
            .query_row(
                "SELECT id, name, ui_locale, content_locale, media_provider, created_at, updated_at
                 FROM profiles WHERE id = ?1",
                params![id.as_str()],
                Self::row_to_profile,
            )
            .optional()?
            .ok_or(AppError::NotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{configure_connection, migrate};
    use crate::domain::profile::ProfileStore;

    fn memory_store() -> Connection {
        let conn = Connection::open_in_memory().expect("memory");
        configure_connection(&conn).expect("pragma");
        migrate(&conn).expect("migrate");
        conn
    }

    #[test]
    fn new_profile_defaults_ui_locale_to_en() {
        let conn = memory_store();
        let store = SqliteProfileStore::new(&conn);
        let created = store
            .create(CreateProfileDto {
                name: "Cong B".into(),
            })
            .expect("create");
        assert_eq!(created.ui_locale, "en");
        assert_eq!(created.content_locale, "E");
        let list = store.list().expect("list");
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn select_makes_profile_current() {
        let conn = memory_store();
        let store = SqliteProfileStore::new(&conn);
        let second = store
            .create(CreateProfileDto {
                name: "Cong B".into(),
            })
            .expect("create");
        let selected = store.select(&second.id).expect("select");
        assert_eq!(selected.id.as_str(), second.id.as_str());
        let current = store.selected().expect("selected");
        assert_eq!(current.id.as_str(), second.id.as_str());
    }
}
