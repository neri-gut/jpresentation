//! File explorer for the operator Multimedia tab. No SQL, no HTTP.

use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::AppError;

/// Per-profile extra folders the explorer may list.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExplorerSetting {
    pub roots: Vec<String>,
}

/// What the operator sees for one directory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExplorerKind {
    Dir,
    Image,
    Video,
    Jwpub,
}

/// One row in the explorer list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerEntryDto {
    pub name: String,
    pub path: String,
    pub kind: ExplorerKind,
}

/// A listed directory (or the virtual root list).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerListDto {
    pub path: String,
    pub parent: Option<String>,
    pub entries: Vec<ExplorerEntryDto>,
}

/// Classifies a file name. Directories are `Dir` (caller checks `is_dir`).
pub fn classify_file(name: &str) -> Option<ExplorerKind> {
    let ext = name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" => Some(ExplorerKind::Image),
        "mp4" | "webm" => Some(ExplorerKind::Video),
        "jwpub" => Some(ExplorerKind::Jwpub),
        _ => None,
    }
}

/// Home, Desktop, Pictures, Videos when those directories exist.
pub fn default_local_roots() -> Vec<ExplorerEntryDto> {
    let mut entries = Vec::new();
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return entries;
    };
    if home.is_dir() {
        push_root(&mut entries, "Home", &home);
    }
    let desktop = std::env::var_os("XDG_DESKTOP_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| home.join("Desktop"));
    if desktop.is_dir() {
        push_root(&mut entries, "Desktop", &desktop);
    }
    for (label, name) in [("Pictures", "Pictures"), ("Videos", "Videos")] {
        let dir = home.join(name);
        if dir.is_dir() {
            push_root(&mut entries, label, &dir);
        }
    }
    entries
}

fn push_root(entries: &mut Vec<ExplorerEntryDto>, name: &str, path: &Path) {
    let as_str = path.to_string_lossy().into_owned();
    if entries.iter().any(|e| e.path == as_str) {
        return;
    }
    entries.push(ExplorerEntryDto {
        name: name.into(),
        path: as_str,
        kind: ExplorerKind::Dir,
    });
}

/// True when `path` is inside `media_root`, a profile root, or a default local root.
pub fn path_allowed(path: &Path, media_root: &Path, roots: &[String]) -> bool {
    let Ok(canon) = path.canonicalize() else {
        return false;
    };
    prefixes(media_root, roots)
        .iter()
        .any(|root| canon.starts_with(root))
}

fn prefixes(media_root: &Path, roots: &[String]) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(root) = media_root.canonicalize() {
        out.push(root);
    }
    for raw in roots {
        if let Ok(root) = Path::new(raw).canonicalize() {
            out.push(root);
        }
    }
    for entry in default_local_roots() {
        if let Ok(root) = Path::new(&entry.path).canonicalize() {
            out.push(root);
        }
    }
    out
}

/// Staging copy name. Unique per snapshot so webviews do not reuse a cached URL.
pub fn stage_file_name(rev: u64, ext: &str) -> String {
    let ext = ext.trim_start_matches('.');
    format!("{rev}.{ext}")
}

/// Lists one directory. Skips hidden names and unknown extensions.
pub fn list_dir(path: &Path) -> Result<Vec<ExplorerEntryDto>, AppError> {
    let mut entries = Vec::new();
    let reader = fs::read_dir(path).map_err(|e| AppError::Io(e.to_string()))?;
    for item in reader {
        let item = item.map_err(|e| AppError::Io(e.to_string()))?;
        let name = item.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let file_type = item.file_type().map_err(|e| AppError::Io(e.to_string()))?;
        let kind = if file_type.is_dir() {
            ExplorerKind::Dir
        } else if let Some(kind) = classify_file(&name) {
            kind
        } else {
            continue;
        };
        entries.push(ExplorerEntryDto {
            name,
            path: item.path().to_string_lossy().into_owned(),
            kind,
        });
    }
    entries.sort_by(|a, b| {
        let dir_a = matches!(a.kind, ExplorerKind::Dir);
        let dir_b = matches!(b.kind, ExplorerKind::Dir);
        dir_b.cmp(&dir_a).then(a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });
    Ok(entries)
}

/// Parent path as string, if any.
pub fn parent_string(path: &Path) -> Option<String> {
    path.parent()
        .map(|p| p.to_string_lossy().into_owned())
}

/// MIME for a classified file.
pub fn mime_for(kind: ExplorerKind, name: &str) -> &'static str {
    let ext = name.rsplit('.').next().unwrap_or("").to_ascii_lowercase();
    match (kind, ext.as_str()) {
        (ExplorerKind::Image, "png") => "image/png",
        (ExplorerKind::Image, "webp") => "image/webp",
        (ExplorerKind::Image, "gif") => "image/gif",
        (ExplorerKind::Image, _) => "image/jpeg",
        (ExplorerKind::Video, "webm") => "video/webm",
        (ExplorerKind::Video, _) => "video/mp4",
        _ => "application/octet-stream",
    }
}

/// Validates explorer roots (absolute, existing dirs, cap 20).
pub fn validate_explorer(value: &ExplorerSetting) -> Result<(), AppError> {
    if value.roots.len() > 20 {
        return Err(AppError::Invariant("too many explorer roots".into()));
    }
    for root in &value.roots {
        if root.trim().is_empty() || root.len() > 512 {
            return Err(AppError::Invariant("invalid explorer root".into()));
        }
        let path = PathBuf::from(root);
        if !path.is_absolute() {
            return Err(AppError::Invariant("explorer root must be absolute".into()));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn classifies_media_and_skips_db() {
        assert_eq!(classify_file("a.JPG"), Some(ExplorerKind::Image));
        assert_eq!(classify_file("clip.mp4"), Some(ExplorerKind::Video));
        assert_eq!(classify_file("w.jwpub"), Some(ExplorerKind::Jwpub));
        assert_eq!(classify_file("mwb.db"), None);
        assert_eq!(classify_file("notes.txt"), None);
    }

    #[test]
    fn list_dir_filters_and_jails() {
        let dir = tempfile::tempdir().expect("tmp");
        fs::write(dir.path().join("pic.jpg"), b"x").expect("jpg");
        fs::write(dir.path().join("skip.txt"), b"x").expect("txt");
        fs::create_dir(dir.path().join("sub")).expect("dir");
        let listed = list_dir(dir.path()).expect("list");
        assert!(listed.iter().any(|e| e.name == "pic.jpg" && e.kind == ExplorerKind::Image));
        assert!(listed.iter().any(|e| e.name == "sub" && e.kind == ExplorerKind::Dir));
        assert!(!listed.iter().any(|e| e.name == "skip.txt"));
        assert!(path_allowed(
            &dir.path().join("pic.jpg"),
            dir.path(),
            &[]
        ));
        assert!(!path_allowed(Path::new("/etc"), dir.path(), &[]));
    }

    #[test]
    fn stage_file_names_differ_by_rev() {
        assert_ne!(stage_file_name(1, "jpg"), stage_file_name(2, "jpg"));
        assert_eq!(stage_file_name(3, ".png"), "3.png");
    }

    #[test]
    fn default_roots_include_home_when_present() {
        let roots = default_local_roots();
        if let Ok(home) = std::env::var("HOME") {
            assert!(roots.iter().any(|e| e.path == home && e.name == "Home"));
            assert!(path_allowed(&PathBuf::from(&home), Path::new("/tmp/jp-none"), &[]));
        }
    }
}
