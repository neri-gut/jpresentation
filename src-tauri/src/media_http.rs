//! Loopback HTTP server for stage video.
//!
//! WebKitGTK does not finish MP4 playback through Tauri's `asset` protocol:
//! each range response is capped and the media backend does not ask for the rest.
//! Audience and speaker `<video>` elements load `http://127.0.0.1/<port>/media?path=…` instead.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use crate::error::AppError;

/// Origin such as `http://127.0.0.1:17311`. Empty when the listener did not start.
pub fn start(media_root: PathBuf) -> Result<String, AppError> {
    let listener = TcpListener::bind("127.0.0.1:0").map_err(|e| AppError::Io(e.to_string()))?;
    let port = listener
        .local_addr()
        .map_err(|e| AppError::Io(e.to_string()))?
        .port();
    let origin = format!("http://127.0.0.1:{port}");
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(stream) = stream else {
                continue;
            };
            let root = media_root.clone();
            thread::spawn(move || {
                let _ = handle_client(stream, &root);
            });
        }
    });
    Ok(origin)
}

fn handle_client(mut stream: TcpStream, media_root: &Path) -> std::io::Result<()> {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(8)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(30)));
    let mut buf = [0u8; 8192];
    let n = stream.read(&mut buf)?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let mut lines = request.split("\r\n");
    let request_line = lines.next().unwrap_or("");
    let mut range: Option<String> = None;
    for line in lines {
        let lower = line.to_ascii_lowercase();
        if let Some(value) = lower.strip_prefix("range:") {
            range = Some(value.trim().to_string());
        }
    }
    let Some(target) = request_target(request_line) else {
        return write_status(&mut stream, 400, "bad request");
    };
    let Some(raw_path) = query_param(&target, "path") else {
        return write_status(&mut stream, 404, "not found");
    };
    let Some(file_path) = resolve_under_root(media_root, &percent_decode(&raw_path)) else {
        return write_status(&mut stream, 404, "not found");
    };
    let mut file = File::open(&file_path)?;
    let len = file.metadata()?.len();
    let mime = mime_for(&file_path);
    let bounds = match range.as_deref() {
        Some(header) => match parse_range(header, len) {
            Some(bounds) => bounds,
            None => {
                let body = format!("bytes */{len}");
                let head = format!(
                    "HTTP/1.1 416 Range Not Satisfiable\r\nContent-Range: {body}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                );
                stream.write_all(head.as_bytes())?;
                return Ok(());
            }
        },
        None => (0, len.saturating_sub(1)),
    };
    let (start, end) = bounds;
    let is_partial = range.is_some();
    let nbytes = end.saturating_sub(start).saturating_add(1);
    let status = if is_partial { 206 } else { 200 };
    let mut head = format!(
        "HTTP/1.1 {status} {}\r\nAccept-Ranges: bytes\r\nContent-Type: {mime}\r\nContent-Length: {nbytes}\r\n",
        if is_partial { "Partial Content" } else { "OK" }
    );
    if is_partial {
        head.push_str(&format!("Content-Range: bytes {start}-{end}/{len}\r\n"));
    }
    head.push_str("Connection: close\r\n\r\n");
    stream.write_all(head.as_bytes())?;
    if request_line.starts_with("HEAD ") {
        return Ok(());
    }
    file.seek(SeekFrom::Start(start))?;
    let mut left = nbytes;
    let mut chunk = [0u8; 64 * 1024];
    while left > 0 {
        let want = std::cmp::min(chunk.len() as u64, left) as usize;
        let read = file.read(&mut chunk[..want])?;
        if read == 0 {
            break;
        }
        stream.write_all(&chunk[..read])?;
        left -= read as u64;
    }
    Ok(())
}

fn write_status(stream: &mut TcpStream, code: u16, text: &str) -> std::io::Result<()> {
    let body = text.as_bytes();
    let head = format!(
        "HTTP/1.1 {code} {text}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body)?;
    Ok(())
}

fn request_target(request_line: &str) -> Option<&str> {
    let mut parts = request_line.split_whitespace();
    let method = parts.next()?;
    if method != "GET" && method != "HEAD" {
        return None;
    }
    parts.next()
}

fn query_param(target: &str, key: &str) -> Option<String> {
    let query = target.split_once('?')?.1;
    for pair in query.split('&') {
        let (name, value) = pair.split_once('=').unwrap_or((pair, ""));
        if name == key {
            return Some(value.to_string());
        }
    }
    None
}

fn mime_for(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .as_deref()
    {
        Some("webm") => "video/webm",
        Some("mp4") | Some("m4v") => "video/mp4",
        _ => "application/octet-stream",
    }
}

/// Decodes `%HH` sequences. `+` stays a plus (this is not form encoding).
pub fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' && index + 2 < bytes.len() {
            if let Ok(value) = u8::from_str_radix(
                std::str::from_utf8(&bytes[index + 1..index + 3]).unwrap_or(""),
                16,
            ) {
                out.push(value);
                index += 3;
                continue;
            }
        }
        out.push(bytes[index]);
        index += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// Canonical path inside `media_root`, or `None` when it escapes the jail.
