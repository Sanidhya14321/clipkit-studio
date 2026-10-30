//! `ck demo reset` (spec 12.1): restores a seeded demo project in well under 10 seconds, with
//! no network, agent or render: a confirmed brief, three recipe scenes filling 15 seconds, a
//! voice script and one open comment. It refuses to replace a project that is not the demo
//! (unless forced) or while a studio serves the folder.

use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

use ck_core::model::{Anchor, Author, Mode};
use ck_core::patch::PatchRequest;
use ck_core::storage::Store;
use ck_core::{CkError, CkResult, ErrorCode, Project};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::clock::SystemClock;
use crate::ops::Ops;
use crate::session::Session;
use crate::session::brief::BriefProposeRequest;
use crate::session::comments::CommentAddRequest;
use crate::session::recipe::RecipeApplyRequest;
use crate::session::voice::{ScriptLine, VoiceScriptRequest};

/// Id of the seeded project; only a project with this id is replaced without `force`.
pub const DEMO_ID: &str = "demo";

pub mod caps {
    pub const DEMO: usize = 1024;
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DemoRequest {
    /// Replace a project that is not the demo.
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DemoResult {
    pub rev: u64,
    pub layers: usize,
    pub duration_sec: f64,
    /// Time the reset took.
    pub ms: u64,
}

fn refuse(msg: &str, hint: &str) -> CkError {
    CkError::new(ErrorCode::Request, msg.to_owned()).with_hint(hint.to_owned())
}

fn patch(s: &Session, ops: serde_json::Value, label: &str) -> CkResult<()> {
    let rev = s.project()?.rev;
    s.patch(&PatchRequest {
        base_rev: rev,
        layer_revs: Default::default(),
        ops: serde_json::from_value(ops)
            .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?,
        author: Author::Human,
        label: label.into(),
    })?;
    Ok(())
}

/// Seeds `<dir>/.clipkit` with the demo project.
pub fn reset(dir: &Path, req: &DemoRequest) -> CkResult<DemoResult> {
    let started = Instant::now();
    let root = dir.join(".clipkit");
    if Ops::connect(&root).and_then(|o| o.status()).is_ok() {
        return Err(refuse(
            "a studio is serving this folder",
            "Stop it (or let it idle out), then reset.",
        ));
    }
    let store = Store::new(&root);
    if store.project_path().is_file() {
        let id = store
            .load()
            .map(|d| d.project().id.clone())
            .unwrap_or_default();
        if id != DEMO_ID && !req.force {
            return Err(refuse(
                &format!("project {id:?} is not the demo"),
                "Use --force to replace it, or reset in another folder.",
            ));
        }
        for sub in ["project.json", "history"] {
            let p = root.join(sub);
            let gone = if p.is_dir() {
                std::fs::remove_dir_all(&p)
            } else {
                std::fs::remove_file(&p)
            };
            gone.map_err(|e| CkError::new(ErrorCode::Storage, format!("{}: {e}", p.display())))?;
        }
    }
    let mut project = Project::new(DEMO_ID, "ClipKit demo", Mode::Generate);
    project.brief.license_mode = Some("commercial-safe".into());
    store.init(&project)?;
    let s = Session::open(&root, Arc::new(SystemClock))?;

    let fields = json!({
        "goal": "Launch teaser for ClipKit Studio",
        "platform": "youtube",
        "length": 15,
        "pacing": "lively",
        "look": "clean-product",
        "cta": "Try ClipKit"
    });
    s.brief_propose(&BriefProposeRequest {
        fields: serde_json::from_value(fields)
            .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?,
        ..BriefProposeRequest::default()
    })?;
    patch(
        &s,
        json!([{ "op": "replace", "path": "/brief/status", "value": "confirmed" }]),
        "Confirm brief",
    )?;
    for scene in [
        json!({ "recipe": "hook-title", "at": 0.0, "duration": 4.0,
                "slots": { "headline": "Edit video by talking" } }),
        json!({ "recipe": "stat-callout", "at": 4.0, "duration": 5.0,
                "slots": { "value": 3, "unit": "x", "label": "faster first cut" } }),
        json!({ "recipe": "outro-cta", "at": 9.0, "duration": 6.0,
                "slots": { "action": "Try ClipKit", "handle": "clipkit.dev" } }),
    ] {
        let req: RecipeApplyRequest = serde_json::from_value(scene)
            .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
        s.recipe_apply(&req)?;
    }
    s.voice_script(&VoiceScriptRequest {
        lines: [
            "Meet the editor you talk to.",
            "Your agent builds the cut while you watch.",
            "Take over any time.",
        ]
        .iter()
        .map(|t| ScriptLine {
            text: (*t).to_owned(),
            window: None,
        })
        .collect(),
        mode: None,
    })?;
    s.comments_add(&CommentAddRequest {
        text: "Make the stat land half a second later".into(),
        anchor: Anchor {
            t_start: Some(4.0),
            ..Anchor::default()
        },
        author: Some(Author::Human),
    })?;
    let p = s.project()?;
    Ok(DemoResult {
        rev: p.rev,
        layers: p.items().count(),
        duration_sec: ck_core::timeline::duration(&p),
        ms: started.elapsed().as_millis() as u64,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_demo_resets_fast_and_only_replaces_itself() {
        let dir = tempfile::tempdir().unwrap();
        let r = reset(dir.path(), &DemoRequest::default()).unwrap();
        assert!(r.ms < 10_000, "{r:?}");
        assert!((r.duration_sec - 15.0).abs() < 1e-6, "{r:?}");
        assert!(r.layers >= 6);
        let again = reset(dir.path(), &DemoRequest::default()).unwrap();
        assert_eq!(again.rev, r.rev, "a reset rebuilds the same history");

        let other = tempfile::tempdir().unwrap();
        Store::new(other.path().join(".clipkit"))
            .init(&Project::new("mine", "Mine", Mode::Edit))
            .unwrap();
        let err = reset(other.path(), &DemoRequest::default()).unwrap_err();
        assert_eq!(err.code, ErrorCode::Request);
        reset(other.path(), &DemoRequest { force: true }).unwrap();
    }
}
