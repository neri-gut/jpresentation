use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, State};

use crate::catalog::md5_hex;
use crate::db::{SqliteProfileStore, SqliteSettingsStore, KEY_EXPLORER};
use crate::domain::explorer::{
    classify_file, default_local_roots, list_dir, mime_for, parent_string, path_allowed,
    stage_file_name, ExplorerEntryDto, ExplorerKind, ExplorerListDto, ExplorerSetting,
};
use crate::domain::jwpub::extract_jwpub_media;
use crate::domain::output::{StageKind, OUTPUT_CHANGED};
use crate::domain::profile::ProfileStore;
use crate::domain::week::{monday_for, today, week_dir, WeekWhich};
use crate::error::{AppError, AppErrorDto};
use crate::state::AppState;

/// Path to list. Empty = virtual roots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerPathDto {
    pub path: String,
}

/// Preview of a local image.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePreviewDto {
    pub mime: String,
    pub data_base64: String,
}

fn profile_and_roots(
    state: &AppState,
) -> Result<(String, String, ExplorerSetting), AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let profile = SqliteProfileStore::new(&conn)
        .selected()
        .map_err(AppErrorDto::from)?;
    let explorer = SqliteSettingsStore::new(&conn)
        .explorer(profile.id.as_str())
        .map_err(AppErrorDto::from)?;
    Ok((
        profile.id.0.clone(),
        profile.content_locale.clone(),
        explorer,
    ))
}

fn ensure_allowed(path: &Path, state: &AppState, roots: &[String]) -> Result<(), AppErrorDto> {
    if path_allowed(path, &state.media_root, roots) {
        Ok(())
    } else {
        Err(AppErrorDto::from(AppError::Invariant(
            "path is outside explorer roots".into(),
        )))
    }
}

/// Lists virtual roots or one directory.
#[tauri::command]
pub fn explorer_list(
    state: State<'_, AppState>,
    payload: ExplorerPathDto,
) -> Result<ExplorerListDto, AppErrorDto> {
    let (_id, locale, explorer) = profile_and_roots(&state)?;
    if payload.path.is_empty() {
        return Ok(virtual_roots(&state, &locale, &explorer));
    }
    let path = PathBuf::from(&payload.path);
    ensure_allowed(&path, &state, &explorer.roots)?;
    let entries = list_dir(&path).map_err(AppErrorDto::from)?;
    Ok(ExplorerListDto {
        path: payload.path,
        parent: parent_string(&path),
        entries,
    })
}

fn virtual_roots(
    state: &AppState,
    locale: &str,
    explorer: &ExplorerSetting,
) -> ExplorerListDto {
    let today = today();
    let mut entries = Vec::new();
    for (which, name) in [
        (WeekWhich::This, "This week"),
        (WeekWhich::Next, "Next week"),
    ] {
        let monday = monday_for(which, today);
        let dir = week_dir(&state.media_root, locale, monday);
        let _ = std::fs::create_dir_all(&dir);
        entries.push(ExplorerEntryDto {
            name: name.into(),
            path: dir.to_string_lossy().into_owned(),
            kind: ExplorerKind::Dir,
        });
    }
    for entry in default_local_roots() {
        entries.push(entry);
    }
    for root in &explorer.roots {
        if entries.iter().any(|e| e.path == *root) {
            continue;
        }
        let path = PathBuf::from(root);
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| root.clone());
        entries.push(ExplorerEntryDto {
            name,
            path: root.clone(),
            kind: ExplorerKind::Dir,
        });
    }
    ExplorerListDto {
        path: String::new(),
        parent: None,
        entries,
    }
}

/// Adds an absolute folder to the profile explorer roots.
#[tauri::command]
pub fn explorer_add_root(
    state: State<'_, AppState>,
    payload: ExplorerPathDto,
) -> Result<ExplorerSetting, AppErrorDto> {
    let path = PathBuf::from(payload.path.trim());
    if !path.is_absolute() || !path.is_dir() {
        return Err(AppErrorDto::from(AppError::Invariant(
            "folder does not exist".into(),
        )));
    }
    let canon = path
        .canonicalize()
        .map_err(|e| AppErrorDto::from(AppError::Io(e.to_string())))?;
    let (profile_id, _locale, mut explorer) = profile_and_roots(&state)?;
    let as_str = canon.to_string_lossy().into_owned();
    if !explorer.roots.iter().any(|r| r == &as_str) {
        explorer.roots.push(as_str);
    }
    crate::domain::explorer::validate_explorer(&explorer).map_err(AppErrorDto::from)?;
    persist_explorer(&state, &profile_id, &explorer)?;
    Ok(explorer)
}

/// Drops a saved root. Does not delete files.
#[tauri::command]
pub fn explorer_remove_root(
    state: State<'_, AppState>,
    payload: ExplorerPathDto,
) -> Result<ExplorerSetting, AppErrorDto> {
    let (profile_id, _locale, mut explorer) = profile_and_roots(&state)?;
    explorer.roots.retain(|r| r != &payload.path);
    persist_explorer(&state, &profile_id, &explorer)?;
    Ok(explorer)
}

