//! Nested-ZIP JWPUB reader and schedule parser.
//!
//! Opens `manifest.json` + `contents` ZIP, then SQLite if the header is
//! `SQLite format 3`. MUST NOT decrypt ciphertext `Document.Content`.

use std::io::{Cursor, Read};
use std::path::Path;

use rusqlite::Connection;
use zip::ZipArchive;

use crate::domain::week::{
    add_days, normalize_issue_tag, CivilDate, MediaItem, MediaKind, MediaRef, MediaStatus,
    MeetingKind, MeetingPart, MeetingWeek,
};
use crate::error::AppError;

const SQLITE_MAGIC: &[u8] = b"SQLite format 3\0";

/// One publication parsed from a `.jwpub` (several calendar weeks).
#[derive(Debug, Clone)]
pub struct ParsedPublication {
    pub symbol: String,
    pub issue: String,
    pub weeks: Vec<MeetingWeek>,
}

/// Reads a `.jwpub` byte blob and extracts embedded files into `img_dir`.
pub fn parse_jwpub(
    bytes: &[u8],
    kind: MeetingKind,
    langwritten: &str,
    img_dir: &Path,
) -> Result<ParsedPublication, AppError> {
    let mut outer = ZipArchive::new(Cursor::new(bytes)).map_err(|_| AppError::UnreadablePub)?;
    let mut contents_bytes = Vec::new();
    {
        let mut contents = outer
            .by_name("contents")
            .map_err(|_| AppError::UnreadablePub)?;
        contents
            .read_to_end(&mut contents_bytes)
            .map_err(|e| AppError::Io(e.to_string()))?;
    }
    let mut inner =
        ZipArchive::new(Cursor::new(contents_bytes)).map_err(|_| AppError::UnreadablePub)?;
    let db_name = inner_db_name(&mut inner)?;
    let db_bytes = read_zip_file(&mut inner, &db_name)?;
    if !db_bytes.starts_with(SQLITE_MAGIC) {
        return Err(AppError::UnreadablePub);
    }
    std::fs::create_dir_all(img_dir).map_err(|e| AppError::Io(e.to_string()))?;
    let tmp = img_dir.join(".pub.db");
    std::fs::write(&tmp, &db_bytes).map_err(|e| AppError::Io(e.to_string()))?;
    let parsed = {
        let conn = Connection::open(&tmp)?;
        parse_sqlite(&conn, kind, langwritten, &mut inner, img_dir)?
    };
    let _ = std::fs::remove_file(&tmp);
    Ok(parsed)
}

const MEDIA_EXTS: &[&str] = &["jpg", "jpeg", "png", "webp", "gif", "mp4", "webm"];

/// Extracts only image/video files from the inner ZIP into `dest`. Skips `.db`.
pub fn extract_jwpub_media(bytes: &[u8], dest: &Path) -> Result<u32, AppError> {
    let mut outer = ZipArchive::new(Cursor::new(bytes)).map_err(|_| AppError::UnreadablePub)?;
    let mut contents_bytes = Vec::new();
    {
        let mut contents = outer
            .by_name("contents")
            .map_err(|_| AppError::UnreadablePub)?;
        contents
            .read_to_end(&mut contents_bytes)
            .map_err(|e| AppError::Io(e.to_string()))?;
    }
    let mut inner =
        ZipArchive::new(Cursor::new(contents_bytes)).map_err(|_| AppError::UnreadablePub)?;
    std::fs::create_dir_all(dest).map_err(|e| AppError::Io(e.to_string()))?;
    let mut count = 0u32;
    for i in 0..inner.len() {
        let mut file = inner.by_index(i).map_err(|_| AppError::UnreadablePub)?;
        if file.is_dir() {
            continue;
        }
        let name = file
            .name()
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(file.name())
            .to_string();
        let ext = name
            .rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        if !MEDIA_EXTS.contains(&ext.as_str()) {
            continue;
        }
        let mut buf = Vec::new();
        file.read_to_end(&mut buf)
            .map_err(|e| AppError::Io(e.to_string()))?;
        std::fs::write(dest.join(&name), buf).map_err(|e| AppError::Io(e.to_string()))?;
        count = count.saturating_add(1);
    }
    Ok(count)
}

