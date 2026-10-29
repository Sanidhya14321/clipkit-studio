//! ffmpeg and ffprobe orchestration and the fast export path.
//!
//! Phase 5: safe tool invocation (`run`), probing (`probe`) and import media (`import`), cut-candidate signals (`detect`) and the fast export path (`export`).

pub mod detect;
pub mod export;
pub mod import;
pub mod probe;
mod run;

pub use detect::{SceneChange, Silence, detect_scenes, detect_silences};
pub use export::{
    ExportSpec, Loudness, LoudnessTarget, Piece, export, filter_graph, measure_loudness,
};
pub use import::{
    AUDIO_SAMPLE_RATE, CachePaths, PROXY_MAX_HEIGHT, extract_audio, hash_file, make_proxy,
};
pub use probe::{AudioStream, MediaInfo, VideoStream, parse_probe, probe};
