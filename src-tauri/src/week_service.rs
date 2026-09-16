//! Orchestrates catalog lookup, JWPUB parse, and `week/` cache. No Vue.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use serde::{Deserialize, Serialize};

use crate::catalog::md5_hex;
use crate::db::SqliteWeekStore;
use crate::domain::jwpub::{parse_jwpub, week_covering};
use crate::domain::media::{
    catalog_key_for_item, CatalogFormat, CatalogKey, MediaResolver, PublicationCatalog,
};
use crate::domain::template::{apply_template, ensure_outline, system_template, EventTemplate};
use crate::domain::week::{
    monday_for, mwb_issue_candidates, should_fetch_to_week, w_issue, week_dir, CivilDate,
    MediaStatus, MeetingKind, MeetingPart, MeetingWeek, WeekWhich,
};
use crate::error::AppError;

/// Event name for fetch/download progress. Operator only.
pub const WEEK_PROGRESS: &str = "week://progress";

/// Progress payload. `label` is a short phase name, never a user path.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekProgressDto {
    pub phase: String,
    pub done: u32,
    pub total: u32,
    pub label: String,
}

/// Both meetings for one Monday, as returned to Vue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekBundleDto {
    pub monday: String,
    pub langwritten: String,
    pub midweek: MeetingWeek,
    pub weekend: MeetingWeek,
}

/// Downloads (if needed) and parses this/next week's `mwb` + `w`.
pub fn fetch_week<C>(
    catalog: &C,
    media_root: &Path,
    langwritten: &str,
    which: WeekWhich,
    today: CivilDate,
    progress: &mut dyn FnMut(WeekProgressDto),
) -> Result<WeekBundleDto, AppError>
where
    C: PublicationCatalog + MediaResolver,
{
    let monday = monday_for(which, today);
    let dir = week_dir(media_root, langwritten, monday);
    fs::create_dir_all(dir.join("pub")).map_err(|e| AppError::Io(e.to_string()))?;
    fs::create_dir_all(dir.join("img")).map_err(|e| AppError::Io(e.to_string()))?;
    fs::create_dir_all(dir.join("vid")).map_err(|e| AppError::Io(e.to_string()))?;

    progress(WeekProgressDto {
        phase: "catalog".into(),
        done: 0,
        total: 2,
        label: "mwb".into(),
    });
    let midweek = fetch_pub(
        catalog,
        &dir,
        langwritten,
        monday,
        MeetingKind::Midweek,
        mwb_issue_candidates(monday),
        true,
    )?;
    progress(WeekProgressDto {
        phase: "catalog".into(),
        done: 1,
        total: 2,
        label: "w".into(),
    });
    let weekend = match fetch_pub(
        catalog,
        &dir,
        langwritten,
        monday,
        MeetingKind::Weekend,
        vec![w_issue(monday)],
        false,
    ) {
        Ok(week) => week,
        Err(AppError::CatalogNotFound) => MeetingWeek::empty(monday, MeetingKind::Weekend, langwritten),
        Err(err) => return Err(err),
    };
    progress(WeekProgressDto {
        phase: "parse".into(),
        done: 2,
        total: 2,
        label: "done".into(),
    });
    Ok(WeekBundleDto {
        monday: monday.to_iso(),
        langwritten: langwritten.to_string(),
        midweek,
        weekend,
    })
}

/// Writes both meetings for a Monday.
pub fn persist_bundle(
    store: &SqliteWeekStore<'_>,
    profile_id: &str,
    bundle: &WeekBundleDto,
) -> Result<(), AppError> {
    store.upsert(profile_id, &bundle.midweek)?;
    store.upsert(profile_id, &bundle.weekend)?;
    Ok(())
}