/// Picks the week whose date range covers `monday`..=Sunday.
pub fn week_covering(pubn: &ParsedPublication, monday: CivilDate) -> Option<&MeetingWeek> {
    let start = monday.to_yyyymmdd();
    let end = add_days(monday, 6).to_yyyymmdd();
    pubn.weeks.iter().find(|week| {
        let Ok(week_monday) = CivilDate::parse_iso(&week.monday) else {
            return false;
        };
        let first = week_monday.to_yyyymmdd();
        let last = add_days(week_monday, 6).to_yyyymmdd();
        first <= end && last >= start
    })
}

fn inner_db_name<R: Read + std::io::Seek>(zip: &mut ZipArchive<R>) -> Result<String, AppError> {
    for i in 0..zip.len() {
        let file = zip.by_index(i).map_err(|_| AppError::UnreadablePub)?;
        let name = file.name().to_string();
        if name.ends_with(".db") && !name.contains('/') && !name.contains('\\') {
            return Ok(name);
        }
    }
    Err(AppError::UnreadablePub)
}

fn read_zip_file<R: Read + std::io::Seek>(
    zip: &mut ZipArchive<R>,
    name: &str,
) -> Result<Vec<u8>, AppError> {
    let mut file = zip.by_name(name).map_err(|_| AppError::UnreadablePub)?;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .map_err(|e| AppError::Io(e.to_string()))?;
    Ok(buf)
}

fn parse_sqlite<R: Read + std::io::Seek>(
    conn: &Connection,
    kind: MeetingKind,
    langwritten: &str,
    zip: &mut ZipArchive<R>,
    img_dir: &Path,
) -> Result<ParsedPublication, AppError> {
    let (symbol, issue) = publication_meta(conn, kind)?;
    let mut dated = conn.prepare(
        "SELECT DocumentId, FirstDateOffset, LastDateOffset FROM DatedText ORDER BY FirstDateOffset",
    )?;
    let rows = dated.query_map([], |row| {
        Ok((
            row.get::<_, i64>(0)?,
            row.get::<_, i64>(1)?,
            row.get::<_, i64>(2)?,
        ))
    })?;
    let mut weeks = Vec::new();
    for row in rows {
        let (document_id, first, last) = row?;
        let Some(monday) = monday_from_offset(first) else {
            continue;
        };
        let _ = last;
        let title = document_title(conn, document_id).unwrap_or_else(|| monday.to_iso());
        let html = document_html(conn, document_id);
        let items = document_media(conn, document_id, langwritten, zip, img_dir)?;
        let parts = parts_from_html(html.as_deref(), &items, document_id);
        weeks.push(MeetingWeek {
            monday: monday.to_iso(),
            kind,
            title,
            langwritten: langwritten.to_string(),
            pub_symbol: symbol.clone(),
            issue: issue.clone(),
            parts,
        });
    }
    Ok(ParsedPublication { symbol, issue, weeks })
}

fn publication_meta(conn: &Connection, kind: MeetingKind) -> Result<(String, String), AppError> {
    let fallback = match kind {
        MeetingKind::Midweek => "mwb",
        MeetingKind::Weekend => "w",
    };
    let row = conn.query_row(
        "SELECT Symbol, IssueTagNumber FROM Publication LIMIT 1",
        [],
        |row| Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?)),
    );
    match row {
        Ok((symbol, tag)) => {
            let undated = if symbol.starts_with("mwb") {
                "mwb".into()
            } else if symbol.starts_with('w') {
                "w".into()
            } else {
                fallback.into()
            };
            Ok((undated, normalize_issue_tag(tag)))
        }
        Err(_) => Ok((fallback.into(), String::new())),
    }
}

