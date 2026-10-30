//! The whole pipeline is one gated path through the real `ck` binary: agents cannot build before
//! the brief, hand off with failing gates, or loop on a failing call; the stage follows the work.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use std::path::Path;
use std::process::Command;

use ck_server::ops::caps;
use common::{Studio, ok};
use serde_json::{Value, json};

fn raw(d: &Path, args: &[&str]) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_ck"))
        .args(args)
        .current_dir(d)
        .output()
        .unwrap();
    let text = [out.stdout, out.stderr].concat();
    serde_json::from_slice(&text).unwrap()
}

fn stage(d: &Path) -> String {
    raw(d, &["status"])["pipeline"]["stage"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn file(d: &Path, name: &str, v: &Value) -> String {
    let p = d.join(name);
    std::fs::write(&p, v.to_string()).unwrap();
    p.to_str().unwrap().to_owned()
}

#[test]
fn the_pipeline_is_enforced_end_to_end() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start_with(d, &[]);
    let hook = file(
        d,
        "a.json",
        &json!({ "recipe": "hook-title", "at": 0.0, "duration": 4.0, "slots": { "headline": "Hello" } }),
    );

    // Refusal 1: the brief before detect.
    let early = file(d, "p.json", &json!({ "fields": {}, "questions": [] }));
    assert_eq!(raw(d, &["brief", "propose", &early])["code"], "E_STAGE");
    assert_eq!(stage(d), "boot");
    assert_eq!(raw(d, &["pipeline", "detect"])["stage"], "detect");

    // Refusal 2: no brief, no build.
    let refused = raw(d, &["recipe", "apply", &hook]);
    assert_eq!(refused["code"], "E_STAGE", "{refused}");

    let brief = file(
        d,
        "b.json",
        &json!({ "fields": { "platform": "youtube" }, "questions": [] }),
    );
    let rev = raw(d, &["brief", "propose", &brief])["rev"]
        .as_u64()
        .unwrap();
    assert_eq!(stage(d), "brief");
    let confirm = json!({ "baseRev": rev, "author": "human",
        "ops": [{ "op": "replace", "path": "/brief/status", "value": "confirmed" }] });
    ok(d, &["project", "patch", &confirm.to_string()], caps::PATCH);
    assert_eq!(stage(d), "plan");

    ok(d, &["recipe", "apply", &hook], caps::RECIPE_APPLY);
    assert_eq!(stage(d), "passes");

    // A gap fails the gaps gate: handoff is refused until an override says why.
    let outro = file(
        d,
        "c.json",
        &json!({ "recipe": "outro-cta", "at": 10.0, "duration": 3.0, "slots": { "action": "Go" } }),
    );
    ok(d, &["recipe", "apply", &outro], caps::RECIPE_APPLY);
    // Refusal 3: handoff mid-pass.
    assert_eq!(raw(d, &["handoff"])["code"], "E_STAGE");
    assert_eq!(stage(d), "passes");
    let mut passes = vec![raw(d, &["status"])["pipeline"]["pass"].clone()];
    loop {
        let step = raw(d, &["pipeline", "next"]);
        if step["stage"] == "verify" {
            break;
        }
        passes.push(step["pass"].clone());
    }
    assert_eq!(
        passes,
        [
            "structure",
            "audio",
            "text",
            "graphicsMotion",
            "transitions",
            "look",
            "polish"
        ]
        .map(|p| json!(p))
    );
    // Refusal 4: failing gates in verify.
    assert_eq!(raw(d, &["handoff"])["code"], "E_STAGE");
    // Refusal 5: verify is left by handoff, not by next.
    assert_eq!(raw(d, &["pipeline", "next"])["code"], "E_STAGE");
    let done = raw(d, &["handoff", "--override-reason", "gap is deliberate"]);
    assert!(done["rev"].is_u64(), "{done}");
    assert_eq!(stage(d), "handoff");
    assert!(
        raw(d, &["project", "get"])
            .to_string()
            .contains("gap is deliberate")
            || raw(d, &["status"])
                .to_string()
                .contains("gap is deliberate")
    );

    // Later agent work is a job in ITERATE.
    let gap = file(
        d,
        "d.json",
        &json!({ "recipe": "stat-callout", "at": 4.0, "duration": 5.0,
                 "slots": { "value": 1, "label": "gap filled" } }),
    );
    ok(d, &["recipe", "apply", &gap], caps::RECIPE_APPLY);
    assert_eq!(stage(d), "iterate");

    // Same failing call: the third is refused.
    let bad = file(
        d,
        "e.json",
        &json!({ "recipe": "no-such-recipe", "at": 0.0, "slots": {} }),
    );
    for _ in 0..2 {
        assert_ne!(raw(d, &["recipe", "apply", &bad])["code"], "E_RETRY");
    }
    assert_eq!(raw(d, &["recipe", "apply", &bad])["code"], "E_RETRY");
}
