use std::thread;
use std::time::Duration;

use tauri::{
    AppHandle, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

use crate::domain::platform::{
    speaker_placement, MonitorDto, PlatformSurface, SpeakerPlacement, SurfacesSetting,
};
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
    ///
    /// Returns `true` when the saved speaker monitor is missing or collides with the
    /// audience monitor (placement degrades to preview). Callers toast `MonitorMissing`.
    pub fn ensure_surfaces(&self, setting: &SurfacesSetting) -> Result<bool, AppError> {
        let ids: Vec<String> = self
            .list_monitors()?
            .into_iter()
            .map(|item| item.id)
            .collect();
        let (placement, missing) = speaker_placement(setting, &ids);
        self.place_audience(setting.audience_monitor_id.as_deref())?;
        self.place_speaker(&placement)?;
        Ok(missing)
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

    fn hide_speaker(&self) -> Result<(), AppError> {
        if let Some(window) = self.app.get_webview_window(SPEAKER) {
            window
                .hide()
                .map_err(|e| AppError::Invariant(e.to_string()))?;
        }
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

    fn place_speaker(&self, placement: &SpeakerPlacement) -> Result<(), AppError> {
        match placement {
            SpeakerPlacement::Hidden => self.hide_speaker(),
            SpeakerPlacement::Preview => {
                let window = self.get_or_create(SPEAKER, "#/speaker", true)?;
                Self::place_preview(&window)
            }
            SpeakerPlacement::Monitor(id) => {
                let window = self.get_or_create(SPEAKER, "#/speaker", false)?;
                match self.monitor_by_id(id)? {
                    Some(monitor) => Self::place_fullscreen(&window, &monitor),
                    None => Self::place_preview(&window),
                }
            }
        }
    }

    fn identify_monitors(&self) -> Result<(), AppError> {
        for (label, window) in self.app.webview_windows() {
            if label.starts_with("identify-") {
                let _ = window.destroy();
            }
        }

        let monitors = self
            .app
            .available_monitors()
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        let mut created: Vec<WebviewWindow> = Vec::new();
        for (index, monitor) in monitors.iter().enumerate() {
            let name = monitor
                .name()
                .map(|s| s.to_string())
                .unwrap_or_else(|| "Display".into());
            let size = monitor.size();
            let pos = monitor.position();
            let label = format!("identify-{index}");
            let hash = format!(
                "#/identify?label={}&size={}x{}",
                encode_query(&name),
                size.width,
                size.height
            );
            let window = WebviewWindowBuilder::new(
                &self.app,
                &label,
                WebviewUrl::App(format!("index.html{hash}").into()),
            )
            .title(format!("{name} ({}×{})", size.width, size.height))
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .inner_size(480.0, 240.0)
            .build()
            .map_err(|e| AppError::Invariant(e.to_string()))?;

            let width = 480i32;
            let height = 240i32;
            let x = pos.x + (size.width as i32 - width).max(0) / 2;
            let y = pos.y + (size.height as i32 - height).max(0) / 2;
            window
                .set_position(Position::Physical(PhysicalPosition { x, y }))
                .map_err(|e| AppError::Invariant(e.to_string()))?;
            window
                .show()
                .map_err(|e| AppError::Invariant(e.to_string()))?;
            created.push(window);
        }

        thread::spawn(move || {
            thread::sleep(Duration::from_secs(2));
            for window in created {
                let _ = window.destroy();
            }
        });
        Ok(())
    }
}

fn monitor_id(monitor: &tauri::Monitor) -> String {
    let pos = monitor.position();
    let name = monitor
        .name()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "Display".into());
    format!("{name}@{},{}", pos.x, pos.y)
}

fn encode_query(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' => {
                out.push(byte as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
