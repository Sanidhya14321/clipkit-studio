use std::path::PathBuf;

use serde_json::Value;

use super::*;

#[test]
fn silences_pair_up_and_an_open_one_closes_at_the_end() {
    let out = "frame:0 pts:0 pts_time:0\nlavfi.silence_start=0\nlavfi.silence_end=1.000021\n\
               lavfi.silence_duration=1.000021\nlavfi.silence_start=11\n";
    let got = parse_silences(out, 12.0);
    assert_eq!(
        got,
        [
            Silence {
                start: 0.0,
                end: 1.000021
            },
            Silence {
                start: 11.0,
                end: 12.0
            }
        ]
    );
}

#[test]
fn scenes_carry_their_score() {
    let out = "frame:60 pts:30720 pts_time:2\nlavfi.scd.mafd=49.4\nlavfi.scd.score=49.4\n\
               lavfi.scd.time=2\nframe:61 pts:1 pts_time:2.03\nlavfi.scd.score=0.000\n";
    assert_eq!(
        parse_scenes(out),
        [SceneChange {
            at: 2.0,
            score: 49.4
        }]
    );
}

#[test]
fn option_like_paths_are_refused() {
    assert!(detect_scenes(Path::new("-i")).is_err());
    assert!(detect_silences(Path::new("-i"), 1.0).is_err());
}

fn ground_truth() -> Option<(PathBuf, Value)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    let text = std::fs::read_to_string(dir.join("ground-truth.json")).ok()?;
    Some((dir, serde_json::from_str(&text).unwrap()))
}

fn ranges(v: &Value) -> Vec<(f64, f64)> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|r| (r[0].as_f64().unwrap(), r[1].as_f64().unwrap()))
        .collect()
}

#[test]
fn finds_fixture_silences_and_scene_changes() {
    let Some((dir, truth)) = ground_truth() else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let tones = &truth["tones"];
    let dur = tones["durationSec"].as_f64().unwrap();
    let got = detect_silences(&dir.join("tones.wav"), dur).unwrap();
    let want = ranges(&tones["silences"]);
    assert_eq!(got.len(), want.len(), "{got:?}");
    for (g, (s, e)) in got.iter().zip(want) {
        assert!(
            (g.start - s).abs() < 0.05 && (g.end - e).abs() < 0.05,
            "{g:?}"
        );
    }

    let scenes = detect_scenes(&dir.join("scenes.mp4")).unwrap();
    let at: Vec<f64> = scenes.iter().map(|s| s.at).collect();
    let want: Vec<f64> = truth["scenes"]["changesAtSec"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_f64().unwrap())
        .collect();
    assert_eq!(at.len(), want.len(), "{at:?}");
    for (g, w) in at.iter().zip(want) {
        assert!((g - w).abs() < 0.05, "{at:?}");
    }
}

#[test]
fn black_and_frozen_stretches_pair_up() {
    let out = "lavfi.black_start=0\nlavfi.freezedetect.freeze_start=0\nlavfi.black_end=1\n\
               lavfi.freezedetect.freeze_duration=1\nlavfi.freezedetect.freeze_end=1\n\
               lavfi.black_start=4\n";
    let (black, frozen) = parse_black_freeze(out, 5.0);
    assert_eq!(
        black,
        [
            Span {
                start: 0.0,
                end: 1.0
            },
            Span {
                start: 4.0,
                end: 5.0
            }
        ]
    );
    assert_eq!(
        frozen,
        [Span {
            start: 0.0,
            end: 1.0
        }]
    );
}
