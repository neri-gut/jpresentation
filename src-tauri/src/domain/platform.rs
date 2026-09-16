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
    /// True when the operator window currently sits on this display.
    #[serde(default)]
    pub is_operator: bool,
    pub position_x: i32,
    pub position_y: i32,
}

/// How the speaker surface paints the stage.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpeakerMode {
    /// Same frame as the audience plus HUD (compact when there is media).
    #[default]
    Mirror,
    /// Clock on black, even if the audience has a resource.
    HudOnly,
}

/// Per-profile assignment of audience/speaker surfaces.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfacesSetting {
    pub audience_monitor_id: Option<String>,
    pub speaker_monitor_id: Option<String>,
    pub use_speaker: bool,
    /// Missing on cells written before 031 → `mirror`.
    #[serde(default)]
    pub speaker_mode: SpeakerMode,
}

impl Default for SurfacesSetting {
    fn default() -> Self {
        Self {
            audience_monitor_id: None,
            speaker_monitor_id: None,
            use_speaker: false,
            speaker_mode: SpeakerMode::Mirror,
        }
    }
}

/// Where the audience surface should live.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudiencePlacement {
    /// Decorated 960×540 window. Never covers the operator display.
    Preview,
    /// Borderless cover of this monitor (not exclusive OS fullscreen).
    Fullscreen(String),
}

/// Where the speaker surface should live after applying a profile's surfaces setting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpeakerPlacement {
    /// `use_speaker = false`. No speaker window.
    Hidden,
    /// Speaker on, but no usable dedicated monitor. Decorated 960×540 preview.
    Preview,
    /// Borderless cover of this monitor id (never the audience or operator monitor).
    Monitor(String),
}

fn covers_operator(id: &str, operator_monitor_id: Option<&str>, known_len: usize) -> bool {
    known_len < 2 || operator_monitor_id == Some(id)
}

/// Maps the stored audience monitor to preview vs cover.
///
/// An empty assignment is always preview — it MUST NOT steal the secondary display.
pub fn audience_placement(
    assigned: Option<&str>,
    known_monitor_ids: &[String],
    operator_monitor_id: Option<&str>,
) -> AudiencePlacement {
    match assigned {
        None => AudiencePlacement::Preview,
        Some(id) => {
            let exists = known_monitor_ids.iter().any(|known| known == id);
            if !exists || covers_operator(id, operator_monitor_id, known_monitor_ids.len()) {
                AudiencePlacement::Preview
            } else {
                AudiencePlacement::Fullscreen(id.to_string())
            }
        }
    }
}

/// Maps stored surfaces to a speaker placement. `missing` is true when the saved
/// speaker id is absent or equal to the audience id; callers toast `MonitorMissing`.
/// Assigning the operator's own display is a valid preview (not missing).
pub fn speaker_placement(
    setting: &SurfacesSetting,
    known_monitor_ids: &[String],
    operator_monitor_id: Option<&str>,
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
            } else if covers_operator(id, operator_monitor_id, known_monitor_ids.len()) {
                (SpeakerPlacement::Preview, false)
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
            is_operator: id == "primary",
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
        let (placement, missing) = speaker_placement(setting, &ids, Some("primary"));
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
            ..SurfacesSetting::default()
        };
        let (placement, missing) =
            speaker_placement(&setting, &["primary".into(), "hdmi".into()], Some("primary"));
        assert_eq!(placement, SpeakerPlacement::Hidden);
        assert!(!missing);
    }

    #[test]
    fn preview_when_on_without_monitor() {
        let setting = SurfacesSetting {
            use_speaker: true,
            speaker_monitor_id: None,
            audience_monitor_id: Some("primary".into()),
            ..SurfacesSetting::default()
        };
        let (placement, missing) =
            speaker_placement(&setting, &["primary".into()], Some("primary"));
        assert_eq!(placement, SpeakerPlacement::Preview);
        assert!(!missing);
    }

    #[test]
    fn preview_and_missing_when_id_unknown_or_same_as_audience() {
        let unknown = SurfacesSetting {
            use_speaker: true,
            speaker_monitor_id: Some("gone".into()),
            audience_monitor_id: Some("primary".into()),
            ..SurfacesSetting::default()
        };
        let (placement, missing) =
            speaker_placement(&unknown, &["primary".into()], Some("primary"));
        assert_eq!(placement, SpeakerPlacement::Preview);
        assert!(missing);

        let same = SurfacesSetting {
            use_speaker: true,
            speaker_monitor_id: Some("primary".into()),
            audience_monitor_id: Some("primary".into()),
            ..SurfacesSetting::default()
        };
        let (placement, missing) =
            speaker_placement(&same, &["primary".into(), "hdmi".into()], Some("primary"));
        assert_eq!(placement, SpeakerPlacement::Preview);
        assert!(missing);
    }

    #[test]
    fn monitor_when_id_exists_and_differs() {
        let setting = SurfacesSetting {
            use_speaker: true,
            speaker_monitor_id: Some("hdmi".into()),
            audience_monitor_id: Some("primary".into()),
            ..SurfacesSetting::default()
        };
        let (placement, missing) =
            speaker_placement(&setting, &["primary".into(), "hdmi".into()], Some("primary"));
        assert_eq!(placement, SpeakerPlacement::Monitor("hdmi".into()));
        assert!(!missing);
    }

    #[test]
    fn speaker_on_operator_display_is_preview_not_missing() {
        let setting = SurfacesSetting {
            use_speaker: true,
            speaker_monitor_id: Some("primary".into()),
            audience_monitor_id: Some("hdmi".into()),
            ..SurfacesSetting::default()
        };
        let (placement, missing) =
            speaker_placement(&setting, &["primary".into(), "hdmi".into()], Some("primary"));
        assert_eq!(placement, SpeakerPlacement::Preview);
        assert!(!missing);
    }

    #[test]
    fn empty_audience_is_preview_even_with_two_displays() {
        let known = vec!["primary".into(), "hdmi".into()];
        assert_eq!(
            audience_placement(None, &known, Some("primary")),
            AudiencePlacement::Preview
        );
    }

    #[test]
    fn audience_on_operator_or_single_display_is_preview() {
        let two = vec!["primary".into(), "hdmi".into()];
        assert_eq!(
            audience_placement(Some("primary"), &two, Some("primary")),
            AudiencePlacement::Preview
        );
        let one = vec!["primary".into()];
        assert_eq!(
            audience_placement(Some("primary"), &one, Some("primary")),
            AudiencePlacement::Preview
        );
    }

    #[test]
    fn audience_on_other_display_is_fullscreen() {
        let known = vec!["primary".into(), "hdmi".into()];
        assert_eq!(
            audience_placement(Some("hdmi"), &known, Some("primary")),
            AudiencePlacement::Fullscreen("hdmi".into())
        );
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
            ..SurfacesSetting::default()
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

    #[test]
    fn surfaces_json_without_speaker_mode_defaults_to_mirror() {
        let parsed: SurfacesSetting = serde_json::from_str(
            r#"{"audience_monitor_id":null,"speaker_monitor_id":null,"use_speaker":false}"#,
        )
        .expect("legacy");
        assert_eq!(parsed.speaker_mode, SpeakerMode::Mirror);
        assert!(!parsed.use_speaker);
    }
}
