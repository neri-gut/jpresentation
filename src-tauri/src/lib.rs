mod commands;
mod db;
pub mod domain;
mod error;
mod jobs;
mod platform;
mod state;

use tauri::Emitter;
use tauri::Manager;

use commands::{
    monitors_list, output_get, profile_create, profile_list, profile_select, profile_update,
    settings_get, settings_set,
};
use db::SqliteSettingsStore;
use domain::output::{OUTPUT_CHANGED, TIMER_CHANGED};
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
                let output = state.lock_output()?;
                let bundle = crate::domain::output::OutputBundleDto {
                    stage: output.stage.clone(),
                    clock: output.clock.clone(),
                };
                drop(output);
                drop(conn);
                (surfaces, bundle)
            };
            app.manage(state);
            DesktopSurface::new(handle.clone()).ensure_surfaces(&surfaces)?;
            let _ = handle.emit(OUTPUT_CHANGED, &bundle.stage);
            let _ = handle.emit(TIMER_CHANGED, &bundle.clock);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            profile_list,
            profile_create,
            profile_select,
            profile_update,
            settings_get,
            settings_set,
            monitors_list,
            output_get
        ])
        .run(tauri::generate_context!())
        .expect("error while running JPresentation");
}
