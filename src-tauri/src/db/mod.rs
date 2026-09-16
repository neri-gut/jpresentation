//! SQLite repository. Vue never issues SQL; commands never open connections themselves.

mod migrations;
mod profile_store;
mod settings_store;
mod template_store;
mod week_store;

pub use migrations::{configure_connection, migrate};
pub use profile_store::SqliteProfileStore;
pub use settings_store::{SqliteSettingsStore, KEY_EXPLORER, KEY_SURFACES};
pub use template_store::SqliteTemplateStore;
pub use week_store::SqliteWeekStore;

#[cfg(test)]
pub(crate) use settings_store::{KEY_APPEARANCE, KEY_MEETING_SCHEDULE};

use std::time::{SystemTime, UNIX_EPOCH};

/// Monotonic-enough timestamp so last-used profile ordering does not collapse at second resolution.
pub(crate) fn now_stamp() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}.{:09}", duration.as_secs(), duration.subsec_nanos())
}
