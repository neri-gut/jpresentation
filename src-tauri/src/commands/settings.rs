use tauri::{AppHandle, State};

use crate::db::{SqliteSettingsStore, KEY_SURFACES};
use crate::domain::platform::SurfacesSetting;
use crate::domain::profile::{SettingDto, SettingKeyDto};
use crate::error::{AppError, AppErrorDto};
use crate::platform::DesktopSurface;
use crate::state::AppState;

/// Reads a settings cell, filling a typed default for known keys.
#[tauri::command]
pub fn settings_get(
    state: State<'_, AppState>,
    payload: SettingKeyDto,
) -> Result<SettingDto, AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    SqliteSettingsStore::new(&conn)
        .get(&payload)
        .map_err(AppErrorDto::from)
}

/// Writes a settings cell. Surface keys also move the audience/speaker windows.
#[tauri::command]
pub fn settings_set(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: SettingDto,
) -> Result<SettingDto, AppErrorDto> {
    let saved = {
        let conn = state.lock_db().map_err(AppErrorDto::from)?;
        SqliteSettingsStore::new(&conn)
            .set(&payload)
            .map_err(AppErrorDto::from)?
    };
    if saved.key == KEY_SURFACES {
        let surfaces: SurfacesSetting =
            serde_json::from_str(&saved.value_json).map_err(|e| AppErrorDto {
                code: "Invariant".into(),
                message: e.to_string(),
                rev: 0,
            })?;
        let missing = DesktopSurface::new(app)
            .ensure_surfaces(&surfaces)
            .map_err(AppErrorDto::from)?;
        if missing {
            return Err(AppErrorDto::from(AppError::MonitorMissing));
        }
    }
    Ok(saved)
}
