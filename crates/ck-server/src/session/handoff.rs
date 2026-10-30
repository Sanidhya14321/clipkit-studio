//! Handoff and the event bridge (spec 4.1 HANDOFF, 6.4, 9.4 steps 8 and 9). `handoff` releases
//! every agent lock and sets the agent idle, so the user can take over. `wait` then blocks until
//! the user (or the studio microphone) changes something after a rev the agent read, and returns
//! those revisions as a compact event list.

use ck_core::model::{AgentState, AgentStatus, Author, CommentStatus, Holder};
use ck_core::{CkResult, Document};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use super::{Session, is_zero};
use crate::event::Event;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HandoffRequest {
    /// What the studio shows for the agent while idle (default "Waiting for you").
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Why the agent hands off with failing gates; shown to the user in the studio.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub override_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct HandoffResult {
    /// Project rev at handoff: pass it to `wait --since`.
    pub rev: u64,
    /// Agent locks released.
    pub released: usize,
    pub open_comments: usize,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WaitRequest {
    /// Report revisions after this rev (from `handoff` or the last `wait`); omitted means the
    /// current rev, so only new changes count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub since: Option<u64>,
    /// Seconds to block (default 20, at most 25); 0 checks once.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_secs: Option<u64>,
}

/// What a user revision changed, from the paths it wrote.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub enum WaitKind {
    /// Comments added or changed: read them with the comment bundle.
    Comment,
    /// The brief status or a checkpoint changed (confirmations).
    Checkpoint,
    /// Brief fields changed: run `brief diff`.
    Brief,
    /// Layers or other project content changed.
    Edit,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct WaitEvent {
    pub rev: u64,
    pub kind: WaitKind,
    pub author: Author,
    pub label: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub touched: Vec<String>,
}

/// `wait`: the user's revisions after `since`, oldest first. Empty on timeout; call again with
/// `rev` as the next `since`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct WaitResult {
    pub rev: u64,
    pub events: Vec<WaitEvent>,
    /// Events left out to keep the reply under its size cap.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub omitted: usize,
}

const IDLE_LABEL: &str = "Waiting for you";
/// The handoff label is persisted and sent in every snapshot; longer labels are cut.
const MAX_LABEL_CHARS: usize = 80;

/// Classify by the first path segment of the revision's operations; the most specific kind
/// any operation hits wins (comment, then checkpoint, then brief).
fn kind_of(paths: &[&str]) -> WaitKind {
    let hits = |pred: &dyn Fn(&str) -> bool| paths.iter().any(|p| pred(p));
    let under = |p: &str, root: &str| p == root || p.starts_with(&format!("{root}/"));
    if hits(&|p| under(p, "/comments")) {
        WaitKind::Comment
    } else if hits(&|p| under(p, "/brief/status") || under(p, "/brief/checkpoints")) {
        WaitKind::Checkpoint
    } else if hits(&|p| under(p, "/brief")) {
        WaitKind::Brief
    } else {
        WaitKind::Edit
    }
}

/// The idle label: the request's label cut to `MAX_LABEL_CHARS`, or the default when blank.
fn handoff_label(label: Option<&str>) -> String {
    label
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.chars().take(MAX_LABEL_CHARS).collect())
        .unwrap_or_else(|| IDLE_LABEL.to_owned())
}

/// The user's (human or voice) revisions after `since`, oldest first.
pub fn user_events(doc: &Document, since: u64) -> Vec<WaitEvent> {
    doc.history()
        .iter()
        .filter(|r| r.rev > since && r.author != Author::Agent)
        .map(|r| {
            let paths: Vec<&str> = r.forward.0.iter().map(|op| op.path().as_str()).collect();
            WaitEvent {
                rev: r.rev,
                kind: kind_of(&paths),
                author: r.author,
                label: r.label.clone(),
                touched: r.touched.clone(),
            }
        })
        .collect()
}

