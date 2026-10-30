//! Voice take processing (spec 8.3): trim leading and trailing silence, normalize loudness to
//! the voice target, then fit the take to its window: within the stretch tolerance it is
//! time-stretched to the window (`atempo`); otherwise it is kept and the fit is reported.

use std::ffi::OsStr;
use std::path::Path;
use std::time::Duration;

use ck_core::{CkError, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::run::{check_path, run};

const TAKE_TIMEOUT: Duration = Duration::from_secs(10 * 60);
/// Integrated loudness for voice takes, LUFS (DECISIONS.md, Phase 7 step 2).
pub const VOICE_LUFS: f64 = -16.0;
/// Takes within this share of the window are stretched to fit it.
pub const STRETCH_TOLERANCE: f64 = 0.08;
/// Level below which leading and trailing audio counts as silence.
const TRIM_DB: f64 = -45.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Fit {
    /// Within tolerance (stretched to the window when needed).
    Fits,
    /// Longer than the window: extend the visuals, shorten the line or re-record.
    TooLong,
    /// Shorter than the window: extend the line, shorten the window or re-record.
    TooShort,
    /// No window to fit.
    NoWindow,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TakeInfo {
    /// Final take length, seconds.
    pub duration_sec: f64,
    /// Length after trimming, before any stretch.
    pub raw_sec: f64,
    pub fit: Fit,
    /// Tempo applied (1.0 when not stretched).
    pub tempo: f64,
}

/// `raw` seconds against a `window` of seconds: the fit and the tempo to apply.
pub fn fit(raw: f64, window: Option<f64>) -> (Fit, f64) {
    let Some(w) = window.filter(|w| *w > 0.0) else {
        return (Fit::NoWindow, 1.0);
    };
    let ratio = raw / w;
    if (ratio - 1.0).abs() <= STRETCH_TOLERANCE {
        (Fit::Fits, ratio)
    } else if ratio > 1.0 {
        (Fit::TooLong, 1.0)
    } else {
        (Fit::TooShort, 1.0)
    }
}

fn ffmpeg(src: &Path, filter: &str, out: &Path) -> Result<(), CkError> {
    let args: Vec<&OsStr> = vec![
        "-hide_banner".as_ref(),
        "-nostdin".as_ref(),
        "-v".as_ref(),
        "error".as_ref(),
        "-y".as_ref(),
        "-i".as_ref(),
        src.as_os_str(),
        "-map".as_ref(),
        "0:a:0".as_ref(),
        "-vn".as_ref(),
        "-af".as_ref(),
        filter.as_ref(),
        "-c:a".as_ref(),
        "pcm_s16le".as_ref(),
        "-f".as_ref(),
        "wav".as_ref(),
        out.as_os_str(),
    ];
    run("ffmpeg", &args, TAKE_TIMEOUT).map(|_| ())
}

/// Writes the processed take to `out` (48 kHz WAV) and reports its fit to `window` seconds.
pub fn prepare_take(src: &Path, out: &Path, window: Option<f64>) -> Result<TakeInfo, CkError> {
    check_path(src)?;
    check_path(out)?;
    let io = |e: std::io::Error| CkError::new(ErrorCode::Media, format!("{}: {e}", out.display()));
    let dir = out.parent().unwrap_or(Path::new("."));
    std::fs::create_dir_all(dir).map_err(io)?;
    let trimmed = dir.join(format!(
        ".tmp-trim-{}",
        out.file_name()
            .and_then(OsStr::to_str)
            .unwrap_or("take.wav")
    ));
    let trim = format!(
        "silenceremove=start_periods=1:start_threshold={TRIM_DB}dB,areverse,\
         silenceremove=start_periods=1:start_threshold={TRIM_DB}dB,areverse,\
         aformat=channel_layouts=mono,loudnorm=I={VOICE_LUFS}:TP=-1.5:LRA=11,aresample=48000"
    );
    let result = (|| {
        ffmpeg(src, &trim, &trimmed)?;
        let raw = crate::probe(&trimmed)?.duration;
        let (fit, tempo) = fit(raw, window);
        if (tempo - 1.0).abs() > 1e-3 {
            ffmpeg(&trimmed, &format!("atempo={tempo:.5}"), out)?;
            let _ = std::fs::remove_file(&trimmed);
        } else {
            std::fs::rename(&trimmed, out).map_err(io)?;
        }
        let duration_sec = crate::probe(out)?.duration;
        Ok(TakeInfo {
            duration_sec,
            raw_sec: raw,
            fit,
            tempo,
        })
    })();
    if result.is_err() {
        let _ = std::fs::remove_file(&trimmed);
    }
    result
}

#[cfg(test)]
mod tests;
