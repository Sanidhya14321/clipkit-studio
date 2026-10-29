use std::path::PathBuf;

use serde_json::{Value, json};

use super::*;

fn parse(v: Value) -> Result<MediaInfo, CkError> {
    parse_probe(v.to_string().as_bytes())
}

fn video(extra: Value) -> Value {
    let mut s = json!({
        "index": 0, "codec_type": "video", "codec_name": "h264", "width": 1920, "height": 1080,
        "r_frame_rate": "30/1", "avg_frame_rate": "30/1"
    });
    if let (Some(s), Some(extra)) = (s.as_object_mut(), extra.as_object()) {
        s.extend(extra.clone());
    }
    s
}

fn file(streams: Vec<Value>) -> Value {
    json!({ "streams": streams, "format": { "format_name": "mov,mp4", "duration": "12.5" } })
}

#[test]
fn constant_rate_landscape() {
    let m = parse(file(vec![video(json!({}))])).unwrap();
    let v = m.video.unwrap();
    assert_eq!(
        (m.duration, v.fps, v.variable_frame_rate),
        (12.5, 30.0, false)
    );
    assert_eq!(
        (v.rotation, v.display_width, v.display_height),
        (0, 1920, 1080)
    );
    assert!(m.audio.is_empty());
}

#[test]
fn variable_frame_rate_is_flagged() {
    let m = parse(file(vec![video(json!({ "avg_frame_rate": "2997/125" }))])).unwrap();
    let v = m.video.unwrap();
    assert!(v.variable_frame_rate);
    assert!((v.fps - 23.976).abs() < 1e-9);
    let ntsc = parse(file(vec![video(
        json!({ "r_frame_rate": "30000/1001", "avg_frame_rate": "30000/1001" }),
    )]));
    assert!(!ntsc.unwrap().video.unwrap().variable_frame_rate);
}

#[test]
fn rotation_from_side_data_or_tag_swaps_display_size() {
    let side = parse(file(vec![video(
        json!({ "side_data_list": [{ "rotation": -90 }] }),
    )]))
    .unwrap();
    let v = side.video.unwrap();
    assert_eq!(
        (v.rotation, v.display_width, v.display_height),
        (270, 1080, 1920)
    );
    let tag = parse(file(vec![video(json!({ "tags": { "rotate": "180" } }))])).unwrap();
    let v = tag.video.unwrap();
    assert_eq!(
        (v.rotation, v.display_width, v.display_height),
        (180, 1920, 1080)
    );
    assert_eq!(normalize_rotation(450.0), 90);
}

#[test]
fn several_audio_tracks_and_cover_art() {
    let cover =
        video(json!({ "index": 0, "codec_name": "mjpeg", "disposition": { "attached_pic": 1 } }));
    let a = |i: u32| json!({ "index": i, "codec_type": "audio", "codec_name": "aac", "sample_rate": "48000", "channels": 2 });
    let m = parse(file(vec![cover, a(1), a(2)])).unwrap();
    assert!(m.video.is_none(), "cover art is not a video stream");
    assert_eq!(m.audio.len(), 2);
    assert_eq!(
        (
            m.audio[1].index,
            m.audio[1].sample_rate,
            m.audio[1].channels
        ),
        (2, 48000, 2)
    );
}

#[test]
fn duration_falls_back_to_streams() {
    let v = json!({ "streams": [video(json!({ "duration": "4.0" }))], "format": { "format_name": "matroska" } });
    assert_eq!(parse(v).unwrap().duration, 4.0);
}

#[test]
fn unusable_files_are_media_errors() {
    for bad in [
        json!({ "streams": [], "format": { "duration": "3" } }),
        json!({ "streams": [video(json!({}))], "format": {} }),
        json!({ "streams": [{ "index": 0, "codec_type": "subtitle" }] }),
    ] {
        assert_eq!(parse(bad).unwrap_err().code, ErrorCode::Media);
    }
    assert_eq!(parse_probe(b"not json").unwrap_err().code, ErrorCode::Media);
}

#[test]
fn option_like_and_missing_paths_are_refused() {
    assert_eq!(
        probe(Path::new("-version")).unwrap_err().code,
        ErrorCode::Request
    );
    assert_eq!(probe(Path::new("")).unwrap_err().code, ErrorCode::Request);
    let missing = std::env::temp_dir().join("ck-media-no-such-file.mp4");
    assert_eq!(probe(&missing).unwrap_err().code, ErrorCode::Media);
}

/// `fixtures/generated/` with its ground truth, or `None` when `just fixtures` has not run.
fn fixtures() -> Option<(PathBuf, Value)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    let truth = std::fs::read(dir.join("ground-truth.json")).ok()?;
    Some((dir, serde_json::from_slice(&truth).unwrap()))
}

#[test]
fn probes_fixture_media_against_ground_truth() {
    let Some((dir, truth)) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let f = |k: &str| dir.join(truth[k]["file"].as_str().unwrap());
    let secs = |k: &str| truth[k]["durationSec"].as_f64().unwrap();

    let counter = probe(&f("counter")).unwrap();
    let v = counter.video.unwrap();
    assert!((counter.duration - secs("counter")).abs() < 0.1);
    assert_eq!((v.width, v.height), (1280, 720));
    assert_eq!(v.fps, truth["fps"].as_f64().unwrap());
    assert!(!v.variable_frame_rate);

    let av = probe(&f("av")).unwrap();
    assert!(av.video.is_some());
    assert_eq!(av.audio.len(), 1);
    assert!((av.duration - secs("av")).abs() < 0.1);

    let tones = probe(&f("tones")).unwrap();
    assert!(tones.video.is_none());
    assert_eq!(
        tones.audio[0].sample_rate,
        truth["tones"]["sampleRate"].as_u64().unwrap() as u32
    );
    assert!((tones.duration - secs("tones")).abs() < 0.1);
}
