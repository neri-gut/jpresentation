//! System and user event templates for the meeting timer.
//!
//! System rows are compiled in. User rows live in SQLite. Applying a template
//! MUST NOT decrypt JWPUB ciphertext and MUST NOT open media.

use serde::{Deserialize, Serialize};

use crate::domain::week::{MediaItem, MediaKind, MediaRef, MeetingKind, MeetingPart, MeetingWeek};
use crate::error::AppError;

/// Where a template came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateSource {
    System,
    User,
}

/// Which meeting skeleton a template is for. `event` is a free-form user programme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TemplateKind {
    Midweek,
    Weekend,
    Event,
}

impl TemplateKind {
    /// Parses the IPC `kind` field.
    pub fn parse(raw: &str) -> Result<Self, AppError> {
        match raw {
            "midweek" => Ok(Self::Midweek),
            "weekend" => Ok(Self::Weekend),
            "event" => Ok(Self::Event),
            _ => Err(AppError::Invariant(
                "template kind must be midweek, weekend, or event".into(),
            )),
        }
    }
}

/// One saved or built-in programme.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventTemplate {
    pub id: String,
    pub name: String,
    pub source: TemplateSource,
    pub kind: TemplateKind,
    pub parts: Vec<MeetingPart>,
}

/// Built-in ids. User templates use UUIDs.
pub const SYS_MIDWEEK: &str = "sys:midweek";
pub const SYS_WEEKEND: &str = "sys:weekend";
pub const SYS_CIRCUIT_MIDWEEK: &str = "sys:circuit-midweek";
pub const SYS_CIRCUIT_WEEKEND: &str = "sys:circuit-weekend";

/// All four system templates, in display order.
pub fn system_templates() -> Vec<EventTemplate> {
    vec![
        EventTemplate {
            id: SYS_MIDWEEK.into(),
            name: "Midweek".into(),
            source: TemplateSource::System,
            kind: TemplateKind::Midweek,
            parts: midweek_parts("sys-mw"),
        },
        EventTemplate {
            id: SYS_WEEKEND.into(),
            name: "Weekend".into(),
            source: TemplateSource::System,
            kind: TemplateKind::Weekend,
            parts: weekend_parts("sys-we"),
        },
        EventTemplate {
            id: SYS_CIRCUIT_MIDWEEK.into(),
            name: "Circuit visit — midweek".into(),
            source: TemplateSource::System,
            kind: TemplateKind::Midweek,
            parts: circuit_midweek_parts("sys-cmw"),
        },
        EventTemplate {
            id: SYS_CIRCUIT_WEEKEND.into(),
            name: "Circuit visit — weekend".into(),
            source: TemplateSource::System,
            kind: TemplateKind::Weekend,
            parts: circuit_weekend_parts("sys-cwe"),
        },
    ]
}

/// Looks up a system template by id.
pub fn system_template(id: &str) -> Option<EventTemplate> {
    system_templates().into_iter().find(|t| t.id == id)
}

/// Midweek / weekend skeleton used when `Document.Content` is not HTML.
pub fn skeleton_parts(kind: MeetingKind, prefix: &str) -> Vec<MeetingPart> {
    match kind {
        MeetingKind::Midweek => midweek_parts(prefix),
        MeetingKind::Weekend => weekend_parts(prefix),
    }
}

/// Fills empty `parts` from the system skeleton and overlays `sjjm` tracks.
pub fn ensure_outline(week: &mut MeetingWeek) {
    if week.parts.is_empty() {
        let prefix = format!("{}-{}", week.pub_symbol, week.monday);
        week.parts = skeleton_parts(week.kind, &prefix);
    }
    let stash = std::mem::take(&mut week.media);
    overlay_songs(&mut week.parts, &stash);
    week.media = leftover_media(&week.parts, stash);
}

/// Replaces `week.parts` with `template`, then overlays songs from existing media.
pub fn apply_template(week: &mut MeetingWeek, template: &EventTemplate) {
    let stash = collect_media(week);
    if template.id == SYS_CIRCUIT_MIDWEEK && !week.parts.is_empty() {
        week.parts = circuit_from_midweek(&week.parts);
    } else {
        week.parts = template
            .parts
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, mut part)| {
                if part.id.is_empty() || part.id.starts_with("sys:") {
                    part.id = format!("{}:{i}", template.id);
                }
                part.items.clear();
                part
            })
            .collect();
    }
    overlay_songs(&mut week.parts, &stash);
    week.media = leftover_media(&week.parts, stash);
}

/// Validates a user-edited row list before persist.
pub fn validate_parts(parts: &[MeetingPart]) -> Result<(), AppError> {
    if parts.len() > 40 {
        return Err(AppError::Invariant("at most 40 parts".into()));
    }
    for part in parts {
        let title = part.title.trim();
        if title.is_empty() || title.len() > 80 {
            return Err(AppError::Invariant("part title must be 1-80 characters".into()));
        }
        if let Some(minutes) = part.minutes {
            if !(1..=180).contains(&minutes) {
                return Err(AppError::Invariant("minutes must be 1-180".into()));
            }
        }
        if !is_tone(&part.tone) {
            return Err(AppError::Invariant(
                "tone must be treasures, ayf, living, song, or other".into(),
            ));
        }
    }
    Ok(())
}

