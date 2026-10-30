//! `reference analyze` (spec 8.1 "reference video (statistics only)"): read a reference video's
//! numbers (duration, aspect, shot lengths, cuts per 15 s, loudness) and suggest brief fields
//! from them with source `reference`. Nothing of the reference is copied; with `apply`, the
//! suggestions fill brief fields the user has not set themselves.

use std::path::Path;

use ck_core::model::{BriefField, BriefSource};
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::Session;

/// Average shot length per pacing level (assets/taste/pacing.md), calm to rapid.
pub const PACING_AVG_SHOT: &[(&str, f64)] = &[
    ("calm", 4.5),
    ("steady", 3.2),
    ("lively", 2.2),
    ("punchy", 1.5),
    ("rapid", 1.0),
];

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReferenceRequest {
    /// Path of the reference video (absolute, or relative to the project folder).
    pub path: String,
    /// Write the suggestions into the brief (fields the user set are kept).
    #[serde(default)]
    pub apply: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceStats {
    pub duration_sec: f64,
    pub width: u32,
    pub height: u32,
    pub shots: usize,
    pub avg_shot_sec: f64,
    pub min_shot_sec: f64,
    pub max_shot_sec: f64,
    pub cuts_per_15s: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loudness_lufs: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReferenceResult {
    pub stats: ReferenceStats,
    /// Brief field to suggested value.
    pub suggested: serde_json::Map<String, Value>,
    /// Fields written to the brief (with `apply`).
    #[serde(default)]
    pub applied: Vec<String>,
    pub rev: u64,
}

/// The pacing level whose average shot length is nearest (in log scale) to `avg`.
pub fn pacing_for(avg: f64) -> &'static str {
    PACING_AVG_SHOT
        .iter()
        .min_by(|a, b| {
            (a.1.ln() - avg.ln())
                .abs()
                .total_cmp(&(b.1.ln() - avg.ln()).abs())
        })
        .map_or("lively", |p| p.0)
}

/// `16:9`, `9:16`, `1:1` or `4:5` when within 2%, else `w:h` reduced.
pub fn aspect_of(w: u32, h: u32) -> String {
    let r = f64::from(w) / f64::from(h.max(1));
    for (name, v) in [
        ("16:9", 16.0 / 9.0),
        ("9:16", 9.0 / 16.0),
        ("1:1", 1.0),
        ("4:5", 0.8),
    ] {
        if (r - v).abs() / v < 0.02 {
            return name.to_owned();
        }
    }
    fn gcd(a: u32, b: u32) -> u32 {
        if b == 0 { a } else { gcd(b, a % b) }
    }
    let g = gcd(w, h).max(1);
    format!("{}:{}", w / g, h / g)
}

pub fn stats(path: &Path) -> CkResult<ReferenceStats> {
    let info = ck_media::probe(path)?;
    let v = info
        .video
        .as_ref()
        .ok_or_else(|| CkError::new(ErrorCode::Media, "the reference has no video"))?;
    let cuts: Vec<f64> = ck_media::detect_scenes(path)?
        .iter()
        .map(|s| s.at)
        .collect();
    let mut bounds = vec![0.0];
    bounds.extend(
        cuts.iter()
            .copied()
            .filter(|c| *c > 0.0 && *c < info.duration),
    );
    bounds.push(info.duration);
    let shots: Vec<f64> = bounds
        .windows(2)
        .map(|w| w[1] - w[0])
        .filter(|s| *s > 0.0)
        .collect();
    let n = shots.len().max(1);
    let loudness = if info.audio.is_empty() {
        None
    } else {
        ck_media::measure_loudness(path)
            .ok()
            .map(|l| l.integrated)
            .filter(|i| *i > ck_media::export::SILENT_LUFS)
    };
    let r2 = |x: f64| (x * 100.0).round() / 100.0;
    Ok(ReferenceStats {
        duration_sec: r2(info.duration),
        width: v.display_width,
        height: v.display_height,
        shots: shots.len(),
        avg_shot_sec: r2(shots.iter().sum::<f64>() / n as f64),
        min_shot_sec: r2(shots.iter().copied().fold(f64::INFINITY, f64::min)),
        max_shot_sec: r2(shots.iter().copied().fold(0.0, f64::max)),
        cuts_per_15s: r2(cuts.len() as f64 * 15.0 / info.duration.max(0.001)),
        loudness_lufs: loudness.map(r2),
    })
}

impl Session {
    pub fn reference_analyze(&self, req: &ReferenceRequest) -> CkResult<ReferenceResult> {
        let path = {
            let p = Path::new(&req.path);
            if p.is_absolute() {
                p.to_path_buf()
            } else {
                self.workdir().join(p)
            }
        };
        let st = stats(&path)?;
        let mut suggested = serde_json::Map::new();
        suggested.insert("length".into(), json!(st.duration_sec.round()));
        suggested.insert("aspect".into(), json!(aspect_of(st.width, st.height)));
        suggested.insert("pacing".into(), json!(pacing_for(st.avg_shot_sec)));
        let mut doc = self.doc()?;
        let p = doc.project();
        let mut applied = Vec::new();
        let mut rev = p.rev;
        if req.apply {
            let ops: Vec<Value> = suggested
                .iter()
                .filter(|(k, _)| {
                    p.brief
                        .fields
                        .get(*k)
                        .is_none_or(|f| f.source != BriefSource::User)
                })
                .map(|(k, v)| {
                    applied.push(k.clone());
                    let field = BriefField {
                        value: v.clone(),
                        source: BriefSource::Reference,
                        confidence: 0.6,
                    };
                    json!({ "op": "add", "path": format!("/brief/fields/{k}"), "value": field })
                })
                .collect();
            if !ops.is_empty() {
                rev = self
                    .commit_agent(
                        &mut doc,
                        Value::Array(ops),
                        "reference: brief fields".into(),
                    )?
                    .rev;
            }
        }
        Ok(ReferenceResult {
            stats: st,
            suggested,
            applied,
            rev,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shot_lengths_map_to_pacing_and_sizes_to_aspects() {
        assert_eq!(pacing_for(4.0), "calm");
        assert_eq!(pacing_for(2.0), "lively");
        assert_eq!(pacing_for(0.9), "rapid");
        assert_eq!(aspect_of(1920, 1080), "16:9");
        assert_eq!(aspect_of(1080, 1920), "9:16");
        assert_eq!(aspect_of(1000, 600), "5:3");
    }

    #[test]
    fn the_scene_fixture_reads_as_five_two_second_shots() {
        let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../fixtures/generated/scenes.mp4");
        if !path.is_file() {
            eprintln!("skipped: run `just fixtures` first");
            return;
        }
        let s = stats(&path).unwrap();
        assert_eq!(s.shots, 5, "{s:?}");
        assert!((s.avg_shot_sec - 2.0).abs() < 0.05);
        assert_eq!(pacing_for(s.avg_shot_sec), "lively");
    }
}
