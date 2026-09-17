//! System and user event templates for the meeting timer.
//!
//! System rows are compiled in. User rows live in SQLite. Applying a template
//! MUST NOT decrypt JWPUB ciphertext, MUST NOT open media, and MUST NOT put
//! song rows on the timer.

use serde::{Deserialize, Serialize};

use crate::domain::week::{MediaItem, MeetingKind, MeetingPart, MeetingWeek};
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

struct Labels {
    opening_comments: &'static str,
    treasures: &'static str,
    gems: &'static str,
    reading: &'static str,
    ayf: &'static str,
    living: &'static str,
    cbs: &'static str,
    review: &'static str,
    circuit_review: &'static str,
    circuit_intro: &'static str,
    service_talk: &'static str,
    public_talk: &'static str,
    public_talk_co: &'static str,
    watchtower: &'static str,
    tpl_midweek: &'static str,
    tpl_weekend: &'static str,
    tpl_circuit_mw: &'static str,
    tpl_circuit_we: &'static str,
}

const EN: Labels = Labels {
    opening_comments: "Opening comments",
    treasures: "Treasures from God's Word",
    gems: "Spiritual Gems",
    reading: "Bible Reading",
    ayf: "Apply Yourself",
    living: "Living as Christians",
    cbs: "Congregation Bible Study",
    review: "Review / announcements",
    circuit_review: "Review, next week, announcements",
    circuit_intro: "Introduction of the circuit overseer",
    service_talk: "Service talk",
    public_talk: "Public talk",
    public_talk_co: "Public talk (circuit overseer)",
    watchtower: "Watchtower Study",
    tpl_midweek: "Midweek",
    tpl_weekend: "Weekend",
    tpl_circuit_mw: "Circuit visit — midweek",
    tpl_circuit_we: "Circuit visit — weekend",
};

const ES: Labels = Labels {
    opening_comments: "Palabras de introducción",
    treasures: "Tesoros de la Biblia",
    gems: "Busquemos perlas escondidas",
    reading: "Lectura de la Biblia",
    ayf: "Asignación",
    living: "Nuestra vida cristiana",
    cbs: "Estudio bíblico de congregación",
    review: "Recapitulación / anuncios",
    circuit_review: "Recapitulación, próxima semana, anuncios",
    circuit_intro: "Presentación del superintendente",
    service_talk: "Discurso de servicio",
    public_talk: "Discurso público",
    public_talk_co: "Discurso público (superintendente)",
    watchtower: "Estudio de La Atalaya",
    tpl_midweek: "Entre semana",
    tpl_weekend: "Fin de semana",
    tpl_circuit_mw: "Visita de circuito — entre semana",
    tpl_circuit_we: "Visita de circuito — fin de semana",
};

fn labels_for(langwritten: &str) -> &'static Labels {
    if langwritten.eq_ignore_ascii_case("S") {
        &ES
    } else {
        &EN
    }
}

/// All four system templates, in display order, titled for `langwritten`.
pub fn system_templates(langwritten: &str) -> Vec<EventTemplate> {
    let labels = labels_for(langwritten);
    vec![
        EventTemplate {
            id: SYS_MIDWEEK.into(),
            name: labels.tpl_midweek.into(),
            source: TemplateSource::System,
            kind: TemplateKind::Midweek,
            parts: midweek_parts("sys-mw", labels),
        },
        EventTemplate {
            id: SYS_WEEKEND.into(),
            name: labels.tpl_weekend.into(),
            source: TemplateSource::System,
            kind: TemplateKind::Weekend,
            parts: weekend_parts("sys-we", labels),
        },
        EventTemplate {
            id: SYS_CIRCUIT_MIDWEEK.into(),
            name: labels.tpl_circuit_mw.into(),
            source: TemplateSource::System,
            kind: TemplateKind::Midweek,
            parts: circuit_midweek_parts("sys-cmw", labels),
        },
        EventTemplate {
            id: SYS_CIRCUIT_WEEKEND.into(),
            name: labels.tpl_circuit_we.into(),
            source: TemplateSource::System,
            kind: TemplateKind::Weekend,
            parts: circuit_weekend_parts("sys-cwe", labels),
        },
    ]
}

/// Looks up a system template by id.
pub fn system_template(id: &str, langwritten: &str) -> Option<EventTemplate> {
    system_templates(langwritten)
        .into_iter()
        .find(|t| t.id == id)
}

/// Midweek / weekend skeleton used when `Document.Content` is not HTML.
pub fn skeleton_parts(kind: MeetingKind, prefix: &str, langwritten: &str) -> Vec<MeetingPart> {
    let labels = labels_for(langwritten);
    match kind {
        MeetingKind::Midweek => midweek_parts(prefix, labels),
        MeetingKind::Weekend => weekend_parts(prefix, labels),
    }
}

/// Fills empty `parts` from the system skeleton. Drops song rows.
pub fn ensure_outline(week: &mut MeetingWeek) {
    if week.parts.is_empty() {
        let prefix = format!("{}-{}", week.pub_symbol, week.monday);
        week.parts = skeleton_parts(week.kind, &prefix, &week.langwritten);
    }
    strip_songs(week);
}

