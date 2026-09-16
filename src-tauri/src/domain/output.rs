use serde::{Deserialize, Serialize};

/// Event name for stage snapshots. Audience and the operator panel subscribe.
pub const OUTPUT_CHANGED: &str = "output://changed";

/// Event name for clock snapshots. Speaker HUD and the operator timer subscribe.
pub const TIMER_CHANGED: &str = "timer://changed";

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
    pub clock: super::clock::ClockSnapshot,
}

impl OutputState {
    /// Idle stage and idle clock at rev 0.
    pub fn idle() -> Self {
        Self {
            stage: StageSnapshot {
                rev: 0,
                kind: StageKind::None,
            },
            clock: super::clock::ClockSnapshot {
                rev: 0,
                state: super::clock::ClockState::Idle,
            },
        }
    }
}

impl OutputPort for OutputState {
    fn snapshot(&self) -> StageSnapshot {
        self.stage.clone()
    }
}

impl super::clock::MeetingClock for OutputState {
    fn snapshot(&self) -> super::clock::ClockSnapshot {
        self.clock.clone()
    }
}

/// Read model returned by `output_get` so a reloaded console can resync without touching the stage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputBundleDto {
    pub stage: StageSnapshot,
    pub clock: super::clock::ClockSnapshot,
}
