use std::path::PathBuf;

use super::*;
use crate::{measure_loudness, probe};

const TARGET: LoudnessTarget = LoudnessTarget {
    integrated: -14.0,
    true_peak: -1.0,
};

fn clip(path: &str, src_in: f64, src_out: f64, at: f64) -> AudioClip {
    AudioClip {
        path: PathBuf::from(path),
        src_in,
        src_out,
        at,
    }
}

#[test]
fn the_graph_places_clips_and_pads_to_the_duration() {
    let g = mix_graph(
        &[clip("a.wav", 1.0, 2.0, 0.5), clip("b.wav", 0.0, 1.0, 3.0)],
        5.0,
        TARGET,
    );
    assert!(g.contains("[0:a:0]atrim=start=1:end=2,"));
    assert!(g.contains("adelay=500|500[a0]"));
    assert!(g.contains("adelay=3000|3000[a1]"));
    assert!(g.contains(
        "[a0][a1]amix=inputs=2:normalize=0:duration=longest,apad=whole_dur=5,atrim=duration=5"
    ));
    assert_eq!(
        mix_graph(&[], 2.0, TARGET),
        "anullsrc=r=48000:cl=stereo,atrim=duration=2[aout]"
    );
}

#[test]
fn mix_and_mux_fixture_media() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    if !dir.join("av.mp4").is_file() {
        eprintln!("skipped: run `just fixtures` first");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let wav = tmp.path().join("mix.wav");
    let tones = dir.join("tones.wav").to_string_lossy().into_owned();
    mix_audio(
        &[clip(&tones, 1.0, 2.0, 0.0), clip(&tones, 3.0, 4.5, 2.0)],
        4.0,
        TARGET,
        &wav,
    )
    .unwrap();
    assert!((probe(&wav).unwrap().duration - 4.0).abs() < 0.05);
    let out = tmp.path().join("out.mp4");
    mux(&dir.join("counter.mp4"), &wav, &out, Some((640, 360))).unwrap();
    let info = probe(&out).unwrap();
    let v = info.video.unwrap();
    assert_eq!((v.width, v.height), (640, 360));
    assert!((info.duration - 4.0).abs() < 0.15, "{}", info.duration);
    let l = measure_loudness(&out).unwrap();
    assert!((l.integrated - TARGET.integrated).abs() < 2.0, "{l:?}");
}