/// Loads weeks for a profile.
pub fn load_week_for_profile(
    store: &SqliteWeekStore<'_>,
    profile_id: &str,
    langwritten: &str,
    which: WeekWhich,
    today: CivilDate,
) -> Result<WeekBundleDto, AppError> {
    let monday = monday_for(which, today);
    let iso = monday.to_iso();
    let midweek = store
        .get(profile_id, &iso, MeetingKind::Midweek)?
        .unwrap_or_else(|| MeetingWeek::empty(monday, MeetingKind::Midweek, langwritten));
    let weekend = store
        .get(profile_id, &iso, MeetingKind::Weekend)?
        .unwrap_or_else(|| MeetingWeek::empty(monday, MeetingKind::Weekend, langwritten));
    Ok(WeekBundleDto {
        monday: iso,
        langwritten: langwritten.to_string(),
        midweek,
        weekend,
    })
}

/// Downloads pending catalog videos (not `sjjm`) into `week/.../vid/`.
pub fn download_week_media<C>(
    catalog: &C,
    media_root: &Path,
    bundle: &mut WeekBundleDto,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(WeekProgressDto),
) -> Result<WeekBundleDto, AppError>
where
    C: PublicationCatalog + MediaResolver,
{
    let monday = CivilDate::parse_iso(&bundle.monday)?;
    let vid_dir = week_dir(media_root, &bundle.langwritten, monday).join("vid");
    fs::create_dir_all(&vid_dir).map_err(|e| AppError::Io(e.to_string()))?;
    let pending: Vec<String> = {
        let mut ids = Vec::new();
        collect_pending_ids(&bundle.midweek, &mut ids);
        collect_pending_ids(&bundle.weekend, &mut ids);
        ids
    };
    let total = pending.len() as u32;
    let lang = bundle.langwritten.clone();
    for (index, id) in pending.iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            return Err(AppError::Cancelled);
        }
        let item = find_item_mut(bundle, id);
        let Some(item) = item else {
            continue;
        };
        progress(WeekProgressDto {
            phase: "download".into(),
            done: index as u32,
            total,
            label: item.title.clone(),
        });
        match download_one(catalog, item, &lang, &vid_dir) {
            Ok(()) => {}
            Err(AppError::Cancelled) => return Err(AppError::Cancelled),
            Err(_) => item.status = MediaStatus::Failed,
        }
    }
    progress(WeekProgressDto {
        phase: "download".into(),
        done: total,
        total,
        label: "done".into(),
    });
    Ok(bundle.clone())
}

fn collect_pending_ids(week: &MeetingWeek, ids: &mut Vec<String>) {
    for part in &week.parts {
        for item in &part.items {
            if should_fetch_to_week(item) {
                ids.push(item.id.clone());
            }
        }
    }
    for item in &week.media {
        if should_fetch_to_week(item) {
            ids.push(item.id.clone());
        }
    }
}

fn find_item_mut<'a>(
    bundle: &'a mut WeekBundleDto,
    id: &str,
) -> Option<&'a mut crate::domain::week::MediaItem> {
    for week in [&mut bundle.midweek, &mut bundle.weekend] {
        for part in &mut week.parts {
            if let Some(item) = part.items.iter_mut().find(|item| item.id == id) {
                return Some(item);
            }
        }
        if let Some(item) = week.media.iter_mut().find(|item| item.id == id) {
            return Some(item);
        }
    }
    None
}

fn download_one<C>(
    catalog: &C,
    item: &mut crate::domain::week::MediaItem,
    langwritten: &str,
    vid_dir: &Path,
) -> Result<(), AppError>
where
    C: PublicationCatalog + MediaResolver,
{
    let Some(key) = catalog_key_for_item(item, langwritten) else {
        return Ok(());
    };
    let dest = video_path(vid_dir, &key);
    if dest.exists() {
        item.status = MediaStatus::Ready;
        item.cache_path = Some(dest.to_string_lossy().into_owned());
        return Ok(());
    }
    let hit = catalog.lookup(&key)?;
    let bytes = catalog.fetch(&key)?;
    if !hit.checksum.is_empty() && md5_hex(&bytes) != hit.checksum.to_ascii_lowercase() {
        item.status = MediaStatus::Failed;
        return Ok(());
    }
    fs::write(&dest, bytes).map_err(|e| AppError::Io(e.to_string()))?;
    item.status = MediaStatus::Ready;
    item.cache_path = Some(dest.to_string_lossy().into_owned());
    Ok(())
}

