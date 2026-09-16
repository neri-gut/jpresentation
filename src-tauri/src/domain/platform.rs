use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// A physical display as shown in Settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorDto {
    pub id: String,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
    pub position_x: i32,
    pub position_y: i32,
}

/// Per-profile assignment of audience/speaker surfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfacesSetting {
    pub audience_monitor_id: Option<String>,
    pub speaker_monitor_id: Option<String>,
    pub use_speaker: bool,
}

impl Default for SurfacesSetting {
    fn default() -> Self {
        Self {
            audience_monitor_id: None,
            speaker_monitor_id: None,
            use_speaker: false,
        }
    }
}

/// Window/monitor adapter. Desktop uses `WebviewWindow`; a future mobile adapter will not.
pub trait PlatformSurface: Send + Sync {
    /// Enumerate attached displays.
    fn list_monitors(&self) -> Result<Vec<MonitorDto>, AppError>;

    /// Place or preview the audience surface. `None` means floating preview on a single display.
    fn place_audience(&self, monitor_id: Option<&str>) -> Result<(), AppError>;

    /// Place the speaker surface, or hide it when `monitor_id` is `None`.
    fn place_speaker(&self, monitor_id: Option<&str>) -> Result<(), AppError>;
}
