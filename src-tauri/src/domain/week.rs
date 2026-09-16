//! Meeting week model, calendar helpers, and cache paths.
//!
//! «This week» is the ISO Monday–Sunday that contains the operator's local date.
//! Fetching MUST NOT happen at process start.

use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Which meeting a persisted `MeetingWeek` describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MeetingKind {
    Midweek,
    Weekend,
}

/// Which rolling week the operator asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WeekWhich {
    This,
    Next,
}

impl WeekWhich {
    /// Parses the IPC `which` field.
    pub fn parse(raw: &str) -> Result<Self, AppError> {
        match raw {
            "this" => Ok(Self::This),
            "next" => Ok(Self::Next),
            _ => Err(AppError::Invariant("which must be this or next".into())),
        }
    }
}

/// Kind of a listed medium.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MediaKind {
    Image,
    Video,
    Song,
}

/// Cache / download state painted in the Multimedia tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaStatus {
    Embedded,
    Ready,
    Pending,
    PendingHymnal,
    Failed,
}

/// How a medium is addressed. Vue never sees CDN URLs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum MediaRef {
    Embedded {
        path: String,
    },
    Catalog {
        key_symbol: String,
        track: u32,
        lang_meps: i64,
        issue_tag: i64,
        mime: String,
    },
}

/// One image, video, or song slot under a part.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaItem {
    pub id: String,
    pub title: String,
    pub media_kind: MediaKind,
    pub status: MediaStatus,
    pub mime: String,
    pub media_ref: MediaRef,
    /// Absolute path under `week/` when the file is on disk. Preview only.
    pub cache_path: Option<String>,
}

fn default_tone() -> String {
    "other".into()
}

/// One row of the meeting outline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingPart {
    pub id: String,
    pub title: String,
    pub minutes: Option<u32>,
    /// `treasures` | `ayf` | `living` | `song` | `other`
    #[serde(default = "default_tone")]
    pub tone: String,
    #[serde(default)]
    pub items: Vec<MediaItem>,
}

/// Normalised programme for one meeting on one Monday.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingWeek {
    pub monday: String,
    pub kind: MeetingKind,
    pub title: String,
    pub langwritten: String,
    pub pub_symbol: String,
    pub issue: String,
    pub parts: Vec<MeetingPart>,
    /// Leftover images/videos not shown as timer rows.
    #[serde(default)]
    pub media: Vec<MediaItem>,
}

impl MeetingWeek {
    /// Empty week used before the first successful fetch.
    pub fn empty(monday: CivilDate, kind: MeetingKind, langwritten: &str) -> Self {
        let pub_symbol = match kind {
            MeetingKind::Midweek => "mwb",
            MeetingKind::Weekend => "w",
        };
        Self {
            monday: monday.to_iso(),
            kind,
            title: String::new(),
            langwritten: langwritten.to_string(),
            pub_symbol: pub_symbol.into(),
            issue: match kind {
                MeetingKind::Midweek => mwb_issue_candidates(monday)
                    .into_iter()
                    .next()
                    .unwrap_or_else(|| issue_yyyymm(monday)),
                MeetingKind::Weekend => w_issue(monday),
            },
            parts: Vec::new(),
            media: Vec::new(),
        }
    }
}

/// Gregorian date without a timezone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CivilDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl CivilDate {
    /// `YYYY-MM-DD`.
    pub fn to_iso(self) -> String {
        format!("{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }

    /// Compact `YYYYMMDD` used in `DatedText`.
    pub fn to_yyyymmdd(self) -> i32 {
        self.year * 10_000 + i32::from(self.month) * 100 + i32::from(self.day)
    }

    /// Parses `YYYY-MM-DD`.
    pub fn parse_iso(raw: &str) -> Result<Self, AppError> {
        let bytes = raw.as_bytes();
        if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
            return Err(AppError::Invariant("date must be YYYY-MM-DD".into()));
        }
        let year: i32 = raw[0..4]
            .parse()
            .map_err(|_| AppError::Invariant("invalid year".into()))?;
        let month: u8 = raw[5..7]
            .parse()
            .map_err(|_| AppError::Invariant("invalid month".into()))?;
        let day: u8 = raw[8..10]
            .parse()
            .map_err(|_| AppError::Invariant("invalid day".into()))?;
        if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
            return Err(AppError::Invariant("invalid calendar date".into()));
        }
        Ok(Self { year, month, day })
    }
}

/// Local civil date (UTC is close enough for hall PCs; operator can still pass a date later).
pub fn today() -> CivilDate {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;
    civil_from_unix_days((secs / 86_400) as i32)
}

/// Monday of the ISO week that contains `date`.
pub fn monday_of(date: CivilDate) -> CivilDate {
    let days = unix_days(date);
    let iso = iso_weekday(days);
    civil_from_unix_days(days - i32::from(iso - 1))
}

/// Adds (possibly negative) days.
pub fn add_days(date: CivilDate, delta: i32) -> CivilDate {
    civil_from_unix_days(unix_days(date) + delta)
}

/// Resolves this/next week to its Monday.
pub fn monday_for(which: WeekWhich, today: CivilDate) -> CivilDate {
    let this = monday_of(today);
    match which {
        WeekWhich::This => this,
        WeekWhich::Next => add_days(this, 7),
    }
}

/// `YYYYMM` of the month that contains `date`.
pub fn issue_yyyymm(date: CivilDate) -> String {
    format!("{:04}{:02}", date.year, date.month)
}

