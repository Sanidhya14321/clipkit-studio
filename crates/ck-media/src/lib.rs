//! ffmpeg and ffprobe orchestration and the fast export path.
//!
//! Phase 5: safe tool invocation (`run`) and probing (`probe`).

pub mod probe;
mod run;

pub use probe::{AudioStream, MediaInfo, VideoStream, parse_probe, probe};
