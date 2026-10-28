use std::borrow::Cow;

use axum::body::Body;
use axum::http::{HeaderMap, Request, StatusCode};
use tower::ServiceExt;

use super::fixture::{Fixture, HOST, PORT, TOKEN};
use crate::api::router_with_assets;
use crate::auth::{Guard, TOKEN_HEADER};
use crate::studio::Assets;

fn fake_build(path: &str) -> Option<Cow<'static, [u8]>> {
    match path {
        "index.html" => Some(Cow::Borrowed(b"<!doctype html><div id=root></div>")),
        "assets/index-abc.js" => Some(Cow::Borrowed(b"console.log(1)")),
        _ => None,
    }
}

fn no_build(_: &str) -> Option<Cow<'static, [u8]>> {
    None
}

async fn fetch(
    assets: Assets,
    method: &str,
    uri: &str,
    token: bool,
) -> (StatusCode, HeaderMap, String) {
    let f = Fixture::new();
    let mut req = Request::builder()
        .method(method)
        .uri(uri)
        .header("host", HOST);
    if token {
        req = req.header(TOKEN_HEADER, TOKEN);
    }
    let router = router_with_assets(f.session.clone(), Guard::new(PORT, TOKEN), assets);
    let resp = router
        .oneshot(req.body(Body::empty()).unwrap())
        .await
        .unwrap();
    let (parts, body) = resp.into_parts();
    let bytes = axum::body::to_bytes(body, usize::MAX).await.unwrap();
    (
        parts.status,
        parts.headers,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

#[tokio::test]
async fn index_is_served_without_a_token() {
    let (code, headers, body) = fetch(fake_build, "GET", "/", false).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(headers["content-type"], "text/html; charset=utf-8");
    assert_eq!(headers["cache-control"], "no-cache");
    assert_eq!(headers["x-content-type-options"], "nosniff");
    assert!(body.contains("id=root"));
}

#[tokio::test]
async fn client_routes_fall_back_to_index() {
    let (code, headers, body) = fetch(fake_build, "GET", "/timeline/t1", false).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(headers["content-type"], "text/html; charset=utf-8");
    assert!(body.contains("id=root"));
}

#[tokio::test]
async fn bundle_files_get_their_type_and_long_cache() {
    let (code, headers, body) = fetch(fake_build, "GET", "/assets/index-abc.js", false).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(headers["content-type"], "text/javascript; charset=utf-8");
    assert!(
        headers["cache-control"]
            .to_str()
            .unwrap()
            .contains("immutable")
    );
    assert_eq!(body, "console.log(1)");
}

#[tokio::test]
async fn missing_files_and_traversal_are_not_found() {
    for uri in [
        "/assets/missing.js",
        "/favicon.ico",
        "/assets/../index.html",
        "/../Cargo.toml",
    ] {
        let (code, _, body) = fetch(fake_build, "GET", uri, false).await;
        assert_eq!(code, StatusCode::NOT_FOUND, "{uri}");
        assert!(body.contains("E_NOT_FOUND"), "{uri}");
    }
}

#[tokio::test]
async fn unknown_api_paths_stay_json_errors() {
    for uri in ["/api/nope", "/api", "/ws/", "//api/x", "/API/x"] {
        let (code, headers, body) = fetch(fake_build, "GET", uri, true).await;
        assert_eq!(code, StatusCode::NOT_FOUND, "{uri}");
        assert_eq!(headers["content-type"], "application/json", "{uri}");
        assert!(body.contains("E_NOT_FOUND"), "{uri}");
    }
}

#[tokio::test]
async fn only_get_and_head_reach_static_files() {
    let (code, headers, body) = fetch(fake_build, "POST", "/", false).await;
    assert_eq!(code, StatusCode::METHOD_NOT_ALLOWED);
    assert_eq!(headers["allow"], "GET, HEAD");
    assert!(body.contains("E_REQUEST"));
    let (code, _, _) = fetch(fake_build, "HEAD", "/", false).await;
    assert_eq!(code, StatusCode::OK);
}

#[tokio::test]
async fn api_still_needs_the_token_next_to_static_files() {
    let (code, _, body) = fetch(fake_build, "GET", "/api/status", false).await;
    assert_eq!(code, StatusCode::FORBIDDEN);
    assert!(body.contains("E_AUTH"));
}

#[tokio::test]
async fn static_files_still_check_host() {
    let f = Fixture::new();
    let router = router_with_assets(f.session.clone(), Guard::new(PORT, TOKEN), fake_build);
    let req = Request::builder()
        .uri("/")
        .header("host", "evil.example:4000")
        .body(Body::empty())
        .unwrap();
    let resp = router.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn missing_build_says_how_to_build() {
    let (code, headers, body) = fetch(no_build, "GET", "/", false).await;
    assert_eq!(code, StatusCode::OK);
    assert_eq!(headers["content-type"], "text/html; charset=utf-8");
    assert!(body.contains("just studio-build"));
}
