//! Append-only revision history. Undo and redo are new revisions; history is never rewritten.

use json_patch::Patch;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::Author;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RevisionKind {
    Edit,
    /// Undid the edit made at revision `of`.
    Undo {
        of: u64,
    },
    /// Re-applied the edit made at revision `of`.
    Redo {
        of: u64,
    },
}

/// One stored revision (one file in `.clipkit/history/`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Revision {
    pub rev: u64,
    pub author: Author,
    pub label: String,
    pub timestamp_ms: u64,
    /// Item ids added, removed or changed.
    pub touched: Vec<String>,
    #[serde(flatten)]
    pub kind: RevisionKind,
    /// Content patch from the previous revision to this one.
    pub forward: Patch,
    /// Content patch back to the previous revision.
    pub inverse: Patch,
}

/// Undo and redo stacks (revision numbers of edits), rebuilt by replaying revision kinds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Stacks {
    pub undo: Vec<u64>,
    pub redo: Vec<u64>,
}

impl Stacks {
    pub fn from_history(revisions: &[Revision]) -> Self {
        let mut s = Stacks::default();
        for r in revisions {
            s.record(r.rev, r.kind);
        }
        s
    }

    pub fn record(&mut self, rev: u64, kind: RevisionKind) {
        match kind {
            RevisionKind::Edit => {
                self.undo.push(rev);
                self.redo.clear();
            }
            RevisionKind::Undo { of } => {
                self.undo.retain(|r| *r != of);
                self.redo.push(of);
            }
            RevisionKind::Redo { of } => {
                self.redo.retain(|r| *r != of);
                self.undo.push(of);
            }
        }
    }
}
