use std::time::Instant;

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Assignment clock. `idle` has no part; `armed` is ready (or paused); `running` ticks in Rust.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClockState {
    Idle,
    Armed,
    Running,
}

/// HUD / panel color. Thresholds: `specs/alertas` `part.warn` / `part.over`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ClockHue {
    Green,
    Amber,
    Red,
}

/// Versioned clock snapshot for the speaker HUD and the operator timer block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClockSnapshot {
    pub rev: u64,
    pub state: ClockState,
    pub title: Option<String>,
    pub assigned_ms: u64,
    pub elapsed_ms: u64,
    pub remaining_ms: i64,
    pub overtime_ms: u64,
    pub progress_pct: u8,
    pub hue: ClockHue,
}

/// Payload for `clock_arm`. Title 1–80 after trim; minutes 1–180.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockArmDto {
    pub title: String,
    pub minutes: u32,
}

/// Meeting assignment clock port. Implementations must not open stage media.
pub trait MeetingClock: Send + Sync {
    /// Current clock without mutating it.
    fn snapshot(&self) -> ClockSnapshot;
}

const MIN_TITLE: usize = 1;
const MAX_TITLE: usize = 80;
const MIN_MINUTES: u32 = 1;
const MAX_MINUTES: u32 = 180;
const WARN_FLOOR_MS: u64 = 60_000;

/// In-memory assignment clock. Vue never owns the tick.
#[derive(Debug, Clone)]
pub struct AssignmentClock {
    rev: u64,
    inner: Inner,
}

#[derive(Debug, Clone)]
enum Inner {
    Idle,
    Armed {
        title: String,
        assigned_ms: u64,
        elapsed_ms: u64,
    },
    Running {
        title: String,
        assigned_ms: u64,
        elapsed_at_start: u64,
        started_at: Instant,
    },
}

impl AssignmentClock {
    /// Idle clock at rev 0.
    pub fn idle() -> Self {
        Self {
            rev: 0,
            inner: Inner::Idle,
        }
    }

    /// Arms (or replaces) a single part. Does not start counting.
    pub fn arm(&mut self, title: &str, minutes: u32) -> Result<(), AppError> {
        let title = validate_title(title)?;
        let assigned_ms = validate_minutes(minutes)?;
        self.rev = self.rev.saturating_add(1);
        self.inner = Inner::Armed {
            title,
            assigned_ms,
            elapsed_ms: 0,
        };
        Ok(())
    }

    /// Starts or resumes from `armed`. Idempotent if already running.
    pub fn start(&mut self) -> Result<(), AppError> {
        self.start_at(Instant::now())
    }

    /// Test seam: start as of `now`.
    pub fn start_at(&mut self, now: Instant) -> Result<(), AppError> {
        match &self.inner {
            Inner::Idle => Err(AppError::ClockNotArmed),
            Inner::Running { .. } => Ok(()),
            Inner::Armed {
                title,
                assigned_ms,
                elapsed_ms,
            } => {
                self.rev = self.rev.saturating_add(1);
                self.inner = Inner::Running {
                    title: title.clone(),
                    assigned_ms: *assigned_ms,
                    elapsed_at_start: *elapsed_ms,
                    started_at: now,
                };
                Ok(())
            }
        }
    }

    /// Freezes elapsed and returns to `armed`.
    pub fn pause(&mut self) -> Result<(), AppError> {
        self.pause_at(Instant::now())
    }

    /// Test seam: pause as of `now`.
    pub fn pause_at(&mut self, now: Instant) -> Result<(), AppError> {
        match &self.inner {
            Inner::Running {
                title,
                assigned_ms,
                elapsed_at_start,
                started_at,
            } => {
                let elapsed_ms = elapsed_since(*elapsed_at_start, *started_at, now);
                self.rev = self.rev.saturating_add(1);
                self.inner = Inner::Armed {
                    title: title.clone(),
                    assigned_ms: *assigned_ms,
                    elapsed_ms,
                };
                Ok(())
            }
            _ => Err(AppError::ClockNotRunning),
        }
    }

    /// Clears the part. Idle stays an error so the console can toast.
    pub fn finish(&mut self) -> Result<(), AppError> {
        if matches!(self.inner, Inner::Idle) {
            return Err(AppError::ClockNotArmed);
        }
        self.rev = self.rev.saturating_add(1);
        self.inner = Inner::Idle;
        Ok(())
    }

    /// Returns true when running so the host can emit `timer://changed`.
    pub fn tick(&mut self) -> bool {
        if matches!(self.inner, Inner::Running { .. }) {
            self.rev = self.rev.saturating_add(1);
            true
        } else {
            false
        }
    }

    /// Snapshot at `Instant::now()`.
    pub fn snapshot(&self) -> ClockSnapshot {
        self.snapshot_at(Instant::now())
    }

