//! Tauri command adapters. Each command validates IPC and delegates to domain/db/platform.

mod clock;
mod monitors;
mod output;
mod profile;
mod settings;

pub use clock::{clock_arm, clock_finish, clock_pause, clock_start};
pub use monitors::{monitors_identify, monitors_list};
pub use output::output_get;
pub use profile::{
    content_languages_list, profile_create, profile_delete, profile_duplicate, profile_list,
    profile_select, profile_update,
};
pub use settings::{settings_get, settings_set};
