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

/// Where the speaker surface should live after applying a profile's surfaces setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeakerPlacement {
    /// `use_speaker = false`. No speaker window.
    Hidden,
    /// Speaker on, but no usable dedicated monitor. Decorated 960×540 preview.
    Preview,
    /// Speaker fullscreen on this monitor id (never the audience monitor).
    Monitor(String),
}

/// Maps stored surfaces to a speaker placement. `missing` is true when the saved
/// speaker id is absent or equal to the audience id; callers toast `MonitorMissing`.
pub fn speaker_placement(
    setting: &SurfacesSetting,
    known_monitor_ids: &[String],
) -> (SpeakerPlacement, bool) {
    if !setting.use_speaker {
        return (SpeakerPlacement::Hidden, false);
    }
    match setting.speaker_monitor_id.as_deref() {
        None => (SpeakerPlacement::Preview, false),
        Some(id) => {
            let exists = known_monitor_ids.iter().any(|known| known == id);
            let same_as_audience = setting.audience_monitor_id.as_deref() == Some(id);
            if !exists || same_as_audience {
                (SpeakerPlacement::Preview, true)
            } else {
                (SpeakerPlacement::Monitor(id.to_string()), false)
            }
        }
    }
}

/// Window/monitor adapter. Desktop uses `WebviewWindow`; a future mobile adapter will not.
pub trait PlatformSurface: Send + Sync {
    /// Enumerate attached displays.
    fn list_monitors(&self) -> Result<Vec<MonitorDto>, AppError>;

    /// Place or preview the audience surface. `None` means floating preview on a single display.
    fn place_audience(&self, monitor_id: Option<&str>) -> Result<(), AppError>;

    /// Place, preview, or hide the speaker surface.
    fn place_speaker(&self, placement: &SpeakerPlacement) -> Result<(), AppError>;

    /// Flash name + resolution on every display for 2 s. Must not move audience or speaker.
    fn identify_monitors(&self) -> Result<(), AppError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct FakeSurface {
        monitors: Vec<MonitorDto>,
        speaker: Mutex<Option<SpeakerPlacement>>,
        audience: Mutex<Option<Option<String>>>,
    }

    impl PlatformSurface for FakeSurface {
        fn list_monitors(&self) -> Result<Vec<MonitorDto>, AppError> {
            Ok(self.monitors.clone())
        }

        fn place_audience(&self, monitor_id: Option<&str>) -> Result<(), AppError> {
            *self.audience.lock().expect("audience") = Some(monitor_id.map(str::to_string));
            Ok(())
        }

        fn place_speaker(&self, placement: &SpeakerPlacement) -> Result<(), AppError> {
            *self.speaker.lock().expect("speaker") = Some(placement.clone());
            Ok(())
        }

        fn identify_monitors(&self) -> Result<(), AppError> {
            Ok(())
        }
    }

    fn monitor(id: &str) -> MonitorDto {
        MonitorDto {
            id: id.into(),
            name: id.into(),
            width: 1920,
            height: 1080,
            is_primary: id == "primary",
            position_x: 0,
            position_y: 0,
        }
    }

    fn apply(surface: &FakeSurface, setting: &SurfacesSetting) -> Result<bool, AppError> {
        let ids: Vec<String> = surface
            .list_monitors()?
            .into_iter()
            .map(|item| item.id)
            .collect();
        let (placement, missing) = speaker_placement(setting, &ids);
        surface.place_audience(setting.audience_monitor_id.as_deref())?;
        surface.place_speaker(&placement)?;
        Ok(missing)
    }

    #[test]
    fn hidden_when_speaker_off() {
        let setting = SurfacesSetting {
            use_speaker: false,
            speaker_monitor_id: Some("hdmi".into()),
            audience_monitor_id: Some("primary".into()),
        };
        let (placement, missing) = speaker_placement(&setting, &["primary".into(), "hdmi".into()]);
        assert_eq!(placement, SpeakerPlacement::Hidden);
        assert!(!missing);
    }

    #[test]
    fn preview_when_on_without_monitor() {
        let setting = SurfacesSetting {
            use_speaker: true,
            speaker_monitor_id: None,
            audience_monitor_id: Some("primary".into()),
        };
        let (placement, missing) = speaker_placement(&setting, &["primary".into()]);
        assert_eq!(placement, SpeakerPlacement::Preview);
        assert!(!missing);
    }

    #[test]
    fn preview_and_missing_when_id_unknown_or_same_as_audience() {
        let unknown = SurfacesSetting {
            use_speaker: true,
            speaker_monitor_id: Some("gone".into()),
            audience_monitor_id: Some("primary".into()),
        };
        let (placement, missing) = speaker_placement(&unknown, &["primary".into()]);
        assert_eq!(placement, SpeakerPlacement::Preview);
        assert!(missing);

        let same = SurfacesSetting {
            use_speaker: true,
            speaker_monitor_id: Some("primary".into()),
            audience_monitor_id: Some("primary".into()),
        };
        let (placement, missing) = speaker_placement(&same, &["primary".into(), "hdmi".into()]);
        assert_eq!(placement, SpeakerPlacement::Preview);
        assert!(missing);
    }

    #[test]
    fn monitor_when_id_exists_and_differs() {
        let setting = SurfacesSetting {
            use_speaker: true,
            speaker_monitor_id: Some("hdmi".into()),
            audience_monitor_id: Some("primary".into()),
        };
        let (placement, missing) = speaker_placement(&setting, &["primary".into(), "hdmi".into()]);
        assert_eq!(placement, SpeakerPlacement::Monitor("hdmi".into()));
        assert!(!missing);
    }

    #[test]
    fn fake_surface_records_preview() {
        let surface = FakeSurface {
            monitors: vec![monitor("primary"), monitor("hdmi")],
            speaker: Mutex::new(None),
            audience: Mutex::new(None),
        };
        let setting = SurfacesSetting {
            use_speaker: true,
            speaker_monitor_id: None,
            audience_monitor_id: Some("hdmi".into()),
        };
        let missing = apply(&surface, &setting).expect("apply");
        assert!(!missing);
        assert_eq!(
            surface.speaker.lock().expect("speaker").clone(),
            Some(SpeakerPlacement::Preview)
        );
        assert_eq!(
            surface.audience.lock().expect("audience").clone(),
            Some(Some("hdmi".into()))
        );
    }
}
