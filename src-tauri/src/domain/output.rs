use serde::{Deserialize, Serialize};

use super::clock::{AssignmentClock, ClockSnapshot, MeetingClock};
use super::platform::SpeakerMode;

/// Event name for stage snapshots. Audience and the operator panel subscribe.
pub const OUTPUT_CHANGED: &str = "output://changed";

/// Event name for clock snapshots. Speaker HUD and the operator timer subscribe.
pub const TIMER_CHANGED: &str = "timer://changed";

/// Event name for speaker mode (and later HUD flags / messages).
pub const SPEAKER_UI_CHANGED: &str = "speaker://ui";

/// What the audience is showing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum StageKind {
    None,
    Image,
    Video,
}

/// Versioned stage snapshot. `rev` is monotonic so late events can be ignored.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StageSnapshot {
    pub rev: u64,
    pub kind: StageKind,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub mime: Option<String>,
    /// Absolute path under `media/stage/` for `convertFileSrc`.
    #[serde(default)]
    pub path: Option<String>,
}

impl StageSnapshot {
    fn idle(rev: u64) -> Self {
        Self {
            rev,
            kind: StageKind::None,
            name: None,
            mime: None,
            path: None,
        }
    }
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
            stage: StageSnapshot::idle(0),
            clock: AssignmentClock::idle(),
            speaker_mode: SpeakerMode::default(),
        }
    }

    /// Puts an image or video on stage. Does not touch the clock.
    pub fn open_media(
        &mut self,
        kind: StageKind,
        name: String,
        mime: String,
        path: String,
    ) -> StageSnapshot {
        self.stage.rev = self.stage.rev.saturating_add(1);
        self.stage.kind = kind;
        self.stage.name = Some(name);
        self.stage.mime = Some(mime);
        self.stage.path = Some(path);
        self.stage.clone()
    }

    /// Clears the stage to black.
    pub fn close_media(&mut self) -> StageSnapshot {
        self.stage = StageSnapshot::idle(self.stage.rev.saturating_add(1));
        self.stage.clone()
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_image_bumps_rev_and_close_clears() {
        let mut out = OutputState::idle();
        let snap = out.open_media(
            StageKind::Image,
            "a.jpg".into(),
            "image/jpeg".into(),
            "/tmp/a.jpg".into(),
        );
        assert_eq!(snap.kind, StageKind::Image);
        assert_eq!(snap.rev, 1);
        let closed = out.close_media();
        assert_eq!(closed.kind, StageKind::None);
        assert_eq!(closed.rev, 2);
        assert!(closed.path.is_none());
    }
}