    /// Test seam: snapshot as of `now`.
    pub fn snapshot_at(&self, now: Instant) -> ClockSnapshot {
        match &self.inner {
            Inner::Idle => ClockSnapshot {
                rev: self.rev,
                state: ClockState::Idle,
                title: None,
                assigned_ms: 0,
                elapsed_ms: 0,
                remaining_ms: 0,
                overtime_ms: 0,
                progress_pct: 0,
                hue: ClockHue::Green,
            },
            Inner::Armed {
                title,
                assigned_ms,
                elapsed_ms,
            } => snapshot_of(
                self.rev,
                ClockState::Armed,
                title,
                *assigned_ms,
                *elapsed_ms,
            ),
            Inner::Running {
                title,
                assigned_ms,
                elapsed_at_start,
                started_at,
            } => {
                let elapsed_ms = elapsed_since(*elapsed_at_start, *started_at, now);
                snapshot_of(
                    self.rev,
                    ClockState::Running,
                    title,
                    *assigned_ms,
                    elapsed_ms,
                )
            }
        }
    }
}

impl MeetingClock for AssignmentClock {
    fn snapshot(&self) -> ClockSnapshot {
        AssignmentClock::snapshot(self)
    }
}

impl Default for AssignmentClock {
    fn default() -> Self {
        Self::idle()
    }
}

fn elapsed_since(elapsed_at_start: u64, started_at: Instant, now: Instant) -> u64 {
    let extra = now.saturating_duration_since(started_at).as_millis() as u64;
    elapsed_at_start.saturating_add(extra)
}

fn snapshot_of(
    rev: u64,
    state: ClockState,
    title: &str,
    assigned_ms: u64,
    elapsed_ms: u64,
) -> ClockSnapshot {
    let remaining_ms = assigned_ms as i64 - elapsed_ms as i64;
    ClockSnapshot {
        rev,
        state,
        title: Some(title.to_string()),
        assigned_ms,
        elapsed_ms,
        remaining_ms,
        overtime_ms: overtime_ms(remaining_ms),
        progress_pct: progress_pct(assigned_ms, elapsed_ms),
        hue: clock_hue(assigned_ms, remaining_ms),
    }
}

/// Last-stretch threshold: max(20 % of assigned, 1 minute).
pub fn warn_threshold_ms(assigned_ms: u64) -> u64 {
    assigned_ms.saturating_div(5).max(WARN_FLOOR_MS)
}

/// Maps remaining time to HUD color.
pub fn clock_hue(assigned_ms: u64, remaining_ms: i64) -> ClockHue {
    if remaining_ms <= 0 {
        ClockHue::Red
    } else if remaining_ms as u64 <= warn_threshold_ms(assigned_ms) {
        ClockHue::Amber
    } else {
        ClockHue::Green
    }
}

fn overtime_ms(remaining_ms: i64) -> u64 {
    if remaining_ms < 0 {
        remaining_ms.unsigned_abs()
    } else {
        0
    }
}

fn progress_pct(assigned_ms: u64, elapsed_ms: u64) -> u8 {
    if assigned_ms == 0 {
        return 0;
    }
    let pct = elapsed_ms.saturating_mul(100) / assigned_ms;
    u8::try_from(pct.min(100)).unwrap_or(100)
}

fn validate_title(title: &str) -> Result<String, AppError> {
    let trimmed = title.trim();
    if trimmed.len() < MIN_TITLE || trimmed.len() > MAX_TITLE {
        return Err(AppError::Invariant(
            "title must be 1–80 characters".into(),
        ));
    }
    Ok(trimmed.to_string())
}