impl Session {
    /// `handoff`: release every agent lock and set the agent idle (no revision is made).
    pub fn handoff(&self, req: &HandoffRequest) -> CkResult<HandoffResult> {
        let project = self.project()?;
        self.gate(&project, ck_pipeline::stage::Op::Handoff)?;
        let failed = if self.pipeline_gates {
            self.verify_now(&Default::default(), project)?.failed
        } else {
            Vec::new()
        };
        let reason = req
            .override_reason
            .as_deref()
            .map(str::trim)
            .filter(|r| !r.is_empty());
        if !failed.is_empty() && reason.is_none() {
            return Err(ck_core::CkError::new(
                ck_core::ErrorCode::Stage,
                format!("quality gates failing: {failed:?}").replace('"', ""),
            )
            .with_hint(
                "Fix them and run ck verify, or pass --override-reason to tell the user why.",
            ));
        }
        let mut doc = self.doc()?;
        let released = doc
            .project()
            .locks
            .iter()
            .filter(|l| l.holder == Holder::Agent)
            .count();
        doc.release(Holder::Agent, None);
        let label = match reason.filter(|_| !failed.is_empty()) {
            Some(r) => handoff_label(Some(&format!("Gates failing ({r})"))),
            None => handoff_label(req.label.as_deref()),
        };
        doc.set_stage(ck_core::model::Stage::Handoff);
        doc.set_agent(AgentState {
            status: AgentStatus::Idle,
            label,
            active_layers: Vec::new(),
            stop_requested: false,
        });
        self.persist(&mut doc)?;
        let _ = std::fs::remove_file(self.store.root().join(ck_assets::retrieve::LEDGER_FILE));
        let project = doc.project();
        self.publish(Event::Locks {
            locks: project.locks.clone(),
        });
        self.publish_snapshot(&doc);
        let project = doc.project();
        Ok(HandoffResult {
            rev: project.rev,
            released,
            open_comments: project
                .comments
                .iter()
                .filter(|c| c.status == CommentStatus::Open)
                .count(),
        })
    }

    /// One `wait` check: the user's revisions after `since` (the current rev when omitted).
    pub fn wait_check(&self, since: Option<u64>) -> CkResult<WaitResult> {
        let doc = self.doc()?;
        let rev = doc.project().rev;
        Ok(WaitResult {
            rev,
            events: user_events(&doc, since.unwrap_or(rev)),
            omitted: 0,
        })
    }
}

impl Session {
    /// Spec 5.2 Stop: the agent's next write is refused and it hands control back. The patch in
    /// progress (already committed or not yet sent) is never cut in half. Studio only.
    pub fn request_stop(&self) -> CkResult<AgentState> {
        let mut doc = self.doc()?;
        let mut agent = doc.project().agent.clone();
        if !agent.stop_requested {
            agent.stop_requested = true;
            doc.set_agent(agent.clone());
            self.persist(&mut doc)?;
            self.publish_snapshot(&doc);
        }
        Ok(agent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds_follow_the_most_specific_path() {
        assert_eq!(kind_of(&["/comments/0"]), WaitKind::Comment);
        assert_eq!(
            kind_of(&["/brief/fields/x", "/comments"]),
            WaitKind::Comment
        );
        assert_eq!(kind_of(&["/brief/status"]), WaitKind::Checkpoint);
        assert_eq!(kind_of(&["/brief/checkpoints/0"]), WaitKind::Checkpoint);
        assert_eq!(kind_of(&["/brief/fields/pacing"]), WaitKind::Brief);
        assert_eq!(kind_of(&["/briefing"]), WaitKind::Edit);
        assert_eq!(kind_of(&["/tracks/0/items/1/at"]), WaitKind::Edit);
    }

    #[test]
    fn handoff_label_is_capped_and_defaulted() {
        assert_eq!(handoff_label(None), IDLE_LABEL);
        assert_eq!(handoff_label(Some("  ")), IDLE_LABEL);
        assert_eq!(handoff_label(Some("Done")), "Done");
        let long = "é".repeat(500);
        assert_eq!(handoff_label(Some(&long)).chars().count(), MAX_LABEL_CHARS);
    }
}
