//! Comments (spec 8.4): anchored to a moment, a range, a layer or an element; lifecycle open,
//! working, resolved. `list` is the send bundle: open and working comments plus summaries of
//! the layers they point at. An agent reply starts work on a comment and may declare the layers
//! it will change besides the anchored one; the scope gate (ck-core) holds the agent to them.

use std::collections::BTreeSet;

use ck_core::model::{Anchor, Author, Comment, CommentStatus, Holder, Project, Reply};
use ck_core::{CkError, CkResult, ErrorCode};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use ts_rs::TS;

use super::{Session, is_zero};
use crate::view::ItemSummary;

/// Longest comment or reply text, in bytes.
pub const MAX_TEXT: usize = 2000;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommentAddRequest {
    pub text: String,
    #[serde(default)]
    pub anchor: Anchor,
    /// Who writes it: the agent (uncertainties) or the human (the studio). Default agent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommentListRequest {
    /// Include resolved comments.
    #[serde(default)]
    pub all: bool,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommentReplyRequest {
    pub id: String,
    pub text: String,
    /// Layers the agent will change besides the anchored one (replaces the declared scope).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommentResolveRequest {
    pub id: String,
    /// Optional closing reply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<Author>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommentResult {
    pub rev: u64,
    pub id: String,
    pub status: CommentStatus,
    /// On resolve: the layers changed between the comment and its resolution (before/after).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed: Vec<String>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommentView {
    pub id: String,
    pub author: Author,
    pub status: CommentStatus,
    pub text: String,
    pub anchor: Anchor,
    pub created_rev: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scope: Vec<String>,
    /// The latest reply, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_reply: Option<Reply>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommentBundle {
    pub rev: u64,
    pub comments: Vec<CommentView>,
    /// The anchored and scoped layers, in short.
    pub layers: Vec<ItemSummary>,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

fn check_text(text: &str) -> CkResult<()> {
    if text.trim().is_empty() || text.len() > MAX_TEXT {
        return Err(CkError::new(
            ErrorCode::Request,
            format!("comment text must be 1 to {MAX_TEXT} bytes"),
        ));
    }
    Ok(())
}

fn holder(author: Option<Author>) -> Holder {
    match author.unwrap_or(Author::Agent) {
        Author::Agent => Holder::Agent,
        Author::Human | Author::Voice => Holder::Human,
    }
}

fn index_of(project: &Project, id: &str) -> CkResult<usize> {
    project
        .comments
        .iter()
        .position(|c| c.id == id)
        .ok_or_else(|| {
            CkError::new(ErrorCode::NotFound, format!("no comment {id:?}"))
                .with_hint("Run ck comments list for the open comments.")
        })
}

/// The bundle for `project`: open and working comments (or all) and their layers.
pub fn bundle(project: &Project, all: bool) -> CommentBundle {
    let comments: Vec<&Comment> = project
        .comments
        .iter()
        .filter(|c| all || c.status != CommentStatus::Resolved)
        .collect();
    let ids: BTreeSet<&str> = comments
        .iter()
        .flat_map(|c| c.anchor.layer_id.iter().chain(c.scope.iter()))
        .map(String::as_str)
        .collect();
    let layers = ids
        .into_iter()
        .filter_map(|id| project.item(id))
        .map(|i| ItemSummary::of(project, i))
        .collect();
    CommentBundle {
        rev: project.rev,
        comments: comments
            .into_iter()
            .map(|c| CommentView {
                id: c.id.clone(),
                author: c.author,
                status: c.status,
                text: c.text.clone(),
                anchor: c.anchor.clone(),
                created_rev: c.created_rev,
                scope: c.scope.clone(),
                last_reply: c.replies.last().cloned(),
            })
            .collect(),
        layers,
        omitted: 0,
    }
}

impl Session {
    pub fn comments_list(&self, req: &CommentListRequest) -> CkResult<CommentBundle> {
        Ok(bundle(self.doc()?.project(), req.all))
    }

    pub fn comments_add(&self, req: &CommentAddRequest) -> CkResult<CommentResult> {
        check_text(&req.text)?;
        let author = req.author.unwrap_or(Author::Agent);
        let mut doc = self.doc()?;
        let project = doc.project();
        if let Some(layer) = &req.anchor.layer_id
            && project.item(layer).is_none()
        {
            return Err(CkError::new(
                ErrorCode::NotFound,
                format!("no layer with id {layer:?}"),
            ));
        }
        let id = (1..)
            .map(|n| format!("c{n}"))
            .find(|id| !project.comments.iter().any(|c| &c.id == id))
            .unwrap_or_default();
        let comment = Comment {
            id: id.clone(),
            author,
            anchor: req.anchor.clone(),
            text: req.text.clone(),
            status: CommentStatus::Open,
            created_rev: project.rev + 1,
            resolved_rev: None,
            replies: Vec::new(),
            scope: Vec::new(),
        };
        let ops = json!([{ "op": "add", "path": "/comments/-", "value": comment }]);
        let label = format!("comment {id}");
        let rev = self.commit_as(&mut doc, holder(req.author), ops, label, &[])?;
        Ok(CommentResult {
            rev: rev.rev,
            id,
            status: CommentStatus::Open,
            changed: Vec::new(),
            omitted: 0,
        })
    }

    /// An agent reply on an open comment starts work on it (status working).
    pub fn comments_reply(&self, req: &CommentReplyRequest) -> CkResult<CommentResult> {
        check_text(&req.text)?;
        let author = req.author.unwrap_or(Author::Agent);
        let mut doc = self.doc()?;
        let project = doc.project();
        let n = index_of(project, &req.id)?;
        let c = &project.comments[n];
        if let Some(bad) = req
            .scope
            .iter()
            .flatten()
            .find(|id| project.item(id).is_none())
        {
            return Err(CkError::new(
                ErrorCode::NotFound,
                format!("scope names no layer {bad:?}"),
            ));
        }
        let reply = Reply {
            author,
            text: req.text.clone(),
            rev: project.rev + 1,
        };
        let mut ops = vec![
            json!({ "op": "add", "path": format!("/comments/{n}/replies/-"),
                                   "value": reply }),
        ];
        let status = if author == Author::Agent && c.status == CommentStatus::Open {
            ops.push(
                json!({ "op": "replace", "path": format!("/comments/{n}/status"),
                             "value": CommentStatus::Working }),
            );
            CommentStatus::Working
        } else {
            c.status
        };
        if let Some(scope) = &req.scope {
            ops.push(json!({ "op": "add", "path": format!("/comments/{n}/scope"),
                             "value": scope }));
        }
        let label = format!("reply {}", req.id);
        let ops = serde_json::Value::Array(ops);
        let rev = self.commit_as(&mut doc, holder(req.author), ops, label, &[])?;
        Ok(CommentResult {
            rev: rev.rev,
            id: req.id.clone(),
            status,
            changed: Vec::new(),
            omitted: 0,
        })
    }

    pub fn comments_resolve(&self, req: &CommentResolveRequest) -> CkResult<CommentResult> {
        if let Some(t) = &req.text {
            check_text(t)?;
        }
        let author = req.author.unwrap_or(Author::Agent);
        let mut doc = self.doc()?;
        let project = doc.project();
        let n = index_of(project, &req.id)?;
        let created = project.comments[n].created_rev;
        let resolved = project.rev + 1;
        let mut ops = Vec::new();
        if let Some(text) = &req.text {
            let reply = Reply {
                author,
                text: text.clone(),
                rev: resolved,
            };
            ops.push(
                json!({ "op": "add", "path": format!("/comments/{n}/replies/-"),
                             "value": reply }),
            );
        }
        ops.push(
            json!({ "op": "replace", "path": format!("/comments/{n}/status"),
                         "value": CommentStatus::Resolved }),
        );
        ops.push(
            json!({ "op": "add", "path": format!("/comments/{n}/resolvedRev"),
                         "value": resolved }),
        );
        // Before/after: the content layers changed since the comment was made.
        let changed: BTreeSet<String> = doc
            .history()
            .iter()
            .filter(|r| r.rev > created)
            .flat_map(|r| r.touched.iter())
            .filter(|id| !id.starts_with('@'))
            .cloned()
            .collect();
        let label = format!("resolve {}", req.id);
        let ops = serde_json::Value::Array(ops);
        let rev = self.commit_as(&mut doc, holder(req.author), ops, label, &[])?;
        Ok(CommentResult {
            rev: rev.rev,
            id: req.id.clone(),
            status: CommentStatus::Resolved,
            changed: changed.into_iter().collect(),
            omitted: 0,
        })
    }
}
