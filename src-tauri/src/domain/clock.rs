use serde::{Deserialize, Serialize};

/// Assignment clock. Change 001 stays idle: finishing a part must not launch media (later changes).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClockState {
    Idle,
}

/// Versioned clock snapshot for the speaker HUD and the operator timer block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClockSnapshot {
    pub rev: u64,
    pub state: ClockState,
}

/// Meeting assignment clock port. Implementations must not open stage media.
pub trait MeetingClock: Send + Sync {
    /// Current clock without mutating it.
    fn snapshot(&self) -> ClockSnapshot;
}
