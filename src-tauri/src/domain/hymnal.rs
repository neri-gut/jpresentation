//! Hymnal domain models, DTOs and filtering logic for `sjjm` 1–163.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// Event name for hymnal download progress.
pub const HYMNAL_PROGRESS: &str = "hymnal://progress";

/// Download / readiness status of a hymnal track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SongStatus {
    /// Downloaded and available in permanent hymnal cache.
    Ready,
    /// Available in catalog but not yet downloaded.
    Pending,
    /// Download failed or checksum mismatch.
    Failed,
}

/// Hymnal song item presented to Vue.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HymnalSongDto {
    pub track: u32,
    pub title: String,
    pub duration_formatted: String,
    pub status: SongStatus,
    #[serde(default)]
    pub cache_path: Option<String>,
    pub filesize: u64,
}

/// The three meeting song slots (Opening, Middle, Concluding).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct HymnalSlotsDto {
    pub start: Option<u32>,
    pub middle: Option<u32>,
    pub end: Option<u32>,
}

/// Progress event payload during bulk or single downloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HymnalProgressDto {
    pub phase: String,
    pub done: u32,
    pub total: u32,
    pub label: String,
}

/// Returned by `hymnal_get` and `hymnal_refresh`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HymnalBundleDto {
    pub langwritten: String,
    pub songs: Vec<HymnalSongDto>,
    pub slots: HymnalSlotsDto,
}

/// Track metadata parsed from GETPUBMEDIALINKS catalog.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HymnalCatalogTrack {
    pub track: u32,
    pub title: String,
    pub duration_secs: Option<u32>,
    pub duration_formatted: String,
    pub label: String,
    pub filesize: u64,
    pub checksum: String,
    pub url: String,
}

/// Returns the permanent hymnal cache directory: `{media_root}/hymnal/{langwritten}`.
pub fn hymnal_dir(media_root: &Path, langwritten: &str) -> PathBuf {
    media_root.join("hymnal").join(langwritten)
}

/// Formats duration in seconds into `MM:SS`.
pub fn format_duration(secs: u32) -> String {
    let mins = secs / 60;
    let rem = secs % 60;
    format!("{:02}:{:02}", mins, rem)
}

/// Normalizes raw duration strings or seconds into `MM:SS`.
pub fn normalize_duration(raw_str: Option<&str>, secs: Option<f64>) -> String {
    if let Some(s) = raw_str {
        let trimmed = s.trim();
        if !trimmed.is_empty() {
            let parts: Vec<&str> = trimmed.split(':').collect();
            if parts.len() == 2 {
                if let (Ok(m), Ok(sec)) = (parts[0].parse::<u32>(), parts[1].parse::<u32>()) {
                    return format!("{:02}:{:02}", m, sec);
                }
            }
            return trimmed.to_string();
        }
    }
    if let Some(d) = secs {
        if d >= 0.0 {
            return format_duration(d.round() as u32);
        }
    }
    String::new()
}

/// Returns true if the track number is valid for congregation meetings (1..=163).
pub fn is_meeting_song_track(track: u32) -> bool {
    (1..=163).contains(&track)
}

/// Finds the cached MP4 path for a track if it exists on disk.
pub fn find_cached_song_file(hymnal_dir: &Path, track: u32) -> Option<PathBuf> {
    if !hymnal_dir.exists() {
        return None;
    }
    let Ok(entries) = std::fs::read_dir(hymnal_dir) else {
        return None;
    };
    let prefix = format!("sjjm_{track}_");
    let fallback_prefix = format!("sjjm_{track}.");
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                if (name.starts_with(&prefix) || name.starts_with(&fallback_prefix))
                    && name.ends_with(".mp4")
                {
                    if let Ok(meta) = entry.metadata() {
                        if meta.len() > 0 {
                            return Some(path);
                        }
                    }
                }
            }
        }
    }
    None
}

/// Generates a fallback list of 163 songs when no catalog is available yet.
pub fn fallback_songs(langwritten: &str) -> Vec<HymnalSongDto> {
    let is_spanish = langwritten.eq_ignore_ascii_case("S");
    (1..=163)
        .map(|track| {
            let title = if is_spanish {
                format!("Cántico {track}")
            } else {
                format!("Song {track}")
            };
            HymnalSongDto {
                track,
                title,
                duration_formatted: String::new(),
                status: SongStatus::Pending,
                cache_path: None,
                filesize: 0,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meeting_song_tracks_are_1_to_163() {
        assert!(is_meeting_song_track(1));
        assert!(is_meeting_song_track(163));
        assert!(!is_meeting_song_track(0));
        assert!(!is_meeting_song_track(164));
        assert!(!is_meeting_song_track(601));
    }

    #[test]
    fn format_duration_handles_seconds() {
        assert_eq!(format_duration(0), "00:00");
        assert_eq!(format_duration(65), "01:05");
        assert_eq!(format_duration(140), "02:20");
    }

    #[test]
    fn normalize_duration_formats_properly() {
        assert_eq!(normalize_duration(Some("2:20"), None), "02:20");
        assert_eq!(normalize_duration(Some("03:45"), None), "03:45");
        assert_eq!(normalize_duration(None, Some(180.0)), "03:00");
        assert_eq!(normalize_duration(None, Some(179.4)), "02:59");
        assert_eq!(normalize_duration(None, None), "");
    }

    #[test]
    fn fallback_songs_contain_163_items() {
        let songs = fallback_songs("S");
        assert_eq!(songs.len(), 163);
        assert_eq!(songs[0].track, 1);
        assert_eq!(songs[0].title, "Cántico 1");
        assert_eq!(songs[162].track, 163);
    }
}