fn fetch_pub<C>(
    catalog: &C,
    dir: &Path,
    langwritten: &str,
    monday: CivilDate,
    kind: MeetingKind,
    issues: Vec<String>,
    retry: bool,
) -> Result<MeetingWeek, AppError>
where
    C: PublicationCatalog + MediaResolver,
{
    let symbol = match kind {
        MeetingKind::Midweek => "mwb",
        MeetingKind::Weekend => "w",
    };
    let mut last_err = AppError::CatalogNotFound;
    for (index, issue) in issues.iter().enumerate() {
        if index > 0 && !retry {
            break;
        }
        let key = CatalogKey {
            langwritten: langwritten.to_string(),
            pub_symbol: symbol.into(),
            issue: Some(issue.clone()),
            track: None,
            format: CatalogFormat::Jwpub,
        };
        match catalog.lookup(&key) {
            Ok(hit) => {
                let pub_path = dir.join("pub").join(format!("{symbol}.jwpub"));
                let bytes = load_or_fetch(catalog, &key, &hit, &pub_path)?;
                let parsed = parse_jwpub(&bytes, kind, langwritten, &dir.join("img"))?;
                if let Some(week) = week_covering(&parsed, monday) {
                    let mut week = week.clone();
                    week.monday = monday.to_iso();
                    week.langwritten = langwritten.to_string();
                    week.issue = issue.clone();
                    return Ok(week);
                }
                let mut empty = MeetingWeek::empty(monday, kind, langwritten);
                empty.issue = issue.clone();
                empty.title = parsed.symbol;
                ensure_outline(&mut empty);
                return Ok(empty);
            }
            Err(AppError::CatalogNotFound) => {
                last_err = AppError::CatalogNotFound;
                continue;
            }
            Err(err) => return Err(err),
        }
    }
    Err(last_err)
}

fn load_or_fetch<C>(
    catalog: &C,
    key: &CatalogKey,
    hit: &crate::domain::media::CatalogHit,
    dest: &Path,
) -> Result<Vec<u8>, AppError>
where
    C: MediaResolver,
{
    if dest.exists() {
        let existing = fs::read(dest).map_err(|e| AppError::Io(e.to_string()))?;
        if hit.checksum.is_empty()
            || md5_hex(&existing) == hit.checksum.to_ascii_lowercase()
        {
            return Ok(existing);
        }
    }
    let bytes = catalog.fetch(key)?;
    if !hit.checksum.is_empty() && md5_hex(&bytes) != hit.checksum.to_ascii_lowercase() {
        return Err(AppError::Invariant("catalog checksum mismatch".into()));
    }
    fs::write(dest, &bytes).map_err(|e| AppError::Io(e.to_string()))?;
    Ok(bytes)
}

/// Applies a template to one meeting of a loaded bundle and persists via the caller.
pub fn apply_template_to_bundle(
    bundle: &mut WeekBundleDto,
    meeting: MeetingKind,
    template: &EventTemplate,
) {
    match meeting {
        MeetingKind::Midweek => apply_template(&mut bundle.midweek, template),
        MeetingKind::Weekend => apply_template(&mut bundle.weekend, template),
    }
}

/// Replaces the part list of one meeting after validation.
pub fn set_bundle_parts(
    bundle: &mut WeekBundleDto,
    meeting: MeetingKind,
    parts: Vec<MeetingPart>,
) -> Result<(), AppError> {
    crate::domain::template::validate_parts(&parts)?;
    match meeting {
        MeetingKind::Midweek => bundle.midweek.parts = parts,
        MeetingKind::Weekend => bundle.weekend.parts = parts,
    }
    Ok(())
}

