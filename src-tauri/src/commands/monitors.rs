use crate::domain::platform::{MonitorDto, PlatformSurface};
use crate::error::AppErrorDto;
use crate::platform::DesktopSurface;
use tauri::AppHandle;

/// Lists attached displays so the operator can assign audience and speaker.
#[tauri::command]
pub fn monitors_list(app: AppHandle) -> Result<Vec<MonitorDto>, AppErrorDto> {
    DesktopSurface::new(app)
        .list_monitors()
        .map_err(AppErrorDto::from)
}

/// Flashes name and resolution on every display for two seconds.
#[tauri::command]
pub fn monitors_identify(app: AppHandle) -> Result<(), AppErrorDto> {
    DesktopSurface::new(app)
        .identify_monitors()
        .map_err(AppErrorDto::from)
}