fn persist_explorer(
    state: &AppState,
    profile_id: &str,
    explorer: &ExplorerSetting,
) -> Result<(), AppErrorDto> {
    let conn = state.lock_db().map_err(AppErrorDto::from)?;
    let value_json =
        serde_json::to_string(explorer).map_err(|e| AppErrorDto::from(AppError::Invariant(e.to_string())))?;
    SqliteSettingsStore::new(&conn)
        .set(&crate::domain::profile::SettingDto {
            profile_id: profile_id.to_string(),
            key: KEY_EXPLORER.into(),
            value_json,
        })
        .map_err(AppErrorDto::from)?;
    Ok(())
}

/// Extracts image/video files from a `.jwpub` into `local-pub/{hash}/`.
#[tauri::command]
pub fn explorer_open_jwpub(
    state: State<'_, AppState>,
    payload: ExplorerPathDto,
) -> Result<ExplorerListDto, AppErrorDto> {
    let (_id, _locale, explorer) = profile_and_roots(&state)?;
    let path = PathBuf::from(&payload.path);
    ensure_allowed(&path, &state, &explorer.roots)?;
    let bytes = std::fs::read(&path)
        .map_err(|e| AppErrorDto::from(AppError::Io(e.to_string())))?;
    let hash = md5_hex(&bytes);
    let dest = state.media_root.join("local-pub").join(&hash);
    extract_jwpub_media(&bytes, &dest).map_err(AppErrorDto::from)?;
    let entries = list_dir(&dest).map_err(AppErrorDto::from)?;
    Ok(ExplorerListDto {
        path: dest.to_string_lossy().into_owned(),
        parent: Some(String::new()),
        entries,
    })
}

/// Image preview as base64. Videos return none.
#[tauri::command]
pub fn explorer_preview(
    state: State<'_, AppState>,
    payload: ExplorerPathDto,
) -> Result<Option<FilePreviewDto>, AppErrorDto> {
    let (_id, _locale, explorer) = profile_and_roots(&state)?;
    let path = PathBuf::from(&payload.path);
    ensure_allowed(&path, &state, &explorer.roots)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let Some(ExplorerKind::Image) = classify_file(&name) else {
        return Ok(None);
    };
    let bytes = std::fs::read(&path)
        .map_err(|e| AppErrorDto::from(AppError::Io(e.to_string())))?;
    Ok(Some(FilePreviewDto {
        mime: mime_for(ExplorerKind::Image, &name).into(),
        data_base64: encode_base64(&bytes),
    }))
}

/// Copies the file into `media/stage/` and publishes the snapshot.
#[tauri::command]
pub fn stage_open(
    app: AppHandle,
    state: State<'_, AppState>,
    payload: ExplorerPathDto,
) -> Result<crate::domain::output::StageSnapshot, AppErrorDto> {
    let (_id, _locale, explorer) = profile_and_roots(&state)?;
    let path = PathBuf::from(&payload.path);
    ensure_allowed(&path, &state, &explorer.roots)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "media".into());
    let kind = classify_file(&name).ok_or_else(|| {
        AppErrorDto::from(AppError::Invariant("not a stageable file".into()))
    })?;
    let stage_kind = match kind {
        ExplorerKind::Image => StageKind::Image,
        ExplorerKind::Video => StageKind::Video,
        ExplorerKind::Jwpub | ExplorerKind::Dir => {
            return Err(AppErrorDto::from(AppError::Invariant(
                "open the package first, then pick a file".into(),
            )));
        }
    };
    let ext = name.rsplit('.').next().unwrap_or("bin");
    let dest_dir = state.media_root.join("stage");
    std::fs::create_dir_all(&dest_dir)
        .map_err(|e| AppErrorDto::from(AppError::Io(e.to_string())))?;
    let next_rev = {
        let output = state.lock_output().map_err(AppErrorDto::from)?;
        output.stage.rev.saturating_add(1)
    };
    let dest = dest_dir.join(stage_file_name(next_rev, ext));
    std::fs::copy(&path, &dest)
        .map_err(|e| AppErrorDto::from(AppError::Io(e.to_string())))?;
    let mime = mime_for(kind, &name);
    let snap = {
        let mut output = state.lock_output().map_err(AppErrorDto::from)?;
        output.open_media(
            stage_kind,
            name,
            mime.to_string(),
            dest.to_string_lossy().into_owned(),
        )
    };
    let _ = app.emit(OUTPUT_CHANGED, &snap);
    Ok(snap)
}

/// Clears the stage. Does not stop the clock.
#[tauri::command]
pub fn stage_close(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<crate::domain::output::StageSnapshot, AppErrorDto> {
    let snap = {
        let mut output = state.lock_output().map_err(AppErrorDto::from)?;
        output.close_media()
    };
    let _ = app.emit(OUTPUT_CHANGED, &snap);
    let stage_dir = state.media_root.join("stage");
    if stage_dir.is_dir() {
        let _ = std::fs::remove_dir_all(&stage_dir);
        let _ = std::fs::create_dir_all(&stage_dir);
    }
    Ok(snap)
}

/// Native folder picker; on confirm, adds the folder as a profile root.
#[tauri::command]
pub fn explorer_pick_root(state: State<'_, AppState>) -> Result<ExplorerSetting, AppErrorDto> {
    let picked = rfd::FileDialog::new()
        .set_title("Folder")
        .pick_folder();
    let Some(path) = picked else {
        let (_id, _locale, explorer) = profile_and_roots(&state)?;
        return Ok(explorer);
    };
    explorer_add_root(
        state,
        ExplorerPathDto {
            path: path.to_string_lossy().into_owned(),
        },
    )
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


