//! Audio mix and mux for the full export path (spec 2.3): the render worker writes muted video;
//! ck mixes the audio clips (footage sound, voice takes) on the timeline with short fades,
//! normalizes the mix and muxes it with the video, scaling to the preset frame when needed.

use std::ffi::OsStr;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ck_core::{CkError, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::export::{JOIN_FADE, LoudnessTarget};
use crate::run::{check_path, run};

const MIX_TIMEOUT: Duration = Duration::from_secs(60 * 60);

/// One audio clip placed on the timeline.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AudioClip {
    pub path: PathBuf,
    pub src_in: f64,
    pub src_out: f64,
    /// Timeline start, seconds.
    pub at: f64,
    /// Linear gain (1.0 unchanged).
    #[serde(default = "unity")]
    pub gain: f64,
    /// Timeline ranges to duck under voice.
    #[serde(default)]
    pub duck: Vec<(f64, f64)>,
}

fn unity() -> f64 {
    1.0
}

fn num(x: f64) -> String {
    let s = format!("{x:.6}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// Gain over music while voice plays (about -12 dB).
const DUCK_GAIN: f64 = 0.25;

/// One timed volume filter per range; runs after `adelay`, so `t` is timeline time.
fn duck_filters(ranges: &[(f64, f64)]) -> String {
    ranges
        .iter()
        .map(|(s, e)| {
            format!(
                ",volume={}:enable='between(t,{},{})'",
                num(DUCK_GAIN),
                num(*s),
                num(*e)
            )
        })
        .collect()
}

/// The `-filter_complex` graph mixing `clips` (input `i` is `clips[i]`) into `[aout]` of
/// exactly `duration` seconds.
pub fn mix_graph(clips: &[AudioClip], duration: f64, loudness: LoudnessTarget) -> String {
    let mut g = String::new();
    let mut labels = String::new();
    for (i, c) in clips.iter().enumerate() {
        let len = (c.src_out - c.src_in).max(0.0);
        let fade = JOIN_FADE.min(len / 2.0);
        let delay = (c.at * 1000.0).round().max(0.0) as u64;
        let _ = write!(
            g,
            "[{i}:a:0]atrim=start={}:end={},asetpts=PTS-STARTPTS,aresample=48000,\
             aformat=channel_layouts=stereo,afade=t=in:d={},afade=t=out:st={}:d={},\
             volume={},adelay={delay}|{delay}{}[a{i}];",
            num(c.src_in),
            num(c.src_out),
            num(fade),
            num(len - fade),
            num(fade),
            num(c.gain.max(0.0)),
            duck_filters(&c.duck)
        );
        let _ = write!(labels, "[a{i}]");
    }
    let d = num(duration);
    if clips.is_empty() {
        let _ = write!(g, "anullsrc=r=48000:cl=stereo,atrim=duration={d}[aout]");
        return g;
    }
    let _ = write!(
        g,
        "{labels}amix=inputs={}:normalize=0:duration=longest,apad=whole_dur={d},\
         atrim=duration={d},loudnorm=I={}:TP={}:LRA=11,aresample=48000:ochl=stereo[aout]",
        clips.len(),
        num(loudness.integrated),
        num(loudness.true_peak)
    );
    g
}

fn ffmpeg(args: &[&OsStr]) -> Result<(), CkError> {
    let mut all: Vec<&OsStr> = ["-hide_banner", "-nostdin", "-v", "error", "-y"]
        .map(OsStr::new)
        .to_vec();
    all.extend_from_slice(args);
    run("ffmpeg", &all, MIX_TIMEOUT).map(|_| ())
}

/// Writes the mixed timeline audio (48 kHz WAV) to `out`.
pub fn mix_audio(
    clips: &[AudioClip],
    duration: f64,
    loudness: LoudnessTarget,
    out: &Path,
) -> Result<(), CkError> {
    check_path(out)?;
    if !(duration > 0.0 && duration.is_finite()) {
        return Err(CkError::new(
            ErrorCode::Request,
            "nothing to mix: zero duration",
        ));
    }
    let mut args: Vec<&OsStr> = Vec::new();
    for c in clips {
        check_path(&c.path)?;
        args.push(OsStr::new("-i"));
        args.push(c.path.as_os_str());
    }
    let graph = mix_graph(clips, duration, loudness);
    args.extend(
        [
            "-filter_complex",
            graph.as_str(),
            "-map",
            "[aout]",
            "-c:a",
            "pcm_s16le",
        ]
        .map(OsStr::new),
    );
    args.push(out.as_os_str());
    ffmpeg(&args)
}

/// Muxes `video` and `audio` into `out` (H.264 and AAC, faststart). With `size`, the frame is
/// scaled and padded to it (re-encoded); otherwise the video stream is copied.
pub fn mux(
    video: &Path,
    audio: &Path,
    out: &Path,
    size: Option<(u32, u32)>,
) -> Result<(), CkError> {
    for p in [video, audio, out] {
        check_path(p)?;
    }
    let fit = size.map(|(w, h)| {
        format!(
            "scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,\
             setsar=1,format=yuv420p"
        )
    });
    let mut args: Vec<&OsStr> = vec![
        "-i".as_ref(),
        video.as_os_str(),
        "-i".as_ref(),
        audio.as_os_str(),
        "-map".as_ref(),
        "0:v:0".as_ref(),
        "-map".as_ref(),
        "1:a:0".as_ref(),
    ];
    match &fit {
        Some(vf) => args.extend(
            [
                "-vf",
                vf.as_str(),
                "-c:v",
                "libx264",
                "-preset",
                "veryfast",
                "-crf",
                "20",
            ]
            .map(OsStr::new),
        ),
        None => args.extend(["-c:v", "copy"].map(OsStr::new)),
    }
    args.extend(
        [
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-shortest",
            "-movflags",
            "+faststart",
            "-f",
            "mp4",
        ]
        .map(OsStr::new),
    );
    args.push(out.as_os_str());
    ffmpeg(&args)
}

/// Tiles `stills` (scaled to `width` px wide) into one image, `cols` per row. A short last row
/// repeats the last frame so the grid is full.
pub fn contact_sheet(
    stills: &[PathBuf],
    width: u32,
    cols: usize,
    out: &Path,
) -> Result<(), CkError> {
    check_path(out)?;
    let Some(last) = stills.last() else {
        return Err(CkError::new(
            ErrorCode::Request,
            "no frames for the contact sheet",
        ));
    };
    let cols = cols.max(1);
    let cells = stills.len().div_ceil(cols) * cols;
    let inputs: Vec<&PathBuf> = (0..cells).map(|i| stills.get(i).unwrap_or(last)).collect();
    let mut args: Vec<&OsStr> = Vec::new();
    for p in &inputs {
        check_path(p)?;
        args.push(OsStr::new("-i"));
        args.push(p.as_os_str());
    }
    let mut g = String::new();
    for i in 0..cells {
        let _ = write!(g, "[{i}:v]scale={width}:-2[s{i}];");
    }
    for i in 0..cells {
        let _ = write!(g, "[s{i}]");
    }
    let layout: Vec<String> = (0..cells)
        .map(|i| {
            let offset = |n: usize, unit: &str| {
                if n == 0 {
                    "0".to_owned()
                } else {
                    vec![unit; n].join("+")
                }
            };
            format!("{}_{}", offset(i % cols, "w0"), offset(i / cols, "h0"))
        })
        .collect();
    let _ = write!(g, "xstack=inputs={cells}:layout={}[out]", layout.join("|"));
    args.extend(
        [
            "-filter_complex",
            g.as_str(),
            "-map",
            "[out]",
            "-frames:v",
            "1",
        ]
        .map(OsStr::new),
    );
    args.push(out.as_os_str());
    ffmpeg(&args)
}

#[cfg(test)]
mod tests;