/// Replaces `week.parts` with `template`. Song rows are dropped.
pub fn apply_template(week: &mut MeetingWeek, template: &EventTemplate) {
    let stash = collect_media(week);
    if template.id == SYS_CIRCUIT_MIDWEEK && !week.parts.is_empty() {
        week.parts = circuit_from_midweek(&week.parts, &week.langwritten);
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
    week.media = leftover_media(&week.parts, stash);
    strip_songs(week);
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

fn ayf_title(labels: &Labels, n: u32) -> String {
    format!("{} {n}", labels.ayf)
}

fn midweek_parts(prefix: &str, labels: &Labels) -> Vec<MeetingPart> {
    vec![
        part(&format!("{prefix}:0"), labels.opening_comments, 1, "other"),
        part(&format!("{prefix}:1"), labels.treasures, 10, "treasures"),
        part(&format!("{prefix}:2"), labels.gems, 10, "treasures"),
        part(&format!("{prefix}:3"), labels.reading, 4, "treasures"),
        part(&format!("{prefix}:4"), &ayf_title(labels, 1), 3, "ayf"),
        part(&format!("{prefix}:5"), &ayf_title(labels, 2), 3, "ayf"),
        part(&format!("{prefix}:6"), &ayf_title(labels, 3), 3, "ayf"),
        part(&format!("{prefix}:7"), &ayf_title(labels, 4), 3, "ayf"),
        part(&format!("{prefix}:8"), labels.living, 15, "living"),
        part(&format!("{prefix}:9"), labels.cbs, 30, "living"),
        part(&format!("{prefix}:10"), labels.review, 3, "living"),
    ]
}

fn weekend_parts(prefix: &str, labels: &Labels) -> Vec<MeetingPart> {
    vec![
        part(&format!("{prefix}:0"), labels.public_talk, 30, "other"),
        part(&format!("{prefix}:1"), labels.watchtower, 60, "living"),
    ]
}

fn circuit_midweek_parts(prefix: &str, labels: &Labels) -> Vec<MeetingPart> {
    let mut parts = midweek_parts(prefix, labels);
    parts.retain(|p| !is_cbs(&p.title) && !is_review(&p.title));
    parts.extend(circuit_block(prefix, labels));
    parts
}

fn circuit_weekend_parts(prefix: &str, labels: &Labels) -> Vec<MeetingPart> {
    vec![
        part(&format!("{prefix}:0"), labels.public_talk_co, 30, "other"),
        part(&format!("{prefix}:1"), labels.watchtower, 60, "living"),
    ]
}

fn circuit_block(prefix: &str, labels: &Labels) -> Vec<MeetingPart> {
    vec![
        part(&format!("{prefix}:r"), labels.circuit_review, 3, "living"),
        part(&format!("{prefix}:i"), labels.circuit_intro, 1, "living"),
        part(&format!("{prefix}:s"), labels.service_talk, 30, "living"),
    ]
}

fn circuit_from_midweek(existing: &[MeetingPart], langwritten: &str) -> Vec<MeetingPart> {
    let labels = labels_for(langwritten);
    let mut kept: Vec<MeetingPart> = existing
        .iter()
        .filter(|p| p.tone != "song" && !is_cbs(&p.title) && !is_review(&p.title))
        .cloned()
        .collect();
    if kept.is_empty() {
        return circuit_midweek_parts("sys-cmw", labels);
    }
    kept.extend(circuit_block("sys-cmw", labels));
    kept
}

fn is_cbs(title: &str) -> bool {
    let t = title.to_ascii_lowercase();
    t.contains("congregation bible")
        || t.contains("estudio biblico")
        || t.contains("estudio bíblico")
}

fn is_review(title: &str) -> bool {
    let t = title.to_ascii_lowercase();
    (t.contains("review") || t.contains("recapitul"))
        && (t.contains("announcement") || t.contains("anuncio") || t.contains("proxima") || t.contains("próxima") || t.contains("/"))
}

fn strip_songs(week: &mut MeetingWeek) {
    let mut extra = Vec::new();
    week.parts.retain(|part| {
        if part.tone == "song" {
            extra.extend(part.items.iter().cloned());
            false
        } else {
            true
        }
    });
    week.media.extend(extra);
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
    use crate::domain::week::{CivilDate, MediaKind, MediaRef, MediaStatus, MeetingKind, MeetingWeek};

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
    fn spanish_skeleton_has_no_songs() {
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
            .any(|p| p.title.contains("Tesoros") && p.minutes == Some(10)));
        assert!(week.parts.iter().any(|p| p.title.contains("perlas")));
        assert!(!week.parts.iter().any(|p| p.tone == "song"));
        assert!(!week.parts.iter().any(|p| p.title.to_ascii_lowercase().contains("song")));
        assert!(week.media.iter().any(|i| i.media_kind == MediaKind::Song));
    }

    #[test]
    fn english_skeleton_when_not_spanish() {
        let parts = skeleton_parts(MeetingKind::Midweek, "x", "E");
        assert!(parts.iter().any(|p| p.title.contains("Treasures")));
        assert!(!parts.iter().any(|p| p.tone == "song"));
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
        week.parts = midweek_parts("mw", labels_for("S"));
        let tpl = system_template(SYS_CIRCUIT_MIDWEEK, "S").expect("tpl");
        apply_template(&mut week, &tpl);
        assert!(!week.parts.iter().any(|p| is_cbs(&p.title)));
        assert!(week
            .parts
            .iter()
            .any(|p| p.title.contains("servicio") && p.minutes == Some(30)));
        assert!(week.parts.iter().any(|p| p.title.contains("Tesoros")));
    }

    #[test]
    fn rejects_empty_template_name() {
        assert!(validate_template_name("").is_err());
        assert!(validate_template_name("Memorial").is_ok());
    }
}
