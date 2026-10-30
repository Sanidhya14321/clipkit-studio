//! Output-only evaluator (spec 10.3 `ck eval`): scores any exported video, from any arm of the
//! benchmark, using only the file and stated expectations. It never looks inside a project, so
//! a run with or without the plugin is judged the same way. Checks that need OCR (safe areas,
//! text size) are listed as not measured rather than guessed.

use std::path::{Path, PathBuf};

use ck_core::{CkError, CkResult, ErrorCode};
use ck_media::{
    LoudnessTarget, detect_black_freeze, detect_scenes, hash_file, measure_loudness, probe,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod caps {
    pub const EVAL: usize = 4 * 1024;
}

/// A black stretch at least this long fails the check, seconds.
pub const BLACK_MAX: f64 = 1.0;
/// A frozen stretch at least this long fails the check, seconds (title holds are shorter).
pub const FROZEN_MAX: f64 = 4.0;
/// Loudness tolerance around the target, LU.
pub const LOUDNESS_TOLERANCE: f64 = 1.0;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Expectations {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<f64>,
    /// Allowed difference, seconds (default 0.5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_tolerance: Option<f64>,
    /// For example `16:9` or `9:16`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub aspect: Option<String>,
    /// Default -14 LUFS and -1 dBTP.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loudness: Option<LoudnessTarget>,
    /// Expected hard cuts in the output, seconds (ground truth).
    #[serde(default)]
    pub cuts_at_sec: Vec<f64>,
    /// Default 0.1 s.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cut_tolerance: Option<f64>,
    /// Source files that must be unchanged: path and SHA-256.
    #[serde(default)]
    pub sources: Vec<SourceHash>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct SourceHash {
    pub path: String,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvalRequest {
    /// The exported video.
    pub video: String,
    #[serde(default)]
    pub expect: Expectations,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Check {
    pub name: String,
    pub pass: bool,
    /// What was measured, in short.
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct EvalReport {
    /// Passed checks over measured checks, 0 to 100.
    pub score: u32,
    pub checks: Vec<Check>,
    pub not_measured: Vec<String>,
}

fn check(name: &str, pass: bool, value: impl Into<String>) -> Check {
    Check {
        name: name.into(),
        pass,
        value: value.into(),
    }
}

/// `w:h` to a ratio; `None` when it does not parse.
fn ratio(aspect: &str) -> Option<f64> {
    let (w, h) = aspect.split_once(':')?;
    let (w, h): (f64, f64) = (w.trim().parse().ok()?, h.trim().parse().ok()?);
    (w > 0.0 && h > 0.0).then_some(w / h)
}

pub fn evaluate(base: &Path, req: &EvalRequest) -> CkResult<EvalReport> {
    let video: PathBuf = {
        let p = Path::new(&req.video);
        if p.is_absolute() {
            p.to_path_buf()
        } else {
            base.join(p)
        }
    };
    if !video.is_file() {
        return Err(CkError::new(
            ErrorCode::NotFound,
            format!("no video {}", video.display()),
        ));
    }
    let e = &req.expect;
    let info = probe(&video)?;
    let mut checks = Vec::new();
    let mut not_measured = vec![
        "safe areas (needs OCR)".to_owned(),
        "text size (needs OCR)".to_owned(),
    ];

    if let Some(want) = e.duration_sec {
        let tol = e.duration_tolerance.unwrap_or(0.5);
        let ok = (info.duration - want).abs() <= tol;
        checks.push(check(
            "duration",
            ok,
            format!("{:.2}s (want {want}±{tol})", info.duration),
        ));
    } else {
        not_measured.push("duration (no expectation)".into());
    }

    match (&info.video, &e.aspect) {
        (Some(v), Some(a)) => {
            let have = f64::from(v.display_width) / f64::from(v.display_height.max(1));
            let ok = ratio(a).is_some_and(|r| (have - r).abs() / r < 0.01);
            checks.push(check(
                "aspect",
                ok,
                format!("{}x{} (want {a})", v.display_width, v.display_height),
            ));
        }
        (None, _) => checks.push(check("video", false, "no video stream")),
        (Some(_), None) => not_measured.push("aspect (no expectation)".into()),
    }

    if info.audio.is_empty() {
        not_measured.push("loudness (no audio)".into());
    } else {
        let l = measure_loudness(&video)?;
        let target = e.loudness.unwrap_or(LoudnessTarget {
            integrated: -14.0,
            true_peak: -1.0,
        });
        if l.integrated <= ck_media::export::SILENT_LUFS {
            not_measured.push("loudness (silent)".into());
        } else {
            let ok = (l.integrated - target.integrated).abs() <= LOUDNESS_TOLERANCE
                && l.true_peak <= target.true_peak + 0.5;
            checks.push(check(
                "loudness",
                ok,
                format!("{:.1} LUFS, {:.1} dBTP", l.integrated, l.true_peak),
            ));
        }
        checks.push(check(
            "clipping",
            l.true_peak < 0.0,
            format!("peak {:.1} dBTP", l.true_peak),
        ));
    }

    if info.video.is_some() {
        let (black, frozen) = detect_black_freeze(&video, 0.5, info.duration)?;
        let longest = |s: &[ck_media::Span]| s.iter().map(|x| x.end - x.start).fold(0.0, f64::max);
        let (b, f) = (longest(&black), longest(&frozen));
        checks.push(check(
            "black frames",
            b < BLACK_MAX,
            format!("longest {b:.2}s"),
        ));
        checks.push(check(
            "frozen frames",
            f < FROZEN_MAX,
            format!("longest {f:.2}s"),
        ));
    }

    if !e.cuts_at_sec.is_empty() {
        let tol = e.cut_tolerance.unwrap_or(0.1);
        let found: Vec<f64> = detect_scenes(&video)?.iter().map(|s| s.at).collect();
        let matched = e
            .cuts_at_sec
            .iter()
            .filter(|want| found.iter().any(|f| (f - *want).abs() <= tol))
            .count();
        checks.push(check(
            "cut accuracy",
            matched == e.cuts_at_sec.len(),
            format!("{matched}/{} cuts within {tol}s", e.cuts_at_sec.len()),
        ));
    }

    for s in &e.sources {
        let path = {
            let p = Path::new(&s.path);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                base.join(p)
            }
        };
        let same = hash_file(&path).is_ok_and(|h| h == s.hash);
        checks.push(check("source unchanged", same, s.path.clone()));
    }

    let passed = checks.iter().filter(|c| c.pass).count();
    let score = if checks.is_empty() {
        0
    } else {
        ((passed * 100) as f64 / checks.len() as f64).round() as u32
    };
    Ok(EvalReport {
        score,
        checks,
        not_measured,
    })
}

#[cfg(test)]
mod tests;
