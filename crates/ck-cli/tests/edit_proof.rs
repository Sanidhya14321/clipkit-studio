//! Phase 5 proof (spec 12): raw footage to a cut to an exported MP4, through the real `ck`
//! binary against a real `ck studio`. The transcript is the fixture transcript attached by a
//! human patch, since `ck transcribe` needs the Fish Audio key (step 4b, BLOCKED).
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use std::path::PathBuf;

use ck_server::ops::caps;
use common::{Studio, err, ok};
use serde_json::json;

fn fixtures() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    dir.join("av.mp4")
        .exists()
        .then(|| dir.canonicalize().unwrap())
}

#[test]
fn raw_footage_to_a_cut_to_an_exported_mp4() {
    let Some(fx) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);

    // Import: probe, hash, proxy, audio; the footage becomes the base layer; mode EDIT.
    let av = fx.join("av.mp4");
    let imported = ok(d, &["import", av.to_str().unwrap()], caps::IMPORT);
    let id = imported["source"]["id"].as_str().unwrap().to_owned();
    let hash = imported["source"]["hash"].as_str().unwrap().to_owned();
    let duration = imported["source"]["durationSec"].as_f64().unwrap();
    let status = ok(d, &["status"], caps::STATUS);
    assert_eq!(status["mode"], "edit", "{status}");

    // Transcribe without a key is a clear voice error; attach the fixture transcript instead.
    assert_eq!(err(d, &["transcribe", &id])["code"], "E_VOICE");
    let rel = format!(".clipkit/cache/transcripts/{hash}.json");
    std::fs::create_dir_all(d.join(".clipkit/cache/transcripts")).unwrap();
    std::fs::copy(fx.join("transcript.json"), d.join(&rel)).unwrap();
    let attach = json!({
        "baseRev": imported["rev"], "author": "human",
        "ops": [{ "op": "add", "path": "/sources/0/transcriptPath", "value": rel }]
    });
    ok(d, &["project", "patch", &attach.to_string()], caps::PATCH);

    // The agent reads the packed transcript and the cut candidates, then cuts.
    let packed = ok(d, &["transcript", "get", &id], caps::TRANSCRIPT);
    assert_eq!(packed["fillers"], 3);
    let suggested = ok(d, &["cut", "suggest", &id], caps::CUT_SUGGEST);
    let kinds: Vec<&str> = suggested["cuts"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["kind"].as_str().unwrap())
        .collect();
    for kind in ["silence", "filler", "retake"] {
        assert!(kinds.contains(&kind), "{kinds:?}");
    }
    let cut = ok(d, &["cut", "apply", &id], caps::CUT_APPLY);
    let kept = cut["durationSec"].as_f64().unwrap();
    assert!(kept < duration - 4.0, "{cut}");

    // Export: a real MP4 of the kept duration at the target loudness.
    let exported = ok(d, &["export", "--preset", "youtube"], caps::EXPORT);
    let path = d.join(exported["path"].as_str().unwrap());
    let info = ck_media::probe(&path).unwrap();
    assert!((info.duration - kept).abs() < 0.15, "{exported}");
    let v = info.video.unwrap();
    assert_eq!((v.width, v.height), (1920, 1080));
    assert_eq!(exported["loudnessPass"], true, "{exported}");
    assert_eq!(info.audio.len(), 1);

    // The evaluator scores the export from the file alone; the source is unchanged.
    let expect = json!({
        "durationSec": kept, "aspect": "16:9",
        "sources": [{ "path": av, "hash": hash }]
    });
    let file = d.join("expect.json");
    std::fs::write(&file, expect.to_string()).unwrap();
    let scored = ok(
        d,
        &[
            "eval",
            exported["path"].as_str().unwrap(),
            "--expect",
            file.to_str().unwrap(),
        ],
        ck_eval::caps::EVAL,
    );
    assert_eq!(scored["score"], 100, "{scored}");
}
