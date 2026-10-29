//! The project model (spec 3.2). Rust types are the source of truth; TS types are generated.

mod item;
mod meta;
mod pipeline;
mod validate;

use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

pub use item::{CaptionFrom, Item, ItemBody, TimeRange, Track, TrackKind};
pub use meta::{
    Anchor, Author, Brief, BriefField, BriefQuestion, BriefSource, BriefStatus, Comment,
    CommentStatus, Holder, Lock, MAX_BRIEF_QUESTIONS, Point, Reply, Source, SourceKind, VoiceMode,
    VoiceSlot, VoiceSlotStatus, VoiceTake,
};
pub use pipeline::{AgentState, AgentStatus, Pass, PipelineState, Stage};

/// Schema version written by this build. Older files are migrated on load (see `migrate`).
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum Mode {
    Edit,
    Hybrid,
    Generate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub fps: u32,
    pub width: u32,
    pub height: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            fps: 30,
            width: 1920,
            height: 1080,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    /// Project revision. Set by the engine, never by patches.
    pub rev: u64,
    pub settings: Settings,
    pub mode: Mode,
    #[serde(default)]
    pub brief: Brief,
    /// Plan written in the PLAN stage. Typed when Phase 4 defines it.
    #[serde(default)]
    pub plan: serde_json::Value,
    /// Design tokens by name.
    #[serde(default)]
    pub tokens: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub sources: Vec<Source>,
    #[serde(default)]
    pub tracks: Vec<Track>,
    #[serde(default)]
    pub voice_slots: Vec<VoiceSlot>,
    #[serde(default)]
    pub comments: Vec<Comment>,
    /// Asset ids referenced by layers.
    #[serde(default)]
    pub assets_used: Vec<String>,
    /// Derived components. Typed when Phase 8 defines them.
    #[serde(default)]
    pub derived: Vec<serde_json::Value>,
    /// Engine-managed; patches may not touch it.
    #[serde(default)]
    pub locks: Vec<Lock>,
    /// Branches. Typed when Phase 9 defines them.
    #[serde(default)]
    pub branches: Vec<serde_json::Value>,
    #[serde(default)]
    pub pipeline: PipelineState,
    #[serde(default)]
    pub agent: AgentState,
}

impl Project {
    /// An empty project at revision 0.
    pub fn new(id: impl Into<String>, name: impl Into<String>, mode: Mode) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            id: id.into(),
            name: name.into(),
            rev: 0,
            settings: Settings::default(),
            mode,
            brief: Brief::default(),
            plan: serde_json::Value::Null,
            tokens: BTreeMap::new(),
            sources: Vec::new(),
            tracks: Vec::new(),
            voice_slots: Vec::new(),
            comments: Vec::new(),
            assets_used: Vec::new(),
            derived: Vec::new(),
            locks: Vec::new(),
            branches: Vec::new(),
            pipeline: PipelineState::default(),
            agent: AgentState::default(),
        }
    }

    pub fn items(&self) -> impl Iterator<Item = &Item> {
        self.tracks.iter().flat_map(|t| t.items.iter())
    }

    pub fn item(&self, id: &str) -> Option<&Item> {
        self.items().find(|i| i.id == id)
    }
}

#[cfg(test)]
pub(crate) mod tests;
