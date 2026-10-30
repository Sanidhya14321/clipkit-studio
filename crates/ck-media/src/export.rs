//! Fast export path (spec 2, spec 8.6): footage-only timelines become one typed ffmpeg filter
//! graph over the original media, with short fades at every join, scale and pad to the preset
//! frame, and loudness normalization (`loudnorm`). The result is measured with `ebur128`.

use std::ffi::OsStr;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use ck_core::{CkError, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::run::{check_path, run};

const EXPORT_TIMEOUT: Duration = Duration::from_secs(60 * 60);
const MEASURE_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// True-peak floor reported for silence, dBTP.
pub const SILENCE_FLOOR_DB: f64 = -144.0;
/// Integrated loudness at or below this is silence (the ebur128 gate is -70 LUFS).
pub const SILENT_LUFS: f64 = -69.0;

/// Audio fade at each join, in seconds.
pub const JOIN_FADE: f64 = 0.03;
/// Loudness range passed to `loudnorm`.
const LOUDNORM_LRA: f64 = 11.0;

/// One piece of the output, in timeline order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum Piece {
    /// `src_in..src_out` of input `input` (an index into [`ExportSpec::inputs`]).
    #[serde(rename_all = "camelCase")]
    Segment {
        input: usize,
        src_in: f64,
        src_out: f64,
        /// False when the input has no audio stream (silence is generated).
        has_audio: bool,
    },
    /// Black frames and silence.
    Gap { dur: f64 },
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoudnessTarget {
    /// Integrated loudness, LUFS.
    pub integrated: f64,
    /// Maximum true peak, dBTP.
    pub true_peak: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExportSpec {
    pub inputs: Vec<PathBuf>,
    pub pieces: Vec<Piece>,
    pub width: u32,
    pub height: u32,
    pub fps: f64,
    pub loudness: LoudnessTarget,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Loudness {
    /// Integrated loudness, LUFS.
    pub integrated: f64,
    /// Highest true peak over all channels, dBTP.
    pub true_peak: f64,
}

fn num(x: f64) -> String {
    let s = format!("{x:.6}");
    s.trim_end_matches('0').trim_end_matches('.').to_owned()
}

/// The `-filter_complex` graph: per piece a video chain `[v<k>]` and an audio chain `[a<k>]`,
/// then `concat`, then loudness. Outputs `[vout]` and `[aout]`.
pub fn filter_graph(spec: &ExportSpec) -> Result<String, CkError> {
    if spec.pieces.is_empty() {
        return Err(CkError::new(ErrorCode::Request, "nothing to export"));
    }
    let (w, h, fps) = (spec.width, spec.height, num(spec.fps));
    let fit = format!(
        "scale={w}:{h}:force_original_aspect_ratio=decrease,pad={w}:{h}:(ow-iw)/2:(oh-ih)/2,\
         setsar=1,fps={fps},format=yuv420p"
    );
    let mut g = String::new();
    for (k, piece) in spec.pieces.iter().enumerate() {
        let dur = match piece {
            Piece::Segment {
                src_in, src_out, ..
            } => src_out - src_in,
            Piece::Gap { dur } => *dur,
        };
        if !(dur > 0.0 && dur.is_finite()) {
            return Err(CkError::new(
                ErrorCode::Request,
                format!("piece {k} has no duration"),
            ));
        }
        let fade = JOIN_FADE.min(dur / 2.0);
        let fades = format!(
            "afade=t=in:d={},afade=t=out:st={}:d={}",
            num(fade),
            num(dur - fade),
            num(fade)
        );
        let silence = format!("anullsrc=r=48000:cl=stereo,atrim=duration={}", num(dur));
        match piece {
            Piece::Segment {
                input,
                src_in,
                src_out,
                has_audio,
            } => {
                if *input >= spec.inputs.len() {
                    return Err(CkError::new(
                        ErrorCode::Request,
                        format!("piece {k} names input {input}"),
                    ));
                }
                let (a, b) = (num(*src_in), num(*src_out));
                let _ = write!(
                    g,
                    "[{input}:v:0]trim=start={a}:end={b},setpts=PTS-STARTPTS,{fit}[v{k}];"
                );
                if *has_audio {
                    let _ = write!(
                        g,
                        "[{input}:a:0]atrim=start={a}:end={b},asetpts=PTS-STARTPTS,\
                         aresample=48000,aformat=channel_layouts=stereo,{fades}[a{k}];"
                    );
                } else {
                    let _ = write!(g, "{silence}[a{k}];");
                }
            }
            Piece::Gap { .. } => {
                let _ = write!(
                    g,
                    "color=c=black:s={w}x{h}:r={fps}:d={},setsar=1,format=yuv420p[v{k}];\
                     {silence}[a{k}];",
                    num(dur)
                );
            }
        }
    }
    for k in 0..spec.pieces.len() {
        let _ = write!(g, "[v{k}][a{k}]");
    }
    let n = spec.pieces.len();
    let l = spec.loudness;
    let _ = write!(
        g,
        "concat=n={n}:v=1:a=1[vout][acat];[acat]loudnorm=I={}:TP={}:LRA={},aresample=48000[aout]",
        num(l.integrated),
        num(l.true_peak),
        num(LOUDNORM_LRA)
    );
    Ok(g)
}

/// Writes the MP4 (H.264 and AAC, faststart) to a temp file next to `out`, then renames it.
pub fn export(spec: &ExportSpec, out: &Path) -> Result<(), CkError> {
    for input in &spec.inputs {
        check_path(input)?;
    }
    check_path(out)?;
    let graph = filter_graph(spec)?;
    let io = |e: std::io::Error| CkError::new(ErrorCode::Media, format!("{}: {e}", out.display()));
    let dir = out.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).map_err(io)?;
    let mut tmp_name = OsStr::new(".tmp-").to_os_string();
    tmp_name.push(out.file_name().unwrap_or(OsStr::new("export.mp4")));
    let tmp = dir.join(tmp_name);
    let mut args: Vec<&OsStr> = ["-hide_banner", "-nostdin", "-v", "error", "-y"]
        .map(OsStr::new)
        .to_vec();
    for input in &spec.inputs {
        args.push(OsStr::new("-i"));
        args.push(input.as_os_str());
    }
    args.extend(
        [
            "-filter_complex",
            graph.as_str(),
            "-map",
            "[vout]",
            "-map",
            "[aout]",
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "20",
            "-c:a",
            "aac",
            "-b:a",
            "192k",
            "-movflags",
            "+faststart",
            "-f",
            "mp4",
        ]
        .map(OsStr::new),
    );
    args.push(tmp.as_os_str());
    let result =
        run("ffmpeg", &args, EXPORT_TIMEOUT).and_then(|_| std::fs::rename(&tmp, out).map_err(io));
    if result.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    result
}

/// Integrated loudness and true peak of the first audio stream, from `ebur128` metadata.
pub fn measure_loudness(path: &Path) -> Result<Loudness, CkError> {
    check_path(path)?;
    let args: Vec<&OsStr> = vec![
        "-hide_banner".as_ref(),
        "-nostdin".as_ref(),
        "-v".as_ref(),
        "error".as_ref(),
        "-i".as_ref(),
        path.as_os_str(),
        "-map".as_ref(),
        "0:a:0".as_ref(),
        "-af".as_ref(),
        "ebur128=metadata=1:peak=true,ametadata=mode=print:file=-".as_ref(),
        "-f".as_ref(),
        "null".as_ref(),
        "-".as_ref(),
    ];
    let out = run("ffmpeg", &args, MEASURE_TIMEOUT)?;
    parse_loudness(&String::from_utf8_lossy(&out.stdout))
}

/// The last running integrated value is the file's integrated loudness; true peaks arrive
/// linear per channel and are reported as the highest in dBTP.
pub fn parse_loudness(out: &str) -> Result<Loudness, CkError> {
    let mut integrated = None;
    let mut peak: f64 = 0.0;
    for line in out.lines() {
        if let Some(v) = line.strip_prefix("lavfi.r128.I=") {
            integrated = v.trim().parse::<f64>().ok().or(integrated);
        } else if let Some((key, v)) = line.split_once('=')
            && key.starts_with("lavfi.r128.true_peaks_ch")
            && let Ok(v) = v.trim().parse::<f64>()
        {
            peak = peak.max(v);
        }
    }
    let integrated = integrated
        .filter(|i: &f64| i.is_finite())
        .ok_or_else(|| CkError::new(ErrorCode::Media, "no loudness measured (silent audio?)"))?;
    // Digital silence has no peak; report the 24-bit floor instead of minus infinity.
    let true_peak = if peak > 0.0 {
        (20.0 * peak.log10()).max(SILENCE_FLOOR_DB)
    } else {
        SILENCE_FLOOR_DB
    };
    Ok(Loudness {
        integrated,
        true_peak,
    })
}

#[cfg(test)]
mod tests;
