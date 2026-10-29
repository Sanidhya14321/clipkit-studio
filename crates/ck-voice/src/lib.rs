//! Fish Audio client, voice cache and loudness normalization.
//!
//! Phase 5: word-level transcripts and the transcriber interface (`transcript`), the packed
//! transcript the agent reads (`pack`) and transcript-based cut candidates (`cuts`).

pub mod cuts;
pub mod pack;
pub mod speech;
pub mod transcript;

pub use cuts::{CutCandidate, CutKind, kept_ranges, suggest_cuts};
pub use pack::{PackedTranscript, pack};
pub use speech::{FishTts, Speaker, cache_key};
pub use transcript::{FishStt, Transcriber, Transcript, Word, load, save};
