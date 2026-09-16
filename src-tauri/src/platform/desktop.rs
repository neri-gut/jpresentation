use std::thread;
use std::time::Duration;

use tauri::{
    AppHandle, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Position, Size, WebviewUrl,
    WebviewWindow, WebviewWindowBuilder,
};

use crate::domain::platform::{
    audience_placement, speaker_placement, AudiencePlacement, MonitorDto, PlatformSurface,
    SpeakerPlacement, SurfacesSetting,
};
use crate::error::AppError;

const AUDIENCE: &str = "audience";
const SPEAKER: &str = "speaker";
const OPERATOR: &str = "operator";
const PREVIEW_WIDTH: f64 = 960.0;
const PREVIEW_HEIGHT: f64 = 540.0;

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
        let operator_id = self.operator_monitor_id()?;
        let audience = audience_placement(
            setting.audience_monitor_id.as_deref(),
            &ids,
            operator_id.as_deref(),
        );
        self.apply_audience(&audience)?;
        let (placement, missing) =
            speaker_placement(setting, &ids, operator_id.as_deref());
        self.place_speaker(&placement)?;
        Ok(missing)
    }

    /// Destroys every webview except the operator. Called when the console closes.
    pub fn close_owned_surfaces(&self) {
        for (label, window) in self.app.webview_windows() {
            if label != OPERATOR {
                let _ = window.destroy();
            }
        }
    }

    fn operator_monitor_id(&self) -> Result<Option<String>, AppError> {
        let Some(operator) = self.app.get_webview_window(OPERATOR) else {
            return Ok(None);
        };
        let monitor = operator
            .current_monitor()
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        Ok(monitor.map(|m| monitor_id(&m)))
    }

    fn surface_window(
        &self,
        label: &str,
        hash: &str,
        title: &str,
        decorated: bool,
    ) -> Result<WebviewWindow, AppError> {
        if let Some(existing) = self.app.get_webview_window(label) {
            let is_decorated = existing.is_decorated().unwrap_or(decorated);
            if is_decorated == decorated {
                let _ = existing.set_title(title);
                return Ok(existing);
            }
            let _ = existing.destroy();
        }

        let builder = WebviewWindowBuilder::new(
            &self.app,
            label,
            WebviewUrl::App(format!("index.html{hash}").into()),
        )
        .title(title)
        .decorations(decorated)
        .resizable(decorated)
        .visible(false);

        let builder = if let Some(operator) = self.app.get_webview_window(OPERATOR) {
            builder
                .parent(&operator)
                .map_err(|e| AppError::Invariant(e.to_string()))?
        } else {
            builder
        };

        builder
            .build()
            .map_err(|e| AppError::Invariant(e.to_string()))
    }

    fn apply_audience(&self, placement: &AudiencePlacement) -> Result<(), AppError> {
        match placement {
            AudiencePlacement::Preview => {
                let window =
                    self.surface_window(AUDIENCE, "#/audience", "JPresentation — Audience", true)?;
                self.place_preview(&window, 0)
            }
            AudiencePlacement::Fullscreen(id) => {
                let window =
                    self.surface_window(AUDIENCE, "#/audience", "JPresentation — Audience", false)?;
                match self.monitor_by_id(id)? {
                    Some(monitor) => Self::place_cover(&window, &monitor),
                    None => self.place_preview(&window, 0),
                }
            }
        }
    }

    fn monitor_by_id(&self, id: &str) -> Result<Option<tauri::Monitor>, AppError> {
        let monitors = self
            .app
            .available_monitors()
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        Ok(monitors.into_iter().find(|m| monitor_id(m) == id))
    }

    fn place_cover(window: &WebviewWindow, monitor: &tauri::Monitor) -> Result<(), AppError> {
        let pos = monitor.position();
        let size = monitor.size();
        let _ = window.set_fullscreen(false);
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

    fn place_preview(&self, window: &WebviewWindow, cascade: u32) -> Result<(), AppError> {
        let _ = window.set_fullscreen(false);
        window
            .set_decorations(true)
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        window
            .set_size(Size::Logical(LogicalSize::new(PREVIEW_WIDTH, PREVIEW_HEIGHT)))
            .map_err(|e| AppError::Invariant(e.to_string()))?;

        let step = 48i32 * cascade as i32;
        let mut x = 80 + step;
        let mut y = 80 + step;
        if let Some(operator) = self.app.get_webview_window(OPERATOR) {
            if let Ok(pos) = operator.outer_position() {
                x = pos.x + 40 + step;
                y = pos.y + 80 + step;
            }
        }
        window
            .set_position(Position::Physical(PhysicalPosition { x, y }))
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        let _ = window.unminimize();
        window
            .show()
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        Ok(())
    }

    fn hide_speaker(&self) -> Result<(), AppError> {
        if let Some(window) = self.app.get_webview_window(SPEAKER) {
            window
                .destroy()
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
        let operator_id = self.operator_monitor_id().ok().flatten();
        let monitors = self
            .app
            .available_monitors()
            .map_err(|e| AppError::Invariant(e.to_string()))?;
        Ok(monitors
            .into_iter()
            .map(|m| {
                let id = monitor_id(&m);
                let is_primary = primary_id.as_deref() == Some(id.as_str());
                let is_operator = operator_id.as_deref() == Some(id.as_str());
                let pos = m.position();
                let size = m.size();
                MonitorDto {
                    id,
                    name: m.name().map(|s| s.to_string()).unwrap_or_else(|| "Display".into()),
                    width: size.width,
                    height: size.height,
                    is_primary,
                    is_operator,
                    position_x: pos.x,
                    position_y: pos.y,
                }
            })
            .collect())
    }

    fn place_audience(&self, monitor_id: Option<&str>) -> Result<(), AppError> {
        let ids: Vec<String> = self
            .list_monitors()?
            .into_iter()
            .map(|item| item.id)
            .collect();
        let operator_id = self.operator_monitor_id()?;
        let placement = audience_placement(monitor_id, &ids, operator_id.as_deref());
        self.apply_audience(&placement)
    }

    fn place_speaker(&self, placement: &SpeakerPlacement) -> Result<(), AppError> {
        match placement {
            SpeakerPlacement::Hidden => self.hide_speaker(),
            SpeakerPlacement::Preview => {
                let window =
                    self.surface_window(SPEAKER, "#/speaker", "JPresentation — Speaker", true)?;
                self.place_preview(&window, 1)
            }
            SpeakerPlacement::Monitor(id) => {
                let window =
                    self.surface_window(SPEAKER, "#/speaker", "JPresentation — Speaker", false)?;
                match self.monitor_by_id(id)? {
                    Some(monitor) => Self::place_cover(&window, &monitor),
                    None => self.place_preview(&window, 1),
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