fn document_title(conn: &Connection, document_id: i64) -> Option<String> {
    conn.query_row(
        "SELECT Title FROM Document WHERE DocumentId = ?1",
        [document_id],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .filter(|s| !s.trim().is_empty())
}

fn document_html(conn: &Connection, document_id: i64) -> Option<String> {
    let blob: Vec<u8> = conn
        .query_row(
            "SELECT Content FROM Document WHERE DocumentId = ?1",
            [document_id],
            |row| row.get(0),
        )
        .ok()?;
    let text = std::str::from_utf8(&blob).ok()?.trim();
    if looks_like_html(text) {
        Some(text.to_string())
    } else {
        None
    }
}

fn looks_like_html(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.starts_with('<') || lower.contains("<html") || lower.contains("<h1")
}

fn document_media<R: Read + std::io::Seek>(
    conn: &Connection,
    document_id: i64,
    langwritten: &str,
    zip: &mut ZipArchive<R>,
    img_dir: &Path,
) -> Result<Vec<MediaItem>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT m.MultimediaId, m.MimeType, m.FilePath, m.KeySymbol, m.Track,
                m.Label, m.Caption, m.MepsLanguageIndex, m.IssueTagNumber,
                dm.BeginParagraphOrdinal
         FROM DocumentMultimedia dm
         JOIN Multimedia m ON m.MultimediaId = dm.MultimediaId
         WHERE dm.DocumentId = ?1
         ORDER BY dm.BeginParagraphOrdinal, m.MultimediaId",
    )?;
    let rows = stmt.query_map([document_id], |row| {
        Ok(RawMedia {
            id: row.get(0)?,
            mime: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            file_path: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
            key_symbol: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
            track: row.get::<_, Option<i64>>(4)?.unwrap_or(0) as u32,
            label: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
            caption: row.get::<_, Option<String>>(6)?.unwrap_or_default(),
            lang_meps: row.get::<_, Option<i64>>(7)?.unwrap_or(0),
            issue_tag: row.get::<_, Option<i64>>(8)?.unwrap_or(0),
            paragraph: row.get::<_, Option<i64>>(9)?.unwrap_or(0),
        })
    })?;
    let mut items = Vec::new();
    for row in rows {
        let raw = row?;
        items.push(materialise_item(raw, document_id, langwritten, zip, img_dir)?);
    }
    Ok(items)
}

struct RawMedia {
    id: i64,
    mime: String,
    file_path: String,
    key_symbol: String,
    track: u32,
    label: String,
    caption: String,
    lang_meps: i64,
    issue_tag: i64,
    paragraph: i64,
}

fn materialise_item<R: Read + std::io::Seek>(
    raw: RawMedia,
    document_id: i64,
    _langwritten: &str,
    zip: &mut ZipArchive<R>,
    img_dir: &Path,
) -> Result<MediaItem, AppError> {
    let id = format!("{document_id}:{}", raw.id);
    let title = first_nonempty(&[&raw.label, &raw.caption, &raw.file_path, &raw.key_symbol]);
    let mime = raw.mime.clone();
    if !raw.file_path.is_empty() {
        let name = file_name(&raw.file_path);
        let dest = img_dir.join(&name);
        if let Ok(bytes) = read_zip_file(zip, &raw.file_path).or_else(|_| read_zip_file(zip, &name))
        {
            std::fs::write(&dest, bytes).map_err(|e| AppError::Io(e.to_string()))?;
        }
        let cache_path = dest.exists().then(|| dest.to_string_lossy().into_owned());
        return Ok(MediaItem {
            id,
            title,
            media_kind: MediaKind::Image,
            status: if cache_path.is_some() {
                MediaStatus::Embedded
            } else {
                MediaStatus::Failed
            },
            mime: if mime.is_empty() {
                "image/jpeg".into()
            } else {
                mime
            },
            media_ref: MediaRef::Embedded {
                path: raw.file_path,
            },
            cache_path,
        });
    }
    let is_song = raw.key_symbol.eq_ignore_ascii_case("sjjm");
    let media_kind = if is_song {
        MediaKind::Song
    } else if mime.starts_with("image/") {
        MediaKind::Image
    } else {
        MediaKind::Video
    };
    let status = if is_song {
        MediaStatus::PendingHymnal
    } else {
        MediaStatus::Pending
    };
    let title = if is_song && raw.track > 0 {
        format!("{}", raw.track)
    } else {
        title
    };
    let _ = raw.paragraph;
    Ok(MediaItem {
        id,
        title,
        media_kind,
        status,
        mime: if mime.is_empty() {
            "video/mp4".into()
        } else {
            mime
        },
        media_ref: MediaRef::Catalog {
            key_symbol: raw.key_symbol,
            track: raw.track,
            lang_meps: raw.lang_meps,
            issue_tag: raw.issue_tag,
            mime: raw.mime,
        },
        cache_path: None,
    })
}

