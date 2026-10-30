//! Phase 6 proof (spec 12): after handoff the user comments on a layer; the agent picks it up
//! through `ck wait`, reads the send bundle and the studio context, starts scoped work, is held
//! to the scope (E_SCOPE), fixes the layer and resolves with a before/after list. Through the
//! real `ck` binary against a real `ck studio`.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use ck_server::ops::caps;
use common::{Studio, err, ok};
use serde_json::json;

#[test]
fn a_user_comment_becomes_a_scoped_agent_job() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);

    let file = d.join("apply.json");
    let recipe = json!({ "recipe": "hook-title", "slots": { "headline": "Ship faster" } });
    std::fs::write(&file, recipe.to_string()).unwrap();
    let applied = ok(
        d,
        &["recipe", "apply", file.to_str().unwrap()],
        caps::RECIPE_APPLY,
    );
    let headline = "hook-title-headline";
    assert!(
        applied["layers"].to_string().contains(headline),
        "{applied}"
    );
    let handed = ok(d, &["handoff"], caps::HANDOFF);
    let since = handed["rev"].as_u64().unwrap();

    // The user comments on the headline in the studio (a human comment).
    let comment = json!({
        "baseRev": since, "author": "human",
        "ops": [{ "op": "add", "path": "/comments/-", "value": {
            "id": "c1", "author": "human", "text": "Say it bolder",
            "anchor": { "layerId": headline }, "createdRev": since + 1 } }]
    });
    ok(d, &["project", "patch", &comment.to_string()], caps::PATCH);

    // The agent's wait returns the comment; the bundle carries the layer in short.
    let since_s = since.to_string();
    let waited = ok(
        d,
        &["wait", "--since", &since_s, "--timeout", "0"],
        caps::WAIT,
    );
    assert_eq!(waited["events"][0]["kind"], "comment", "{waited}");
    let bundle = ok(d, &["comments", "list"], caps::COMMENTS);
    assert_eq!(bundle["comments"][0]["id"], "c1");
    assert_eq!(bundle["layers"][0]["id"], headline);
    let context = ok(d, &["context"], caps::CONTEXT);
    assert_eq!(context["openComments"][0]["layerId"], headline);

    // Scoped work: only the anchored layer.
    let replied = ok(
        d,
        &["comments", "reply", "c1", "Making it bolder", "--scope", ""],
        caps::COMMENT_WRITE,
    );
    assert_eq!(replied["status"], "working");
    ok(d, &["lock", headline, "hook-title-backdrop"], caps::LOCK);
    let project = ok(d, &["status"], caps::STATUS);
    let rev = project["rev"].as_u64().unwrap();
    let outside = json!({
        "baseRev": rev, "author": "agent",
        "ops": [{ "op": "replace", "path": "/tracks/0/items/0/name", "value": "Moved" }]
    });
    let refused = err(d, &["project", "patch", &outside.to_string()]);
    assert_eq!(refused["code"], "E_SCOPE", "{refused}");

    let summary = ok(d, &["project", "get", "--summary"], caps::SUMMARY);
    let track = summary["tracks"]
        .as_array()
        .unwrap()
        .iter()
        .position(|t| t["items"].to_string().contains(headline))
        .unwrap();
    let inside = json!({
        "baseRev": rev, "author": "agent",
        "ops": [{ "op": "replace", "path": format!("/tracks/{track}/items/0/name"),
                  "value": "Bold headline" }]
    });
    ok(d, &["project", "patch", &inside.to_string()], caps::PATCH);

    let resolved = ok(
        d,
        &["comments", "resolve", "c1", "--text", "Bolder now"],
        caps::COMMENT_WRITE,
    );
    assert_eq!(resolved["status"], "resolved");
    assert_eq!(resolved["changed"], json!([headline]));
    ok(d, &["unlock"], caps::LOCK);
    let open = ok(d, &["comments", "list"], caps::COMMENTS);
    assert_eq!(open["comments"], json!([]));

    // Debuggability (Phase 9): history replays to the same project; a trace bundle is written
    // and keeps the session token out.
    let replay = ok(d, &["replay"], ck_server::debug::caps::REPLAY);
    assert_eq!(replay["reproduces"], true, "{replay}");
    let trace = ok(d, &["trace", "bundle"], ck_server::debug::caps::TRACE);
    let text = std::fs::read_to_string(d.join(trace["path"].as_str().unwrap())).unwrap();
    let token: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(d.join(".clipkit/server.json")).unwrap())
            .unwrap();
    assert!(!text.contains(token["token"].as_str().unwrap()));
    assert!(text.contains("Bolder now"));
}
