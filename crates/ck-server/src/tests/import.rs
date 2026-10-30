//! `import` (spec 3.2 Source): probe, cache media and register a source as one revision.

use std::path::PathBuf;

use ck_core::ErrorCode;
use ck_core::model::{ItemBody, Mode, SourceKind, TrackKind};

use super::fixture::Fixture;
use crate::session::import::ImportRequest;

/// `fixtures/generated/`, or `None` when `just fixtures` has not run.
fn fixtures() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    dir.join("av.mp4")
        .exists()
        .then(|| dir.canonicalize().unwrap())
}

fn req(path: PathBuf) -> ImportRequest {
    ImportRequest {
        path: path.to_string_lossy().into_owned(),
    }
}

#[test]
fn video_import_caches_media_and_registers_once() {
    let Some(dir) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let f = Fixture::new();
    let first = f.session.import(&req(dir.join("av.mp4"))).unwrap();
    assert!(first.added);
    assert_eq!(first.rev, 1);
    let src = &first.source;
    assert_eq!(src.kind, SourceKind::Video);
    assert_eq!(src.id, format!("src-{}", &src.hash[..8]));
    assert!(src.duration_sec.unwrap() > 0.0);
    let proxy = src.proxy_path.as_deref().unwrap();
    assert!(
        f.dir
            .path()
            .join("cache/proxies")
            .read_dir()
            .unwrap()
            .count()
            == 1
    );
    assert!(proxy.ends_with(&format!("{}.mp4", src.hash)), "{proxy}");
    assert!(
        f.dir
            .path()
            .join(format!("cache/audio/{}.wav", src.hash))
            .exists()
    );
    let p = f.session.project().unwrap();
    assert_eq!(p.sources, vec![src.clone()]);

    let again = f.session.import(&req(dir.join("av.mp4"))).unwrap();
    assert!(!again.added);
    assert_eq!((again.rev, &again.source), (1, src));
}

#[test]
fn audio_and_image_imports_get_their_kinds() {
    let Some(dir) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let f = Fixture::new();
    let audio = f.session.import(&req(dir.join("tones.wav"))).unwrap();
    assert_eq!(audio.source.kind, SourceKind::Audio);
    assert_eq!(audio.source.proxy_path, None);
    let image = f
        .session
        .import(&req(dir.join("images/square.png")))
        .unwrap();
    assert_eq!(image.source.kind, SourceKind::Image);
    assert_eq!(image.source.duration_sec, None);
    assert_eq!(image.rev, 2);
}

#[test]
fn relative_and_missing_paths_are_refused() {
    let f = Fixture::new();
    let rel = f.session.import(&req("clip.mp4".into())).unwrap_err();
    assert_eq!(rel.code, ErrorCode::Request);
    let missing = f
        .session
        .import(&req("/no/such/clip.mp4".into()))
        .unwrap_err();
    assert_eq!(missing.code, ErrorCode::Media);
    assert_eq!(f.session.project().unwrap().rev, 0);
}

#[test]
fn video_into_generated_work_turns_hybrid_with_the_base_layer_below() {
    let Some(dir) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let f = Fixture::new();
    let r = f.session.import(&req(dir.join("av.mp4"))).unwrap();
    let p = f.session.project().unwrap();
    assert_eq!(p.mode, Mode::Hybrid);
    // The base layer sits on a new bottom track; the existing title is kept.
    assert_eq!(p.tracks[0].id, "footage-1");
    assert_eq!(p.tracks[0].kind, TrackKind::Video);
    let ItemBody::Footage {
        src_in,
        src_out,
        at,
        ..
    } = p.tracks[0].items[0].body
    else {
        panic!("footage expected");
    };
    assert_eq!((src_in, at), (0.0, 0.0));
    assert!((src_out - r.source.duration_sec.unwrap()).abs() < 0.001);
    assert!(p.item("title").is_some());

    // Generated layers that target the video track go over the footage instead.
    let applied = f
        .session
        .recipe_apply(
            &serde_json::from_value(
                serde_json::json!({ "recipe": "hook-title", "slots": { "headline": "Hi" } }),
            )
            .unwrap(),
        )
        .unwrap();
    let p = f.session.project().unwrap();
    assert_eq!(
        p.tracks[0].items.len(),
        1,
        "footage track holds footage only"
    );
    for id in &applied.layers {
        let track = p
            .tracks
            .iter()
            .find(|t| t.items.iter().any(|i| &i.id == id))
            .unwrap();
        assert_ne!(track.kind, TrackKind::Video, "{id}");
    }
}

#[test]
fn video_into_an_empty_project_turns_edit_and_audio_keeps_the_mode() {
    let Some(dir) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let f = Fixture::new();
    f.session.import(&req(dir.join("tones.wav"))).unwrap();
    assert_eq!(f.session.project().unwrap().mode, Mode::Generate);

    let empty = tempfile::tempdir().unwrap();
    ck_core::storage::Store::new(empty.path())
        .init(&ck_core::Project::new("p2", "Empty", Mode::Generate))
        .unwrap();
    let s = crate::session::Session::open(
        empty.path(),
        std::sync::Arc::new(super::fixture::FakeClock::default()),
    )
    .unwrap();
    s.import(&req(dir.join("av.mp4"))).unwrap();
    assert_eq!(s.project().unwrap().mode, Mode::Edit);
}

#[tokio::test]
async fn the_proxy_is_served_in_ranges_with_the_token_in_the_query() {
    use axum::http::{Request, StatusCode};

    use super::fixture::{HOST, TOKEN, body_of};

    let Some(dir) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let f = Fixture::new();
    let hash = f
        .session
        .import(&req(dir.join("av.mp4")))
        .unwrap()
        .source
        .hash;
    let get = |uri: String, range: Option<&str>| {
        let mut b = Request::builder().uri(uri).header("host", HOST);
        if let Some(r) = range {
            b = b.header("range", r);
        }
        b.body(body_of(None)).unwrap()
    };
    let uri = format!("/media/{hash}.mp4?token={TOKEN}");
    let (status, _) = f.send(get(uri.clone(), Some("bytes=0-99"))).await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    let (status, _) = f.send(get(uri, None)).await;
    assert_eq!(status, StatusCode::OK);
    let (status, _) = f.send(get(format!("/media/{hash}.mp4"), None)).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    let (status, _) = f
        .send(get(
            format!("/media/{}.mp4?token={TOKEN}", "0".repeat(64)),
            None,
        ))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = f
        .send(get(format!("/media/..%2Fx.mp4?token={TOKEN}"), None))
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn originals_are_served_for_imported_sources_only() {
    use axum::http::{Request, StatusCode};

    use super::fixture::{HOST, TOKEN, body_of};

    let Some(dir) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let f = Fixture::new();
    let hash = f
        .session
        .import(&req(dir.join("av.mp4")))
        .unwrap()
        .source
        .hash;
    let get = |uri: String| {
        Request::builder()
            .uri(uri)
            .header("host", HOST)
            .header("range", "bytes=0-99")
            .body(body_of(None))
            .unwrap()
    };
    let (status, _) = f
        .send(get(format!("/media/source/{hash}?token={TOKEN}")))
        .await;
    assert_eq!(status, StatusCode::PARTIAL_CONTENT);
    let (status, _) = f
        .send(get(format!(
            "/media/source/{}?token={TOKEN}",
            "a".repeat(64)
        )))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = f.send(get(format!("/media/source/{hash}"))).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}
