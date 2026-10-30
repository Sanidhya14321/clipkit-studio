//! Pipeline and agent state stored in the project (spec 3.2, 4.1). Transition rules live in ck-pipeline.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Pipeline stages in order (spec 4.1).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema, TS,
)]
#[serde(rename_all = "camelCase")]
pub enum Stage {
    Boot,
    Detect,
    Brief,
    Plan,
    Base,
    Passes,
    Verify,
    Handoff,
    Iterate,
}

impl Stage {
    pub const ALL: &'static [Stage] = &[
        Stage::Boot,
        Stage::Detect,
        Stage::Brief,
        Stage::Plan,
        Stage::Base,
        Stage::Passes,
        Stage::Verify,
        Stage::Handoff,
        Stage::Iterate,
    ];
}

/// Passes inside the PASSES stage, in order (spec 4.1).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, JsonSchema, TS,
)]
#[serde(rename_all = "camelCase")]
pub enum Pass {
    Structure,
    Audio,
    Text,
    GraphicsMotion,
    Transitions,
    Look,
    Polish,
}

impl Pass {
    pub const ALL: &'static [Pass] = &[
        Pass::Structure,
        Pass::Audio,
        Pass::Text,
        Pass::GraphicsMotion,
        Pass::Transitions,
        Pass::Look,
        Pass::Polish,
    ];

    pub fn next(self) -> Option<Pass> {
        let i = Pass::ALL.iter().position(|p| *p == self)?;
        Pass::ALL.get(i + 1).copied()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct PipelineState {
    pub stage: Stage,
    /// Current pass while `stage` is `passes`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pass: Option<Pass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_completed_pass: Option<Pass>,
}

impl Default for PipelineState {
    fn default() -> Self {
        Self {
            stage: Stage::Boot,
            pass: None,
            last_completed_pass: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum AgentStatus {
    #[default]
    Idle,
    Waiting,
    Working,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct AgentState {
    pub status: AgentStatus,
    /// Plain-words label of what the agent is doing (activity feed).
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub active_layers: Vec<String>,
    /// The user pressed Stop (spec 5.2): agent patches are refused until `handoff`.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub stop_requested: bool,
}
