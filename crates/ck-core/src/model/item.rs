//! Tracks and timeline items (spec 3.2). Times are seconds; frames only at render time.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::pipeline::Pass;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum TrackKind {
    Video,
    Overlay,
    Audio,
    Caption,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Track {
    pub id: String,
    pub kind: TrackKind,
    #[serde(default)]
    pub items: Vec<Item>,
}

/// A layer on a track. Common fields plus one kind-specific body, tagged by `type`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    #[serde(default)]
    pub name: String,
    /// Why this layer exists, in a few words (shown to the user and the agent).
    #[serde(default)]
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass: Option<Pass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub asset_ref: Option<String>,
    /// Brief field names this layer depends on (drives re-planning when the brief changes).
    #[serde(default)]
    pub brief_deps: Vec<String>,
    /// Where the layer came from, for recipe-expanded and asset-derived layers (spec 7.4).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provenance: Option<Provenance>,
    /// Project revision that last touched this layer. Set by the engine, never by patches.
    #[serde(default)]
    pub rev: u64,
    #[serde(flatten)]
    pub body: ItemBody,
}

/// Where a layer came from (spec 7.4: sourceAsset, hash, mode, adaptations, license).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct Provenance {
    /// `recipe:<id>` or an asset id.
    pub source_asset: String,
    /// Content hash of the asset file, hex.
    pub hash: String,
    pub mode: String,
    /// What was adapted (for a recipe: the slots filled, in recipe order).
    pub adaptations: Vec<String>,
    pub license: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ItemBody {
    /// A reference into a source with in and out points. Source media is never modified.
    #[serde(rename_all = "camelCase")]
    Footage {
        /// Source id.
        src: String,
        src_in: f64,
        src_out: f64,
        at: f64,
        #[serde(default = "one")]
        speed: f64,
        #[serde(default = "one")]
        volume: f64,
    },
    /// A component clip (native, recipe-expanded or derived).
    #[serde(rename_all = "camelCase")]
    Clip {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        clip_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        derived_id: Option<String>,
        at: f64,
        dur: f64,
        #[serde(default)]
        props: serde_json::Value,
    },
    /// A voice slot placed on the timeline; its duration comes from the slot's active take.
    #[serde(rename_all = "camelCase")]
    Voice { slot_id: String, at: f64 },
    /// Music or effects: source ranges laid end to end from `at`.
    #[serde(rename_all = "camelCase")]
    Audio {
        /// Source id.
        src: String,
        at: f64,
        #[serde(default)]
        ranges: Vec<TimeRange>,
        #[serde(default = "one")]
        volume: f64,
        /// Duck under voice.
        #[serde(default)]
        duck: bool,
    },
    /// Captions generated from a transcript or a voice slot.
    #[serde(rename_all = "camelCase")]
    Caption {
        from: CaptionFrom,
        /// Id of the source or voice slot the words come from.
        #[serde(rename = "ref")]
        reference: String,
        /// Caption style id (resolved against the project's tokens).
        #[serde(default)]
        style: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum CaptionFrom {
    Transcript,
    Voice,
}

/// A half-open time range in seconds, `[start, end)`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
pub struct TimeRange {
    pub start: f64,
    pub end: f64,
}

fn one() -> f64 {
    1.0
}
