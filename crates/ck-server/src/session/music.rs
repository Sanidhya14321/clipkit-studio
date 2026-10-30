//! Analysis never mutates timing or approves a music license.
use super::Session;
use ck_core::{CkError, CkResult, ErrorCode};
use ck_media::music::MusicAnalysis;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
static ANALYSIS_JOB: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MusicAnalyzeRequest {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct MusicAnalyzeResult {
    pub hash: String,
    pub analysis: MusicAnalysis,
    pub beat_count: usize,
    pub onset_count: usize,
    pub energy_count: usize,
    pub omitted: usize,
}

/// Trusted orchestration callback used by CLI and MCP ingestion.
pub fn asset_analysis(path: &Path) -> CkResult<serde_json::Value> {
    let analysis = ck_media::music::analyze(path)?;
    let mut value = serde_json::to_value(analysis)
        .map_err(|e| CkError::new(ErrorCode::Media, e.to_string()))?;
    let duration = value["durationSec"].as_f64().unwrap_or(0.0);
    let loudness = if duration >= 0.4 {
        Some(ck_media::measure_loudness(path)?)
    } else {
        None
    };
    value["loudnessLufs"] = if let Some(loudness) = loudness
        .as_ref()
        .filter(|l| l.integrated.is_finite() && l.integrated > ck_media::export::SILENT_LUFS)
    {
        serde_json::json!(loudness.integrated)
    } else {
        serde_json::Value::Null
    };
    value["truePeakDbtp"] = loudness
        .filter(|l| l.true_peak.is_finite())
        .map_or(serde_json::Value::Null, |l| serde_json::json!(l.true_peak));
    Ok(value)
}

impl Session {
    pub fn music_analyze(&self, req: &MusicAnalyzeRequest) -> CkResult<MusicAnalyzeResult> {
        let _job = ANALYSIS_JOB
            .lock()
            .map_err(|_| CkError::new(ErrorCode::Server, "audio analyzer lock poisoned"))?;
        let input = Path::new(&req.path);
        if req.path.is_empty() || req.path.starts_with('-') {
            return Err(CkError::new(ErrorCode::Request, "invalid audio path"));
        }
        let path = if input.is_absolute() {
            input.to_path_buf()
        } else {
            self.workdir().join(input)
        };
        let meta =
            std::fs::metadata(&path).map_err(|e| CkError::new(ErrorCode::Media, e.to_string()))?;
        if !meta.is_file() || meta.len() > 128 * 1024 * 1024 {
            return Err(CkError::new(
                ErrorCode::Media,
                "audio must be a regular file under 128 MiB",
            ));
        }
        let hash = ck_media::hash_file(&path)?;
        let cache = self.cache_dir().join("music");
        std::fs::create_dir_all(&cache)
            .map_err(|e| CkError::new(ErrorCode::Storage, e.to_string()))?;
        let cache_path = cache.join(format!("{hash}-energy-v1.json"));
        let cached = std::fs::metadata(&cache_path)
            .ok()
            .filter(|m| m.len() <= 512 * 1024)
            .and_then(|_| std::fs::read(&cache_path).ok())
            .and_then(|b| serde_json::from_slice::<MusicAnalysis>(&b).ok())
            .filter(|a| {
                !a.confirmed
                    && a.duration_sec > 0.0
                    && a.duration_sec <= 600.0
                    && a.confidence.is_finite()
                    && (0.0..=1.0).contains(&a.confidence)
                    && a.bpm
                        .is_none_or(|b| b.is_finite() && (60.0..=240.0).contains(&b))
                    && [&a.beats, &a.onsets].iter().all(|v| {
                        v.windows(2).all(|p| p[1] > p[0])
                            && v.iter()
                                .all(|t| t.is_finite() && *t >= 0.0 && *t < a.duration_sec)
                    })
                    && a.energy.len() <= 600
                    && a.energy.iter().all(|e| e.is_finite() && *e >= 0.0)
            });
        let mut analysis = match cached {
            Some(a) => a,
            None => ck_media::music::analyze(&path)?,
        };
        let bytes = serde_json::to_vec(&analysis)
            .map_err(|e| CkError::new(ErrorCode::Media, e.to_string()))?;
        // Full data stays in the content-addressed cache; replies are compact.
        ck_core::storage::atomic_write(&cache_path, &bytes)?;
        let (beat_count, onset_count, energy_count) = (
            analysis.beats.len(),
            analysis.onsets.len(),
            analysis.energy.len(),
        );
        analysis.beats.truncate(16);
        analysis.onsets.truncate(16);
        analysis.energy.truncate(16);
        let omitted = beat_count + onset_count + energy_count
            - analysis.beats.len()
            - analysis.onsets.len()
            - analysis.energy.len();
        Ok(MusicAnalyzeResult {
            hash,
            analysis,
            beat_count,
            onset_count,
            energy_count,
            omitted,
        })
    }
}