/// Re-parses cached `mwb`/`w` JWPUB files without hitting the catalog.
pub fn restore_from_cache(
    media_root: &Path,
    langwritten: &str,
    which: WeekWhich,
    today: CivilDate,
) -> Result<WeekBundleDto, AppError> {
    let monday = monday_for(which, today);
    let dir = week_dir(media_root, langwritten, monday);
    let midweek = parse_cached_pub(&dir, langwritten, monday, MeetingKind::Midweek);
    let weekend = parse_cached_pub(&dir, langwritten, monday, MeetingKind::Weekend);
    Ok(WeekBundleDto {
        monday: monday.to_iso(),
        langwritten: langwritten.to_string(),
        midweek,
        weekend,
    })
}

fn parse_cached_pub(
    dir: &Path,
    langwritten: &str,
    monday: CivilDate,
    kind: MeetingKind,
) -> MeetingWeek {
    let symbol = match kind {
        MeetingKind::Midweek => "mwb",
        MeetingKind::Weekend => "w",
    };
    let path = dir.join("pub").join(format!("{symbol}.jwpub"));
    let Ok(bytes) = fs::read(&path) else {
        let mut empty = MeetingWeek::empty(monday, kind, langwritten);
        ensure_outline(&mut empty);
        return empty;
    };
    match parse_jwpub(&bytes, kind, langwritten, &dir.join("img")) {
        Ok(parsed) => {
            if let Some(week) = week_covering(&parsed, monday) {
                let mut week = week.clone();
                week.monday = monday.to_iso();
                week.langwritten = langwritten.to_string();
                ensure_outline(&mut week);
                week
            } else {
                let mut empty = MeetingWeek::empty(monday, kind, langwritten);
                empty.title = parsed.symbol;
                ensure_outline(&mut empty);
                empty
            }
        }
        Err(_) => {
            let mut empty = MeetingWeek::empty(monday, kind, langwritten);
            ensure_outline(&mut empty);
            empty
        }
    }
}

/// Resolves a system template id.
pub fn resolve_system_template(id: &str) -> Option<EventTemplate> {
    system_template(id)
}

pub fn parse_meeting_kind(raw: &str) -> Result<MeetingKind, AppError> {
    match raw {
        "midweek" => Ok(MeetingKind::Midweek),
        "weekend" => Ok(MeetingKind::Weekend),
        _ => Err(AppError::Invariant("meeting must be midweek or weekend".into())),
    }
}

