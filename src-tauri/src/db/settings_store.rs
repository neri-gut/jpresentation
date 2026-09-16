use rusqlite::{params, Connection, OptionalExtension};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::domain::platform::SurfacesSetting;
use crate::domain::profile::{AppearanceSetting, PanelSetting, SettingDto, SettingKeyDto};
use crate::error::AppError;

/// Known settings keys. Values are named DTOs encoded as JSON text.
pub const KEY_APPEARANCE: &str = "appearance";
pub const KEY_SURFACES: &str = "surfaces";
pub const KEY_PANEL: &str = "panel";

/// rusqlite settings cells keyed by profile + name.
pub struct SqliteSettingsStore<'a> {
    conn: &'a Connection,
}

impl<'a> SqliteSettingsStore<'a> {
    /// Borrow an open connection. The caller holds the `AppState` mutex.
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Reads a cell or the default for a known key.
    pub fn get(&self, key: &SettingKeyDto) -> Result<SettingDto, AppError> {
        if let Some(value_json) = self
            .conn
            .query_row(
                "SELECT value_json FROM settings WHERE profile_id = ?1 AND key = ?2",
                params![key.profile_id, key.key],
                |row| row.get::<_, String>(0),
            )
            .optional()?
        {
            return Ok(SettingDto {
                profile_id: key.profile_id.clone(),
                key: key.key.clone(),
                value_json,
            });
        }
        Ok(SettingDto {
            profile_id: key.profile_id.clone(),
            key: key.key.clone(),
            value_json: default_json(&key.key)?,
        })
    }

    /// Upserts a cell after validating the JSON against the known DTO for that key.
    pub fn set(&self, value: &SettingDto) -> Result<SettingDto, AppError> {
        validate_value(&value.key, &value.value_json)?;
        self.conn.execute(
            "INSERT INTO settings (profile_id, key, value_json) VALUES (?1, ?2, ?3)
             ON CONFLICT(profile_id, key) DO UPDATE SET value_json = excluded.value_json",
            params![value.profile_id, value.key, value.value_json],
        )?;
        Ok(value.clone())
    }

    /// Typed helper used by `PlatformSurface` when placing windows.
    pub fn surfaces(&self, profile_id: &str) -> Result<SurfacesSetting, AppError> {
        let cell = self.get(&SettingKeyDto {
            profile_id: profile_id.to_string(),
            key: KEY_SURFACES.into(),
        })?;
        parse_json(&cell.value_json)
    }
}

fn default_json(key: &str) -> Result<String, AppError> {
    match key {
        KEY_APPEARANCE => encode(&AppearanceSetting::default()),
        KEY_SURFACES => encode(&SurfacesSetting::default()),
        KEY_PANEL => encode(&PanelSetting::default()),
        _ => Err(AppError::Invariant(format!("unknown setting key: {key}"))),
    }
}

fn validate_value(key: &str, value_json: &str) -> Result<(), AppError> {
    match key {
        KEY_APPEARANCE => parse_json::<AppearanceSetting>(value_json).map(|_| ()),
        KEY_SURFACES => parse_json::<SurfacesSetting>(value_json).map(|_| ()),
        KEY_PANEL => parse_json::<PanelSetting>(value_json).map(|_| ()),
        _ => Err(AppError::Invariant(format!("unknown setting key: {key}"))),
    }
}

fn parse_json<T: DeserializeOwned>(value_json: &str) -> Result<T, AppError> {
    serde_json::from_str(value_json).map_err(|e| AppError::Invariant(e.to_string()))
}

fn encode<T: Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_string(value).map_err(|e| AppError::Invariant(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{configure_connection, migrate, SqliteProfileStore};
    use crate::domain::profile::{CreateProfileDto, ProfileStore};

    #[test]
    fn missing_appearance_returns_system_compact() {
        let conn = Connection::open_in_memory().expect("memory");
        configure_connection(&conn).expect("pragma");
        migrate(&conn).expect("migrate");
        let profiles = SqliteProfileStore::new(&conn);
        let profile = profiles
            .create(CreateProfileDto {
                name: "Cong A".into(),
            })
            .expect("create");
        let settings = SqliteSettingsStore::new(&conn);
        let cell = settings
            .get(&SettingKeyDto {
                profile_id: profile.id.0,
                key: KEY_APPEARANCE.into(),
            })
            .expect("get");
        let appearance: AppearanceSetting = parse_json(&cell.value_json).expect("json");
        assert_eq!(appearance.theme, "system");
        assert_eq!(appearance.density, "compact");
    }
}
