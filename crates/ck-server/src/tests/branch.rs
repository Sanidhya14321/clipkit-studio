//! History, branches and variants through the session.

use ck_core::ErrorCode;
use ck_core::model::Author;
use ck_core::patch::PatchRequest;
use serde_json::json;

use super::fixture::Fixture;
use crate::session::branch::{BranchRequest, HistoryRequest, VariantsRequest};

fn rename(f: &Fixture, name: &str) {
    let rev = f.session.project().unwrap().rev;
    f.session
        .patch(&PatchRequest {
            base_rev: rev,
            layer_revs: Default::default(),
            ops: serde_json::from_value(
                json!([{ "op": "replace", "path": "/tracks/0/items/0/name", "value": name }]),
            )
            .unwrap(),
            author: Author::Human,
            label: format!("rename to {name}"),
        })
        .unwrap();
}

fn title(f: &Fixture) -> String {
    f.session
        .project()
        .unwrap()
        .item("title")
        .unwrap()
        .name
        .clone()
}

#[test]
fn history_lists_newest_first() {
    let f = Fixture::new();
    rename(&f, "A");
    rename(&f, "B");
    let h = f
        .session
        .history(&HistoryRequest { limit: Some(1) })
        .unwrap();
    assert_eq!(h.lines, ["2 human rename to B (1 layers)"]);
}

#[test]
fn a_branch_switch_is_one_undoable_revision() {
    let f = Fixture::new();
    rename(&f, "Calm");
    let br = |n: &str| BranchRequest {
        name: n.into(),
        author: None,
    };
    f.session.branch_save(&br("calm")).unwrap();
    rename(&f, "Punchy");
    let switched = f.session.branch_switch(&br("calm")).unwrap();
    assert_eq!(title(&f), "Calm");
    assert_eq!(switched.touched, ["title"]);
    // Switching to where you are is a no-op.
    let again = f.session.branch_switch(&br("calm")).unwrap();
    assert_eq!(again.rev, switched.rev);
    // Undo goes back to Punchy.
    f.session
        .undo(&crate::session::history::HistoryRequest::default())
        .unwrap();
    assert_eq!(title(&f), "Punchy");
    assert_eq!(
        f.session.branch_list().unwrap().branches,
        ["calm (saved at rev 1)"]
    );

    assert_eq!(
        f.session.branch_switch(&br("nope")).unwrap_err().code,
        ErrorCode::NotFound
    );
    assert_eq!(
        f.session.branch_save(&br("Bad Name")).unwrap_err().code,
        ErrorCode::Request
    );
}

#[test]
fn variants_compare_two_or_three_known_looks() {
    let f = Fixture::new();
    let req = |looks: &[&str]| VariantsRequest {
        looks: looks.iter().map(|l| (*l).to_owned()).collect(),
        frame: None,
    };
    assert_eq!(
        f.session.variants(&req(&["cinematic"])).unwrap_err().code,
        ErrorCode::Request
    );
    assert_eq!(
        f.session
            .variants(&req(&["cinematic", "nope"]))
            .unwrap_err()
            .code,
        ErrorCode::NotFound
    );
}
