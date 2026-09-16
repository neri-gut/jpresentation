//! Domain ports and DTOs. This module must not import Tauri windowing.
//!
//! Meeting flow, profiles, output, and the clock live here so a future mobile
//! adapter can reuse them. Commands only validate IPC and delegate.
#![deny(clippy::unwrap_used)]

pub mod clock;
pub mod media;
pub mod output;
pub mod platform;
pub mod profile;

pub use clock::{ClockSnapshot, ClockState, MeetingClock};
pub use media::{MediaProvider, ProviderRegistry};
pub use output::{OutputPort, OutputState, StageKind, StageSnapshot, OUTPUT_CHANGED, TIMER_CHANGED};
pub use platform::{MonitorDto, PlatformSurface, SurfacesSetting};
pub use profile::{
    AppearanceSetting, CreateProfileDto, PanelSetting, ProfileDto, ProfileId, ProfileStore,
    SelectProfileDto, SettingDto, SettingKeyDto, UpdateProfileDto,
};
