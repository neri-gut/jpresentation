//! Domain ports and DTOs. This module must not import Tauri windowing.
//!
//! Meeting flow, profiles, output, and the clock live here so a future mobile
//! adapter can reuse them. Commands only validate IPC and delegate.
#![deny(clippy::unwrap_used)]

pub mod clock;
pub mod content;
pub mod media;
pub mod output;
pub mod platform;
pub mod profile;
pub mod schedule;

pub use clock::{
    clock_hue, warn_threshold_ms, AssignmentClock, ClockArmDto, ClockHue, ClockSnapshot,
    ClockState, MeetingClock,
};
pub use content::{content_languages, validate_content_locale, ContentLanguageDto};
pub use media::{MediaProvider, ProviderRegistry};
pub use output::{
    OutputPort, OutputState, SpeakerUiDto, StageKind, StageSnapshot, OUTPUT_CHANGED,
    SPEAKER_UI_CHANGED, TIMER_CHANGED,
};
pub use platform::{
    audience_placement, speaker_placement, AudiencePlacement, MonitorDto, PlatformSurface,
    SpeakerMode, SpeakerPlacement, SurfacesSetting,
};
pub use profile::{
    validate_appearance, validate_profile_name, AppearanceSetting, CreateProfileDto,
    DeleteProfileDto, DuplicateProfileDto, PanelSetting, ProfileDto, ProfileId, ProfileStore,
    SelectProfileDto, SettingDto, SettingKeyDto, UpdateProfileDto,
};
pub use schedule::{validate_meeting_schedule, MeetingScheduleSetting};
