//! Hymnal service orchestrating catalog discovery, permanent cache, download, and stage playback.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::catalog::md5_hex;
use crate::domain::explorer::copy_into_stage;
use crate::domain::hymnal::{
    find_cached_song_file, hymnal_dir, is_meeting_song_track, HymnalCatalogTrack,
    HymnalProgressDto, HymnalSongDto, SongStatus,
};
use crate::domain::media::{CatalogFormat, CatalogKey, MediaResolver, PublicationCatalog};
use crate::domain::output::{OutputState, StageKind, StageSnapshot};
use crate::error::AppError;

const CATALOG_FILENAME: &str = "catalog.json";

/// Loads tracks metadata from catalog.json or from catalog API.
pub fn get_catalog_tracks<C>(
    catalog: &C,
    media_root: &Path,
    langwritten: &str,
    refresh: bool,
) -> Result<Vec<HymnalCatalogTrack>, AppError>
where
    C: PublicationCatalog,
{
    let dir = hymnal_dir(media_root, langwritten);
    fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;
    let catalog_path = dir.join(CATALOG_FILENAME);

    if !refresh && catalog_path.exists() {
        if let Ok(bytes) = fs::read(&catalog_path) {
            if let Ok(tracks) = serde_json::from_slice::<Vec<HymnalCatalogTrack>>(&bytes) {
                if !tracks.is_empty() {
                    return Ok(tracks);
                }
            }
        }
    }

    let tracks = catalog.list_hymnal_tracks(langwritten)?;
    if let Ok(json) = serde_json::to_vec_pretty(&tracks) {
        let _ = fs::write(&catalog_path, json);
    }
    Ok(tracks)
}

/// Loads the song list for a language.
/// If `catalog.json` exists in `hymnal/{langwritten}`, loads from cache without network.
/// If not, or if `refresh` is true, fetches from the catalog adapter and caches it.
pub fn load_or_fetch_songs<C>(
    catalog: &C,
    media_root: &Path,
    langwritten: &str,
    refresh: bool,
) -> Result<Vec<HymnalSongDto>, AppError>
where
    C: PublicationCatalog,
{
    let dir = hymnal_dir(media_root, langwritten);
    fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;

    let tracks = get_catalog_tracks(catalog, media_root, langwritten, refresh)?;
    let songs = tracks
        .into_iter()
        .filter(|t| is_meeting_song_track(t.track))
        .map(|t| HymnalSongDto {
            track: t.track,
            title: t.title,
            duration_formatted: t.duration_formatted,
            status: SongStatus::Pending,
            cache_path: None,
            filesize: t.filesize,
        })
        .collect();

    Ok(check_disk_status(songs, &dir))
}

fn check_disk_status(mut songs: Vec<HymnalSongDto>, dir: &Path) -> Vec<HymnalSongDto> {
    for song in &mut songs {
        if let Some(path) = find_cached_song_file(dir, song.track) {
            song.status = SongStatus::Ready;
            song.cache_path = Some(path.to_string_lossy().into_owned());
            if song.filesize == 0 {
                if let Ok(meta) = path.metadata() {
                    song.filesize = meta.len();
                }
            }
        } else {
            song.status = SongStatus::Pending;
            song.cache_path = None;
        }
    }
    songs
}

/// Downloads a single hymnal track on demand.
pub fn download_single_song<C>(
    catalog: &C,
    media_root: &Path,
    langwritten: &str,
    track: u32,
) -> Result<HymnalSongDto, AppError>
where
    C: PublicationCatalog + MediaResolver,
{
    if !is_meeting_song_track(track) {
        return Err(AppError::Invariant("invalid song track".into()));
    }
    let dir = hymnal_dir(media_root, langwritten);
    fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;

    let tracks = get_catalog_tracks(catalog, media_root, langwritten, false).ok();
    let item = tracks
        .as_ref()
        .and_then(|list| list.iter().find(|t| t.track == track).cloned());

    if let Some(path) = find_cached_song_file(&dir, track) {
        return Ok(song_from_disk(track, langwritten, item.as_ref(), &path));
    }

    if let Some(item) = item {
        write_catalog_track(catalog, langwritten, &dir, &item)?;
    } else {
        let key = CatalogKey {
            langwritten: langwritten.to_string(),
            pub_symbol: "sjjm".into(),
            issue: None,
            track: Some(track),
            format: CatalogFormat::Mp4,
        };
        let hit = catalog.lookup(&key)?;
        let bytes = catalog.fetch(&key)?;
        write_song_bytes(&dir, track, "720p", &bytes, &hit.checksum)?;
    }

    let path = find_cached_song_file(&dir, track)
        .ok_or_else(|| AppError::Invariant("song was not written to hymnal cache".into()))?;
    let item = tracks
        .as_ref()
        .and_then(|list| list.iter().find(|t| t.track == track).cloned());
    Ok(song_from_disk(track, langwritten, item.as_ref(), &path))
}

