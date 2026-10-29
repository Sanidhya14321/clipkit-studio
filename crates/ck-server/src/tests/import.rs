//! `import` (spec 3.2 Source): probe, cache media and register a source as one revision.

use std::path::PathBuf;

use ck_core::ErrorCode;
use ck_core::model::SourceKind;

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
