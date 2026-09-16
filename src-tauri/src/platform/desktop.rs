use tauri::{
    AppHandle, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

use crate::domain::platform::{MonitorDto, PlatformSurface, SurfacesSetting};
use crate::error::AppError;

const AUDIENCE: &str = "audience";
const SPEAKER: &str = "speaker";

/// Desktop adapter: `WebviewWindow` + OS monitors. Domain code never calls this type directly.
pub struct DesktopSurface {
    app: AppHandle,
}

impl DesktopSurface {
    /// Wraps the running Tauri app.
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    /// Creates missing surfaces and places them from the active profile's settings.
    pub fn ensure_surfaces(&self, setting: &SurfacesSetting) -> Result<(), AppError> {
        self.place_audience(setting.audience_monitor_id.as_deref())?;
        if setting.use_speaker {
            self.place_speaker(setting.speaker_monitor_id.as_deref())?;
        } else {
            self.place_speaker(None)?;
        }
        Ok(())
    }

    fn get_or_create(&self, label: &str, hash: &str, decorated: bool) -> Result<WebviewWindow, AppError> {
        if let Some(existing) = self.app.get_webview_window(label) {
            return Ok(existing);
        }
        let window = WebviewWindowBuilder::new(
            &self.app,
            label,
            WebviewUrl::App(format!("index.html{hash}").into()),
        )
        .title("JPresentation")
        .decorations(decorated)
        .visible(false)
        .build()
        .map_err(|e| AppError::Invariant(e.to_string()))?;
        Ok(window)
    }

    fn monitor_by_id(&self, id: &str) -> Result<Option<tauri::Monitor>, AppError> {
        let monitors = self
            .app
            .available_monitors()
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        Ok(monitors.into_iter().find(|m| monitor_id(m) == id))
    }

    fn place_fullscreen(window: &WebviewWindow, monitor: &tauri::Monitor) -> Result<(), AppError> {
        let pos = monitor.position();
        let size = monitor.size();
        window
            .set_fullscreen(false)
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        window
            .set_decorations(false)
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        window
            .set_position(Position::Physical(PhysicalPosition { x: pos.x, y: pos.y }))
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        window
            .set_size(Size::Physical(PhysicalSize {
                width: size.width,
                height: size.height,
            }))
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        window
            .show()
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        Ok(())
    }

    fn place_preview(window: &WebviewWindow) -> Result<(), AppError> {
        window
            .set_fullscreen(false)
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        window
            .set_decorations(true)
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        window
            .set_size(Size::Logical(LogicalSize::new(960.0, 540.0)))
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        window
            .show()
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        Ok(())
    }
}

impl PlatformSurface for DesktopSurface {
    fn list_monitors(&self) -> Result<Vec<MonitorDto>, AppError> {
        let primary = self
            .app
            .primary_monitor()
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        let primary_id = primary.as_ref().map(monitor_id);
        let monitors = self
            .app
            .available_monitors()
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        Ok(monitors
            .into_iter()
            .map(|m| {
                let id = monitor_id(&m);
                let is_primary = primary_id.as_deref() == Some(id.as_str());
                let pos = m.position();
                let size = m.size();
                MonitorDto {
                    id,
                    name: m.name().map(|s| s.to_string()).unwrap_or_else(|| "Display".into()),
                    width: size.width,
                    height: size.height,
                    is_primary,
                    position_x: pos.x,
                    position_y: pos.y,
                }
            })
            .collect())
    }

    fn place_audience(&self, monitor_id: Option<&str>) -> Result<(), AppError> {
        let window = self.get_or_create(AUDIENCE, "#/audience", false)?;
        match monitor_id {
            Some(id) => match self.monitor_by_id(id)? {
                Some(monitor) => Self::place_fullscreen(&window, &monitor),
                None => Self::place_preview(&window),
            },
            None => {
                let count = self.list_monitors()?.len();
                if count >= 2 {
                    if let Some(secondary) = self
                        .list_monitors()?
                        .into_iter()
                        .find(|m| !m.is_primary)
                    {
                        return self.place_audience(Some(&secondary.id));
                    }
                }
                Self::place_preview(&window)
            }
        }
    }

    fn place_speaker(&self, monitor_id: Option<&str>) -> Result<(), AppError> {
        match monitor_id {
            None => {
                if let Some(window) = self.app.get_webview_window(SPEAKER) {
                    window
                        .hide()
                        .map_err(|e| AppError::Invariant(e.to_string()))?;
                }
                Ok(())
            }
            Some(id) => {
                let window = self.get_or_create(SPEAKER, "#/speaker", false)?;
                match self.monitor_by_id(id)? {
                    Some(monitor) => Self::place_fullscreen(&window, &monitor),
                    None => Self::place_preview(&window),
                }
            }
        }
    }
}

fn monitor_id(monitor: &tauri::Monitor) -> String {
    let pos = monitor.position();
    let name = monitor.name().map(|s| s.to_string()).unwrap_or_else(|| "Display".into());
    format!("{name}@{},{}", pos.x, pos.y)
}