/// Downloads all missing hymnal songs with progress and cancellation.
pub fn download_all_songs<C>(
    catalog: &C,
    media_root: &Path,
    langwritten: &str,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(HymnalProgressDto),
) -> Result<Vec<HymnalSongDto>, AppError>
where
    C: PublicationCatalog + MediaResolver,
{
    download_songs(
        catalog,
        media_root,
        langwritten,
        None,
        cancel,
        progress,
    )
}

/// Downloads the given tracks (or every pending track when `only_tracks` is `None`).
pub fn download_songs<C>(
    catalog: &C,
    media_root: &Path,
    langwritten: &str,
    only_tracks: Option<&[u32]>,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(HymnalProgressDto),
) -> Result<Vec<HymnalSongDto>, AppError>
where
    C: PublicationCatalog + MediaResolver,
{
    let dir = hymnal_dir(media_root, langwritten);
    fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;

    let tracks = get_catalog_tracks(catalog, media_root, langwritten, false)?;
    let pending: Vec<HymnalCatalogTrack> = tracks
        .into_iter()
        .filter(|item| is_meeting_song_track(item.track))
        .filter(|item| match only_tracks {
            Some(wanted) => wanted.contains(&item.track),
            None => true,
        })
        .filter(|item| find_cached_song_file(&dir, item.track).is_none())
        .collect();
    let total = pending.len() as u32;

    for (index, item) in pending.iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            return Err(AppError::Cancelled);
        }
        progress(HymnalProgressDto {
            phase: "download".into(),
            done: index as u32,
            total,
            label: item.title.clone(),
        });
        match write_catalog_track(catalog, langwritten, &dir, item) {
            Ok(()) => {}
            Err(AppError::Cancelled) => return Err(AppError::Cancelled),
            Err(_) => {}
        }
    }

    progress(HymnalProgressDto {
        phase: "download".into(),
        done: total,
        total,
        label: "done".into(),
    });

    load_or_fetch_songs(catalog, media_root, langwritten, false)
}

/// Plays a song on the stage. If not downloaded yet, attempts on-demand download.
pub fn play_song_on_stage<C>(
    catalog: &C,
    media_root: &Path,
    langwritten: &str,
    track: u32,
    output: &mut OutputState,
) -> Result<StageSnapshot, AppError>
where
    C: PublicationCatalog + MediaResolver,
{
    let song = download_single_song(catalog, media_root, langwritten, track)?;
    let Some(path) = song.cache_path else {
        return Err(AppError::Invariant("song has no cache path".into()));
    };
    let next_rev = output.stage.rev.saturating_add(1);
    let dest = copy_into_stage(media_root, Path::new(&path), next_rev, "mp4")?;

    Ok(output.open_media(
        StageKind::Video,
        song.title,
        "video/mp4".into(),
        dest.to_string_lossy().into_owned(),
    ))
}

fn write_catalog_track<C>(
    catalog: &C,
    langwritten: &str,
    dir: &Path,
    item: &HymnalCatalogTrack,
) -> Result<(), AppError>
where
    C: PublicationCatalog + MediaResolver,
{
    let bytes = if !item.url.is_empty() {
        catalog.fetch_url(&item.url)?
    } else {
        let key = CatalogKey {
            langwritten: langwritten.to_string(),
            pub_symbol: "sjjm".into(),
            issue: None,
            track: Some(item.track),
            format: CatalogFormat::Mp4,
        };
        catalog.fetch(&key)?
    };
    let label = if item.label.is_empty() {
        "720p"
    } else {
        item.label.as_str()
    };
    write_song_bytes(dir, item.track, label, &bytes, &item.checksum)?;
    Ok(())
}

