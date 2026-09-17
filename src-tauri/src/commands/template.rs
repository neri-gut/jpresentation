use serde::{Deserialize, Serialize};

use tauri::State;

use super::week::WeekScopeDto;
use crate::db::{SqliteProfileStore, SqliteTemplateStore, SqliteWeekStore};
use crate::domain::profile::ProfileStore;
use crate::domain::template::{EventTemplate, TemplateKind};
use crate::domain::week::{today, MeetingPart, WeekWhich};
use crate::error::AppErrorDto;
use crate::state::AppState;
use crate::week_service::{
    apply_template_to_bundle, load_week_for_profile, parse_meeting_kind, persist_bundle,
    resolve_system_template, restore_from_cache, set_bundle_parts, WeekBundleDto,
};

/// Create or update a user template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveTemplateDto {
    pub id: Option<String>,
    pub name: String,
    pub kind: String,
    pub parts: Vec<MeetingPart>,
}

/// Delete a user template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateIdDto {
    pub id: String,
}

/// Apply a template to this/next week's midweek or weekend meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyTemplateDto {
    pub which: String,
    pub meeting: String,
    pub template_id: String,
}

/// Replace the part list of one meeting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetPartsDto {
    pub which: String,
    pub meeting: String,
    pub parts: Vec<MeetingPart>,
}

/// System templates plus this profile's user templates.
#[tauri::command]
pub fn template_list(state: State<'_, AppState>) -> Result<Vec<EventTemplate>, AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    SqliteTemplateStore::new(&conn)
        .list(profile.id.as_str(), &profile.content_locale)
        .map_err(AppErrorDto::from)
}

/// Creates or updates a user template. System ids are rejected.
#[tauri::command]
pub fn template_save(
    state: State<'_, AppState>,
    payload: SaveTemplateDto,
) -> Result<EventTemplate, AppErrorDto> {
    let kind = TemplateKind::parse(&payload.kind).map_err(AppErrorDto::from)?;
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    SqliteTemplateStore::new(&conn)
        .save(
            profile.id.as_str(),
            payload.id.as_deref(),
            &payload.name,
            kind,
            payload.parts,
        )
        .map_err(AppErrorDto::from)
}

/// Deletes a user template.
#[tauri::command]
pub fn template_delete(
    state: State<'_, AppState>,
    payload: TemplateIdDto,
) -> Result<(), AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    SqliteTemplateStore::new(&conn)
        .delete(profile.id.as_str(), &payload.id)
        .map_err(AppErrorDto::from)
}

/// Applies a system or user template to the open week.
#[tauri::command]
pub fn template_apply(
    state: State<'_, AppState>,
    payload: ApplyTemplateDto,
) -> Result<WeekBundleDto, AppErrorDto> {
    let which = WeekWhich::parse(&payload.which).map_err(AppErrorDto::from)?;
    let meeting = parse_meeting_kind(&payload.meeting).map_err(AppErrorDto::from)?;
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    let template = SqliteTemplateStore::new(&conn)
        .get(
            profile.id.as_str(),
            &payload.template_id,
            &profile.content_locale,
        )
        .map_err(AppErrorDto::from)?
        .or_else(|| resolve_system_template(&payload.template_id, &profile.content_locale))
        .ok_or_else(|| AppErrorDto::from(crate::error::AppError::NotFound))?;
    let mut bundle = load_week_for_profile(
        &SqliteWeekStore::new(&conn),
        profile.id.as_str(),
        &profile.content_locale,
        which,
        today(),
    )
    .map_err(AppErrorDto::from)?;
    apply_template_to_bundle(&mut bundle, meeting, &template);
    persist_bundle(&SqliteWeekStore::new(&conn), profile.id.as_str(), &bundle)
        .map_err(AppErrorDto::from)?;
    Ok(bundle)
}

/// Replaces the part rows of one meeting (add / modify / delete in the UI).
#[tauri::command]
pub fn week_set_parts(
    state: State<'_, AppState>,
    payload: SetPartsDto,
) -> Result<WeekBundleDto, AppErrorDto> {
    let which = WeekWhich::parse(&payload.which).map_err(AppErrorDto::from)?;
    let meeting = parse_meeting_kind(&payload.meeting).map_err(AppErrorDto::from)?;
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    let mut bundle = load_week_for_profile(
        &SqliteWeekStore::new(&conn),
        profile.id.as_str(),
        &profile.content_locale,
        which,
        today(),
    )
    .map_err(AppErrorDto::from)?;
    set_bundle_parts(&mut bundle, meeting, payload.parts).map_err(AppErrorDto::from)?;
    persist_bundle(&SqliteWeekStore::new(&conn), profile.id.as_str(), &bundle)
        .map_err(AppErrorDto::from)?;
    Ok(bundle)
}

/// Re-reads cached JWPUB files (or the system skeleton) without the catalog.
#[tauri::command]
pub fn week_restore(
    state: State<'_, AppState>,
    payload: WeekScopeDto,
) -> Result<WeekBundleDto, AppErrorDto> {
    let which = WeekWhich::parse(&payload.which).map_err(AppErrorDto::from)?;
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    let bundle = restore_from_cache(
        &state.media_root,
        &profile.content_locale,
        which,
        today(),
    )
    .map_err(AppErrorDto::from)?;
    persist_bundle(&SqliteWeekStore::new(&conn), profile.id.as_str(), &bundle)
        .map_err(AppErrorDto::from)?;
    Ok(bundle)
}
