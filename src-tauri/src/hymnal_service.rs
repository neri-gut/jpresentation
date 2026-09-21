//! Hymnal service orchestrating catalog discovery, permanent cache, download, and stage playback.

use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::catalog::md5_hex;
use crate::domain::hymnal::{
    fallback_songs, find_cached_song_file, hymnal_dir, is_meeting_song_track,
    HymnalCatalogTrack, HymnalProgressDto, HymnalSongDto, SongStatus,
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

    let tracks = match get_catalog_tracks(catalog, media_root, langwritten, refresh) {
        Ok(t) => t,
        Err(_) => {
            return Ok(check_disk_status(fallback_songs(langwritten), &dir));
        }
    };

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

    // If already on disk, return ready
    if let Some(path) = find_cached_song_file(&dir, track) {
        let title = format_default_title(langwritten, track);
        return Ok(HymnalSongDto {
            track,
            title,
            duration_formatted: String::new(),
            status: SongStatus::Ready,
            cache_path: Some(path.to_string_lossy().into_owned()),
            filesize: path.metadata().map(|m| m.len()).unwrap_or(0),
        });
    }

    let tracks_res = get_catalog_tracks(catalog, media_root, langwritten, false);
    let (bytes, title, duration_formatted, label) = match tracks_res {
        Ok(tracks) => {
            if let Some(item) = tracks.iter().find(|t| t.track == track) {
                let bytes = if !item.url.is_empty() {
                    catalog.fetch_url(&item.url)?
                } else {
                    let key = CatalogKey {
                        langwritten: langwritten.to_string(),
                        pub_symbol: "sjjm".into(),
                        issue: None,
                        track: Some(track),
                        format: CatalogFormat::Mp4,
                    };
                    catalog.fetch(&key)?
                };
                if !item.checksum.is_empty() && md5_hex(&bytes) != item.checksum.to_ascii_lowercase() {
                    return Err(AppError::Invariant("hymnal song checksum mismatch".into()));
                }
                let label = if item.label.is_empty() { "720p".to_string() } else { item.label.clone() };
                (bytes, item.title.clone(), item.duration_formatted.clone(), label)
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
                if !hit.checksum.is_empty() && md5_hex(&bytes) != hit.checksum.to_ascii_lowercase() {
                    return Err(AppError::Invariant("hymnal song checksum mismatch".into()));
                }
                (bytes, format_default_title(langwritten, track), String::new(), "720p".into())
            }
        }
        Err(_) => {
            let key = CatalogKey {
                langwritten: langwritten.to_string(),
                pub_symbol: "sjjm".into(),
                issue: None,
                track: Some(track),
                format: CatalogFormat::Mp4,
            };
            let hit = catalog.lookup(&key)?;
            let bytes = catalog.fetch(&key)?;
            if !hit.checksum.is_empty() && md5_hex(&bytes) != hit.checksum.to_ascii_lowercase() {
                return Err(AppError::Invariant("hymnal song checksum mismatch".into()));
            }
            (bytes, format_default_title(langwritten, track), String::new(), "720p".into())
        }
    };

    let dest = dir.join(format!("sjjm_{track}_{label}.mp4"));
    fs::write(&dest, &bytes).map_err(|e| AppError::Io(e.to_string()))?;

    Ok(HymnalSongDto {
        track,
        title,
        duration_formatted,
        status: SongStatus::Ready,
        cache_path: Some(dest.to_string_lossy().into_owned()),
        filesize: bytes.len() as u64,
    })
}

/// Downloads all missing hymnal songs in background with progress and cancellation.
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
    let dir = hymnal_dir(media_root, langwritten);
    fs::create_dir_all(&dir).map_err(|e| AppError::Io(e.to_string()))?;

    let tracks = get_catalog_tracks(catalog, media_root, langwritten, false)?;
    let total = tracks.len() as u32;

    for (index, item) in tracks.iter().enumerate() {
        if cancel.load(Ordering::SeqCst) {
            return Err(AppError::Cancelled);
        }
        progress(HymnalProgressDto {
            phase: "download".into(),
            done: index as u32,
            total,
            label: item.title.clone(),
        });

        if find_cached_song_file(&dir, item.track).is_none() {
            let bytes_res = if !item.url.is_empty() {
                catalog.fetch_url(&item.url)
            } else {
                let key = CatalogKey {
                    langwritten: langwritten.to_string(),
                    pub_symbol: "sjjm".into(),
                    issue: None,
                    track: Some(item.track),
                    format: CatalogFormat::Mp4,
                };
                catalog.fetch(&key)
            };

            if let Ok(bytes) = bytes_res {
                if item.checksum.is_empty()
                    || md5_hex(&bytes) == item.checksum.to_ascii_lowercase()
                {
                    let label = if item.label.is_empty() { "720p" } else { &item.label };
                    let dest = dir.join(format!("sjjm_{}_{}.mp4", item.track, label));
                    let _ = fs::write(dest, bytes);
                }
            }
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

    Ok(output.open_media(StageKind::Video, song.title, "video/mp4".into(), path))
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
        assert!(snap.path.is_some());
    }
}