fn write_song_bytes(
    dir: &Path,
    track: u32,
    label: &str,
    bytes: &[u8],
    checksum: &str,
) -> Result<(), AppError> {
    if !checksum.is_empty() && md5_hex(bytes) != checksum.to_ascii_lowercase() {
        return Err(AppError::Invariant("hymnal song checksum mismatch".into()));
    }
    let dest = dir.join(format!("sjjm_{track}_{label}.mp4"));
    let tmp = dir.join(format!("sjjm_{track}_{label}.mp4.part"));
    fs::write(&tmp, bytes).map_err(|e| AppError::Io(e.to_string()))?;
    fs::rename(&tmp, &dest).map_err(|e| {
        let _ = fs::remove_file(&tmp);
        AppError::Io(e.to_string())
    })?;
    Ok(())
}

fn song_from_disk(
    track: u32,
    langwritten: &str,
    item: Option<&HymnalCatalogTrack>,
    path: &Path,
) -> HymnalSongDto {
    let title = item
        .map(|t| t.title.clone())
        .unwrap_or_else(|| format_default_title(langwritten, track));
    let duration_formatted = item
        .map(|t| t.duration_formatted.clone())
        .unwrap_or_default();
    let filesize = path.metadata().map(|m| m.len()).unwrap_or(0);
    HymnalSongDto {
        track,
        title,
        duration_formatted,
        status: SongStatus::Ready,
        cache_path: Some(path.to_string_lossy().into_owned()),
        filesize,
    }
}