/// Validates a user template name.
pub fn validate_template_name(name: &str) -> Result<(), AppError> {
    let name = name.trim();
    if name.is_empty() || name.len() > 80 {
        return Err(AppError::Invariant("template name must be 1-80 characters".into()));
    }
    Ok(())
}

fn is_tone(tone: &str) -> bool {
    matches!(tone, "treasures" | "ayf" | "living" | "song" | "other")
}

fn part(id: &str, title: &str, minutes: u32, tone: &str) -> MeetingPart {
    MeetingPart {
        id: id.into(),
        title: title.into(),
        minutes: Some(minutes),
        tone: tone.into(),
        items: Vec::new(),
    }
}

fn midweek_parts(prefix: &str) -> Vec<MeetingPart> {
    vec![
        part(&format!("{prefix}:0"), "Opening song + prayer", 5, "song"),
        part(&format!("{prefix}:1"), "Opening comments", 1, "other"),
        part(
            &format!("{prefix}:2"),
            "Treasures from God's Word",
            10,
            "treasures",
        ),
        part(&format!("{prefix}:3"), "Spiritual Gems", 10, "treasures"),
        part(&format!("{prefix}:4"), "Bible Reading", 4, "treasures"),
        part(&format!("{prefix}:5"), "Apply Yourself 1", 3, "ayf"),
        part(&format!("{prefix}:6"), "Apply Yourself 2", 3, "ayf"),
        part(&format!("{prefix}:7"), "Apply Yourself 3", 3, "ayf"),
        part(&format!("{prefix}:8"), "Apply Yourself 4", 3, "ayf"),
        part(&format!("{prefix}:9"), "Middle song", 3, "song"),
        part(
            &format!("{prefix}:10"),
            "Living as Christians",
            15,
            "living",
        ),
        part(
            &format!("{prefix}:11"),
            "Congregation Bible Study",
            30,
            "living",
        ),
        part(
            &format!("{prefix}:12"),
            "Review / announcements",
            3,
            "living",
        ),
        part(&format!("{prefix}:13"), "Closing song + prayer", 5, "song"),
    ]
}

fn weekend_parts(prefix: &str) -> Vec<MeetingPart> {
    vec![
        part(&format!("{prefix}:0"), "Opening song + prayer", 5, "song"),
        part(&format!("{prefix}:1"), "Public talk", 30, "other"),
        part(&format!("{prefix}:2"), "Middle song", 3, "song"),
        part(&format!("{prefix}:3"), "Watchtower Study", 60, "living"),
        part(&format!("{prefix}:4"), "Closing song + prayer", 5, "song"),
    ]
}

fn circuit_midweek_parts(prefix: &str) -> Vec<MeetingPart> {
    let mut parts = midweek_parts(prefix);
    parts.retain(|p| !is_cbs(&p.title));
    // Drop the generic review; circuit has its own recap + overseer block.
    parts.retain(|p| !is_review(&p.title));
    let closing = parts
        .iter()
        .rposition(|p| p.tone == "song")
        .unwrap_or(parts.len().saturating_sub(1));
    let extra = vec![
        part(
            &format!("{prefix}:r"),
            "Review, next week, announcements",
            3,
            "living",
        ),
        part(
            &format!("{prefix}:i"),
            "Introduction of the circuit overseer",
            1,
            "living",
        ),
        part(&format!("{prefix}:s"), "Service talk", 30, "living"),
    ];
    parts.splice(closing..closing, extra);
    parts
}

fn circuit_weekend_parts(prefix: &str) -> Vec<MeetingPart> {
    let mut parts = weekend_parts(prefix);
    if let Some(talk) = parts.iter_mut().find(|p| p.title == "Public talk") {
        talk.title = "Public talk (circuit overseer)".into();
    }
    parts
}

fn circuit_from_midweek(existing: &[MeetingPart]) -> Vec<MeetingPart> {
    let mut kept: Vec<MeetingPart> = existing
        .iter()
        .filter(|p| !is_cbs(&p.title) && !is_review(&p.title))
        .cloned()
        .collect();
    if kept.is_empty() {
        return circuit_midweek_parts("sys-cmw");
    }
    let closing = kept
        .iter()
        .rposition(|p| p.tone == "song")
        .unwrap_or(kept.len());
    let extra = vec![
        part(
            "sys-cmw:r",
            "Review, next week, announcements",
            3,
            "living",
        ),
        part(
            "sys-cmw:i",
            "Introduction of the circuit overseer",
            1,
            "living",
        ),
        part("sys-cmw:s", "Service talk", 30, "living"),
    ];
    kept.splice(closing..closing, extra);
    kept
}

