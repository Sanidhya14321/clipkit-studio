use std::path::PathBuf;

use serde_json::Value;

use super::*;
use crate::pack::pack;
use crate::transcript::{FishStt, Transcriber, load, save};

/// The same words as `fixtures/generated/transcript.json` (fixtures/generate.sh).
fn sample() -> Transcript {
    let words = [
        ("Welcome", 1.00, 1.40),
        ("um", 1.45, 1.70),
        ("to", 1.75, 1.85),
        ("ClipKit", 1.85, 2.00),
        ("today", 3.00, 3.30),
        ("we", 3.30, 3.45),
        ("will", 3.45, 3.60),
        ("show", 3.60, 3.90),
        ("uh", 3.95, 4.20),
        ("today", 6.50, 6.70),
        ("we'll", 6.70, 6.85),
        ("show", 6.85, 7.00),
        ("you", 9.00, 9.30),
        ("like", 9.35, 9.60),
        ("the", 9.65, 9.80),
        ("editor", 9.80, 10.40),
        ("live", 10.40, 11.00),
    ];
    Transcript {
        language: "en".into(),
        words: words
            .iter()
            .enumerate()
            .map(|(i, (t, s, e))| Word {
                i,
                text: (*t).into(),
                start: *s,
                end: *e,
            })
            .collect(),
    }
}

#[test]
fn finds_the_fixture_fillers_and_retake() {
    let cuts = suggest_cuts(&sample(), 12.0);
    let fillers: Vec<usize> = cuts
        .iter()
        .filter(|c| c.kind == CutKind::Filler)
        .map(|c| c.words.unwrap()[0])
        .collect();
    // Filler 8 ("uh") sits inside the abandoned take, so the retake cut covers it.
    assert_eq!(fillers, [1, 13]);
    let retake = cuts.iter().find(|c| c.kind == CutKind::Retake).unwrap();
    assert_eq!(retake.words, Some([4, 8]));
    assert_eq!((retake.start, retake.end), (3.0, 6.5));
    // No overlaps, sorted, numbered.
    for pair in cuts.windows(2) {
        assert!(pair[0].end <= pair[1].start, "{cuts:?}");
    }
    assert_eq!(cuts[0].id, "cut-1");
    // Leading and trailing silence are cut to the edge.
    assert_eq!(
        (cuts[0].kind, cuts[0].start, cuts[0].end),
        (CutKind::Silence, 0.0, 0.85)
    );
    let last = cuts.last().unwrap();
    assert_eq!((last.kind, last.end), (CutKind::Silence, 12.0));
}

#[test]
fn kept_ranges_complement_the_cuts() {
    let cuts = suggest_cuts(&sample(), 12.0);
    let ranges: Vec<(f64, f64)> = cuts.iter().map(|c| (c.start, c.end)).collect();
    let kept = kept_ranges(&ranges, 12.0);
    let cut: f64 = cuts.iter().map(|c| c.end - c.start).sum();
    let keep: f64 = kept.iter().map(|(s, e)| e - s).sum();
    assert!((cut + keep - 12.0).abs() < 1e-9);
    assert_eq!(kept_ranges(&[], 5.0), [(0.0, 5.0)]);
}

#[test]
fn pack_marks_fillers_and_silences_compactly() {
    let p = pack(&sample());
    assert_eq!((p.words, p.fillers), (17, 3));
    assert_eq!(p.lines[0], "~ 0.00-1.00 silence");
    assert_eq!(p.lines[1], "#0 1.00-2.00 Welcome (um) to ClipKit");
    assert!(
        p.lines
            .contains(&"#4 3.00-4.20 today we will show (uh)".to_owned())
    );
    // About 12 KB per 10 minutes: this 12 s sample stays far below 1 KB.
    assert!(serde_json::to_vec(&p).unwrap().len() < 1024);
}

#[test]
fn transcripts_round_trip_and_bad_ones_are_refused() {
    let dir = tempfile::tempdir().unwrap();
    let f = dir.path().join("t/x.json");
    save(&f, &sample()).unwrap();
    assert_eq!(load(&f).unwrap(), sample());
    let mut bad = sample();
    bad.words.swap(0, 1);
    assert_eq!(save(&f, &bad).unwrap_err().code, ck_core::ErrorCode::Voice);
    let missing = load(&dir.path().join("none.json")).unwrap_err();
    assert_eq!(missing.code, ck_core::ErrorCode::NotFound);
}

#[test]
fn fish_stt_without_a_key_is_a_voice_error() {
    let err = FishStt::default()
        .transcribe(std::path::Path::new("a.wav"))
        .unwrap_err();
    assert_eq!(err.code, ck_core::ErrorCode::Voice);
}

#[test]
fn matches_fixture_ground_truth() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    let Ok(text) = std::fs::read_to_string(dir.join("ground-truth.json")) else {
        eprintln!("skipped: run `just fixtures` first");
        return;
    };
    let truth: Value = serde_json::from_str(&text).unwrap();
    let t = load(&dir.join("transcript.json")).unwrap();
    assert_eq!(t, sample());
    let p = pack(&t);
    let want: Vec<usize> = truth["transcript"]["fillerWordIndices"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as usize)
        .collect();
    let got: Vec<usize> = t
        .words
        .iter()
        .filter(|w| crate::pack::is_filler(w))
        .map(|w| w.i)
        .collect();
    assert_eq!(got, want);
    assert_eq!(p.fillers, want.len());
    let r = &truth["transcript"]["retakes"][0]["abandoned"];
    let cuts = suggest_cuts(&t, 12.0);
    let retake = cuts.iter().find(|c| c.kind == CutKind::Retake).unwrap();
    let words = retake.words.unwrap();
    assert_eq!(
        [words[0] as u64, words[1] as u64],
        [r[0].as_u64().unwrap(), r[1].as_u64().unwrap()]
    );
}

#[test]
fn no_words_is_no_evidence_of_silence() {
    let t = Transcript::default();
    assert!(suggest_cuts(&t, 60.0).is_empty());
}
