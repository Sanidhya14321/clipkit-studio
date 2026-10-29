//! Word-level transcripts (spec 3.2: on disk only, never inlined into the project) and the
//! transcriber interface. Fish Audio STT is the cloud provider; local whisper is a later
//! fallback (DECISIONS.md, Phase 5 step 4).

use std::path::Path;

use ck_core::storage::atomic_write;
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Environment variable holding the Fish Audio key. Never logged or printed.
pub const FISH_KEY_ENV: &str = "FISH_AUDIO_API_KEY";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Word {
    /// Index in the transcript, from 0.
    pub i: usize,
    pub text: String,
    /// Source time in seconds.
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Transcript {
    #[serde(default)]
    pub language: String,
    pub words: Vec<Word>,
}

impl Transcript {
    /// Refuses words out of order, with negative or reversed times, or misnumbered.
    pub fn check(&self) -> CkResult<()> {
        let mut last = 0.0;
        for (n, w) in self.words.iter().enumerate() {
            let ok = w.i == n && w.start >= last && w.end >= w.start && w.start.is_finite();
            if !ok || !w.end.is_finite() {
                return Err(CkError::new(
                    ErrorCode::Voice,
                    format!("transcript word {n} ({:?}) is out of order", w.text),
                ));
            }
            last = w.start;
        }
        Ok(())
    }
}

/// Turns a 48 kHz WAV into a word-level transcript.
pub trait Transcriber: Send + Sync {
    fn transcribe(&self, wav: &Path) -> CkResult<Transcript>;
}

/// Fish Audio speech to text.
///
/// PLACEHOLDER: the request and response shape are not verified yet (PROGRESS.md, Phase 5
/// step 4), so even with a key this refuses with `E_VOICE` instead of guessing the API.
#[derive(Debug, Default)]
pub struct FishStt {
    configured: bool,
}

impl FishStt {
    pub fn from_env() -> Self {
        Self {
            configured: std::env::var_os(FISH_KEY_ENV).is_some_and(|v| !v.is_empty()),
        }
    }
}

impl Transcriber for FishStt {
    fn transcribe(&self, _wav: &Path) -> CkResult<Transcript> {
        let message = if self.configured {
            "Fish Audio STT client is not built yet (API unverified)".to_owned()
        } else {
            format!("no transcriber: {FISH_KEY_ENV} is not set")
        };
        Err(CkError::new(ErrorCode::Voice, message))
    }
}

pub fn load(path: &Path) -> CkResult<Transcript> {
    let bytes = std::fs::read(path).map_err(|e| {
        CkError::new(
            ErrorCode::NotFound,
            format!("transcript {}: {e}", path.display()),
        )
    })?;
    let t: Transcript = serde_json::from_slice(&bytes).map_err(|e| {
        CkError::new(
            ErrorCode::Voice,
            format!("transcript {}: {e}", path.display()),
        )
    })?;
    t.check()?;
    Ok(t)
}

pub fn save(path: &Path, transcript: &Transcript) -> CkResult<()> {
    transcript.check()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| CkError::new(ErrorCode::Storage, format!("{}: {e}", dir.display())))?;
    }
    let bytes = serde_json::to_vec(transcript)
        .map_err(|e| CkError::new(ErrorCode::Voice, e.to_string()))?;
    atomic_write(path, &bytes)
}
