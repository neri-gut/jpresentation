//! JW publication catalog adapter. Host and path stay in this module.

use std::collections::HashMap;
#[cfg(test)]
use std::sync::Mutex;

use serde::Deserialize;

use crate::domain::media::{CatalogFormat, CatalogHit, CatalogKey, MediaResolver, PublicationCatalog};
use crate::error::AppError;

const DEFAULT_ENDPOINT: &str = "https://b.jw-cdn.org/apis/pub-media/GETPUBMEDIALINKS";

/// Live GETPUBMEDIALINKS client. Vue never receives the file URL.
pub struct JwCdnCatalog {
    client: reqwest::blocking::Client,
    endpoint: String,
}

impl JwCdnCatalog {
    /// Default CDN endpoint from research. Override only in tests.
    pub fn new() -> Result<Self, AppError> {
        let client = reqwest::blocking::Client::builder()
            .user_agent("JPresentation/0.1")
            .build()
            .map_err(|e| AppError::Network(e.to_string()))?;
        Ok(Self {
            client,
            endpoint: DEFAULT_ENDPOINT.into(),
        })
    }

    fn lookup_file(&self, key: &CatalogKey) -> Result<(CatalogHit, String), AppError> {
        let mut url = reqwest::Url::parse(&self.endpoint)
            .map_err(|e| AppError::Network(e.to_string()))?;
        {
            let mut q = url.query_pairs_mut();
            q.append_pair("output", "json");
            q.append_pair("langwritten", &key.langwritten);
            q.append_pair("pub", &key.pub_symbol);
            q.append_pair("fileformat", key.format.as_str());
            if let Some(issue) = &key.issue {
                q.append_pair("issue", issue);
            }
            if let Some(track) = key.track {
                q.append_pair("track", &track.to_string());
            }
        }
        let response = self
            .client
            .get(url)
            .header("Accept", "application/json")
            .send()
            .map_err(|e| AppError::Network(e.to_string()))?;
        let status = response.status();
        if status.as_u16() == 404 {
            return Err(AppError::CatalogNotFound);
        }
        if !status.is_success() {
            return Err(AppError::Network(format!("catalog HTTP {status}")));
        }
        let body: CatalogResponse = response
            .json()
            .map_err(|e| AppError::Network(e.to_string()))?;
        pick_file(&body, key)
    }
}

impl Default for JwCdnCatalog {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            client: reqwest::blocking::Client::new(),
            endpoint: DEFAULT_ENDPOINT.into(),
        })
    }
}

impl PublicationCatalog for JwCdnCatalog {
    fn lookup(&self, key: &CatalogKey) -> Result<CatalogHit, AppError> {
        Ok(self.lookup_file(key)?.0)
    }
}

impl MediaResolver for JwCdnCatalog {
    fn fetch(&self, key: &CatalogKey) -> Result<Vec<u8>, AppError> {
        let (_hit, file_url) = self.lookup_file(key)?;
        let response = self
            .client
            .get(&file_url)
            .send()
            .map_err(|e| AppError::Network(e.to_string()))?;
        if !response.status().is_success() {
            return Err(AppError::Network(format!(
                "download HTTP {}",
                response.status()
            )));
        }
        response
            .bytes()
            .map(|b| b.to_vec())
            .map_err(|e| AppError::Network(e.to_string()))
    }
}

#[derive(Debug, Deserialize)]
struct CatalogResponse {
    files: HashMap<String, HashMap<String, Vec<CatalogFile>>>,
}

#[derive(Debug, Deserialize)]
struct CatalogFile {
    #[serde(default)]
    label: String,
    #[serde(default)]
    filesize: u64,
    file: CatalogFileUrl,
}

#[derive(Debug, Deserialize)]
struct CatalogFileUrl {
    url: String,
    #[serde(default)]
    checksum: String,
}

fn pick_file(body: &CatalogResponse, key: &CatalogKey) -> Result<(CatalogHit, String), AppError> {
    let by_lang = body
        .files
        .get(&key.langwritten)
        .or_else(|| body.files.values().next())
        .ok_or(AppError::CatalogNotFound)?;
    let list = by_lang
        .get(key.format.as_str())
        .ok_or(AppError::CatalogNotFound)?;
    if list.is_empty() {
        return Err(AppError::CatalogNotFound);
    }
    let chosen = match key.format {
        CatalogFormat::Jwpub => list.first(),
        CatalogFormat::Mp4 => list.iter().max_by_key(|item| label_rank(&item.label)),
    }
    .ok_or(AppError::CatalogNotFound)?;
    Ok((
        CatalogHit {
            key: key.clone(),
            checksum: chosen.file.checksum.clone(),
            size: chosen.filesize,
        },
        chosen.file.url.clone(),
    ))
}

