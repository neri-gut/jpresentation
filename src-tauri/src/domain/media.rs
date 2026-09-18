//! Media ports. HTTP and filesystem live in adapters, not here.

use crate::domain::week::MediaItem;
use crate::error::AppError;

/// Catalog/media source. Change 001 registers none; later changes add JW and local adapters.
pub trait MediaProvider: Send + Sync {
    /// Stable provider id used on media handles (`jw-org`, `local`, …).
    fn id(&self) -> &'static str;
}

/// Holds `MediaProvider` implementations.
pub struct ProviderRegistry {
    inner: Vec<Box<dyn MediaProvider>>,
}

impl ProviderRegistry {
    /// Empty registry. Playback commands must not special-case provider ids.
    pub fn new() -> Self {
        Self { inner: Vec::new() }
    }

    /// True when no provider has been registered yet.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Registers a provider.
    pub fn register(&mut self, provider: Box<dyn MediaProvider>) {
        self.inner.push(provider);
    }
}

impl Default for ProviderRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Lookup key for GETPUBMEDIALINKS. No host or URL.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogKey {
    pub langwritten: String,
    pub pub_symbol: String,
    pub issue: Option<String>,
    pub track: Option<u32>,
    pub format: CatalogFormat,
}

/// File kind requested from the catalog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogFormat {
    Jwpub,
    Mp4,
}

impl CatalogFormat {
    /// Wire token used by the catalog API.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Jwpub => "JWPUB",
            Self::Mp4 => "MP4",
        }
    }
}

/// Metadata the domain is allowed to see. Checksum is compared to disk; URL stays in the adapter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CatalogHit {
    pub key: CatalogKey,
    pub checksum: String,
    pub size: u64,
}

/// Discovers a publication or track without downloading it.
pub trait PublicationCatalog: Send + Sync {
    /// Resolves metadata for a catalog key. `NotFound` on HTTP 404.
    fn lookup(&self, key: &CatalogKey) -> Result<CatalogHit, AppError>;

    /// Lists hymnal tracks (1..=163) from `pub=sjjm`.
    fn list_hymnal_tracks(
        &self,
        langwritten: &str,
    ) -> Result<Vec<crate::domain::hymnal::HymnalCatalogTrack>, AppError>;
}

/// Downloads bytes for a catalog key. Implementations talk HTTP.
pub trait MediaResolver: Send + Sync {
    /// Fetches the file body. Caller writes to `week/` and verifies checksum.
    fn fetch(&self, key: &CatalogKey) -> Result<Vec<u8>, AppError>;
}

/// Builds a catalog key for a pending video item.
pub fn catalog_key_for_item(item: &MediaItem, langwritten: &str) -> Option<CatalogKey> {
    match &item.media_ref {
        crate::domain::week::MediaRef::Catalog {
            key_symbol,
            track,
            issue_tag,
            ..
        } => {
            if key_symbol.eq_ignore_ascii_case("sjjm") {
                return None;
            }
            let issue = if *issue_tag == 0 {
                None
            } else {
                Some(crate::domain::week::normalize_issue_tag(*issue_tag))
            };
            Some(CatalogKey {
                langwritten: langwritten.to_string(),
                pub_symbol: key_symbol.clone(),
                issue,
                track: Some(*track),
                format: CatalogFormat::Mp4,
            })
        }
        crate::domain::week::MediaRef::Embedded { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_empty() {
        assert!(ProviderRegistry::new().is_empty());
    }

    #[test]
    fn sjjm_has_no_week_catalog_key() {
        let item = crate::domain::week::MediaItem {
            id: "s".into(),
            title: "1".into(),
            media_kind: crate::domain::week::MediaKind::Song,
            status: crate::domain::week::MediaStatus::PendingHymnal,
            mime: "video/mp4".into(),
            media_ref: crate::domain::week::MediaRef::Catalog {
                key_symbol: "sjjm".into(),
                track: 1,
                lang_meps: 1,
                issue_tag: 0,
                mime: "video/mp4".into(),
            },
            cache_path: None,
        };
        assert!(catalog_key_for_item(&item, "S").is_none());
    }
}
