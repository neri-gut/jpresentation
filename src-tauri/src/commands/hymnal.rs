//! Hymnal commands for listing, downloading, and playing `sjjm` meeting songs.

use std::sync::atomic::Ordering;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::catalog::JwCdnCatalog;
use crate::db::SqliteProfileStore;
use crate::domain::hymnal::{
    HymnalBundleDto, HymnalProgressDto, HymnalSlotsDto, HymnalSongDto, HYMNAL_PROGRESS,
};
use crate::domain::output::{StageSnapshot, OUTPUT_CHANGED};
use crate::domain::profile::ProfileStore;
use crate::error::{AppError, AppErrorDto};
use crate::hymnal_service::{
    download_all_songs, download_single_song, load_or_fetch_songs, play_song_on_stage,
};
use crate::state::AppState;

/// Request payload to download or play a specific song track.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HymnalTrackRequestDto {
    pub track: u32,
}

/// Request payload to update song slots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HymnalSetSlotsRequestDto {
    pub start: Option<u32>,
    pub middle: Option<u32>,
    pub end: Option<u32>,
}

/// Returns the song list and slots for the active profile's content language.
#[tauri::command]
pub fn hymnal_get(state: State<'_, AppState>) -> Result<HymnalBundleDto, AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    drop(conn);

    let catalog = JwCdnCatalog::new().map_err(AppErrorDto::from)?;
    let songs = load_or_fetch_songs(
        &catalog,
        &state.media_root,
        &profile.content_locale,
        false,
    )
    .map_err(AppErrorDto::from)?;

    Ok(HymnalBundleDto {
        langwritten: profile.content_locale,
        songs,
        slots: HymnalSlotsDto::default(),
    })
}

/// Forces a fresh catalog fetch from the CDN and updates local cache.
#[tauri::command]
pub fn hymnal_refresh(state: State<'_, AppState>) -> Result<HymnalBundleDto, AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    drop(conn);

    let catalog = JwCdnCatalog::new().map_err(AppErrorDto::from)?;
    let songs = load_or_fetch_songs(
        &catalog,
        &state.media_root,
        &profile.content_locale,
        true,
    )
    .map_err(AppErrorDto::from)?;

    Ok(HymnalBundleDto {
        langwritten: profile.content_locale,
        songs,
        slots: HymnalSlotsDto::default(),
    })
}

/// Downloads a single hymnal track on demand.
#[tauri::command]
pub fn hymnal_download_song(
    payload: HymnalTrackRequestDto,
    state: State<'_, AppState>,
) -> Result<HymnalSongDto, AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    drop(conn);

    let catalog = JwCdnCatalog::new().map_err(AppErrorDto::from)?;
    download_single_song(
        &catalog,
        &state.media_root,
        &profile.content_locale,
        payload.track,
    )
    .map_err(AppErrorDto::from)
}

/// Downloads all missing hymnal songs in background with progress events.
#[tauri::command]
pub fn hymnal_download_all(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<HymnalSongDto>, AppErrorDto> {
    if state
        .hymnal_busy
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        return Err(AppErrorDto::from(AppError::Busy));
    }
    state.hymnal_cancel.store(false, Ordering::SeqCst);

    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    drop(conn);

    let catalog = match JwCdnCatalog::new() {
        Ok(c) => c,
        Err(err) => {
            state.hymnal_busy.store(false, Ordering::SeqCst);
            return Err(AppErrorDto::from(err));
        }
    };

    let result = download_all_songs(
        &catalog,
        &state.media_root,
        &profile.content_locale,
        &state.hymnal_cancel,
        &mut |progress: HymnalProgressDto| {
            let _ = app.emit(HYMNAL_PROGRESS, &progress);
        },
    );

    state.hymnal_busy.store(false, Ordering::SeqCst);
    result.map_err(AppErrorDto::from)
}

/// Cancels an in-flight bulk hymnal download.
#[tauri::command]
pub fn hymnal_cancel(state: State<'_, AppState>) -> Result<(), AppErrorDto> {
    state.hymnal_cancel.store(true, Ordering::SeqCst);
    Ok(())
}

/// Plays a song track on stage (Auditorio with video & audio, Speaker in mirror/HUD).
#[tauri::command]
pub fn hymnal_play(
    payload: HymnalTrackRequestDto,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<StageSnapshot, AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    drop(conn);

    let catalog = JwCdnCatalog::new().map_err(AppErrorDto::from)?;
    let mut output = state.lock_output().map_err(AppErrorDto::from)?;

    let snap = play_song_on_stage(
        &catalog,
        &state.media_root,
        &profile.content_locale,
        payload.track,
        &mut output,
    )
    .map_err(AppErrorDto::from)?;

    let _ = app.emit(OUTPUT_CHANGED, &snap);
    Ok(snap)
}
