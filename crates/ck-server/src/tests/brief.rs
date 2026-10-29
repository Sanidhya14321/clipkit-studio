//! Brief Studio essentials through the client: propose, get, wait, diff (spec 8.1).

use std::time::{Duration, Instant};

use ck_core::ErrorCode;
use ck_core::model::{BriefSource, BriefStatus, Holder};
use ck_core::patch::PROJECT_LAYER;
use serde_json::{Value, json};

use super::ops::Live;
use crate::ops::{Ops, caps, trimmed};
use crate::session::LockRequest;
use crate::session::brief::{
    BriefChange, BriefDiff, BriefDiffRequest, BriefProposeRequest, BriefWaitRequest,
};

fn propose(v: Value) -> BriefProposeRequest {
    serde_json::from_value(v).unwrap()
}

fn essentials() -> BriefProposeRequest {
    propose(json!({
        "fields": { "platform": "youtube", "length": 15, "pacing": "brisk" },
        "confidence": { "platform": 0.9 },
        "questions": [{ "id": "price", "text": "Show the price?", "affects": ["cta"] }]
    }))
}

/// The studio's confirm: a human patch that sets one field and the status.
fn confirm(ops: &Ops, rev: u64, length: Value) {
    let req = json!({
        "baseRev": rev, "author": "human", "label": "confirm brief",
        "ops": [
            { "op": "replace", "path": "/brief/fields/length",
              "value": { "value": length, "source": "user", "confidence": 1.0 } },
            { "op": "replace", "path": "/brief/status", "value": "confirmed" }
        ]
    });
    ops.patch(&serde_json::from_value(req).unwrap()).unwrap();
}

fn wait(ops: &Ops, secs: u64) -> crate::session::brief::BriefWaitResult {
    ops.brief_wait(&BriefWaitRequest {
        timeout_secs: Some(secs),
    })
    .unwrap()
}

#[test]
fn propose_get_confirm_and_diff() {
    let live = Live::start();
    let ops = live.ops();

    let proposed = ops.brief_propose(&essentials()).unwrap();
    assert_eq!((proposed.rev, proposed.status), (1, BriefStatus::Proposed));
    assert!(
        ops.status().unwrap().locks.is_empty(),
        "the project lock is released"
    );

    let brief = ops.brief_get().unwrap();
    assert_eq!(brief.rev, 1);
    assert_eq!(brief.fields["platform"].source, BriefSource::Inferred);
    assert_eq!(brief.questions[0].affects, ["cta"]);
    assert!(!wait(&ops, 0).ready, "not ready until the user confirms");

    confirm(&ops, 1, json!(20));
    let ready = wait(&ops, 0);
    assert!(ready.ready);
    assert_eq!(ready.brief.unwrap().fields["length"].value, 20);

    let diff = ops.brief_diff(&BriefDiffRequest { since: 1 }).unwrap();
    let fields: Vec<&str> = diff.changes.iter().map(|c| c.field.as_str()).collect();
    assert_eq!(fields, ["status", "length"]);
    assert_eq!(diff.changes[1].source, Some(BriefSource::User));
    assert_eq!((diff.since, diff.rev), (1, 2));
    assert!(
        ops.brief_diff(&BriefDiffRequest { since: 2 })
            .unwrap()
            .changes
            .is_empty()
    );

    // A new proposal never overwrites what the user set, and asks for confirmation again.
    let mut again = essentials();
    again.fields.insert("length".into(), json!(10));
    let res = ops.brief_propose(&again).unwrap();
    assert_eq!(
        (res.status, res.kept.as_slice()),
        (BriefStatus::Proposed, &["length".to_owned()][..])
    );
    assert_eq!(ops.brief_get().unwrap().fields["length"].value, 20);

    // Proposing the same values twice writes no revision.
    let rev = ops.status().unwrap().rev;
    assert_eq!(ops.brief_propose(&again).unwrap().rev, rev);
}

#[test]
fn wait_blocks_until_the_user_confirms() {
    let live = Live::start();
    let ops = live.ops();
    ops.brief_propose(&essentials()).unwrap();

    let waiter = {
        let dir = live.f.dir.path().to_owned();
        std::thread::spawn(move || {
            let start = Instant::now();
            let out = wait(&Ops::connect(&dir).unwrap(), 10);
            (out, start.elapsed())
        })
    };
    std::thread::sleep(Duration::from_millis(300));
    confirm(&ops, 1, json!(15));
    let (out, took) = waiter.join().unwrap();
    assert!(out.ready);
    assert_eq!(out.status, BriefStatus::Confirmed);
    assert!(
        took < Duration::from_secs(5),
        "woken by the change, took {took:?}"
    );

    // An auto brief is ready at once.
    let auto = propose(json!({ "fields": { "platform": "tiktok" }, "auto": true }));
    assert_eq!(ops.brief_propose(&auto).unwrap().status, BriefStatus::Auto);
    assert!(wait(&ops, 0).ready);
}

#[test]
fn bad_proposals_are_refused() {
    let live = Live::start();
    let ops = live.ops();
    let q = |id: &str| json!({ "id": id, "text": "Why?", "affects": ["hook"] });
    let refused = |v: Value| ops.brief_propose(&propose(v)).unwrap_err().code;

    assert_eq!(
        refused(json!({ "fields": {}, "questions": [q("a"), q("b"), q("c"), q("d")] })),
        ErrorCode::Request
    );
    assert_eq!(
        refused(json!({ "fields": { "goal": "x".repeat(5000) } })),
        ErrorCode::Request
    );
    assert_eq!(
        refused(json!({ "fields": { "bad name": 1 } })),
        ErrorCode::Request
    );
    assert_eq!(
        refused(json!({ "fields": { "a": 1 }, "confidence": { "a": 2.0 } })),
        ErrorCode::Request
    );
    assert_eq!(
        refused(
            json!({ "fields": {}, "questions": [{ "id": "a", "text": "Why?", "affects": [] }] })
        ),
        ErrorCode::Schema
    );

    // While the human holds the project lock the agent cannot write the brief.
    ops.lock(&LockRequest {
        holder: Holder::Human,
        layer_ids: vec![PROJECT_LAYER.to_owned()],
    })
    .unwrap();
    assert_eq!(
        ops.brief_propose(&essentials()).unwrap_err().code,
        ErrorCode::LockHeld
    );
    assert_eq!(ops.status().unwrap().rev, 0);

    assert_eq!(
        ops.brief_diff(&BriefDiffRequest { since: 9 })
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
}

#[test]
fn brief_replies_stay_under_their_caps() {
    let changes = (0..200)
        .map(|i| BriefChange {
            field: format!("field-{i:03}"),
            from: Some(json!("x".repeat(40))),
            to: None,
            source: None,
        })
        .collect();
    let big = BriefDiff {
        since: 0,
        rev: 1,
        changes,
        omitted: 0,
    };
    let out = trimmed("brief diff", caps::BRIEF, big).unwrap();
    assert!(serde_json::to_vec(&out).unwrap().len() <= caps::BRIEF);
    assert_eq!(out.changes.len() + out.omitted, 200);

    // The largest proposal the server accepts reads back under the get cap.
    let live = Live::start();
    let ops = live.ops();
    let fields: serde_json::Map<String, Value> = (0..40)
        .map(|i| (format!("f{i:02}"), json!("v".repeat(60))))
        .collect();
    ops.brief_propose(&propose(json!({ "fields": fields })))
        .unwrap();
    let got = ops.brief_get().unwrap();
    assert!(serde_json::to_vec(&got).unwrap().len() <= caps::BRIEF);
}
