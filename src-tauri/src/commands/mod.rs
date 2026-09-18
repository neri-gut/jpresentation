//! Tauri command adapters. Each command validates IPC and delegates to domain/db/platform.

mod clock;
mod explorer;
mod hymnal;
mod monitors;
mod output;
mod profile;
mod settings;
mod template;
mod week;

pub use clock::{clock_arm, clock_finish, clock_pause, clock_start};
pub use explorer::{
    explorer_add_root, explorer_list, explorer_open_jwpub, explorer_pick_root, explorer_preview,
    explorer_remove_root, stage_close, stage_open,
};
pub use hymnal::{
    hymnal_cancel, hymnal_download_all, hymnal_download_song, hymnal_get, hymnal_play,
    hymnal_refresh,
};
pub use monitors::{monitors_identify, monitors_list};
pub use output::output_get;
pub use template::{
    template_apply, template_delete, template_list, template_save, week_restore, week_set_parts,
};
pub use week::{week_cancel, week_download_media, week_fetch, week_get, week_preview};
pub use profile::{
    content_languages_list, profile_create, profile_delete, profile_duplicate, profile_list,
    profile_select, profile_update,
};
pub use settings::{settings_get, settings_set};
