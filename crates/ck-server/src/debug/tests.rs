use super::*;
use crate::tests::fixture::Fixture;
use ck_core::model::Author;
use ck_core::patch::PatchRequest;

fn patch(f: &Fixture, ops: Value, label: &str) {
    let rev = f.session.project().unwrap().rev;
    f.session
        .patch(&PatchRequest {
            base_rev: rev,
            layer_revs: Default::default(),
            ops: serde_json::from_value(ops).unwrap(),
            author: Author::Human,
            label: label.into(),
        })
        .unwrap();
}

/// Revisions 1 to 3 rename the title; revision 4 adds a clip with placeholder text; 5 renames.
fn history() -> Fixture {
    let f = Fixture::new();
    for n in 1..=3 {
        patch(
            &f,
            json!([{ "op": "replace", "path": "/tracks/0/items/0/name", "value": format!("T{n}") }]),
            "rename",
        );
    }
    patch(
        &f,
        json!([{ "op": "add", "path": "/tracks/0/items/-", "value": {
            "id": "dup", "name": "Dup", "type": "clip", "at": 2.0, "dur": 1.0,
            "props": { "text": "Lorem ipsum dolor" } } }]),
        "add placeholder clip",
    );
    patch(
        &f,
        json!([{ "op": "replace", "path": "/tracks/0/items/0/name", "value": "T5" }]),
        "rename",
    );
    f
}

#[test]
fn replay_reproduces_and_rebuilds_an_earlier_revision() {
    let f = history();
    let r = replay(f.dir.path(), &ReplayRequest { to: Some(3) }).unwrap();
    assert!(r.reproduces, "{r:?}");
    assert_eq!(r.rev, 5);
    assert_eq!(r.at.unwrap().layers, 1);
}

#[test]
fn bisect_finds_the_revision_that_broke_the_gates() {
    let f = history();
    let r = bisect(
        f.dir.path(),
        &BisectRequest {
            gate: Some("placeholders".into()),
        },
    )
    .unwrap();
    assert_eq!(r.first_failing, Some(4), "{r:?}");
    assert_eq!(r.label.as_deref(), Some("add placeholder clip"));
    assert_eq!(r.gates, ["placeholders"]);
    assert!(r.checked.len() <= 5, "binary search: {:?}", r.checked);
}

#[test]
fn the_trace_bundle_is_redacted() {
    let f = history();
    std::fs::write(
        f.dir.path().join("server.json"),
        r#"{"token":"secret-123"}"#,
    )
    .unwrap();
    let r = trace_bundle(f.dir.path(), &TraceRequest {}).unwrap();
    let text = std::fs::read_to_string(f.dir.path().join("traces/trace-5.json")).unwrap();
    assert!(!text.contains("secret-123"));
    assert!(text.contains("\"kind\": \"clipkit-trace\""));
    assert!(text.contains("add placeholder clip"));
    assert_eq!(r.path, ".clipkit/traces/trace-5.json");

    let mut v =
        json!({ "apiKey": "k", "url": "http://x/media/a.mp4?token=abc", "p": "/home/me/x" });
    let mut n = 0;
    redact(&mut v, Some("/home/me"), &mut n);
    assert_eq!(
        v,
        json!({ "apiKey": "[redacted]", "url": "http://x/media/a.mp4?token=[redacted]", "p": "~/x" })
    );
    assert_eq!(n, 3);
}
