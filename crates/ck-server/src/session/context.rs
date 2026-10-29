//! `context` (spec 6.4, Lane A): what the user is pointing at in the studio, so "this", "here"
//! and "that" resolve. The studio reports its selection, hovered element, playhead and loop
//! range (UI state, not a revision); the answer adds open comments, in about 300 tokens.

use ck_core::model::{CommentStatus, Project};
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Session, is_zero};

/// Comment text is cut to this many characters in the context answer.
const TEXT_CHARS: usize = 80;

/// The studio's pointer state, as the studio last reported it.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StudioContext {
    /// Selected layer id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection: Option<String>,
    /// Hovered element: layer id and, inside it, the element path (data-ck-path).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hovered: Option<Hovered>,
    /// Playhead in seconds.
    #[serde(default)]
    pub playhead: f64,
    /// Loop range in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub loop_range: Option<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Hovered {
    pub layer_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextRequest {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OpenComment {
    pub id: String,
    pub text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub t_start: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ContextView {
    pub rev: u64,
    #[serde(flatten)]
    pub studio: StudioContext,
    /// Name of the selected layer, so the agent need not read it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_name: Option<String>,
    pub open_comments: Vec<OpenComment>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

fn cut(text: &str) -> String {
    if text.chars().count() <= TEXT_CHARS {
        return text.to_owned();
    }
    let mut s: String = text.chars().take(TEXT_CHARS - 1).collect();
    s.push('…');
    s
}

pub fn view(project: &Project, studio: &StudioContext) -> ContextView {
    ContextView {
        rev: project.rev,
        studio: studio.clone(),
        selection_name: studio
            .selection
            .as_deref()
            .and_then(|id| project.item(id))
            .map(|i| i.name.clone()),
        open_comments: project
            .comments
            .iter()
            .filter(|c| c.status != CommentStatus::Resolved)
            .map(|c| OpenComment {
                id: c.id.clone(),
                text: cut(&c.text),
                layer_id: c.anchor.layer_id.clone(),
                t_start: c.anchor.t_start,
            })
            .collect(),
        omitted: 0,
    }
}

impl Session {
    pub fn context(&self) -> CkResult<ContextView> {
        let studio = self
            .studio_context
            .lock()
            .map_err(|_| CkError::new(ErrorCode::Server, "context state poisoned"))?
            .clone();
        Ok(view(self.doc()?.project(), &studio))
    }

    /// The studio reports its pointer state (no revision, no event).
    pub fn set_context(&self, ctx: StudioContext) -> CkResult<StudioContext> {
        if !ctx.playhead.is_finite() || ctx.playhead < 0.0 {
            return Err(CkError::new(
                ErrorCode::Request,
                "playhead must be 0 or more",
            ));
        }
        let mut slot = self
            .studio_context
            .lock()
            .map_err(|_| CkError::new(ErrorCode::Server, "context state poisoned"))?;
        *slot = ctx.clone();
        Ok(ctx)
    }
}
