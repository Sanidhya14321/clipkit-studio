use std::path::PathBuf;

use super::*;

fn fixtures() -> Option<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    dir.join("scenes.mp4").is_file().then_some(dir)
}

fn req(video: &Path, expect: Expectations) -> EvalRequest {
    EvalRequest {
        video: video.to_string_lossy().into_owned(),
        expect,
    }
}

#[test]
fn ratios_parse() {
    assert_eq!(ratio("16:9"), Some(16.0 / 9.0));
    assert_eq!(ratio("x"), None);
    assert_eq!(ratio("0:9"), None);
}

#[test]
fn the_scene_fixture_meets_its_ground_truth() {
    let Some(dir) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let truth: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("ground-truth.json")).unwrap())
            .unwrap();
    let cuts: Vec<f64> = truth["scenes"]["changesAtSec"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    let video = dir.join("scenes.mp4");
    let r = evaluate(
        &dir,
        &req(
            &video,
            Expectations {
                duration_sec: Some(10.0),
                aspect: Some("16:9".into()),
                cuts_at_sec: cuts,
                sources: vec![SourceHash {
                    path: "scenes.mp4".into(),
                    hash: hash_file(&video).unwrap(),
                }],
                ..Default::default()
            },
        ),
    )
    .unwrap();
    let names: Vec<(&str, bool)> = r.checks.iter().map(|c| (c.name.as_str(), c.pass)).collect();
    assert!(names.contains(&("cut accuracy", true)), "{names:?}");
    assert!(names.contains(&("duration", true)));
    assert!(names.contains(&("source unchanged", true)));
    // Each solid-colour scene holds 2 s: below the frozen limit.
    assert!(names.contains(&("frozen frames", true)), "{:?}", r.checks);
    assert!(r.checks.iter().any(|c| c.name == "loudness" && !c.pass));
}

#[test]
fn wrong_expectations_fail_and_lower_the_score() {
    let Some(dir) = fixtures() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let r = evaluate(
        &dir,
        &req(
            &dir.join("av.mp4"),
            Expectations {
                duration_sec: Some(30.0),
                aspect: Some("9:16".into()),
                sources: vec![SourceHash {
                    path: "av.mp4".into(),
                    hash: "0".repeat(64),
                }],
                ..Default::default()
            },
        ),
    )
    .unwrap();
    let failed: Vec<&str> = r
        .checks
        .iter()
        .filter(|c| !c.pass)
        .map(|c| c.name.as_str())
        .collect();
    for name in ["duration", "aspect", "source unchanged"] {
        assert!(failed.contains(&name), "{name}: {:?}", r.checks);
    }
    assert!(r.score < 100);
    let missing = evaluate(&dir, &req(Path::new("none.mp4"), Expectations::default())).unwrap_err();
    assert_eq!(missing.code, ErrorCode::NotFound);
}