fn validate_minutes(minutes: u32) -> Result<u64, AppError> {
    if !(MIN_MINUTES..=MAX_MINUTES).contains(&minutes) {
        return Err(AppError::Invariant(
            "minutes must be between 1 and 180".into(),
        ));
    }
    Ok(u64::from(minutes) * 60_000)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn arm_ten_minutes_is_green() {
        let mut clock = AssignmentClock::idle();
        clock.arm("Tesoro", 10).expect("arm");
        let snap = clock.snapshot();
        assert_eq!(snap.state, ClockState::Armed);
        assert_eq!(snap.title.as_deref(), Some("Tesoro"));
        assert_eq!(snap.assigned_ms, 600_000);
        assert_eq!(snap.remaining_ms, 600_000);
        assert_eq!(snap.hue, ClockHue::Green);
        assert_eq!(snap.progress_pct, 0);
    }

    #[test]
    fn last_two_minutes_of_ten_are_amber() {
        assert_eq!(warn_threshold_ms(600_000), 120_000);
        assert_eq!(clock_hue(600_000, 120_001), ClockHue::Green);
        assert_eq!(clock_hue(600_000, 120_000), ClockHue::Amber);
        assert_eq!(clock_hue(600_000, 1), ClockHue::Amber);
    }

    #[test]
    fn four_minute_part_warns_at_one_minute_floor() {
        assert_eq!(warn_threshold_ms(240_000), 60_000);
        assert_eq!(clock_hue(240_000, 60_001), ClockHue::Green);
        assert_eq!(clock_hue(240_000, 60_000), ClockHue::Amber);
    }

    #[test]
    fn overtime_is_red_and_full_bar() {
        let mut clock = AssignmentClock::idle();
        clock.arm("Tesoro", 10).expect("arm");
        let t0 = Instant::now();
        clock.start_at(t0).expect("start");
        let snap = clock.snapshot_at(t0 + Duration::from_millis(11 * 60 * 1000 + 20_000));
        assert_eq!(snap.state, ClockState::Running);
        assert_eq!(snap.remaining_ms, -80_000);
        assert_eq!(snap.overtime_ms, 80_000);
        assert_eq!(snap.hue, ClockHue::Red);
        assert_eq!(snap.progress_pct, 100);
        assert_eq!(clock_hue(600_000, 0), ClockHue::Red);
    }

    #[test]
    fn pause_keeps_elapsed_and_start_resumes() {
        let mut clock = AssignmentClock::idle();
        clock.arm("Tesoro", 10).expect("arm");
        let t0 = Instant::now();
        clock.start_at(t0).expect("start");
        clock
            .pause_at(t0 + Duration::from_secs(4 * 60))
            .expect("pause");
        let paused = clock.snapshot_at(t0 + Duration::from_secs(4 * 60));
        assert_eq!(paused.state, ClockState::Armed);
        assert_eq!(paused.elapsed_ms, 4 * 60_000);
        assert_eq!(paused.remaining_ms, 6 * 60_000);
        let t1 = t0 + Duration::from_secs(4 * 60);
        clock.start_at(t1).expect("resume");
        let resumed = clock.snapshot_at(t1 + Duration::from_secs(60));
        assert_eq!(resumed.state, ClockState::Running);
        assert_eq!(resumed.elapsed_ms, 5 * 60_000);
        assert_eq!(resumed.remaining_ms, 5 * 60_000);
    }

    #[test]
    fn start_from_idle_is_not_armed() {
        let mut clock = AssignmentClock::idle();
        assert!(matches!(clock.start(), Err(AppError::ClockNotArmed)));
    }

    #[test]
    fn pause_from_armed_is_not_running() {
        let mut clock = AssignmentClock::idle();
        clock.arm("Tesoro", 10).expect("arm");
        assert!(matches!(clock.pause(), Err(AppError::ClockNotRunning)));
    }

    #[test]
    fn finish_clears_to_idle() {
        let mut clock = AssignmentClock::idle();
        clock.arm("Tesoro", 10).expect("arm");
        clock.finish().expect("finish");
        assert_eq!(clock.snapshot().state, ClockState::Idle);
        assert!(matches!(clock.finish(), Err(AppError::ClockNotArmed)));
    }

    #[test]
    fn start_while_running_is_idempotent() {
        let mut clock = AssignmentClock::idle();
        clock.arm("Tesoro", 10).expect("arm");
        let t0 = Instant::now();
        clock.start_at(t0).expect("start");
        clock.start_at(t0 + Duration::from_secs(1)).expect("again");
        let snap = clock.snapshot_at(t0 + Duration::from_secs(1));
        assert_eq!(snap.elapsed_ms, 1_000);
    }

    #[test]
    fn rejects_empty_title_and_out_of_range_minutes() {
        let mut clock = AssignmentClock::idle();
        assert!(clock.arm("   ", 10).is_err());
        assert!(clock.arm("Tesoro", 0).is_err());
        assert!(clock.arm("Tesoro", 181).is_err());
        clock.arm("  Tesoro  ", 1).expect("trim");
        assert_eq!(clock.snapshot().assigned_ms, 60_000);
    }

    #[test]
    fn tick_only_when_running() {
        let mut clock = AssignmentClock::idle();
        assert!(!clock.tick());
        clock.arm("Tesoro", 10).expect("arm");
        assert!(!clock.tick());
        clock.start().expect("start");
        assert!(clock.tick());
    }

    #[test]
    fn arm_replaces_a_running_part() {
        let mut clock = AssignmentClock::idle();
        clock.arm("Tesoro", 10).expect("arm");
        clock.start().expect("start");
        clock.arm("Perlas", 10).expect("replace");
        let snap = clock.snapshot();
        assert_eq!(snap.state, ClockState::Armed);
        assert_eq!(snap.title.as_deref(), Some("Perlas"));
        assert_eq!(snap.elapsed_ms, 0);
    }
}