fn is_cbs(title: &str) -> bool {
    let t = title.to_ascii_lowercase();
    t.contains("congregation bible") || t.contains("estudio bíblico") || t.contains("estudio biblico")
}

fn is_review(title: &str) -> bool {
    let t = title.to_ascii_lowercase();
    t.contains("review") || t.contains("recapitul") || t.contains("announcement")
}

fn overlay_songs(parts: &mut [MeetingPart], items: &[MediaItem]) {
    let songs: Vec<&MediaItem> = items
        .iter()
        .filter(|item| {
            item.media_kind == MediaKind::Song
                || matches!(
                    &item.media_ref,
                    MediaRef::Catalog { key_symbol, .. }
                        if key_symbol.eq_ignore_ascii_case("sjjm")
                )
        })
        .collect();
    if songs.is_empty() {
        return;
    }
    let mut i = 0usize;
    for part in parts.iter_mut() {
        if part.tone != "song" {
            continue;
        }
        if i >= songs.len() {
            break;
        }
        let song = songs[i];
        i += 1;
        if let MediaRef::Catalog { track, .. } = song.media_ref {
            if (1..=163).contains(&track) {
                part.title = format!("Song {track}");
            }
        }
        if !part.items.iter().any(|item| item.id == song.id) {
            part.items.push(song.clone());
        }
    }
}

fn collect_media(week: &MeetingWeek) -> Vec<MediaItem> {
    let mut out = Vec::new();
    for part in &week.parts {
        out.extend(part.items.iter().cloned());
    }
    out.extend(week.media.iter().cloned());
    out
}

fn leftover_media(parts: &[MeetingPart], items: Vec<MediaItem>) -> Vec<MediaItem> {
    let used: Vec<&str> = parts
        .iter()
        .flat_map(|p| p.items.iter().map(|i| i.id.as_str()))
        .collect();
    items
        .into_iter()
        .filter(|item| !used.contains(&item.id.as_str()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::week::{CivilDate, MediaStatus, MeetingKind, MeetingWeek};

    fn song_item(track: u32) -> MediaItem {
        MediaItem {
            id: format!("sjjm:{track}"),
            title: format!("{track}"),
            media_kind: MediaKind::Song,
            status: MediaStatus::PendingHymnal,
            mime: "video/mp4".into(),
            media_ref: MediaRef::Catalog {
                key_symbol: "sjjm".into(),
                track,
                lang_meps: 1,
                issue_tag: 0,
                mime: "video/mp4".into(),
            },
            cache_path: None,
        }
    }

    #[test]
    fn ciphertext_week_gets_midweek_skeleton_and_songs() {
        let monday = CivilDate {
            year: 2026,
            month: 9,
            day: 14,
        };
        let mut week = MeetingWeek::empty(monday, MeetingKind::Midweek, "S");
        week.media = vec![song_item(1), song_item(128), song_item(143)];
        ensure_outline(&mut week);
        assert!(week
            .parts
            .iter()
            .any(|p| p.title.contains("Treasures") && p.minutes == Some(10)));
        assert!(week.parts.iter().any(|p| p.title == "Song 1"));
        assert!(week.parts.iter().any(|p| p.title == "Song 128"));
        assert!(week.parts.iter().any(|p| p.title == "Song 143"));
        assert!(week.parts.iter().any(|p| p.title.contains("Congregation")));
        assert!(!week.parts.iter().any(|p| p.title == "Media"));
    }

    #[test]
    fn html_parts_are_not_replaced() {
        let monday = CivilDate {
            year: 2026,
            month: 9,
            day: 14,
        };
        let mut week = MeetingWeek::empty(monday, MeetingKind::Midweek, "S");
        week.parts = vec![part("a", "Treasures from God's Word", 10, "treasures")];
        ensure_outline(&mut week);
        assert_eq!(week.parts.len(), 1);
        assert_eq!(week.parts[0].title, "Treasures from God's Word");
    }

    #[test]
    fn circuit_drops_cbs_and_inserts_service_talk() {
        let mut week = MeetingWeek::empty(
            CivilDate {
                year: 2026,
                month: 9,
                day: 14,
            },
            MeetingKind::Midweek,
            "S",
        );
        week.parts = midweek_parts("mw");
        let tpl = system_template(SYS_CIRCUIT_MIDWEEK).expect("tpl");
        apply_template(&mut week, &tpl);
        assert!(!week.parts.iter().any(|p| is_cbs(&p.title)));
        assert!(week
            .parts
            .iter()
            .any(|p| p.title.contains("Service talk") && p.minutes == Some(30)));
        assert!(week.parts.iter().any(|p| p.title.contains("Treasures")));
    }

    #[test]
    fn rejects_empty_template_name() {
        assert!(validate_template_name("").is_err());
        assert!(validate_template_name("Memorial").is_ok());
    }
}
