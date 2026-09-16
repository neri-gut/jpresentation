//! Tauri command adapters. Each command validates IPC and delegates to domain/db/platform.

mod monitors;
mod output;
mod profile;
mod settings;

pub use monitors::monitors_list;
pub use output::output_get;
pub use profile::{profile_create, profile_list, profile_select, profile_update};
pub use settings::{settings_get, settings_set};
