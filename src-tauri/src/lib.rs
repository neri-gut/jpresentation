mod catalog;
mod commands;
mod db;
pub mod domain;
mod error;
mod jobs;
mod platform;
mod state;
mod week_service;

use tauri::Emitter;
use tauri::Manager;

use std::thread;
use std::time::Duration;

use commands::{
    clock_arm, clock_finish, clock_pause, clock_start, content_languages_list, explorer_add_root,
    explorer_list, explorer_open_jwpub, explorer_pick_root, explorer_preview, explorer_remove_root,
    monitors_identify, monitors_list, output_get, profile_create, profile_delete, profile_duplicate,
    profile_list, profile_select, profile_update, settings_get, settings_set, stage_close,
    stage_open, template_apply, template_delete, template_list, template_save, week_cancel,
    week_download_media, week_fetch, week_get, week_preview, week_restore, week_set_parts,
};
use db::SqliteSettingsStore;
use domain::output::{OUTPUT_CHANGED, SPEAKER_UI_CHANGED, TIMER_CHANGED};
use domain::output::SpeakerUiDto;
use domain::profile::ProfileStore;
use platform::DesktopSurface;
use state::AppState;

/// Starts the desktop runtime. Panics only if the event loop cannot start.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle().clone();
            let state = AppState::open(&handle)?;
            let (surfaces, bundle) = {
                let conn = state.lock_db()?;
                let profile = crate::db::SqliteProfileStore::new(&conn).selected()?;
                let surfaces = SqliteSettingsStore::new(&conn).surfaces(profile.id.as_str())?;
                drop(conn);
                let mut output = state.lock_output()?;
                output.speaker_mode = surfaces.speaker_mode;
                let bundle = crate::domain::output::OutputBundleDto {
                    stage: output.stage.clone(),
                    clock: output.clock.snapshot(),
                    speaker_mode: output.speaker_mode,
                };
                drop(output);
                (surfaces, bundle)
            };
            app.manage(state);
            let _missing = DesktopSurface::new(handle.clone()).ensure_surfaces(&surfaces)?;
            let _ = handle.emit(OUTPUT_CHANGED, &bundle.stage);
            let _ = handle.emit(TIMER_CHANGED, &bundle.clock);
            let _ = handle.emit(
                SPEAKER_UI_CHANGED,
                SpeakerUiDto {
                    mode: bundle.speaker_mode,
                    message: String::new(),
                },
            );
            spawn_clock_tick(handle.clone());
            if let Some(operator) = handle.get_webview_window("operator") {
                let owned = handle.clone();
                let _ = operator.on_window_event(move |event| {
                    if matches!(
                        event,
                        tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed
                    ) {
                        DesktopSurface::new(owned.clone()).close_owned_surfaces();
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            profile_list,
            profile_create,
            profile_select,
            profile_update,
            profile_duplicate,
            profile_delete,
            content_languages_list,
            settings_get,
            settings_set,
            monitors_list,
            monitors_identify,
            output_get,
            clock_arm,
            clock_start,
            clock_pause,
            clock_finish,
            week_get,
            week_fetch,
            week_download_media,
            week_cancel,
            week_preview,
            week_restore,
            week_set_parts,
            template_list,
            template_save,
            template_delete,
            template_apply,
            explorer_list,
            explorer_add_root,
            explorer_pick_root,
            explorer_remove_root,
            explorer_open_jwpub,
            explorer_preview,
            stage_open,
            stage_close
        ])
        .run(tauri::generate_context!())
        .expect("error while running JPresentation");
}

/// ~4 Hz tick. Vue must not own a counting interval.
fn spawn_clock_tick(app: tauri::AppHandle) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_millis(250));
        let Some(state) = app.try_state::<AppState>() else {
            continue;
        };
        let snapshot = {
            let Ok(mut output) = state.lock_output() else {
                continue;
            };
            if !output.clock.tick() {
                continue;
            }
            output.clock.snapshot()
        };
        let _ = app.emit(TIMER_CHANGED, &snapshot);
    });
}