pub fn resolve_under_root(media_root: &Path, raw: &str) -> Option<PathBuf> {
    let root = media_root.canonicalize().ok()?;
    let candidate = PathBuf::from(raw);
    let path = candidate.canonicalize().ok()?;
    if path.starts_with(&root) && path.is_file() {
        Some(path)
    } else {
        None
    }
}

/// Inclusive byte range. `None` when the header cannot be satisfied.
pub fn parse_range(header: &str, len: u64) -> Option<(u64, u64)> {
    if len == 0 {
        return None;
    }
    let spec = header.trim().strip_prefix("bytes=")?.trim();
    let spec = match spec.split_once(',') {
        Some((first, _)) => first,
        None => spec,
    };
    let (start_raw, end_raw) = spec.split_once('-')?;
    if start_raw.is_empty() {
        let tail: u64 = end_raw.parse().ok()?;
        if tail == 0 {
            return None;
        }
        let start = len.saturating_sub(tail);
        return Some((start, len - 1));
    }
    let start: u64 = start_raw.parse().ok()?;
    if start >= len {
        return None;
    }
    let end = if end_raw.is_empty() {
        len - 1
    } else {
        let parsed: u64 = end_raw.parse().ok()?;
        parsed.min(len - 1)
    };
    if end < start {
        return None;
    }
    Some((start, end))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Read;
    use std::net::TcpStream;

    #[test]
    fn range_covers_the_whole_file_when_open_ended() {
        assert_eq!(parse_range("bytes=0-", 7850496), Some((0, 7850495)));
        assert_eq!(parse_range("bytes=100-199", 1000), Some((100, 199)));
        assert_eq!(parse_range("bytes=100-", 150), Some((100, 149)));
        assert!(parse_range("bytes=5000-", 100).is_none());
    }

    #[test]
    fn rejects_paths_outside_the_media_root() {
        let dir = tempfile::tempdir().expect("tmp");
        let inside = dir.path().join("stage").join("1.mp4");
        std::fs::create_dir_all(inside.parent().expect("parent")).expect("dir");
        std::fs::write(&inside, b"mp4").expect("write");
        let outside = std::env::temp_dir().join(format!("jp-outside-{}.mp4", std::process::id()));
        std::fs::write(&outside, b"no").expect("out");
        assert!(resolve_under_root(dir.path(), &inside.to_string_lossy()).is_some());
        assert!(resolve_under_root(dir.path(), &outside.to_string_lossy()).is_none());
        let _ = std::fs::remove_file(&outside);
    }

    #[test]
    fn server_returns_the_full_open_range() {
        let dir = tempfile::tempdir().expect("tmp");
        let file = dir.path().join("clip.mp4");
        let body = vec![7u8; 2500];
        std::fs::write(&file, &body).expect("write");
        let origin = start(dir.path().to_path_buf()).expect("listen");
        let host = origin.trim_start_matches("http://");
        let mut stream = TcpStream::connect(host).expect("connect");
        let req = format!(
            "GET /media?path={} HTTP/1.1\r\nHost: {host}\r\nRange: bytes=0-\r\nConnection: close\r\n\r\n",
            percent_encode(&file.to_string_lossy())
        );
        stream.write_all(req.as_bytes()).expect("write");
        let mut response = Vec::new();
        stream.read_to_end(&mut response).expect("read");
        let text = String::from_utf8_lossy(&response);
        assert!(text.starts_with("HTTP/1.1 206"), "{text}");
        assert!(text.contains("Content-Range: bytes 0-2499/2500"), "{text}");
        let split = response.windows(4).position(|w| w == b"\r\n\r\n").expect("hdr");
        assert_eq!(&response[split + 4..], body.as_slice());
    }

    fn percent_encode(value: &str) -> String {
        let mut out = String::new();
        for byte in value.bytes() {
            match byte {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                    out.push(byte as char);
                }
                _ => out.push_str(&format!("%{byte:02X}")),
            }
        }
        out
    }
}
