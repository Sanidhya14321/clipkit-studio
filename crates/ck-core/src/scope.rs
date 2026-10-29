//! Scope gate (spec 4): while the agent works on a comment, its patches may touch only the
//! anchored layer, the layers it declared as dependents and the comments. Anything else is
//! refused with E_SCOPE.

use std::collections::BTreeSet;

use crate::model::CommentStatus;
use crate::patch::COMMENTS_LAYER;
use crate::{CkError, CkResult, ErrorCode, Project};

/// Lock ids the agent may touch while working on comments; `None` when no comment is working.
pub fn allowed(project: &Project) -> Option<BTreeSet<String>> {
    let mut working = project
        .comments
        .iter()
        .filter(|c| c.status == CommentStatus::Working)
        .peekable();
    working.peek()?;
    let mut ids: BTreeSet<String> = BTreeSet::from([COMMENTS_LAYER.to_owned()]);
    for c in working {
        ids.extend(c.anchor.layer_id.iter().cloned());
        ids.extend(c.scope.iter().cloned());
    }
    Some(ids)
}

pub fn check(project: &Project, touched: &[String]) -> CkResult<()> {
    let Some(ok) = allowed(project) else {
        return Ok(());
    };
    let outside: Vec<String> = touched
        .iter()
        .filter(|id| !ok.contains(*id))
        .cloned()
        .collect();
    if outside.is_empty() {
        return Ok(());
    }
    Err(CkError::new(
        ErrorCode::Scope,
        "a comment is being worked on; this patch reaches outside its scope",
    )
    .with_layers(outside))
}
