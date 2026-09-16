use serde::{Deserialize, Serialize};

use super::clock::{AssignmentClock, ClockSnapshot, MeetingClock};
use super::platform::SpeakerMode;

/// Event name for stage snapshots. Audience and the operator panel subscribe.
pub const OUTPUT_CHANGED: &str = "output://changed";

/// Event name for clock snapshots. Speaker HUD and the operator timer subscribe.
pub const TIMER_CHANGED: &str = "timer://changed";

/// Event name for speaker mode (and later HUD flags / messages).
pub const SPEAKER_UI_CHANGED: &str = "speaker://ui";

/// What the audience is showing. Change 001 only has `none` (black stage).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StageKind {
    None,
}

/// Versioned stage snapshot. `rev` is monotonic so late events can be ignored.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageSnapshot {
    pub rev: u64,
    pub kind: StageKind,
}

/// In-memory stage owned by Rust. Vue never holds the source of truth.
pub trait OutputPort: Send + Sync {
    /// Current stage without mutating it.
    fn snapshot(&self) -> StageSnapshot;
}

/// Combined output + clock holder used as `AppState.output`.
#[derive(Debug, Clone)]
pub struct OutputState {
    pub stage: StageSnapshot,
    pub clock: AssignmentClock,
    pub speaker_mode: SpeakerMode,
}

impl OutputState {
    /// Idle stage and idle clock at rev 0.
    pub fn idle() -> Self {
        Self {
            stage: StageSnapshot {
                rev: 0,
                kind: StageKind::None,
            },
            clock: AssignmentClock::idle(),
            speaker_mode: SpeakerMode::default(),
        }
    }
}

impl OutputPort for OutputState {
    fn snapshot(&self) -> StageSnapshot {
        self.stage.clone()
    }
}

impl MeetingClock for OutputState {
    fn snapshot(&self) -> ClockSnapshot {
        self.clock.snapshot()
    }
}

/// Read model returned by `output_get` so a reloaded console can resync without touching the stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBundleDto {
    pub stage: StageSnapshot,
    pub clock: ClockSnapshot,
    pub speaker_mode: SpeakerMode,
}

/// Speaker HUD flags published when surfaces change. `message` stays empty until a later change.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SpeakerUiDto {
    pub mode: SpeakerMode,
    pub message: String,
}
