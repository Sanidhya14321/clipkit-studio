//! `look apply` (spec 8.1): re-theme the project through design tokens. Layer props keep their
//! `$role.name` token refs, so switching the look is one write of `tokens.look` and no layer is
//! rewritten (zero agent tokens per layer). The reply is the impact preview: the layers whose
//! look changes. `dryRun` returns the preview without writing.

use ck_assets::library;
use ck_core::model::{Author, Holder, ItemBody, Project};
use ck_core::patch::{PROJECT_LAYER, PatchRequest};
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::recipe::DEFAULT_LOOK;
use super::{Session, is_zero};
use crate::event::Event;

/// Key of the stored project look in `Project.tokens`.
pub const LOOK_TOKEN: &str = "look";

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LookApplyRequest {
    /// Look id (from `look list`).
    pub look: String,
    /// Only preview the impact; write nothing.
    #[serde(default)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct LookApplyResult {
    /// Project revision after the write (unchanged on a dry run or when the look is the same).
    pub rev: u64,
    pub previous: String,
    pub look: String,
    /// True when `tokens.look` was written.
    pub applied: bool,
    /// Layers that use look tokens and so change appearance.
    pub impacted: Vec<String>,
    /// Layer ids left out of `impacted` to keep the reply under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

/// The project's look: the stored `tokens.look`, else the brief's `look` field, else the default.
pub fn project_look_id(project: &Project) -> String {
    let stored = project.tokens.get(LOOK_TOKEN).and_then(Value::as_str);
    let brief = project
        .brief
        .fields
        .get("look")
        .and_then(|f| f.value.as_str());
    stored
        .or(brief)
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_LOOK)
        .to_owned()
}

fn has_token_ref(v: &Value) -> bool {
    match v {
        Value::String(s) => s.starts_with('$'),
        Value::Array(a) => a.iter().any(has_token_ref),
        Value::Object(o) => o.values().any(has_token_ref),
        _ => false,
    }
}

/// Ids of the clip layers whose props hold a `$token` ref, in track order.
pub fn token_layers(project: &Project) -> Vec<String> {
    project
        .tracks
        .iter()
        .flat_map(|t| &t.items)
        .filter(|i| matches!(&i.body, ItemBody::Clip { props, .. } if has_token_ref(props)))
        .map(|i| i.id.clone())
        .collect()
}

fn server_err(e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Server, e.to_string())
}

impl Session {
    /// `look apply`: write `tokens.look` as one agent revision. Like `brief propose`, the agent
    /// briefly takes the project lock unless it holds it, so a human mid-edit refuses the write.
    pub fn look_apply(&self, req: &LookApplyRequest) -> CkResult<LookApplyResult> {
        // Unknown ids fail here with the library's error and hint.
        library::look_get(&library::IdRequest {
            id: req.look.clone(),
        })?;
        let now = self.now_ms();
        let mut doc = self.doc()?;
        let project = doc.project();
        let previous = project_look_id(project);
        let stored = project.tokens.get(LOOK_TOKEN).and_then(Value::as_str);
        let mut result = LookApplyResult {
            rev: project.rev,
            impacted: if previous == req.look {
                Vec::new()
            } else {
                token_layers(project)
            },
            previous,
            look: req.look.clone(),
            applied: false,
            omitted: 0,
        };
        if req.dry_run || stored == Some(req.look.as_str()) {
            return Ok(result);
        }
        let ops = serde_json::from_value(json!([
            { "op": "add", "path": format!("/tokens/{LOOK_TOKEN}"), "value": req.look }
        ]))
        .map_err(server_err)?;
        let patch = PatchRequest {
            base_rev: project.rev,
            layer_revs: Default::default(),
            ops,
            author: Author::Agent,
            label: format!("look: {}", req.look),
        };
        let project_lock = [PROJECT_LAYER.to_owned()];
        let held = project
            .locks
            .iter()
            .any(|l| l.layer_id == PROJECT_LAYER && l.holder == Holder::Agent && l.is_live(now));
        if !held {
            doc.acquire(Holder::Agent, &project_lock, now)?;
        }
        let committed = doc.patch(&patch, now).cloned();
        if !held {
            doc.release(Holder::Agent, Some(&project_lock));
        }
        let rev = committed?;
        self.persist(&mut doc)?;
        self.publish(Event::Revision {
            rev: rev.rev,
            author: rev.author,
            label: rev.label,
            touched: rev.touched,
        });
        self.publish_snapshot(&doc);
        result.rev = rev.rev;
        result.applied = true;
        Ok(result)
    }
}
