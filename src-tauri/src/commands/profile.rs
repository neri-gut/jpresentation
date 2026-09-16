use tauri::{AppHandle, State};

use crate::db::{SqliteProfileStore, SqliteSettingsStore};
use crate::domain::content::content_languages;
use crate::domain::content::ContentLanguageDto;
use crate::domain::profile::{
    CreateProfileDto, DeleteProfileDto, DuplicateProfileDto, ProfileDto, ProfileId, ProfileStore,
    SelectProfileDto, UpdateProfileDto,
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
    let _missing = DesktopSurface::new(app)
        .ensure_surfaces(&surfaces)
        .map_err(AppErrorDto::from)?;
    Ok(profile)
}

/// Updates name, UI locale, and/or content locale of a profile.
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

/// Copies a profile and its settings. Does not select the copy.
#[tauri::command]
pub fn profile_duplicate(
    state: State<'_, AppState>,
    payload: DuplicateProfileDto,
) -> Result<ProfileDto, AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    SqliteProfileStore::new(&conn)
        .duplicate(&ProfileId(payload.id))
        .map_err(AppErrorDto::from)
}

/// Deletes a profile and returns the profile that remains selected.
#[tauri::command]
pub fn profile_delete(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: DeleteProfileDto,
) -> Result<ProfileDto, AppErrorDto> {
    let (remaining, surfaces) = {
        let conn = state.lock_db().map_err(AppErrorDto::from)?;
        let remaining = SqliteProfileStore::new(&conn)
            .delete(&ProfileId(payload.id))
            .map_err(AppErrorDto::from)?;
        let surfaces = SqliteSettingsStore::new(&conn)
            .surfaces(remaining.id.as_str())
            .map_err(AppErrorDto::from)?;
        (remaining, surfaces)
    };
    let _missing = DesktopSurface::new(app)
        .ensure_surfaces(&surfaces)
        .map_err(AppErrorDto::from)?;
    Ok(remaining)
}

/// Embedded JW content-language seed. No network.
#[tauri::command]
pub fn content_languages_list() -> Result<Vec<ContentLanguageDto>, AppErrorDto> {
    content_languages().map_err(AppErrorDto::from)
}
