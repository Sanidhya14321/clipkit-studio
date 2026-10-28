//! The built studio (`studio/dist`), embedded with rust-embed. Release builds carry the files in
//! the binary; debug builds read them from disk, so `just studio-build` shows up without a
//! rebuild. Static files hold no project data and need no token; the site guard (Host, Origin)
//! still runs in front of them.

use std::borrow::Cow;

use axum::http::{HeaderValue, Method, StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};
use ck_core::{CkError, ErrorCode};
use rust_embed::RustEmbed;

use crate::http_error::ApiError;

#[derive(RustEmbed)]
#[folder = "../../studio/dist"]
#[allow_missing = true]
struct Dist;

/// Looks up one file of the built studio by its path relative to `dist/`.
pub type Assets = fn(&str) -> Option<Cow<'static, [u8]>>;

/// The embedded studio build.
pub fn embedded(path: &str) -> Option<Cow<'static, [u8]>> {
    Dist::get(path).map(|f| f.data)
}

const INDEX: &str = "index.html";

/// Shown when the binary was built before the studio was.
const NOT_BUILT: &str = "<!doctype html><title>ClipKit Studio</title>\
<p>The studio is not built into this binary. Run <code>just studio-build</code>, then rebuild ck.</p>";

/// Fallback for every path the API does not route: a file of the build, or `index.html` for
/// client-side routes (paths without an extension). Unknown API paths stay JSON errors.
pub async fn serve(assets: Assets, method: Method, uri: Uri) -> Response {
    let path = uri.path();
    let rel = path.trim_start_matches('/');
    let first = rel.split('/').next().unwrap_or("").to_ascii_lowercase();
    if first == "api" || first == "ws" {
        return not_found(path);
    }
    if method != Method::GET && method != Method::HEAD {
        let err = CkError::new(
            ErrorCode::Request,
            format!("{method} is not allowed on {path}"),
        )
        .with_hint("Studio files are read-only; send edits to /api/patch");
        let mut resp = ApiError(err).into_response();
        *resp.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        resp.headers_mut()
            .insert(header::ALLOW, HeaderValue::from_static("GET, HEAD"));
        return resp;
    }
    if rel.split('/').any(|seg| seg == ".." || seg.contains('\\')) {
        return not_found(path);
    }
    match assets(rel).filter(|_| !rel.is_empty()) {
        Some(data) => file_response(data, rel),
        None if has_extension(rel) => not_found(path),
        None => {
            let index = assets(INDEX).unwrap_or(Cow::Borrowed(NOT_BUILT.as_bytes()));
            file_response(index, INDEX)
        }
    }
}

fn file_response(data: Cow<'static, [u8]>, file: &str) -> Response {
    // Hashed bundle files never change; index.html must be re-read to pick up a new build.
    let cache = if file.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "no-cache"
    };
    let mut resp = data.into_owned().into_response();
    let headers = resp.headers_mut();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(content_type(file)),
    );
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static(cache));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(CSP),
    );
    headers.insert(header::X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    resp
}

/// Studio page policy: only our own scripts and our own server (API and `/ws`); no framing.
/// Inline styles stay allowed because React and Remotion set `style` attributes; the Player
/// loads `data:` media (seen as a media-src violation in the e2e run), so media allows `data:`.
pub const CSP: &str = "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; \
    img-src 'self' data: blob:; media-src 'self' data: blob:; font-src 'self' data:; \
    connect-src 'self'; worker-src 'self' blob:; object-src 'none'; base-uri 'none'; \
    form-action 'self'; frame-ancestors 'none'";

fn not_found(path: &str) -> Response {
    let err = CkError::new(
        ErrorCode::NotFound,
        format!("no route or studio file {path}"),
    )
    .with_hint("Check the URL; the API lives under /api and the studio at /");
    ApiError(err).into_response()
}

fn has_extension(rel: &str) -> bool {
    rel.rsplit('/')
        .next()
        .is_some_and(|name| name.contains('.'))
}

fn content_type(file: &str) -> &'static str {
    match file.rsplit_once('.').map(|(_, ext)| ext) {
        Some("html") => "text/html; charset=utf-8",
        Some("js" | "mjs") => "text/javascript; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("json" | "map") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("ico") => "image/x-icon",
        Some("woff2") => "font/woff2",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}
