use super::*;
use crate::probe;

const TARGET: LoudnessTarget = LoudnessTarget {
    integrated: -14.0,
    true_peak: -1.0,
};

fn spec(inputs: Vec<PathBuf>, pieces: Vec<Piece>) -> ExportSpec {
    ExportSpec {
        inputs,
        pieces,
        width: 1280,
        height: 720,
        fps: 30.0,
        loudness: TARGET,
    }
}

fn seg(input: usize, src_in: f64, src_out: f64, has_audio: bool) -> Piece {
    Piece::Segment {
        input,
        src_in,
        src_out,
        has_audio,
    }
}

#[test]
fn graph_trims_fades_concats_and_normalizes() {
    let s = spec(
        vec!["a.mp4".into()],
        vec![
            seg(0, 0.0, 1.5, true),
            Piece::Gap { dur: 0.5 },
            seg(0, 2.0, 3.0, false),
        ],
    );
    let g = filter_graph(&s).unwrap();
    assert!(g.contains("[0:v:0]trim=start=0:end=1.5,setpts=PTS-STARTPTS,scale=1280:720"));
    assert!(g.contains("afade=t=out:st=1.47:d=0.03[a0]"));
    assert!(g.contains("color=c=black:s=1280x720:r=30:d=0.5"));
    assert!(g.contains("anullsrc=r=48000:cl=stereo,atrim=duration=1[a2]"));
    assert!(g.contains("[v0][a0][v1][a1][v2][a2]concat=n=3:v=1:a=1[vout][acat]"));
    assert!(g.ends_with("loudnorm=I=-14:TP=-1:LRA=11,aresample=48000[aout]"));
}

#[test]
fn bad_specs_are_refused() {
    let empty = spec(vec![], vec![]);
    assert_eq!(filter_graph(&empty).unwrap_err().code, ErrorCode::Request);
    let missing = spec(vec![], vec![seg(0, 0.0, 1.0, true)]);
    assert_eq!(filter_graph(&missing).unwrap_err().code, ErrorCode::Request);
    let reversed = spec(vec!["a".into()], vec![seg(0, 2.0, 1.0, true)]);
    assert_eq!(
        filter_graph(&reversed).unwrap_err().code,
        ErrorCode::Request
    );
}

#[test]
fn loudness_takes_the_last_integrated_and_the_highest_peak() {
    let out = "lavfi.r128.I=-20.0\nlavfi.r128.true_peaks_ch0=0.5\nlavfi.r128.true_peaks_ch1=0.25\n\
               lavfi.r128.I=-14.2\n";
    let l = parse_loudness(out).unwrap();
    assert_eq!(l.integrated, -14.2);
    assert!((l.true_peak - (-6.0206)).abs() < 1e-3);
    assert_eq!(parse_loudness("").unwrap_err().code, ErrorCode::Media);
}

#[test]
fn exports_fixture_footage_to_the_target_loudness() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated");
    if !dir.join("av.mp4").is_file() {
        eprintln!("skipped: run `just fixtures` first");
        return;
    }
    let tmp = tempfile::tempdir().unwrap();
    let out = tmp.path().join("cut.mp4");
    // Keep 1 to 2 s and 3 to 4.5 s of the tone bursts, plus a 0.5 s gap.
    let s = spec(
        vec![dir.join("av.mp4")],
        vec![
            seg(0, 1.0, 2.0, true),
            Piece::Gap { dur: 0.5 },
            seg(0, 3.0, 4.5, true),
        ],
    );
    export(&s, &out).unwrap();
    let info = probe(&out).unwrap();
    assert!((info.duration - 3.0).abs() < 0.1, "{}", info.duration);
    let v = info.video.unwrap();
    assert_eq!((v.width, v.height), (1280, 720));
    assert_eq!(info.audio.len(), 1);
    let l = measure_loudness(&out).unwrap();
    assert!((l.integrated - TARGET.integrated).abs() <= 1.5, "{l:?}");
    assert!(l.true_peak <= TARGET.true_peak + 0.5, "{l:?}");
}