fn parts_from_html(html: Option<&str>, items: &[MediaItem], document_id: i64) -> Vec<MeetingPart> {
    let mut parts: Vec<MeetingPart> = Vec::new();
    if let Some(html) = html {
        for track in extract_song_numbers(html) {
            if let Some(item) = items
                .iter()
                .find(|item| matches!(&item.media_ref, MediaRef::Catalog { key_symbol, track: t, .. } if key_symbol == "sjjm" && *t == track))
            {
                parts.push(MeetingPart {
                    id: format!("{document_id}:song:{track}"),
                    title: format!("Song {track}"),
                    minutes: Some(5),
                    items: vec![item.clone()],
                });
            }
        }
    }
    let used: Vec<String> = parts
        .iter()
        .flat_map(|part| part.items.iter().map(|item| item.id.clone()))
        .collect();
    let leftover: Vec<MediaItem> = items
        .iter()
        .filter(|item| !used.contains(&item.id))
        .cloned()
        .collect();
    if !leftover.is_empty() || parts.is_empty() {
        parts.push(MeetingPart {
            id: format!("{document_id}:media"),
            title: "Media".into(),
            minutes: None,
            items: leftover,
        });
    }
    parts
}

fn extract_song_numbers(html: &str) -> Vec<u32> {
    let lower = html.to_ascii_lowercase();
    let mut out = Vec::new();
    for key in ["song ", "canción ", "cancion ", "cántico ", "cantico "] {
        let mut rest = lower.as_str();
        while let Some(idx) = rest.find(key) {
            rest = &rest[idx + key.len()..];
            let digits: String = rest.chars().take_while(|c| c.is_ascii_digit()).collect();
            if let Ok(n) = digits.parse::<u32>() {
                if (1..=163).contains(&n) && !out.contains(&n) {
                    out.push(n);
                }
            }
        }
    }
    out
}

fn monday_from_offset(offset: i64) -> Option<CivilDate> {
    let raw = offset.to_string();
    if raw.len() != 8 {
        return None;
    }
    let year: i32 = raw[0..4].parse().ok()?;
    let month: u8 = raw[4..6].parse().ok()?;
    let day: u8 = raw[6..8].parse().ok()?;
    Some(CivilDate { year, month, day })
}

fn file_name(path: &str) -> String {
    path.rsplit(['/', '\\'])
        .next()
        .unwrap_or(path)
        .to_string()
}

fn first_nonempty(parts: &[&str]) -> String {
    parts
        .iter()
        .find(|s| !s.trim().is_empty())
        .map(|s| (*s).to_string())
        .unwrap_or_else(|| "Media".into())
}

/// Builds a nested ZIP + SQLite sample used by tests. Not an official publication.
#[cfg(test)]
pub fn sample_mwb_bytes() -> Vec<u8> {
    sample_pub_bytes(true)
}

