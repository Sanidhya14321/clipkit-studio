//! Compact read views for agents: status and project summary. Never the whole project.

use ck_core::Project;
use ck_core::model::{AgentState, Lock, Mode, PipelineState, Settings, TrackKind};
use ck_core::timeline;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// `ck status`: where the project and the agent are.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub project_id: String,
    pub rev: u64,
    pub mode: Mode,
    pub pipeline: PipelineState,
    pub agent: AgentState,
    /// Live locks only.
    pub locks: Vec<Lock>,
}

/// `ck project get --summary`: ids, kinds and spans per track, no props or text bodies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub id: String,
    pub name: String,
    pub rev: u64,
    pub mode: Mode,
    pub settings: Settings,
    pub duration_sec: f64,
    pub tracks: Vec<TrackSummary>,
    pub sources: usize,
    pub voice_slots: usize,
    pub open_comments: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TrackSummary {
    pub id: String,
    pub kind: TrackKind,
    pub items: Vec<ItemSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ItemSummary {
    pub id: String,
    /// The item body's `type` tag (footage, clip, voice, audio, caption).
    #[serde(rename = "type")]
    pub kind: String,
    pub name: String,
    pub rev: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<f64>,
}

impl Status {
    pub fn of(project: &Project, now_ms: u64) -> Self {
        Self {
            project_id: project.id.clone(),
            rev: project.rev,
            mode: project.mode,
            pipeline: project.pipeline.clone(),
            agent: project.agent.clone(),
            locks: project
                .locks
                .iter()
                .filter(|l| l.is_live(now_ms))
                .cloned()
                .collect(),
        }
    }
}

impl Summary {
    pub fn of(project: &Project) -> Self {
        let tracks = project
            .tracks
            .iter()
            .map(|t| TrackSummary {
                id: t.id.clone(),
                kind: t.kind,
                items: t
                    .items
                    .iter()
                    .map(|i| {
                        let span = timeline::span(project, i);
                        ItemSummary {
                            id: i.id.clone(),
                            kind: item_kind(i),
                            name: i.name.clone(),
                            rev: i.rev,
                            start: span.map(|s| s.start),
                            end: span.map(|s| s.end),
                        }
                    })
                    .collect(),
            })
            .collect();
        Self {
            id: project.id.clone(),
            name: project.name.clone(),
            rev: project.rev,
            mode: project.mode,
            settings: project.settings.clone(),
            duration_sec: timeline::duration(project),
            tracks,
            sources: project.sources.len(),
            voice_slots: project.voice_slots.len(),
            open_comments: project
                .comments
                .iter()
                .filter(|c| c.status != ck_core::model::CommentStatus::Resolved)
                .count(),
        }
    }
}

fn item_kind(item: &ck_core::model::Item) -> String {
    use ck_core::model::ItemBody;
    match item.body {
        ItemBody::Footage { .. } => "footage",
        ItemBody::Clip { .. } => "clip",
        ItemBody::Voice { .. } => "voice",
        ItemBody::Audio { .. } => "audio",
        ItemBody::Caption { .. } => "caption",
    }
    .to_owned()
}
