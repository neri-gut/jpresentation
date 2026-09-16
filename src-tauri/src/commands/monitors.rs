use tauri::AppHandle;

use crate::domain::platform::{MonitorDto, PlatformSurface};
use crate::error::AppErrorDto;
use crate::platform::DesktopSurface;

/// Lists attached displays so the operator can assign audience and speaker.
#[tauri::command]
pub fn monitors_list(app: AppHandle) -> Result<Vec<MonitorDto>, AppErrorDto> {
    DesktopSurface::new(app)
        .list_monitors()
        .map_err(AppErrorDto::from)
}
