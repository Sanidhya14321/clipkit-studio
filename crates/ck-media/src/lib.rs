//! ffmpeg and ffprobe orchestration and the fast export path.
//!
//! Phase 5: safe tool invocation (`run`), probing (`probe`) and import media (`import`), cut-candidate signals (`detect`) and the fast export path (`export`).

pub mod beats;
pub mod detect;
pub mod engine;
pub mod export;
pub mod import;
pub mod mix;
pub mod music;
pub mod probe;
mod run;
pub mod take;

pub use beats::{Beats, measure_beats};
pub use detect::{SceneChange, Silence, Span, detect_black_freeze, detect_scenes, detect_silences};
pub use export::{
    ExportSpec, Loudness, LoudnessTarget, Piece, export, filter_graph, measure_loudness,
};
pub use import::{
    AUDIO_SAMPLE_RATE, CachePaths, PROXY_MAX_HEIGHT, extract_audio, hash_file, make_proxy,
};
pub use mix::{AudioClip, contact_sheet, mix_audio, mux};
pub use probe::{AudioStream, MediaInfo, VideoStream, parse_probe, probe};
pub use take::{Fit, TakeInfo, prepare_take};
