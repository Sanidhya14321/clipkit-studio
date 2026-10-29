//! A project plus its history: the one place patches, the lock gate and undo/redo meet.

use crate::error::{CkError, CkResult, ErrorCode};
use crate::history::{Revision, RevisionKind, Stacks};
use crate::locks;
use crate::model::{AgentState, Author, Holder, Project};
use crate::patch::{self, PatchRequest};

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    project: Project,
    history: Vec<Revision>,
    stacks: Stacks,
}

impl Document {
    pub fn new(project: Project) -> Self {
        Self {
            project,
            history: Vec::new(),
            stacks: Stacks::default(),
        }
    }

    /// Rebuild from a stored project and its history (used by storage on load).
    pub fn from_parts(project: Project, history: Vec<Revision>) -> Self {
        let stacks = Stacks::from_history(&history);
        Self {
            project,
            history,
            stacks,
        }
    }

    pub fn project(&self) -> &Project {
        &self.project
    }

    pub fn history(&self) -> &[Revision] {
        &self.history
    }

    /// Apply a patch through the revision, schema and lock gates and record it.
    pub fn patch(&mut self, req: &PatchRequest, now_ms: u64) -> CkResult<&Revision> {
        self.commit(req, RevisionKind::Edit, now_ms)
    }

    /// Undo the most recent edit that is not already undone, as a new revision.
    pub fn undo(&mut self, author: Author, now_ms: u64) -> CkResult<&Revision> {
        let target = self.target(self.stacks.undo.last().copied(), "nothing to undo")?;
        let req = self.internal_request(author, format!("undo: {}", target.label), target.inverse);
        self.commit(&req, RevisionKind::Undo { of: target.rev }, now_ms)
    }

    /// Re-apply the most recently undone edit, as a new revision.
    pub fn redo(&mut self, author: Author, now_ms: u64) -> CkResult<&Revision> {
        let target = self.target(self.stacks.redo.last().copied(), "nothing to redo")?;
        let req = self.internal_request(author, format!("redo: {}", target.label), target.forward);
        self.commit(&req, RevisionKind::Redo { of: target.rev }, now_ms)
    }

    pub fn acquire(&mut self, holder: Holder, layer_ids: &[String], now_ms: u64) -> CkResult<()> {
        locks::acquire(
            &mut self.project.locks,
            holder,
            layer_ids,
            now_ms,
            locks::DEFAULT_TTL_MS,
        )
    }

    pub fn heartbeat(&mut self, holder: Holder, now_ms: u64) -> usize {
        locks::heartbeat(&mut self.project.locks, holder, now_ms)
    }

    /// Set the engine-owned agent state. Like locks it is not content, so no revision is made.
    pub fn set_agent(&mut self, agent: AgentState) {
        self.project.agent = agent;
    }

    pub fn release(&mut self, holder: Holder, layer_ids: Option<&[String]>) {
        locks::release(&mut self.project.locks, holder, layer_ids);
    }

    /// The project content (engine-owned fields removed) as it was at revision `rev`, rebuilt
    /// by applying the inverse of every later revision. E_NOT_FOUND when history does not reach
    /// back that far or `rev` is in the future.
    pub fn content_at(&self, rev: u64) -> CkResult<serde_json::Value> {
        let current = self.project.rev;
        let later: Vec<&Revision> = self.history.iter().filter(|r| r.rev > rev).collect();
        // Revisions are numbered one apart, so a full history holds exactly current - rev later ones.
        let reachable = rev <= current && later.len() as u64 == current - rev;
        if !reachable {
            return Err(CkError::new(
                ErrorCode::NotFound,
                format!("revision {rev} is not in history (current is {current})"),
            ));
        }
        let full = serde_json::to_value(&self.project)
            .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
        let mut content = patch::content(&full);
        for r in later.iter().rev() {
            json_patch::patch(&mut content, &r.inverse).map_err(|e| {
                CkError::new(ErrorCode::Server, format!("replaying rev {}: {e}", r.rev))
            })?;
        }
        Ok(content)
    }

    fn target(&self, rev: Option<u64>, empty: &str) -> CkResult<Revision> {
        rev.and_then(|r| self.history.iter().find(|h| h.rev == r))
            .cloned()
            .ok_or_else(|| CkError::new(ErrorCode::HistoryEmpty, empty))
    }

    fn internal_request(
        &self,
        author: Author,
        label: String,
        ops: json_patch::Patch,
    ) -> PatchRequest {
        PatchRequest {
            base_rev: self.project.rev,
            layer_revs: Default::default(),
            ops,
            author,
            label,
        }
    }

    fn commit(
        &mut self,
        req: &PatchRequest,
        kind: RevisionKind,
        now_ms: u64,
    ) -> CkResult<&Revision> {
        let applied = patch::apply(&self.project, req)?;
        locks::check_patch(&self.project.locks, req.author, &applied.touched, now_ms)?;
        if req.author == Author::Agent {
            crate::scope::check(&self.project, &applied.touched)?;
            consent_unchanged(&self.project, &applied.project)?;
        }
        let mut project = applied.project;
        // Locks are engine state, not content: carry them over and drop expired ones.
        project.locks = std::mem::take(&mut self.project.locks);
        locks::prune(&mut project.locks, now_ms);
        let rev = Revision {
            rev: project.rev,
            author: req.author,
            label: req.label.clone(),
            timestamp_ms: now_ms,
            touched: applied.touched,
            kind,
            forward: applied.forward,
            inverse: applied.inverse,
        };
        tracing::info!(
            rev = rev.rev,
            author = ?rev.author,
            kind = ?rev.kind,
            layerIds = ?rev.touched,
            "revision committed"
        );
        self.project = project;
        self.stacks.record(rev.rev, kind);
        self.history.push(rev);
        self.history
            .last()
            .ok_or_else(|| CkError::new(ErrorCode::HistoryEmpty, "revision was not recorded"))
    }
}

#[cfg(test)]
mod tests;

/// Only a human gives or withdraws cloning consent (spec 8.3): an agent patch that changes any
/// take's `consented` flag, or adds a take that carries it, is refused.
fn consent_unchanged(before: &Project, after: &Project) -> CkResult<()> {
    let consents = |p: &Project| -> Vec<(String, String)> {
        let mut v: Vec<(String, String)> = p
            .voice_slots
            .iter()
            .flat_map(|s| {
                s.takes
                    .iter()
                    .filter(|t| t.consented)
                    .map(|t| (s.id.clone(), t.id.clone()))
            })
            .collect();
        v.sort();
        v
    };
    if consents(before) == consents(after) {
        return Ok(());
    }
    Err(CkError::new(
        ErrorCode::PatchInvalid,
        "only the user can give or withdraw cloning consent",
    )
    .with_hint("Ask the user to consent in the studio's voice panel."))
}
