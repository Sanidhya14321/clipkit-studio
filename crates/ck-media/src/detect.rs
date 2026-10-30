//! Cut-candidate signals (spec 8, spec 14 wave 1): silences from `silencedetect` and hard scene
//! changes from `scdet`. Both filters print their metadata to stdout through `ametadata` and
//! `metadata` (`file=-`), so nothing is scraped from the ffmpeg log.

use std::ffi::OsStr;
use std::path::Path;
use std::time::Duration;

use ck_core::CkError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::run::{check_path, run};

const DETECT_TIMEOUT: Duration = Duration::from_secs(30 * 60);

/// Level below which audio counts as silence.
pub const SILENCE_NOISE_DB: f64 = -40.0;
/// Shortest silence reported, in seconds.
pub const SILENCE_MIN_SEC: f64 = 0.3;
/// `scdet` score (0 to 100) at or above which a frame starts a new scene.
pub const SCENE_THRESHOLD: f64 = 10.0;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Silence {
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SceneChange {
    /// Time of the first frame of the new scene.
    pub at: f64,
    /// `scdet` score, 0 to 100.
    pub score: f64,
}

/// Silences in the first audio stream. A silence still open at the end of the file closes at
/// `duration`.
pub fn detect_silences(src: &Path, duration: f64) -> Result<Vec<Silence>, CkError> {
    let filter = format!(
        "silencedetect=n={SILENCE_NOISE_DB}dB:d={SILENCE_MIN_SEC},ametadata=mode=print:file=-"
    );
    let out = ffmpeg_null(src, &["-map", "0:a:0", "-vn", "-af", &filter])?;
    Ok(parse_silences(&String::from_utf8_lossy(&out), duration))
}

/// Hard cuts in the first video stream. Scored in RGB, since two colours of equal luma would
/// otherwise look the same.
pub fn detect_scenes(src: &Path) -> Result<Vec<SceneChange>, CkError> {
    let filter =
        format!("format=gbrp,scdet=threshold={SCENE_THRESHOLD},metadata=mode=print:file=-");
    let out = ffmpeg_null(src, &["-map", "0:v:0", "-an", "-vf", &filter])?;
    Ok(parse_scenes(&String::from_utf8_lossy(&out)))
}

fn ffmpeg_null(src: &Path, middle: &[&str]) -> Result<Vec<u8>, CkError> {
    check_path(src)?;
    let mut args: Vec<&OsStr> = vec![
        "-hide_banner".as_ref(),
        "-nostdin".as_ref(),
        "-v".as_ref(),
        "error".as_ref(),
        "-i".as_ref(),
        src.as_os_str(),
    ];
    args.extend(middle.iter().map(OsStr::new));
    args.extend(["-f", "null", "-"].map(OsStr::new));
    Ok(run("ffmpeg", &args, DETECT_TIMEOUT)?.stdout)
}

fn value(line: &str, key: &str) -> Option<f64> {
    line.strip_prefix(key)?
        .strip_prefix('=')?
        .trim()
        .parse()
        .ok()
}

pub fn parse_silences(out: &str, duration: f64) -> Vec<Silence> {
    let mut silences = Vec::new();
    let mut open = None;
    for line in out.lines() {
        if let Some(start) = value(line, "lavfi.silence_start") {
            open = Some(start.max(0.0));
        } else if let Some(end) = value(line, "lavfi.silence_end")
            && let Some(start) = open.take()
        {
            silences.push(Silence { start, end });
        }
    }
    if let Some(start) = open.filter(|s| *s < duration) {
        silences.push(Silence {
            start,
            end: duration,
        });
    }
    silences
}

pub fn parse_scenes(out: &str) -> Vec<SceneChange> {
    let mut scenes = Vec::new();
    let mut score = 0.0;
    for line in out.lines() {
        if line.starts_with("frame:") {
            score = 0.0;
        } else if let Some(s) = value(line, "lavfi.scd.score") {
            score = s;
        } else if let Some(at) = value(line, "lavfi.scd.time") {
            scenes.push(SceneChange { at, score });
        }
    }
    scenes
}

/// A time range in seconds (black or frozen stretch).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub start: f64,
    pub end: f64,
}

/// Black stretches (at least `min` seconds) and frozen stretches (no change for `min` seconds)
/// in the first video stream; open stretches close at `duration`.
pub fn detect_black_freeze(
    src: &Path,
    min: f64,
    duration: f64,
) -> Result<(Vec<Span>, Vec<Span>), CkError> {
    let filter = format!(
        "blackdetect=d={min}:pix_th=0.1,freezedetect=n=0.001:d={min},metadata=mode=print:file=-"
    );
    let out = ffmpeg_null(src, &["-map", "0:v:0", "-an", "-vf", &filter])?;
    Ok(parse_black_freeze(&String::from_utf8_lossy(&out), duration))
}

pub fn parse_black_freeze(out: &str, duration: f64) -> (Vec<Span>, Vec<Span>) {
    let pair = |start_key: &str, end_key: &str| {
        let mut spans = Vec::new();
        let mut open = None;
        for line in out.lines() {
            if let Some(s) = value(line, start_key) {
                open = Some(s);
            } else if let Some(e) = value(line, end_key)
                && let Some(s) = open.take()
            {
                spans.push(Span { start: s, end: e });
            }
        }
        if let Some(s) = open.filter(|s| *s < duration) {
            spans.push(Span {
                start: s,
                end: duration,
            });
        }
        spans
    };
    (
        pair("lavfi.black_start", "lavfi.black_end"),
        pair(
            "lavfi.freezedetect.freeze_start",
            "lavfi.freezedetect.freeze_end",
        ),
    )
}

#[cfg(test)]
mod tests;