fn format_default_title(langwritten: &str, track: u32) -> String {
    if langwritten.eq_ignore_ascii_case("S") {
        format!("{track}. Cántico {track}")
    } else {
        format!("{track}. Song {track}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::MemoryCatalog;
    use tempfile::tempdir;

    #[test]
    fn load_or_fetch_songs_uses_cache_and_marks_ready() {
        let dir = tempdir().expect("tempdir");
        let media_root = dir.path();
        let catalog = MemoryCatalog::new();
        catalog.insert_hymnal_tracks(
            "S",
            vec![
                HymnalCatalogTrack {
                    track: 1,
                    title: "1. Las cualidades de Jehová".into(),
                    duration_secs: Some(140),
                    duration_formatted: "02:20".into(),
                    label: "720p".into(),
                    filesize: 1024,
                    checksum: "abc".into(),
                    url: "http://example.com/1.mp4".into(),
                },
                HymnalCatalogTrack {
                    track: 2,
                    title: "2. Tu nombre es Jehová".into(),
                    duration_secs: Some(180),
                    duration_formatted: "03:00".into(),
                    label: "720p".into(),
                    filesize: 2048,
                    checksum: "def".into(),
                    url: "http://example.com/2.mp4".into(),
                },
            ],
        );

        let songs = load_or_fetch_songs(&catalog, media_root, "S", false).expect("songs");
        assert_eq!(songs.len(), 2);
        assert_eq!(songs[0].status, SongStatus::Pending);

        // Simulate downloading song 1
        let song1_path = hymnal_dir(media_root, "S").join("sjjm_1_720p.mp4");
        fs::write(&song1_path, b"mp4data").expect("write");

        let reloaded = load_or_fetch_songs(&catalog, media_root, "S", false).expect("reloaded");
        assert_eq!(reloaded[0].status, SongStatus::Ready);
        assert_eq!(reloaded[0].cache_path, Some(song1_path.to_string_lossy().into_owned()));
        assert_eq!(reloaded[1].status, SongStatus::Pending);
    }

    #[test]
    fn download_single_song_downloads_and_verifies() {
        let dir = tempdir().expect("tempdir");
        let media_root = dir.path();
        let catalog = MemoryCatalog::new();
        let video_bytes = b"fake-hymn-video";
        let checksum = md5_hex(video_bytes);

        catalog.insert(
            CatalogKey {
                langwritten: "S".into(),
                pub_symbol: "sjjm".into(),
                issue: None,
                track: Some(38),
                format: CatalogFormat::Mp4,
            },
            video_bytes.to_vec(),
            checksum,
        );

        let song = download_single_song(&catalog, media_root, "S", 38).expect("download");
        assert_eq!(song.status, SongStatus::Ready);
        assert!(song.cache_path.is_some());
        assert!(Path::new(&song.cache_path.unwrap()).exists());
    }

    #[test]
    fn play_song_opens_video_stage() {
        let dir = tempdir().expect("tempdir");
        let media_root = dir.path();
        let catalog = MemoryCatalog::new();
        let video_bytes = b"song-bytes";
        let checksum = md5_hex(video_bytes);

        catalog.insert(
            CatalogKey {
                langwritten: "S".into(),
                pub_symbol: "sjjm".into(),
                issue: None,
                track: Some(151),
                format: CatalogFormat::Mp4,
            },
            video_bytes.to_vec(),
            checksum,
        );

        let mut output = OutputState::idle();
        let snap = play_song_on_stage(&catalog, media_root, "S", 151, &mut output).expect("play");
        assert_eq!(snap.kind, StageKind::Video);
        assert_eq!(snap.rev, 1);
        let path = snap.path.expect("stage path");
        assert!(path.contains("stage"));
        assert!(Path::new(&path).exists());
    }

    #[test]
    fn download_all_writes_pending_and_skips_ready() {
        let dir = tempdir().expect("tempdir");
        let media_root = dir.path();
        let catalog = MemoryCatalog::new();
        let video_bytes = b"fake-test-song";
        catalog.insert_hymnal_tracks(
            "S",
            vec![
                HymnalCatalogTrack {
                    track: 1,
                    title: "1. One".into(),
                    duration_secs: Some(140),
                    duration_formatted: "02:20".into(),
                    label: "720p".into(),
                    filesize: video_bytes.len() as u64,
                    checksum: String::new(),
                    url: "http://example.com/1.mp4".into(),
                },
                HymnalCatalogTrack {
                    track: 2,
                    title: "2. Two".into(),
                    duration_secs: Some(180),
                    duration_formatted: "03:00".into(),
                    label: "720p".into(),
                    filesize: video_bytes.len() as u64,
                    checksum: String::new(),
                    url: "http://example.com/2.mp4".into(),
                },
            ],
        );
        let already = hymnal_dir(media_root, "S").join("sjjm_1_720p.mp4");
        fs::create_dir_all(already.parent().expect("parent")).expect("dir");
        fs::write(&already, video_bytes).expect("write");

        let mut last = HymnalProgressDto {
            phase: String::new(),
            done: 0,
            total: 0,
            label: String::new(),
        };
        let songs = download_songs(
            &catalog,
            media_root,
            "S",
            None,
            &AtomicBool::new(false),
            &mut |p| last = p,
        )
        .expect("download all");
        assert_eq!(last.done, 1);
        assert_eq!(last.total, 1);
        assert_eq!(songs.len(), 2);
        assert_eq!(songs[0].status, SongStatus::Ready);
        assert_eq!(songs[1].status, SongStatus::Ready);
        assert!(hymnal_dir(media_root, "S")
            .join("sjjm_2_720p.mp4")
            .exists());
    }

    #[test]
    fn download_songs_respects_track_filter() {
        let dir = tempdir().expect("tempdir");
        let media_root = dir.path();
        let catalog = MemoryCatalog::new();
        catalog.insert_hymnal_tracks(
            "S",
            vec![
                HymnalCatalogTrack {
                    track: 3,
                    title: "3. Three".into(),
                    duration_secs: None,
                    duration_formatted: String::new(),
                    label: "720p".into(),
                    filesize: 4,
                    checksum: String::new(),
                    url: "http://example.com/3.mp4".into(),
                },
                HymnalCatalogTrack {
                    track: 4,
                    title: "4. Four".into(),
                    duration_secs: None,
                    duration_formatted: String::new(),
                    label: "720p".into(),
                    filesize: 4,
                    checksum: String::new(),
                    url: "http://example.com/4.mp4".into(),
                },
            ],
        );
        let songs = download_songs(
            &catalog,
            media_root,
            "S",
            Some(&[3]),
            &AtomicBool::new(false),
            &mut |_| {},
        )
        .expect("selected");
        assert_eq!(songs[0].status, SongStatus::Ready);
        assert_eq!(songs[1].status, SongStatus::Pending);
    }
}