fn label_rank(label: &str) -> u32 {
    let digits: String = label.chars().filter(|c| c.is_ascii_digit()).collect();
    digits.parse().unwrap_or(0)
}

/// In-memory catalog for tests. Maps a `pub/issue` key to bytes + checksum.
#[cfg(test)]
pub struct MemoryCatalog {
    files: Mutex<HashMap<String, (CatalogHit, Vec<u8>)>>,
    missing: Mutex<Vec<String>>,
}

#[cfg(test)]
impl MemoryCatalog {
    /// Empty fake catalog.
    pub fn new() -> Self {
        Self {
            files: Mutex::new(HashMap::new()),
            missing: Mutex::new(Vec::new()),
        }
    }

    /// Registers a blob. `NotFound` keys can be listed with [`Self::mark_missing`].
    pub fn insert(&self, key: CatalogKey, bytes: Vec<u8>, checksum: String) {
        let hit = CatalogHit {
            key: key.clone(),
            checksum,
            size: bytes.len() as u64,
        };
        self.files
            .lock()
            .expect("catalog")
            .insert(memory_id(&key), (hit, bytes));
    }

    /// Next lookup for this id returns `CatalogNotFound` once.
    pub fn mark_missing(&self, key: &CatalogKey) {
        self.missing.lock().expect("missing").push(memory_id(key));
    }
}

#[cfg(test)]
impl Default for MemoryCatalog {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl PublicationCatalog for MemoryCatalog {
    fn lookup(&self, key: &CatalogKey) -> Result<CatalogHit, AppError> {
        let id = memory_id(key);
        {
            let mut missing = self.missing.lock().expect("missing");
            if let Some(pos) = missing.iter().position(|m| m == &id) {
                missing.remove(pos);
                return Err(AppError::CatalogNotFound);
            }
        }
        self.files
            .lock()
            .expect("files")
            .get(&id)
            .map(|(hit, _)| hit.clone())
            .ok_or(AppError::CatalogNotFound)
    }
}

#[cfg(test)]
impl MediaResolver for MemoryCatalog {
    fn fetch(&self, key: &CatalogKey) -> Result<Vec<u8>, AppError> {
        self.files
            .lock()
            .expect("files")
            .get(&memory_id(key))
            .map(|(_, bytes)| bytes.clone())
            .ok_or(AppError::CatalogNotFound)
    }
}

#[cfg(test)]
fn memory_id(key: &CatalogKey) -> String {
    format!(
        "{}:{}:{}:{}:{}",
        key.langwritten,
        key.pub_symbol,
        key.issue.as_deref().unwrap_or("-"),
        key.track.unwrap_or(0),
        key.format.as_str()
    )
}

/// Hex MD5 used to compare catalog checksums with disk.
pub fn md5_hex(bytes: &[u8]) -> String {
    use md5::{Digest, Md5};
    format!("{:x}", Md5::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pick_best_mp4_by_label() {
        let body = CatalogResponse {
            files: HashMap::from([(
                "S".into(),
                HashMap::from([(
                    "MP4".into(),
                    vec![
                        CatalogFile {
                            label: "240p".into(),
                            filesize: 1,
                            file: CatalogFileUrl {
                                url: "http://x/240".into(),
                                checksum: "a".into(),
                            },
                        },
                        CatalogFile {
                            label: "720p".into(),
                            filesize: 2,
                            file: CatalogFileUrl {
                                url: "http://x/720".into(),
                                checksum: "b".into(),
                            },
                        },
                    ],
                )]),
            )]),
        };
        let key = CatalogKey {
            langwritten: "S".into(),
            pub_symbol: "mwbv".into(),
            issue: Some("202609".into()),
            track: Some(1),
            format: CatalogFormat::Mp4,
        };
        let (hit, url) = pick_file(&body, &key).expect("pick");
        assert_eq!(url, "http://x/720");
        assert_eq!(hit.size, 2);
    }
}
