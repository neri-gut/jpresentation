//! Tauri command adapters. Each command validates IPC and delegates to domain/db/platform.

mod clock;
mod explorer;
mod monitors;
mod output;
mod profile;
mod settings;
mod week;

pub use clock::{clock_arm, clock_finish, clock_pause, clock_start};
pub use explorer::{
    explorer_add_root, explorer_list, explorer_open_jwpub, explorer_pick_root, explorer_preview,
    explorer_remove_root, stage_close, stage_open,
};
pub use monitors::{monitors_identify, monitors_list};
pub use output::output_get;
pub use week::{week_cancel, week_download_media, week_fetch, week_get, week_preview};
pub use profile::{
    content_languages_list, profile_create, profile_delete, profile_duplicate, profile_list,
    profile_select, profile_update,
};
pub use settings::{settings_get, settings_set};
