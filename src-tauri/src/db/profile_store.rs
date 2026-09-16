use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::content::validate_content_locale;
use crate::domain::profile::{
    duplicate_name, validate_profile_name, CreateProfileDto, ProfileDto, ProfileId, ProfileStore,
    UpdateProfileDto,
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

    fn count(&self) -> Result<i64, AppError> {
        self.conn
            .query_row("SELECT COUNT(*) FROM profiles", [], |row| row.get(0))
            .map_err(AppError::from)
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
        let content_locale = match input.content_locale {
            Some(raw) => validate_content_locale(&raw)?,
            None => current.content_locale,
        };
        self.conn.execute(
            "UPDATE profiles SET name = ?1, ui_locale = ?2, content_locale = ?3, updated_at = ?4 WHERE id = ?5",
            params![name, ui_locale, content_locale, crate::db::now_stamp(), input.id],
        )?;
        self.get(&ProfileId(input.id))
    }

    fn duplicate(&self, id: &ProfileId) -> Result<ProfileDto, AppError> {
        let source = self.get(id)?;
        let current = self.selected()?;
        let name = duplicate_name(&source.name);
        let new_id = uuid::Uuid::new_v4().to_string();
        let stamp = crate::db::now_stamp();
        self.conn.execute(
            "INSERT INTO profiles (id, name, ui_locale, content_locale, media_provider, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)",
            params![
                new_id,
                name,
                source.ui_locale,
                source.content_locale,
                source.media_provider,
                stamp
            ],
        )?;
        self.conn.execute(
            "INSERT INTO settings (profile_id, key, value_json)
             SELECT ?1, key, value_json FROM settings WHERE profile_id = ?2",
            params![new_id, id.as_str()],
        )?;
        // The copy must not become last-used; restore whoever was selected.
        self.select(&current.id)?;
        self.get(&ProfileId(new_id))
    }

    fn delete(&self, id: &ProfileId) -> Result<ProfileDto, AppError> {
        let _existing = self.get(id)?;
        if self.count()? <= 1 {
            return Err(AppError::LastProfile);
        }
        let current = self.selected()?;
        let deleting_current = current.id.as_str() == id.as_str();
        self.conn
            .execute("DELETE FROM profiles WHERE id = ?1", params![id.as_str()])?;
        if deleting_current {
            self.selected()
        } else {
            Ok(current)
        }
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
    use crate::db::{
        configure_connection, migrate, SqliteSettingsStore, KEY_APPEARANCE, KEY_MEETING_SCHEDULE,
    };
    use crate::domain::profile::{ProfileStore, SettingDto, SettingKeyDto};
    use crate::domain::schedule::MeetingScheduleSetting;

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

    #[test]
    fn update_rejects_unknown_content_locale() {
        let conn = memory_store();
        let store = SqliteProfileStore::new(&conn);
        let created = store
            .create(CreateProfileDto {
                name: "Cong B".into(),
            })
            .expect("create");
        let err = store
            .update(UpdateProfileDto {
                id: created.id.0.clone(),
                name: None,
                ui_locale: None,
                content_locale: Some("ZZZ".into()),
            })
            .expect_err("unknown");
        assert!(matches!(err, AppError::UnknownContentLanguage));
    }

    #[test]
    fn update_accepts_seed_content_locale() {
        let conn = memory_store();
        let store = SqliteProfileStore::new(&conn);
        let created = store
            .create(CreateProfileDto {
                name: "Cong B".into(),
            })
            .expect("create");
        let updated = store
            .update(UpdateProfileDto {
                id: created.id.0.clone(),
                name: None,
                ui_locale: Some("es".into()),
                content_locale: Some("TG".into()),
            })
            .expect("update");
        assert_eq!(updated.ui_locale, "es");
        assert_eq!(updated.content_locale, "TG");
    }

    #[test]
    fn duplicate_copies_settings_and_keeps_source_selected() {
        let conn = memory_store();
        let store = SqliteProfileStore::new(&conn);
        let source = store.selected().expect("default");
        let settings = SqliteSettingsStore::new(&conn);
        settings
            .set(&SettingDto {
                profile_id: source.id.0.clone(),
                key: KEY_APPEARANCE.into(),
                value_json: r#"{"theme":"dark","accent":"teal","density":"comfortable"}"#.into(),
            })
            .expect("appearance");
        settings
            .set(&SettingDto {
                profile_id: source.id.0.clone(),
                key: KEY_MEETING_SCHEDULE.into(),
                value_json: serde_json::to_string(&MeetingScheduleSetting {
                    midweek_weekday: 3,
                    midweek_time: "19:30".into(),
                    weekend_weekday: 7,
                    weekend_time: "10:00".into(),
                })
                .expect("json"),
            })
            .expect("schedule");

        let copy = store.duplicate(&source.id).expect("duplicate");
        assert_ne!(copy.id.as_str(), source.id.as_str());
        assert_eq!(copy.name, "Default (copy)");
        assert_eq!(copy.ui_locale, source.ui_locale);
        assert_eq!(copy.content_locale, source.content_locale);

        let copied_appearance = settings
            .get(&SettingKeyDto {
                profile_id: copy.id.0.clone(),
                key: KEY_APPEARANCE.into(),
            })
            .expect("copied appearance");
        assert!(copied_appearance.value_json.contains("teal"));

        let copied_schedule = settings
            .get(&SettingKeyDto {
                profile_id: copy.id.0.clone(),
                key: KEY_MEETING_SCHEDULE.into(),
            })
            .expect("copied schedule");
        assert!(copied_schedule.value_json.contains("19:30"));

        let current = store.selected().expect("selected");
        assert_eq!(current.id.as_str(), source.id.as_str());
    }

    #[test]
    fn delete_last_profile_is_rejected() {
        let conn = memory_store();
        let store = SqliteProfileStore::new(&conn);
        let only = store.selected().expect("default");
        let err = store.delete(&only.id).expect_err("last");
        assert!(matches!(err, AppError::LastProfile));
        assert_eq!(store.list().expect("list").len(), 1);
    }

    #[test]
    fn delete_active_selects_remaining() {
        let conn = memory_store();
        let store = SqliteProfileStore::new(&conn);
        let default = store.selected().expect("default");
        let other = store
            .create(CreateProfileDto {
                name: "Cong B".into(),
            })
            .expect("create");
        store.select(&default.id).expect("select default");
        let remaining = store.delete(&default.id).expect("delete");
        assert_eq!(remaining.id.as_str(), other.id.as_str());
        assert_eq!(store.selected().expect("selected").id.as_str(), other.id.as_str());
    }
}
