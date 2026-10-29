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
fn agent_cannot_edit_project_or_track_content_without_a_lock() {
    use crate::patch::PROJECT_LAYER;
    let mut doc = Document::new(sample());
    let fps = |d: &Document, who| {
        edit(
            d,
            who,
            json!([{"op": "replace", "path": "/settings/fps", "value": 25}]),
        )
    };
    let err = doc.patch(&fps(&doc, Author::Agent), 0).unwrap_err();
    assert_eq!(
        (err.code, err.layer_ids.clone()),
        (ErrorCode::LockNotHeld, vec![PROJECT_LAYER.to_owned()])
    );
    let reorder = edit(
        &doc,
        Author::Agent,
        json!([{"op": "move", "from": "/tracks/1/items/1", "path": "/tracks/1/items/0"}]),
    );
    assert_eq!(
        doc.patch(&reorder, 0).unwrap_err().layer_ids,
        vec!["t2".to_owned()]
    );

    doc.acquire(Holder::Agent, &[PROJECT_LAYER.to_owned()], 0)
        .unwrap();
    assert_eq!(
        doc.patch(&fps(&doc, Author::Human), 1).unwrap_err().code,
        ErrorCode::LockHeld
    );
    doc.patch(&fps(&doc, Author::Agent), 2).unwrap();
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

#[test]
fn content_at_rebuilds_earlier_revisions() {
    let mut doc = Document::new(sample());
    let start = content_of(&doc);
    doc.acquire(Holder::Agent, &["b".to_owned()], 0).unwrap();
    for at in [3.0, 4.0] {
        let req = edit(
            &doc,
            Author::Agent,
            json!([{"op": "replace", "path": "/tracks/1/items/0/at", "value": at}]),
        );
        doc.patch(&req, 0).unwrap();
    }
    assert_eq!(doc.content_at(0).unwrap(), start);
    assert_eq!(
        doc.content_at(1).unwrap()["tracks"][1]["items"][0]["at"],
        3.0
    );
    assert_eq!(doc.content_at(2).unwrap(), content_of(&doc));
    assert_eq!(doc.content_at(3).unwrap_err().code, ErrorCode::NotFound);
    let partial = Document::from_parts(doc.project().clone(), doc.history()[1..].to_vec());
    assert_eq!(partial.content_at(0).unwrap_err().code, ErrorCode::NotFound);
}

#[test]
fn brief_questions_are_capped_and_need_affects() {
    use crate::model::BriefQuestion;
    let q = |id: &str, affects: &[&str]| BriefQuestion {
        id: id.into(),
        text: "Show pricing?".into(),
        affects: affects.iter().map(|s| (*s).to_owned()).collect(),
        answer: None,
    };
    let mut p = sample();
    p.brief.questions = vec![q("a", &["cta"]), q("b", &["cta"]), q("c", &["cta"])];
    p.validate().unwrap();
    p.brief.questions.push(q("d", &["cta"]));
    assert_eq!(p.validate().unwrap_err().code, ErrorCode::Schema);
    p.brief.questions = vec![q("a", &[])];
    assert_eq!(p.validate().unwrap_err().code, ErrorCode::Schema);
}

fn comment(id: &str, layer: Option<&str>, status: &str, scope: &[&str]) -> Value {
    json!({ "id": id, "author": "human", "anchor": { "layerId": layer }, "text": "fix",
            "status": status, "createdRev": 0, "scope": scope })
}

#[test]
fn comments_have_their_own_lock_id_so_the_user_can_comment_while_the_agent_works() {
    let mut doc = Document::new(sample());
    doc.acquire(Holder::Agent, &[crate::patch::PROJECT_LAYER.to_owned()], 0)
        .unwrap();
    let add = edit(
        &doc,
        Author::Human,
        json!([{ "op": "add", "path": "/comments/-", "value": comment("c1", Some("b"), "open", &[]) }]),
    );
    let rev = doc.patch(&add, 1).unwrap();
    assert_eq!(rev.touched, vec![crate::patch::COMMENTS_LAYER.to_owned()]);
}

#[test]
fn structural_changes_take_the_global_lock_and_freeze_human_edits() {
    let mut doc = Document::new(sample());
    let swap = json!([{ "op": "move", "from": "/tracks/1", "path": "/tracks/0" }]);
    let err = doc
        .patch(&edit(&doc, Author::Agent, swap.clone()), 0)
        .unwrap_err();
    assert!(
        err.layer_ids
            .contains(&crate::patch::STRUCTURE_LAYER.to_owned())
    );

    let mode = json!([{ "op": "replace", "path": "/mode", "value": "generate" }]);
    let touched = crate::patch::apply(doc.project(), &edit(&doc, Author::Agent, mode))
        .unwrap()
        .touched;
    assert!(touched.contains(&crate::patch::STRUCTURE_LAYER.to_owned()));

    // Appending a track is not structural.
    let append = json!([{ "op": "add", "path": "/tracks/-",
                          "value": { "id": "t3", "kind": "audio", "items": [] } }]);
    let touched = crate::patch::apply(doc.project(), &edit(&doc, Author::Agent, append))
        .unwrap()
        .touched;
    assert!(!touched.contains(&crate::patch::STRUCTURE_LAYER.to_owned()));

    // While the agent holds the structure lock, even an unrelated human edit is refused.
    doc.acquire(
        Holder::Agent,
        &[crate::patch::STRUCTURE_LAYER.to_owned()],
        0,
    )
    .unwrap();
    let rename = json!([{ "op": "replace", "path": "/tracks/1/items/0/name", "value": "x" }]);
    let err = doc
        .patch(&edit(&doc, Author::Human, rename), 1)
        .unwrap_err();
    assert_eq!(err.code, ErrorCode::LockHeld);
}

#[test]
fn the_scope_gate_keeps_the_agent_inside_a_working_comment() {
    let mut p = sample();
    p.comments =
        serde_json::from_value(json!([comment("c1", Some("b"), "working", &["c"])])).unwrap();
    let mut doc = Document::new(p);
    let all: Vec<String> = ["a", "b", "c", crate::patch::COMMENTS_LAYER]
        .iter()
        .map(|s| (*s).to_owned())
        .collect();
    doc.acquire(Holder::Agent, &all, 0).unwrap();

    // The anchored layer and the declared dependent are fine.
    let inside = json!([
        { "op": "replace", "path": "/tracks/1/items/0/name", "value": "b2" },
        { "op": "replace", "path": "/tracks/1/items/1/name", "value": "c2" }
    ]);
    doc.patch(&edit(&doc, Author::Agent, inside), 1).unwrap();

    // Layer `a` is outside the scope.
    let outside = json!([{ "op": "replace", "path": "/tracks/0/items/0/name", "value": "a2" }]);
    let err = doc
        .patch(&edit(&doc, Author::Agent, outside.clone()), 2)
        .unwrap_err();
    assert_eq!(
        (err.code, err.layer_ids),
        (ErrorCode::Scope, vec!["a".to_owned()])
    );

    // Humans are not gated; once the comment resolves, the agent is not either.
    doc.release(Holder::Agent, None);
    doc.patch(&edit(&doc, Author::Human, outside), 3).unwrap();
    let resolve = json!([{ "op": "replace", "path": "/comments/0/status", "value": "resolved" }]);
    doc.acquire(Holder::Agent, &all, 4).unwrap();
    doc.patch(&edit(&doc, Author::Agent, resolve), 4).unwrap();
    let later = json!([{ "op": "replace", "path": "/tracks/0/items/0/name", "value": "a3" }]);
    doc.patch(&edit(&doc, Author::Agent, later), 5).unwrap();
}
