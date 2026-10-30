//! Handoff and `wait` through the client (spec 4.1 HANDOFF, 6.4 event bridge).

use std::time::{Duration, Instant};

use ck_core::model::{AgentStatus, Author, Holder};
use ck_core::patch::PROJECT_LAYER;
use serde_json::{Value, json};

use super::ops::Live;
use crate::ops::Ops;
use crate::session::LockRequest;
use crate::session::handoff::{HandoffRequest, WaitKind, WaitRequest, WaitResult};

/// One patch; the agent takes the project lock for it (humans edit without one).
fn patch(ops: &Ops, author: &str, ops_json: Value) {
    let agent = author == "agent";
    if agent {
        ops.lock(&LockRequest {
            holder: Holder::Agent,
            layer_ids: vec![PROJECT_LAYER.into()],
        })
        .unwrap();
    }
    let rev = ops.status().unwrap().rev;
    let req = json!({ "baseRev": rev, "author": author, "label": "edit", "ops": ops_json });
    ops.patch(&serde_json::from_value(req).unwrap()).unwrap();
    if agent {
        ops.handoff(&HandoffRequest::default()).unwrap();
    }
}

fn wait(ops: &Ops, since: Option<u64>, secs: u64) -> WaitResult {
    ops.wait(&WaitRequest {
        since,
        timeout_secs: Some(secs),
    })
    .unwrap()
}

#[test]
fn handoff_releases_agent_locks_and_sets_idle() {
    let live = Live::start();
    let ops = live.ops();
    ops.lock(&LockRequest {
        holder: Holder::Agent,
        layer_ids: vec!["a".into(), "b".into()],
    })
    .unwrap();
    ops.lock(&LockRequest {
        holder: Holder::Human,
        layer_ids: vec!["c".into()],
    })
    .unwrap();

    let out = ops.handoff(&HandoffRequest::default()).unwrap();
    assert_eq!((out.rev, out.released, out.open_comments), (0, 2, 0));
    let status = ops.status().unwrap();
    assert_eq!(status.agent.status, AgentStatus::Idle);
    assert_eq!(status.agent.label, "Waiting for you");
    let holders: Vec<Holder> = status.locks.iter().map(|l| l.holder).collect();
    assert_eq!(holders, [Holder::Human], "the human's lock stays");
    assert_eq!(status.rev, 0, "handoff makes no revision");

    let labelled = ops
        .handoff(&HandoffRequest {
            label: Some("Done: 15 s promo".into()),
            override_reason: None,
        })
        .unwrap();
    assert_eq!(labelled.released, 0);
    assert_eq!(ops.status().unwrap().agent.label, "Done: 15 s promo");
}

#[test]
fn wait_returns_user_revisions_by_kind_and_skips_the_agent() {
    let live = Live::start();
    let ops = live.ops();
    let since = ops.handoff(&HandoffRequest::default()).unwrap().rev;
    assert!(wait(&ops, Some(since), 0).events.is_empty());

    patch(
        &ops,
        "agent",
        json!([{ "op": "add", "path": "/tokens/x", "value": 1 }]),
    );
    patch(
        &ops,
        "human",
        json!([{ "op": "replace", "path": "/brief/status", "value": "confirmed" }]),
    );
    patch(
        &ops,
        "voice",
        json!([{ "op": "add", "path": "/tokens/y", "value": 2 }]),
    );

    let out = wait(&ops, Some(since), 0);
    assert_eq!(out.rev, 3);
    let got: Vec<(u64, WaitKind, Author)> = out
        .events
        .iter()
        .map(|e| (e.rev, e.kind, e.author))
        .collect();
    assert_eq!(
        got,
        [
            (2, WaitKind::Checkpoint, Author::Human),
            (3, WaitKind::Edit, Author::Voice)
        ]
    );
    assert!(wait(&ops, Some(out.rev), 0).events.is_empty());
    assert!(
        wait(&ops, None, 0).events.is_empty(),
        "default since is now"
    );
}

#[test]
fn wait_blocks_until_the_user_acts() {
    let live = Live::start();
    let ops = live.ops();
    let waiter = {
        let dir = live.f.dir.path().to_owned();
        std::thread::spawn(move || {
            let start = Instant::now();
            let out = wait(&Ops::connect(&dir).unwrap(), Some(0), 10);
            (out, start.elapsed())
        })
    };
    std::thread::sleep(Duration::from_millis(300));
    // An agent revision does not end the wait; the human's does.
    patch(
        &ops,
        "agent",
        json!([{ "op": "add", "path": "/tokens/x", "value": 1 }]),
    );
    std::thread::sleep(Duration::from_millis(200));
    patch(
        &ops,
        "human",
        json!([{ "op": "add", "path": "/brief/fields/pacing",
                 "value": { "value": "calm", "source": "user", "confidence": 1.0 } }]),
    );
    let (out, took) = waiter.join().unwrap();
    assert_eq!(out.events.len(), 1);
    assert_eq!(
        (out.events[0].rev, out.events[0].kind),
        (2, WaitKind::Brief)
    );
    assert!(
        took < Duration::from_secs(5),
        "woken by the change, took {took:?}"
    );
}

#[test]
fn stop_refuses_the_agents_next_write_until_handoff() {
    use crate::session::handoff::HandoffRequest;
    use ck_core::ErrorCode;
    use ck_core::model::{Author, Holder};
    use ck_core::patch::PatchRequest;

    let f = super::fixture::Fixture::new();
    let rename = |author: Author, name: &str| {
        let rev = f.session.project().unwrap().rev;
        f.session.patch(&PatchRequest {
            base_rev: rev,
            layer_revs: Default::default(),
            ops: serde_json::from_value(serde_json::json!([
                { "op": "replace", "path": "/tracks/0/items/0/name", "value": name }
            ]))
            .unwrap(),
            author,
            label: "rename".into(),
        })
    };
    f.session
        .lock(&crate::session::LockRequest {
            holder: Holder::Agent,
            layer_ids: vec!["title".into()],
        })
        .unwrap();
    rename(Author::Agent, "A").unwrap();

    let agent = f.session.request_stop().unwrap();
    assert!(agent.stop_requested);
    assert!(f.session.project().unwrap().agent.stop_requested);
    let err = rename(Author::Agent, "B").unwrap_err();
    assert_eq!(err.code, ErrorCode::Stage);

    // Handing back clears it and releases the locks; the user edits, the agent may resume.
    f.session.handoff(&HandoffRequest::default()).unwrap();
    assert!(!f.session.project().unwrap().agent.stop_requested);
    rename(Author::Human, "C").unwrap();
}
