//! `GET /media/<hash>.mp4`: the 720p proxy of an imported source, for the studio preview. A
//! `<video>` element cannot send headers, so this path (like `/ws`) takes the token from the
//! `token` query parameter. Byte ranges are served so the player can seek.

use std::io::{Read, Seek, SeekFrom};
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::{HeaderMap, HeaderValue, StatusCode, header};
use axum::response::{IntoResponse, Response};
use ck_core::{CkError, ErrorCode};
use ck_media::CachePaths;

use crate::http_error::ApiError;
use crate::session::Session;

/// Largest slice sent for one open-ended range request.
const MAX_CHUNK: u64 = 4 * 1024 * 1024;

/// `bytes=a-b` or `bytes=a-` against a file of `len` bytes, as an inclusive `(start, end)`.
pub fn parse_range(value: &str, len: u64) -> Option<(u64, u64)> {
    let (a, b) = value.strip_prefix("bytes=")?.split_once('-')?;
    let start: u64 = a.trim().parse().ok()?;
    if start >= len {
        return None;
    }
    let end = match b.trim() {
        "" => (start + MAX_CHUNK - 1).min(len - 1),
        b => b.parse::<u64>().ok()?.min(len - 1),
    };
    (end >= start).then_some((start, end))
}

fn media_err(e: impl std::fmt::Display) -> ApiError {
    ApiError(CkError::new(ErrorCode::Media, e.to_string()))
}

pub async fn proxy(
    State(s): State<Arc<Session>>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let hash = name.strip_suffix(".mp4").unwrap_or("");
    let paths = CachePaths::new(&s.cache_dir(), hash)?;
    serve_file(paths.proxy, "video/mp4", &headers).await
}

/// `GET /media/source/<hash>`: the original media of an imported source, for the render
/// worker (spec 2.3: render against original media). Only files the project imported.
pub async fn original(
    State(s): State<Arc<Session>>,
    Path(hash): Path<String>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let found = tokio::task::spawn_blocking(move || s.source_file(&hash))
        .await
        .map_err(media_err)??;
    let kind = match found
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_lowercase)
        .as_deref()
    {
        Some("webm") => "video/webm",
        Some("mov") => "video/quicktime",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "video/mp4",
    };
    serve_file(found, kind, &headers).await
}

/// Serves `path` with byte ranges (open ranges capped at 4 MB per reply).
async fn serve_file(
    path: std::path::PathBuf,
    kind: &'static str,
    headers: &HeaderMap,
) -> Result<Response, ApiError> {
    let range = headers
        .get(header::RANGE)
        .and_then(|v| v.to_str().ok())
        .map(str::to_owned);
    let read = tokio::task::spawn_blocking(move || -> Result<_, ApiError> {
        let mut file = std::fs::File::open(&path).map_err(|_| {
            ApiError(CkError::new(
                ErrorCode::NotFound,
                format!("no media {}", path.display()),
            ))
        })?;
        let len = file.metadata().map_err(media_err)?.len();
        let span = match range {
            Some(r) => Some(parse_range(&r, len).ok_or_else(|| {
                ApiError(CkError::new(ErrorCode::Request, format!("bad range {r:?}")))
            })?),
            None => None,
        };
        let (start, end) = span.unwrap_or((0, len.saturating_sub(1)));
        file.seek(SeekFrom::Start(start)).map_err(media_err)?;
        let mut buf = Vec::new();
        file.take(end + 1 - start)
            .read_to_end(&mut buf)
            .map_err(media_err)?;
        Ok((buf, span.map(|_| (start, end, len))))
    })
    .await
    .map_err(media_err)??;
    let (body, span) = read;
    let mut resp = body.into_response();
    let h = resp.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static(kind));
    h.insert(header::ACCEPT_RANGES, HeaderValue::from_static("bytes"));
    if let Some((start, end, len)) = span {
        *resp.status_mut() = StatusCode::PARTIAL_CONTENT;
        let value = format!("bytes {start}-{end}/{len}");
        if let Ok(v) = HeaderValue::from_str(&value) {
            resp.headers_mut().insert(header::CONTENT_RANGE, v);
        }
    }
    Ok(resp)
}

/// `GET /media/storyboard/latest.png`: the newest storyboard sheet, for the studio's `<img>`
/// (token in the query, like the proxies).
pub async fn storyboard_sheet(State(s): State<Arc<Session>>) -> Result<Response, ApiError> {
    let dir = s.cache_dir().with_file_name("storyboard");
    let newest = std::fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| {
            p.extension().is_some_and(|e| e == "png")
                && p.file_name()
                    .is_some_and(|n| n.to_string_lossy().starts_with("sheet-"))
        })
        .max_by_key(|p| std::fs::metadata(p).and_then(|m| m.modified()).ok());
    let path =
        newest.ok_or_else(|| ApiError(CkError::new(ErrorCode::NotFound, "no storyboard yet")))?;
    let bytes = std::fs::read(&path).map_err(media_err)?;
    let mut resp = bytes.into_response();
    resp.headers_mut()
        .insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
    resp.headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    Ok(resp)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranges_are_clamped_to_the_file() {
        assert_eq!(parse_range("bytes=0-99", 1000), Some((0, 99)));
        assert_eq!(parse_range("bytes=900-", 1000), Some((900, 999)));
        assert_eq!(parse_range("bytes=10-5000", 1000), Some((10, 999)));
        assert_eq!(
            parse_range("bytes=0-", 10 * MAX_CHUNK).unwrap().1,
            MAX_CHUNK - 1
        );
        assert_eq!(parse_range("bytes=1000-", 1000), None);
        assert_eq!(parse_range("bytes=5-1", 1000), None);
        assert_eq!(parse_range("items=0-1", 1000), None);
    }
}
