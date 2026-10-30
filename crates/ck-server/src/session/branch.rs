//! History, branches and variants (spec 5.4, 8.6). `history` lists recent revisions in short.
//! A branch is a saved copy of the project content under `.clipkit/branches/<name>.json`;
//! switching applies the difference as one revision, so it can be undone like any edit.
//! Variants are token-only treatments: one still per look at a chosen frame, rendered by the
//! worker; picking one is `look apply`.

use std::path::PathBuf;

use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::Session;
use super::look::LOOK_TOKEN;

pub mod caps {
    pub const HISTORY: usize = 4 * 1024;
    pub const BRANCH: usize = 2 * 1024;
    pub const VARIANTS: usize = 2 * 1024;
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HistoryRequest {
    /// Newest revisions to list (default 10, at most 50).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct HistoryView {
    pub rev: u64,
    /// `rev author label (n layers)`, newest first.
    pub lines: Vec<String>,
    #[serde(default, skip_serializing_if = "super::is_zero")]
    pub omitted: usize,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BranchRequest {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BranchResult {
    pub rev: u64,
    pub name: String,
    /// Layers changed by a switch.
    #[serde(default)]
    pub touched: Vec<String>,
    #[serde(default, skip_serializing_if = "super::is_zero")]
    pub omitted: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BranchList {
    /// `name (saved at rev N)`.
    pub branches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct VariantsRequest {
    /// Two or three look ids.
    pub looks: Vec<String>,
    /// Timeline frame to compare at (default the middle).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct VariantsResult {
    pub frame: u64,
    /// `look: path` per still, relative to the project folder.
    pub stills: Vec<String>,
}

fn valid_name(name: &str) -> CkResult<()> {
    let ok = !name.is_empty()
        && name.len() <= 40
        && name
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if ok {
        Ok(())
    } else {
        Err(CkError::new(
            ErrorCode::Request,
            format!("branch name {name:?} must be lowercase letters, digits and dashes"),
        ))
    }
}

fn io(e: impl std::fmt::Display) -> CkError {
    CkError::new(ErrorCode::Storage, e.to_string())
}

impl Session {
    fn branch_path(&self, name: &str) -> PathBuf {
        self.store
            .root()
            .join("branches")
            .join(format!("{name}.json"))
    }

    pub fn history(&self, req: &HistoryRequest) -> CkResult<HistoryView> {
        let limit = req.limit.unwrap_or(10).clamp(1, 50);
        let doc = self.doc()?;
        let lines = doc
            .history()
            .iter()
            .rev()
            .take(limit)
            .map(|r| {
                let author = serde_json::to_value(r.author)
                    .ok()
                    .and_then(|v| v.as_str().map(str::to_owned))
                    .unwrap_or_default();
                format!(
                    "{} {author} {} ({} layers)",
                    r.rev,
                    r.label,
                    r.touched.len()
                )
            })
            .collect();
        Ok(HistoryView {
            rev: doc.project().rev,
            lines,
            omitted: 0,
        })
    }

    /// Save the current content as branch `name` (replacing an earlier save of that name).
    pub fn branch_save(&self, req: &BranchRequest) -> CkResult<BranchResult> {
        valid_name(&req.name)?;
        let doc = self.doc()?;
        let full = serde_json::to_value(doc.project()).map_err(io)?;
        let saved = json!({ "rev": doc.project().rev, "content": ck_core::patch::content(&full) });
        let path = self.branch_path(&req.name);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(io)?;
        }
        ck_core::storage::atomic_write(&path, saved.to_string().as_bytes())?;
        Ok(BranchResult {
            rev: doc.project().rev,
            name: req.name.clone(),
            touched: Vec::new(),
            omitted: 0,
        })
    }

    pub fn branch_list(&self) -> CkResult<BranchList> {
        let dir = self.store.root().join("branches");
        let mut branches = Vec::new();
        if dir.is_dir() {
            for e in std::fs::read_dir(&dir).map_err(io)? {
                let p = e.map_err(io)?.path();
                let Some(name) = p.file_stem().and_then(|n| n.to_str()) else {
                    continue;
                };
                let rev = std::fs::read_to_string(&p)
                    .ok()
                    .and_then(|t| serde_json::from_str::<Value>(&t).ok())
                    .and_then(|v| v["rev"].as_u64())
                    .unwrap_or(0);
                branches.push(format!("{name} (saved at rev {rev})"));
            }
        }
        branches.sort();
        Ok(BranchList { branches })
    }

    /// Make the project content equal branch `name`, as one undoable agent revision.
    pub fn branch_switch(&self, req: &BranchRequest) -> CkResult<BranchResult> {
        valid_name(&req.name)?;
        let path = self.branch_path(&req.name);
        let saved: Value = std::fs::read_to_string(&path)
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .ok_or_else(|| {
                CkError::new(ErrorCode::NotFound, format!("no branch {:?}", req.name))
                    .with_hint("List branches with ck branch list.")
            })?;
        let mut doc = self.doc()?;
        let full = serde_json::to_value(doc.project()).map_err(io)?;
        let current = ck_core::patch::content(&full);
        let ops =
            serde_json::to_value(json_patch::diff(&current, &saved["content"])).map_err(io)?;
        if ops.as_array().is_some_and(Vec::is_empty) {
            return Ok(BranchResult {
                rev: doc.project().rev,
                name: req.name.clone(),
                touched: Vec::new(),
                omitted: 0,
            });
        }
        let rev = self.commit_agent(&mut doc, ops, format!("switch to branch {}", req.name))?;
        Ok(BranchResult {
            rev: rev.rev,
            name: req.name.clone(),
            touched: rev.touched,
            omitted: 0,
        })
    }

    /// One still per look at `frame`, the project otherwise unchanged (token-only treatments).
    pub fn variants(&self, req: &VariantsRequest) -> CkResult<VariantsResult> {
        if !(2..=3).contains(&req.looks.len()) {
            return Err(CkError::new(
                ErrorCode::Request,
                "variants compare 2 or 3 looks",
            ));
        }
        for id in &req.looks {
            ck_assets::library::look_get(&ck_assets::library::IdRequest { id: id.clone() })?;
        }
        let project = self.project()?;
        let fps = f64::from(project.settings.fps);
        let last = (ck_core::timeline::duration(&project) * fps)
            .ceil()
            .max(1.0) as u64
            - 1;
        let frame = req.frame.unwrap_or(last / 2).min(last);
        let looks = serde_json::to_value(ck_assets::library::looks()?).map_err(io)?;
        let dir = self.store.root().join("variants");
        std::fs::create_dir_all(&dir).map_err(io)?;
        let mut stills = Vec::new();
        for id in &req.looks {
            let mut p = project.clone();
            p.tokens
                .insert(LOOK_TOKEN.into(), Value::String(id.clone()));
            let out = dir.join(format!("{id}-{frame}.png"));
            let mut job = self.job(&p, &looks)?;
            job["out"] = json!(out);
            job["still"] = json!(frame);
            self.run_worker(&job)?;
            stills.push(format!("{id}: .clipkit/variants/{id}-{frame}.png"));
        }
        Ok(VariantsResult { frame, stills })
    }
}