fn video_path(vid_dir: &Path, key: &CatalogKey) -> PathBuf {
    let issue = key.issue.as_deref().unwrap_or("0");
    let track = key.track.unwrap_or(0);
    vid_dir.join(format!("{}_{issue}_{track}.mp4", key.pub_symbol))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::{md5_hex, MemoryCatalog};
    use crate::db::{configure_connection, migrate, SqliteProfileStore};
    use crate::domain::jwpub::sample_mwb_bytes;
    use crate::domain::profile::{CreateProfileDto, ProfileStore};
    use rusqlite::Connection;

    fn setup() -> (Connection, String, PathBuf, MemoryCatalog) {
        let conn = Connection::open_in_memory().expect("db");
        configure_connection(&conn).expect("pragma");
        migrate(&conn).expect("migrate");
        let profile = SqliteProfileStore::new(&conn)
            .create(CreateProfileDto { name: "Cong".into() })
            .expect("profile");
        let dir = tempfile::tempdir().expect("tmp").into_path();
        (conn, profile.id.0, dir, MemoryCatalog::new())
    }

    #[test]
    fn fetch_uses_previous_month_when_first_mwb_is_missing() {
        let (conn, profile_id, media, catalog) = setup();
        let bytes = sample_mwb_bytes();
        let checksum = md5_hex(&bytes);
        let monday = CivilDate {
            year: 2026,
            month: 9,
            day: 7,
        };
        catalog.mark_missing(&CatalogKey {
            langwritten: "S".into(),
            pub_symbol: "mwb".into(),
            issue: Some("202609".into()),
            track: None,
            format: CatalogFormat::Jwpub,
        });
        catalog.insert(
            CatalogKey {
                langwritten: "S".into(),
                pub_symbol: "mwb".into(),
                issue: Some("202608".into()),
                track: None,
                format: CatalogFormat::Jwpub,
            },
            bytes,
            checksum,
        );
        let store = SqliteWeekStore::new(&conn);
        let mut n = 0;
        let bundle = fetch_week(
            &catalog,
            &media,
            "S",
            WeekWhich::This,
            monday,
            &mut |_| n += 1,
        )
        .expect("fetch");
        store.upsert(&profile_id, &bundle.midweek).expect("mw");
        store.upsert(&profile_id, &bundle.weekend).expect("we");
        assert_eq!(bundle.monday, "2026-09-07");
        assert_eq!(bundle.midweek.issue, "202608");
        assert!(media.join("week/S/2026-09-07/pub/mwb.jwpub").exists());
        assert!(media.join("week/S/2026-09-07/img/test.jpg").exists());
        let song_copied = bundle
            .midweek
            .parts
            .iter()
            .flat_map(|p| p.items.iter())
            .any(|i| i.media_kind == crate::domain::week::MediaKind::Song && i.cache_path.is_some());
        assert!(!song_copied);
        assert!(n > 0);
    }

    #[test]
    fn download_skips_sjjm_and_writes_mwbv() {
        let (conn, profile_id, media, catalog) = setup();
        let bytes = sample_mwb_bytes();
        catalog.insert(
            CatalogKey {
                langwritten: "S".into(),
                pub_symbol: "mwb".into(),
                issue: Some("202609".into()),
                track: None,
                format: CatalogFormat::Jwpub,
            },
            bytes.clone(),
            md5_hex(&bytes),
        );
        let video = b"fake-mp4-bytes";
        catalog.insert(
            CatalogKey {
                langwritten: "S".into(),
                pub_symbol: "mwbv".into(),
                issue: Some("202609".into()),
                track: Some(1),
                format: CatalogFormat::Mp4,
            },
            video.to_vec(),
            md5_hex(video),
        );
        let store = SqliteWeekStore::new(&conn);
        let monday = CivilDate {
            year: 2026,
            month: 9,
            day: 7,
        };
        let mut bundle = fetch_week(
            &catalog,
            &media,
            "S",
            WeekWhich::This,
            monday,
            &mut |_| {},
        )
        .expect("fetch");
        let cancel = AtomicBool::new(false);
        bundle = download_week_media(
            &catalog,
            &media,
            &mut bundle,
            &cancel,
            &mut |_| {},
        )
        .expect("dl");
        store.upsert(&profile_id, &bundle.midweek).expect("mw");
        store.upsert(&profile_id, &bundle.weekend).expect("we");
        let items: Vec<_> = bundle
            .midweek
            .parts
            .iter()
            .flat_map(|p| p.items.iter())
            .chain(bundle.midweek.media.iter())
            .collect();
        assert!(items.iter().any(|i| {
            matches!(
                &i.media_ref,
                crate::domain::week::MediaRef::Catalog { key_symbol, .. } if key_symbol == "mwbv"
            ) && i.status == MediaStatus::Ready
                && i.cache_path.is_some()
        }));
        assert!(items.iter().all(|i| {
            i.media_kind != crate::domain::week::MediaKind::Song
                || i.status == crate::domain::week::MediaStatus::PendingHymnal
        }));
        assert!(!media
            .join("week/S/2026-09-07/vid")
            .read_dir()
            .map(|d| d.filter_map(|e| e.ok()).any(|e| e
                .file_name()
                .to_string_lossy()
                .contains("sjjm")))
            .unwrap_or(false));
    }
}
