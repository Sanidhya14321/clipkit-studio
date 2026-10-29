use std::path::PathBuf;

use super::*;
use crate::probe;

const HASH: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";

#[test]
fn hash_is_sha256_of_the_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("abc.bin");
    std::fs::write(&f, b"abc").unwrap();
    assert_eq!(hash_file(&f).unwrap(), HASH);
}

#[test]
fn hash_of_a_missing_file_is_a_media_error() {
    let err = hash_file(Path::new("/no/such/file.mp4")).unwrap_err();
    assert_eq!(err.code, ErrorCode::Media);
}

#[test]
fn cache_paths_are_keyed_by_hash() {
    let p = CachePaths::new(Path::new(".clipkit/cache"), HASH).unwrap();
    assert_eq!(
        p.proxy,
        PathBuf::from(format!(".clipkit/cache/proxies/{HASH}.mp4"))
    );
    assert_eq!(
        p.audio,
        PathBuf::from(format!(".clipkit/cache/audio/{HASH}.wav"))
    );
}

#[test]
fn cache_paths_refuse_a_key_that_is_not_hex() {
    for bad in [
        "",
        "abc",
        "../../etc/passwd/aaaaaaaaaaaaaaaa",
        "zzzzzzzzzzzzzzzzzz",
    ] {
        let err = CachePaths::new(Path::new("c"), bad).unwrap_err();
        assert_eq!(err.code, ErrorCode::Request, "{bad:?}");
    }
}

#[test]
fn transcode_refuses_option_like_paths() {
    let err = make_proxy(Path::new("-i"), Path::new("out.mp4")).unwrap_err();
    assert_eq!(err.code, ErrorCode::Request);
    let err = extract_audio(Path::new("in.mp4"), Path::new("-y")).unwrap_err();
    assert_eq!(err.code, ErrorCode::Request);
}

/// `fixtures/generated/`, or `None` when `just fixtures` has not run.
fn fixtures() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    dir.join("ground-truth.json").is_file().then_some(dir)
}

#[test]
fn proxy_and_audio_from_fixture_media() {
    let Some(dir) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let src = dir.join("av.mp4");
    let out = tempfile::tempdir().unwrap();
    let paths = CachePaths::new(out.path(), &hash_file(&src).unwrap()).unwrap();

    make_proxy(&src, &paths.proxy).unwrap();
    let p = probe(&paths.proxy).unwrap();
    let v = p.video.unwrap();
    assert!(v.display_height <= PROXY_MAX_HEIGHT && v.display_height.is_multiple_of(2));
    assert_eq!(v.codec, "h264");
    assert_eq!(p.audio.len(), 1);
    assert!((p.duration - 12.0).abs() < 0.2, "{}", p.duration);

    extract_audio(&src, &paths.audio).unwrap();
    let a = probe(&paths.audio).unwrap();
    assert!(a.video.is_none());
    assert_eq!(a.audio[0].sample_rate, AUDIO_SAMPLE_RATE);
    assert!((a.duration - 12.0).abs() < 0.2, "{}", a.duration);

    // No temp files left behind; a second call is a cache hit and leaves the file as is.
    let names: Vec<_> = std::fs::read_dir(paths.audio.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(names.len(), 1);
    let before = std::fs::metadata(&paths.audio).unwrap().modified().unwrap();
    extract_audio(&src, &paths.audio).unwrap();
    let after = std::fs::metadata(&paths.audio).unwrap().modified().unwrap();
    assert_eq!(before, after);
}

#[test]
fn audio_from_a_silent_video_is_a_media_error_and_leaves_no_file() {
    let Some(dir) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let src = dir.join("counter.mp4");
    if !probe(&src).unwrap().audio.is_empty() {
        eprintln!("skipped: counter.mp4 has audio");
        return;
    }
    let out = tempfile::tempdir().unwrap();
    let wav = out.path().join("audio/x.wav");
    let err = extract_audio(&src, &wav).unwrap_err();
    assert_eq!(err.code, ErrorCode::Media);
    let left = std::fs::read_dir(wav.parent().unwrap()).unwrap().count();
    assert_eq!(left, 0);
}