#[cfg(test)]
pub fn sample_unreadable_bytes() -> Vec<u8> {
    use std::io::Write;
    use zip::write::FileOptions;
    use zip::ZipWriter;
    let mut outer = ZipWriter::new(Cursor::new(Vec::new()));
    let opts = FileOptions::default();
    outer.start_file("manifest.json", opts).expect("manifest");
    outer.write_all(b"{\"symbol\":\"mwb\"}").expect("json");
    let mut inner = ZipWriter::new(Cursor::new(Vec::new()));
    inner.start_file("mwb_S_202609.db", opts).expect("db");
    inner.write_all(b"NOTSQLITE").expect("garbage");
    let inner_bytes = inner.finish().expect("inner").into_inner();
    outer.start_file("contents", opts).expect("contents");
    outer.write_all(&inner_bytes).expect("write");
    outer.finish().expect("outer").into_inner()
}

#[cfg(test)]
fn sample_pub_bytes(with_html: bool) -> Vec<u8> {
    use std::io::Write;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    let dir = tempfile::tempdir().expect("tmp");
    let db_path = dir.path().join("mwb.db");
    {
        let conn = Connection::open(&db_path).expect("db");
        conn.execute_batch(
            r#"
            CREATE TABLE Publication (Symbol TEXT, IssueTagNumber INTEGER);
            INSERT INTO Publication VALUES ('mwb26', 20260900);
            CREATE TABLE Document (DocumentId INTEGER PRIMARY KEY, Class INTEGER, Title TEXT, Content BLOB);
            CREATE TABLE DatedText (DocumentId INTEGER, FirstDateOffset INTEGER, LastDateOffset INTEGER);
            CREATE TABLE Multimedia (
                MultimediaId INTEGER PRIMARY KEY, MimeType TEXT, FilePath TEXT, KeySymbol TEXT,
                Track INTEGER, Label TEXT, Caption TEXT, MepsLanguageIndex INTEGER, IssueTagNumber INTEGER
            );
            CREATE TABLE DocumentMultimedia (
                DocumentMultimediaId INTEGER PRIMARY KEY, DocumentId INTEGER, MultimediaId INTEGER,
                BeginParagraphOrdinal INTEGER
            );
            "#,
        )
        .expect("schema");
        let html = if with_html {
            b"<html><h1>September 7-13</h1><h3>Song 1</h3><p>Treasures (10 min)</p></html>".as_slice()
        } else {
            &[0xff, 0x00, 0xae, 0x11]
        };
        conn.execute(
            "INSERT INTO Document (DocumentId, Class, Title, Content) VALUES (1, 106, 'September 7-13, 2026', ?1)",
            [html],
        )
        .expect("doc");
        conn.execute(
            "INSERT INTO DatedText VALUES (1, 20260907, 20260913)",
            [],
        )
        .expect("dated");
        conn.execute(
            "INSERT INTO Multimedia VALUES (1, 'image/jpeg', 'test.jpg', '', 0, 'Illustration', '', 1, 0)",
            [],
        )
        .expect("img");
        conn.execute(
            "INSERT INTO Multimedia VALUES (2, 'video/mp4', '', 'mwbv', 1, 'Video 1', '', 1, 20260900)",
            [],
        )
        .expect("vid");
        conn.execute(
            "INSERT INTO Multimedia VALUES (3, 'video/mp4', '', 'sjjm', 1, 'Song 1', '', 1, 0)",
            [],
        )
        .expect("song");
        conn.execute("INSERT INTO DocumentMultimedia VALUES (1, 1, 1, 9)", [])
            .expect("dm1");
        conn.execute("INSERT INTO DocumentMultimedia VALUES (2, 1, 2, 31)", [])
            .expect("dm2");
        conn.execute("INSERT INTO DocumentMultimedia VALUES (3, 1, 3, 3)", [])
            .expect("dm3");
    }
    let db_bytes = std::fs::read(&db_path).expect("read db");
    let jpeg = minimal_jpeg();
    let opts = FileOptions::default();
    let mut inner = ZipWriter::new(Cursor::new(Vec::new()));
    inner.start_file("mwb_S_202609.db", opts).expect("db file");
    inner.write_all(&db_bytes).expect("db bytes");
    inner.start_file("test.jpg", opts).expect("jpg");
    inner.write_all(&jpeg).expect("jpg bytes");
    let inner_bytes = inner.finish().expect("inner").into_inner();
    let mut outer = ZipWriter::new(Cursor::new(Vec::new()));
    outer
        .start_file("manifest.json", opts)
        .expect("manifest");
    outer
        .write_all(br#"{"publication":{"symbol":"mwb26"}}"#)
        .expect("json");
    outer.start_file("contents", opts).expect("contents");
    outer.write_all(&inner_bytes).expect("contents bytes");
    outer.finish().expect("outer").into_inner()
}

#[cfg(test)]
fn minimal_jpeg() -> Vec<u8> {
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01, 0x01, 0x00, 0x00,
        0x01, 0x00, 0x01, 0x00, 0x00, 0xFF, 0xD9,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::week::CivilDate;

    #[test]
    fn parses_sample_week_and_extracts_jpeg() {
        let bytes = sample_mwb_bytes();
        let dir = tempfile::tempdir().expect("tmp");
        let img = dir.path().join("img");
        let parsed = parse_jwpub(&bytes, MeetingKind::Midweek, "S", &img).expect("parse");
        assert_eq!(parsed.symbol, "mwb");
        assert_eq!(parsed.issue, "202609");
        let monday = CivilDate {
            year: 2026,
            month: 9,
            day: 7,
        };
        let week = week_covering(&parsed, monday).expect("week");
        assert_eq!(week.monday, "2026-09-07");
        let items: Vec<&MediaItem> = week.parts.iter().flat_map(|p| p.items.iter()).collect();
        assert!(items.iter().any(|i| i.status == MediaStatus::Embedded
            && matches!(i.media_ref, MediaRef::Embedded { .. })));
        assert!(items.iter().any(|i| {
            matches!(
                &i.media_ref,
                MediaRef::Catalog { key_symbol, track, .. } if key_symbol == "mwbv" && *track == 1
            ) && i.status == MediaStatus::Pending
        }));
        assert!(items.iter().any(|i| {
            i.media_kind == MediaKind::Song && i.status == MediaStatus::PendingHymnal
        }));
        assert!(img.join("test.jpg").exists());
        assert!(week.parts.iter().any(|p| p.title.starts_with("Song")));
    }

    #[test]
    fn ciphertext_content_still_yields_media_folder() {
        let bytes = sample_pub_bytes(false);
        let dir = tempfile::tempdir().expect("tmp");
        let parsed =
            parse_jwpub(&bytes, MeetingKind::Midweek, "S", &dir.path().join("img")).expect("parse");
        let week = &parsed.weeks[0];
        assert!(week.parts.iter().any(|p| p.title == "Media"));
        assert!(
            week.parts
                .iter()
                .flat_map(|p| p.items.iter())
                .count()
                > 0
        );
    }

    #[test]
    fn extract_media_skips_sqlite() {
        let bytes = sample_mwb_bytes();
        let dir = tempfile::tempdir().expect("tmp");
        let dest = dir.path().join("out");
        let n = extract_jwpub_media(&bytes, &dest).expect("extract");
        assert_eq!(n, 1);
        assert!(dest.join("test.jpg").exists());
        assert!(!dest.join("mwb_S_202609.db").exists());
    }

    #[test]
    fn garbage_db_is_unreadable() {
        let bytes = sample_unreadable_bytes();
        let dir = tempfile::tempdir().expect("tmp");
        let err = parse_jwpub(&bytes, MeetingKind::Midweek, "S", &dir.path().join("img"))
            .expect_err("unreadable");
        assert!(matches!(err, AppError::UnreadablePub));
    }
}
