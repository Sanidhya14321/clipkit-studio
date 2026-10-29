//! Sources, voice slots, comments, brief and locks (spec 3.2).

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::item::TimeRange;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum SourceKind {
    Video,
    Audio,
    Image,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    pub id: String,
    pub kind: SourceKind,
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proxy_path: Option<String>,
    /// Content hash of the original media.
    #[serde(default)]
    pub hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_sec: Option<f64>,
    /// Word-level transcript on disk (never inlined into the project).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub transcript_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum VoiceMode {
    Ai,
    Recorded,
    Cloned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum VoiceSlotStatus {
    #[default]
    Empty,
    Pending,
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct VoiceTake {
    pub id: String,
    pub path: String,
    pub duration_sec: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSlot {
    pub id: String,
    pub text: String,
    pub mode: VoiceMode,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub voice_ref: Option<String>,
    #[serde(default)]
    pub takes: Vec<VoiceTake>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_take: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_window: Option<TimeRange>,
    #[serde(default)]
    pub status: VoiceSlotStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum Author {
    Human,
    Agent,
    Voice,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Where a comment points: a time range, a layer, a property and/or a point on the frame.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Anchor {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_start: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_end: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prop_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub point: Option<Point>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum CommentStatus {
    #[default]
    Open,
    Working,
    Resolved,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Reply {
    pub author: Author,
    pub text: String,
    pub rev: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Comment {
    pub id: String,
    pub author: Author,
    #[serde(default)]
    pub anchor: Anchor,
    pub text: String,
    #[serde(default)]
    pub status: CommentStatus,
    pub created_rev: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_rev: Option<u64>,
    #[serde(default)]
    pub replies: Vec<Reply>,
    /// Layers the agent declared it will change besides the anchored one (scope gate).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scope: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum BriefSource {
    Inferred,
    Default,
    Profile,
    Reference,
    User,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct BriefField {
    pub value: serde_json::Value,
    pub source: BriefSource,
    /// 0.0 to 1.0.
    pub confidence: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum BriefStatus {
    #[default]
    Proposed,
    Confirmed,
    /// The user asked for no questions or autopilot is on; shown afterwards as an editable card.
    Auto,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Brief {
    /// Field name (audience, pacing, platform, look, voice, ...) to value.
    #[serde(default)]
    pub fields: BTreeMap<String, BriefField>,
    #[serde(default)]
    pub status: BriefStatus,
    #[serde(default)]
    pub checkpoints: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effort: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license_mode: Option<String>,
    /// Dynamic questions the agent adds to the Brief page (at most [`MAX_BRIEF_QUESTIONS`]).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub questions: Vec<BriefQuestion>,
}

/// Most dynamic questions one brief may carry (spec 8.1).
pub const MAX_BRIEF_QUESTIONS: usize = 3;

/// A question the agent adds to the brief, stating which fields its answer affects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct BriefQuestion {
    pub id: String,
    pub text: String,
    /// Brief fields or parts of the video the answer changes (never empty).
    pub affects: Vec<String>,
    /// The user's answer, once given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum Holder {
    Agent,
    Human,
}

/// A lock on one layer. Expires when no heartbeat arrives within `ttl_ms`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Lock {
    pub layer_id: String,
    pub holder: Holder,
    pub acquired_at_ms: u64,
    pub heartbeat_at_ms: u64,
    pub ttl_ms: u64,
}

impl Lock {
    pub fn is_live(&self, now_ms: u64) -> bool {
        now_ms < self.heartbeat_at_ms.saturating_add(self.ttl_ms)
    }
}