/// Previous calendar month as `YYYYMM`.
pub fn previous_month_issue(issue: &str) -> Result<String, AppError> {
    if issue.len() != 6 {
        return Err(AppError::Invariant("issue must be YYYYMM".into()));
    }
    let year: i32 = issue[0..4]
        .parse()
        .map_err(|_| AppError::Invariant("invalid issue year".into()))?;
    let month: u8 = issue[4..6]
        .parse()
        .map_err(|_| AppError::Invariant("invalid issue month".into()))?;
    if !(1..=12).contains(&month) {
        return Err(AppError::Invariant("invalid issue month".into()));
    }
    if month == 1 {
        Ok(format!("{:04}12", year - 1))
    } else {
        Ok(format!("{:04}{:02}", year, month - 1))
    }
}

/// `mwb` is bimonthly: try the Monday's month, then the previous month.
pub fn mwb_issue_candidates(monday: CivilDate) -> Vec<String> {
    let first = issue_yyyymm(monday);
    match previous_month_issue(&first) {
        Ok(prev) => vec![first, prev],
        Err(_) => vec![first],
    }
}

/// Watchtower study issue is monthly.
pub fn w_issue(monday: CivilDate) -> String {
    issue_yyyymm(monday)
}

/// Normalises `20260900` / `202609` to `YYYYMM`.
pub fn normalize_issue_tag(tag: i64) -> String {
    let raw = tag.to_string();
    if raw.len() >= 6 {
        raw[..6].to_string()
    } else {
        raw
    }
}

/// `week/{langwritten}/{yyyy-mm-dd}` under the media root.
pub fn week_dir(media_root: &Path, langwritten: &str, monday: CivilDate) -> PathBuf {
    media_root
        .join("week")
        .join(langwritten)
        .join(monday.to_iso())
}

/// True when this catalog row should be downloaded into `week/` (not hymnal).
pub fn should_fetch_to_week(item: &MediaItem) -> bool {
    matches!(item.status, MediaStatus::Pending | MediaStatus::Failed)
        && item.media_kind != MediaKind::Song
        && matches!(item.media_ref, MediaRef::Catalog { .. })
}

fn iso_weekday(unix_days: i32) -> u8 {
    let v = (unix_days.rem_euclid(7) + 3).rem_euclid(7) + 1;
    v as u8
}

fn unix_days(date: CivilDate) -> i32 {
    days_from_civil(date.year, date.month, date.day) - 719_468
}

fn civil_from_unix_days(unix_days: i32) -> CivilDate {
    civil_from_days(unix_days + 719_468)
}

/// Howard Hinnant's public-domain days_from_civil.
fn days_from_civil(mut y: i32, m: u8, d: u8) -> i32 {
    y -= i32::from(m <= 2);
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = (y - era * 400) as u32;
    let m = u32::from(m);
    let mp = if m > 2 { m - 3 } else { m + 9 };
    let doy = (153 * mp + 2) / 5 + u32::from(d) - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe as i32
}

fn civil_from_days(z: i32) -> CivilDate {
    let z = z as i64;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    CivilDate {
        year,
        month: m as u8,
        day: d as u8,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn monday_of_wednesday_is_that_week() {
        let wed = CivilDate {
            year: 2026,
            month: 9,
            day: 9,
        };
        assert_eq!(monday_of(wed).to_iso(), "2026-09-07");
        assert_eq!(monday_of(wed).to_yyyymmdd(), 20260907);
    }

    #[test]
    fn next_week_is_plus_seven() {
        let today = CivilDate {
            year: 2026,
            month: 9,
            day: 16,
        };
        assert_eq!(monday_for(WeekWhich::This, today).to_iso(), "2026-09-14");
        assert_eq!(monday_for(WeekWhich::Next, today).to_iso(), "2026-09-21");
    }

    #[test]
    fn mwb_retries_previous_month() {
        let monday = CivilDate {
            year: 2026,
            month: 9,
            day: 7,
        };
        assert_eq!(mwb_issue_candidates(monday), vec!["202609", "202608"]);
        assert_eq!(w_issue(monday), "202609");
        assert_eq!(previous_month_issue("202601").expect("jan"), "202512");
    }

    #[test]
    fn issue_tag_strips_trailing_zeros_pair() {
        assert_eq!(normalize_issue_tag(20260900), "202609");
        assert_eq!(normalize_issue_tag(202609), "202609");
    }

    #[test]
    fn sjjm_is_not_fetched_into_week() {
        let song = MediaItem {
            id: "sjjm:1".into(),
            title: "1".into(),
            media_kind: MediaKind::Song,
            status: MediaStatus::PendingHymnal,
            mime: "video/mp4".into(),
            media_ref: MediaRef::Catalog {
                key_symbol: "sjjm".into(),
                track: 1,
                lang_meps: 1,
                issue_tag: 0,
                mime: "video/mp4".into(),
            },
            cache_path: None,
        };
        assert!(!should_fetch_to_week(&song));
        let video = MediaItem {
            id: "mwbv:1".into(),
            title: "Video".into(),
            media_kind: MediaKind::Video,
            status: MediaStatus::Pending,
            mime: "video/mp4".into(),
            media_ref: MediaRef::Catalog {
                key_symbol: "mwbv".into(),
                track: 1,
                lang_meps: 1,
                issue_tag: 20260900,
                mime: "video/mp4".into(),
            },
            cache_path: None,
        };
        assert!(should_fetch_to_week(&video));
    }

    #[test]
    fn epoch_thursday() {
        let epoch = CivilDate {
            year: 1970,
            month: 1,
            day: 1,
        };
        assert_eq!(iso_weekday(unix_days(epoch)), 4);
        assert_eq!(monday_of(epoch).to_iso(), "1969-12-29");
    }
}
