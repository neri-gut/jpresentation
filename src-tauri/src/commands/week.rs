use std::sync::atomic::Ordering;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::catalog::JwCdnCatalog;
use crate::db::{SqliteProfileStore, SqliteWeekStore};
use crate::domain::profile::ProfileStore;
use crate::domain::week::{today, MediaKind, MediaStatus, WeekWhich};
use crate::error::AppErrorDto;
use crate::state::AppState;
use crate::week_service::{
    download_week_media, fetch_week, load_week_for_profile, persist_bundle, WeekBundleDto,
    WeekProgressDto, WEEK_PROGRESS,
};

/// `which`: `this` or `next`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekScopeDto {
    pub which: String,
}

/// Preview request for one tree item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekPreviewRequestDto {
    pub which: String,
    pub item_id: String,
}

/// PNG/JPEG bytes as a data URL payload. Never a CDN URL.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekPreviewDto {
    pub mime: String,
    pub data_base64: String,
}

fn parse_which(raw: &str) -> Result<WeekWhich, AppErrorDto> {
    WeekWhich::parse(raw).map_err(AppErrorDto::from)
}

/// Loads the last fetched week (empty shells if none).
#[tauri::command]
pub fn week_get(
    state: State<'_, AppState>,
    payload: WeekScopeDto,
) -> Result<WeekBundleDto, AppErrorDto> {
    let which = parse_which(&payload.which)?;
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    let store = SqliteWeekStore::new(&conn);
    load_week_for_profile(
        &store,
        profile.id.as_str(),
        &profile.content_locale,
        which,
        today(),
    )
    .map_err(AppErrorDto::from)
}

/// Downloads (if needed) and parses `mwb` + `w` for this or next week.
#[tauri::command]
pub fn week_fetch(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: WeekScopeDto,
) -> Result<WeekBundleDto, AppErrorDto> {
    if state
        .week_busy
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(AppErrorDto::from(crate::error::AppError::Busy));
    }
    state.week_cancel.store(false, Ordering::SeqCst);
    let result = (|| {
        let which = parse_which(&payload.which)?;
        let catalog = JwCdnCatalog::new().map_err(AppErrorDto::from)?;
        let (profile_id, locale) = {
            let conn = state.lock_db().map_err(AppErrorDto::from)?;
            let profile = SqliteProfileStore::new(&conn)
                .selected()
                .map_err(AppErrorDto::from)?;
            (profile.id.0.clone(), profile.content_locale.clone())
        };
        let mut emit = |dto: WeekProgressDto| {
            let _ = app.emit(WEEK_PROGRESS, &dto);
        };
        let bundle = fetch_week(
            &catalog,
            &state.media_root,
            &locale,
            which,
            today(),
            &mut emit,
        )
        .map_err(AppErrorDto::from)?;
        let conn = state.lock_db().map_err(AppErrorDto::from)?;
        persist_bundle(&SqliteWeekStore::new(&conn), &profile_id, &bundle)
            .map_err(AppErrorDto::from)?;
        Ok(bundle)
    })();
    state.week_busy.store(false, Ordering::SeqCst);
    result
}

/// Downloads pending catalog videos into `week/`. Does not fetch `sjjm`.
#[tauri::command]
pub fn week_download_media(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: WeekScopeDto,
) -> Result<WeekBundleDto, AppErrorDto> {
    if state
        .week_busy
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(AppErrorDto::from(crate::error::AppError::Busy));
    }
    state.week_cancel.store(false, Ordering::SeqCst);
    let result = (|| {
        let which = parse_which(&payload.which)?;
        let catalog = JwCdnCatalog::new().map_err(AppErrorDto::from)?;
        let (profile_id, mut bundle) = {
            let conn = state.lock_db().map_err(AppErrorDto::from)?;
            let profile = SqliteProfileStore::new(&conn)
                .selected()
                .map_err(AppErrorDto::from)?;
            let bundle = load_week_for_profile(
                &SqliteWeekStore::new(&conn),
                profile.id.as_str(),
                &profile.content_locale,
                which,
                today(),
            )
            .map_err(AppErrorDto::from)?;
            (profile.id.0.clone(), bundle)
        };
        let mut emit = |dto: WeekProgressDto| {
            let _ = app.emit(WEEK_PROGRESS, &dto);
        };
        let bundle = download_week_media(
            &catalog,
            &state.media_root,
            &mut bundle,
            &state.week_cancel,
            &mut emit,
        )
        .map_err(AppErrorDto::from)?;
        let conn = state.lock_db().map_err(AppErrorDto::from)?;
        persist_bundle(&SqliteWeekStore::new(&conn), &profile_id, &bundle)
            .map_err(AppErrorDto::from)?;
        Ok(bundle)
    })();
    state.week_busy.store(false, Ordering::SeqCst);
    result
}

/// Asks an in-flight `week_download_media` to stop after the current file.
#[tauri::command]
pub fn week_cancel(state: State<'_, AppState>) -> Result<(), AppErrorDto> {
    state.week_cancel.store(true, Ordering::SeqCst);
    Ok(())
}

/// Returns a data-URL payload for an extracted image. Not used for video.
#[tauri::command]
pub fn week_preview(
    state: State<'_, AppState>,
    payload: WeekPreviewRequestDto,
) -> Result<Option<WeekPreviewDto>, AppErrorDto> {
    let which = parse_which(&payload.which)?;
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    let store = SqliteWeekStore::new(&conn);
    let bundle = load_week_for_profile(
        &store,
        profile.id.as_str(),
        &profile.content_locale,
        which,
        today(),
    )
    .map_err(AppErrorDto::from)?;
    drop(conn);
    let item = bundle
        .midweek
        .parts
        .iter()
        .chain(bundle.weekend.parts.iter())
        .flat_map(|part| part.items.iter())
        .find(|item| item.id == payload.item_id);
    let Some(item) = item else {
        return Ok(None);
    };
    if item.media_kind != MediaKind::Image {
        return Ok(None);
    }
    if !matches!(
        item.status,
        MediaStatus::Embedded | MediaStatus::Ready
    ) {
        return Ok(None);
    }
    let Some(path) = item.cache_path.as_deref() else {
        return Ok(None);
    };
    let root = state.media_root.canonicalize().ok();
    let file = std::path::Path::new(path);
    if let (Ok(canon), Some(root)) = (file.canonicalize(), root.as_ref()) {
        if !canon.starts_with(root) {
            return Err(AppErrorDto::from(crate::error::AppError::Invariant(
                "preview path outside media root".into(),
            )));
        }
    }
    let bytes = std::fs::read(file).map_err(|e| {
        AppErrorDto::from(crate::error::AppError::Io(e.to_string()))
    })?;
    Ok(Some(WeekPreviewDto {
        mime: if item.mime.is_empty() {
            "image/jpeg".into()
        } else {
            item.mime.clone()
        },
        data_base64: encode_base64(&bytes),
    }))
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::new();
    let mut i = 0;
    while i < bytes.len() {
        let b0 = bytes[i];
        let b1 = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
        let b2 = if i + 2 < bytes.len() { bytes[i + 2] } else { 0 };
        out.push(TABLE[(b0 >> 2) as usize] as char);
        out.push(TABLE[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        if i + 1 < bytes.len() {
            out.push(TABLE[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char);
        } else {
            out.push('=');
        }
        if i + 2 < bytes.len() {
            out.push(TABLE[(b2 & 0x3f) as usize] as char);
        } else {
            out.push('=');
        }
        i += 3;
    }
    out
}
