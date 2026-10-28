use proptest::prelude::*;
use serde_json::{Value, json};

use super::*;
use crate::model::tests::sample;
use crate::patch::content;

fn content_of(doc: &Document) -> Value {
    content(&serde_json::to_value(doc.project()).unwrap())
}

fn edit(doc: &Document, author: Author, ops: Value) -> PatchRequest {
    PatchRequest {
        base_rev: doc.project().rev,
        layer_revs: Default::default(),
        ops: serde_json::from_value(ops).unwrap(),
        author,
        label: "edit".into(),
    }
}

#[test]
fn agent_needs_locks_and_human_is_refused_while_agent_holds() {
    let mut doc = Document::new(sample());
    let move_b = |d: &Document, who| {
        edit(
            d,
            who,
            json!([{"op": "replace", "path": "/tracks/1/items/0/at", "value": 3.0}]),
        )
    };

    let err = doc.patch(&move_b(&doc, Author::Agent), 0).unwrap_err();
    assert_eq!(
        (err.code, err.layer_ids.clone()),
        (ErrorCode::LockNotHeld, vec!["b".to_owned()])
    );

    doc.acquire(Holder::Agent, &["b".to_owned()], 0).unwrap();
    let err = doc.patch(&move_b(&doc, Author::Human), 10).unwrap_err();
    assert_eq!(err.code, ErrorCode::LockHeld);
    assert_eq!(doc.project().rev, 0, "refused patches change nothing");

    doc.patch(&move_b(&doc, Author::Agent), 20).unwrap();
    assert_eq!(doc.project().rev, 1);
    assert_eq!(doc.project().locks.len(), 1, "locks survive a commit");

    // Human undo of an agent-held layer is refused too; after release it works.
    assert_eq!(
        doc.undo(Author::Human, 30).unwrap_err().code,
        ErrorCode::LockHeld
    );
    doc.release(Holder::Agent, None);
    let undo = doc.undo(Author::Human, 40).unwrap();
    assert_eq!((undo.rev, undo.kind), (2, RevisionKind::Undo { of: 1 }));
}

#[test]
fn empty_history_errors() {
    let mut doc = Document::new(sample());
    assert_eq!(
        doc.undo(Author::Human, 0).unwrap_err().code,
        ErrorCode::HistoryEmpty
    );
    assert_eq!(
        doc.redo(Author::Human, 0).unwrap_err().code,
        ErrorCode::HistoryEmpty
    );
}

#[derive(Debug, Clone)]
enum Action {
    SetAt(usize, f64),
    Rename(usize, String),
    Add(f64, f64),
    Remove(usize),
    RenameProject(String),
    Undo,
    Redo,
}

fn action() -> impl Strategy<Value = Action> {
    prop_oneof![
        (any::<usize>(), 0.0f64..100.0).prop_map(|(i, v)| Action::SetAt(i, v)),
        (any::<usize>(), "[a-z]{1,6}").prop_map(|(i, n)| Action::Rename(i, n)),
        (0.0f64..100.0, 0.01f64..10.0).prop_map(|(a, d)| Action::Add(a, d)),
        any::<usize>().prop_map(Action::Remove),
        "[A-Z][a-z]{0,5}".prop_map(Action::RenameProject),
        Just(Action::Undo),
        Just(Action::Redo),
    ]
}

fn to_ops(doc: &Document, a: &Action, next_id: &mut u32) -> Option<Value> {
    let n = doc.project().tracks[1].items.len();
    let path = |i: usize, f: &str| format!("/tracks/1/items/{}{f}", i % n.max(1));
    Some(match a {
        Action::SetAt(i, v) if n > 0 => {
            json!([{"op": "replace", "path": path(*i, "/at"), "value": v}])
        }
        Action::Rename(i, s) if n > 0 => {
            json!([{"op": "replace", "path": path(*i, "/name"), "value": s}])
        }
        Action::Remove(i) if n > 0 => json!([{"op": "remove", "path": path(*i, "")}]),
        Action::Add(at, dur) => {
            *next_id += 1;
            json!([{"op": "add", "path": "/tracks/1/items/-", "value": {
                "id": format!("n{next_id}"), "type": "clip", "at": at, "dur": dur
            }}])
        }
        Action::RenameProject(s) => json!([{"op": "replace", "path": "/name", "value": s}]),
        _ => return None,
    })
}

proptest! {
    /// Against a reference model of linear undo: every state matches, revs only go up,
    /// and refused patches leave the document unchanged.
    #[test]
    fn edits_undo_redo_match_reference(actions in prop::collection::vec(action(), 1..40)) {
        let mut doc = Document::new(sample());
        let (mut past, mut future): (Vec<Value>, Vec<Value>) = (vec![], vec![]);
        let mut current = content_of(&doc);
        let mut next_id = 0;
        let mut now = 0;

        for a in &actions {
            now += 1;
            let rev_before = doc.project().rev;
            let outcome = match a {
                Action::Undo => doc.undo(Author::Human, now).map(|_| ()),
                Action::Redo => doc.redo(Author::Human, now).map(|_| ()),
                _ => match to_ops(&doc, a, &mut next_id) {
                    Some(ops) => doc.patch(&edit(&doc, Author::Human, ops), now).map(|_| ()),
                    None => continue,
                },
            };
            match (a, outcome) {
                (Action::Undo, Ok(())) => { future.push(current.clone()); current = past.pop().unwrap(); }
                (Action::Redo, Ok(())) => { past.push(current.clone()); current = future.pop().unwrap(); }
                (Action::Undo, Err(e)) => { prop_assert!(past.is_empty()); prop_assert_eq!(e.code, ErrorCode::HistoryEmpty); }
                (Action::Redo, Err(e)) => { prop_assert!(future.is_empty()); prop_assert_eq!(e.code, ErrorCode::HistoryEmpty); }
                (_, Ok(())) => { past.push(current.clone()); current = content_of(&doc); future.clear(); }
                (_, Err(_)) => { prop_assert_eq!(doc.project().rev, rev_before); }
            }
            prop_assert_eq!(content_of(&doc), current.clone());
            if doc.project().rev != rev_before {
                prop_assert_eq!(doc.project().rev, rev_before + 1);
                prop_assert_eq!(doc.history().last().unwrap().rev, doc.project().rev);
            }
            doc.project().validate().unwrap();
        }

        // Rebuilding from stored parts gives the same undo behaviour.
        let mut rebuilt = Document::from_parts(doc.project().clone(), doc.history().to_vec());
        let a = doc.undo(Author::Human, now + 1).map(|r| r.kind);
        let b = rebuilt.undo(Author::Human, now + 1).map(|r| r.kind);
        prop_assert_eq!(a, b);
    }
}
