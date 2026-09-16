use tauri::{AppHandle, State};

use crate::db::{SqliteProfileStore, SqliteSettingsStore};
use crate::domain::profile::{
    CreateProfileDto, ProfileDto, ProfileId, ProfileStore, SelectProfileDto, UpdateProfileDto,
};
use crate::error::AppErrorDto;
use crate::platform::DesktopSurface;
use crate::state::AppState;

/// Lists congregation profiles.
#[tauri::command]
pub fn profile_list(state: State<'_, AppState>) -> Result<Vec<ProfileDto>, AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    SqliteProfileStore::new(&conn)
        .list()
        .map_err(AppErrorDto::from)
}

/// Creates a profile. UI locale is `en` regardless of the operator's current language.
#[tauri::command]
pub fn profile_create(
    state: State<'_, AppState>,
    payload: CreateProfileDto,
) -> Result<ProfileDto, AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    SqliteProfileStore::new(&conn)
        .create(payload)
        .map_err(AppErrorDto::from)
}

/// Selects the active profile and reapplies its surface placement.
#[tauri::command]
pub fn profile_select(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: SelectProfileDto,
) -> Result<ProfileDto, AppErrorDto> {
    let (profile, surfaces) = {
        let conn = state.lock_db().map_err(AppErrorDto::from)?;
        let store = SqliteProfileStore::new(&conn);
        let profile = store
            .select(&ProfileId(payload.id))
            .map_err(AppErrorDto::from)?;
        let surfaces = SqliteSettingsStore::new(&conn)
            .surfaces(profile.id.as_str())
            .map_err(AppErrorDto::from)?;
        (profile, surfaces)
    };
    DesktopSurface::new(app)
        .ensure_surfaces(&surfaces)
        .map_err(AppErrorDto::from)?;
    Ok(profile)
}

/// Updates name and/or UI locale of a profile.
#[tauri::command]
pub fn profile_update(
    state: State<'_, AppState>,
    payload: UpdateProfileDto,
) -> Result<ProfileDto, AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    SqliteProfileStore::new(&conn)
        .update(payload)
        .map_err(AppErrorDto::from)
}
