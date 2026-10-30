use std::path::PathBuf;

use super::*;

#[test]
fn browser_mono_recording_normalizes_with_an_explicit_output_layout() {
    if std::process::Command::new("ffmpeg")
        .arg("-version")
        .output()
        .is_err()
    {
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let src = tmp.path().join("browser.webm");
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-hide_banner",
            "-nostdin",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=1",
            "-ac",
            "1",
            "-c:a",
            "libopus",
        ])
        .arg(&src)
        .status()
        .unwrap();
    assert!(status.success());
    let out = tmp.path().join("take.wav");
    let info = prepare_take(&src, &out, None).unwrap();
    assert!(
        info.duration_sec > 0.9 && info.duration_sec < 1.1,
        "{info:?}"
    );
    let probe = crate::probe(&out).unwrap();
    assert_eq!(probe.audio[0].sample_rate, 48000);
    assert_eq!(probe.audio[0].channels, 1);
}

#[test]
fn fit_stretches_within_tolerance_only() {
    assert_eq!(fit(2.1, Some(2.0)), (Fit::Fits, 1.05));
    assert_eq!(fit(2.0, Some(2.0)), (Fit::Fits, 1.0));
    assert_eq!(fit(3.0, Some(2.0)), (Fit::TooLong, 1.0));
    assert_eq!(fit(1.0, Some(2.0)), (Fit::TooShort, 1.0));
    assert_eq!(fit(1.0, None), (Fit::NoWindow, 1.0));
}

#[test]
fn a_take_is_trimmed_normalized_and_stretched_to_its_window() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    if !dir.join("tones.wav").is_file() {
        eprintln!("skipped: run `just fixtures` first");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("take.wav");
    // tones.wav: 1 s silence, then bursts until 11 s, then 1 s silence; trimmed it is 10 s.
    let info = prepare_take(&dir.join("tones.wav"), &out, Some(9.5)).unwrap();
    assert!((info.raw_sec - 10.0).abs() < 0.1, "{info:?}");
    assert_eq!(info.fit, Fit::Fits);
    assert!((info.duration_sec - 9.5).abs() < 0.1, "{info:?}");
    let l = crate::measure_loudness(&out).unwrap();
    assert!((l.integrated - VOICE_LUFS).abs() < 2.0, "{l:?}");

    let long = prepare_take(&dir.join("tones.wav"), &tmp.path().join("l.wav"), Some(5.0)).unwrap();
    assert_eq!(long.fit, Fit::TooLong);
    assert!((long.duration_sec - 10.0).abs() < 0.1);
}
