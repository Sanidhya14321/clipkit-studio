//! Stabilization sprint: the wrong-result bugs, through the real binary.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use std::path::Path;
use std::process::Command;

use common::Studio;
use serde_json::{Value, json};

fn raw(d: &Path, args: &[&str]) -> Value {
    let out = Command::new(env!("CARGO_BIN_EXE_ck"))
        .args(args)
        .current_dir(d)
        .output()
        .unwrap();
    serde_json::from_slice(&[out.stdout, out.stderr].concat()).unwrap()
}

fn input(d: &Path, name: &str, v: &Value) -> String {
    let p = d.join(name);
    std::fs::write(&p, v.to_string()).unwrap();
    p.to_str().unwrap().to_owned()
}

fn settings(d: &Path) -> (u64, u64) {
    let p: Value =
        serde_json::from_str(&std::fs::read_to_string(d.join(".clipkit/project.json")).unwrap())
            .unwrap();
    (
        p["settings"]["width"].as_u64().unwrap(),
        p["settings"]["height"].as_u64().unwrap(),
    )
}

fn scene(d: &Path, name: &str, at: f64, dur: f64) -> String {
    input(
        d,
        name,
        &json!({ "recipe": "hook-title", "at": at, "duration": dur, "slots": { "headline": "Hi" } }),
    )
}

#[test]
fn brief_sets_the_canvas_and_edits_follow() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    let b = input(
        d,
        "b.json",
        &json!({ "fields": { "aspect": "9:16" }, "questions": [] }),
    );
    raw(d, &["brief", "propose", &b]);
    assert_eq!(settings(d), (1080, 1920));
    let rev = raw(d, &["status"])["rev"].clone();
    let edit = json!({ "baseRev": rev, "author": "human", "ops": [
        { "op": "replace", "path": "/brief/fields/aspect/value", "value": "1:1" }] });
    raw(d, &["project", "patch", &edit.to_string()]);
    assert_eq!(settings(d), (1080, 1080));
}

#[test]
fn verify_fails_when_size_and_brief_disagree() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    let b = input(
        d,
        "b.json",
        &json!({ "fields": { "aspect": "9:16" }, "questions": [] }),
    );
    raw(d, &["brief", "propose", &b]);
    let rev = raw(d, &["status"])["rev"].clone();
    let edit = json!({ "baseRev": rev, "author": "human", "ops": [
        { "op": "replace", "path": "/settings/width", "value": 1920 },
        { "op": "replace", "path": "/settings/height", "value": 1080 }] });
    raw(d, &["project", "patch", &edit.to_string()]);
    let v = raw(d, &["verify"]);
    assert!(
        v["failed"].as_array().unwrap().contains(&json!("canvas")),
        "{v}"
    );
}

#[test]
fn export_refuses_failing_gates_and_a_reframing_preset() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    let b = input(
        d,
        "b.json",
        &json!({ "fields": { "aspect": "9:16" }, "questions": [] }),
    );
    raw(d, &["brief", "propose", &b]);
    raw(d, &["recipe", "apply", &scene(d, "a.json", 0.0, 3.0)]);
    raw(d, &["recipe", "apply", &scene(d, "c.json", 8.0, 3.0)]);

    let refused = raw(d, &["export"]);
    assert_eq!(refused["code"], "E_GATES", "{refused}");
    assert!(
        refused["message"].as_str().unwrap().contains("gaps"),
        "{refused}"
    );
    let bare = raw(d, &["export", "--force"]);
    assert_eq!(bare["code"], "E_REQUEST", "force needs a reason: {bare}");

    let reframed = raw(
        d,
        &["export", "--preset", "youtube", "--force", "--reason", "x"],
    );
    assert_eq!(reframed["code"], "E_REQUEST", "{reframed}");
    assert!(
        reframed["message"].as_str().unwrap().contains("reframe"),
        "{reframed}"
    );
}

#[test]
fn forced_export_records_the_reason_and_silence_fails_loudness() {
    let worker =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../render-worker/bundle/index.html");
    if !worker.is_file() {
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    raw(d, &["recipe", "apply", &scene(d, "a.json", 0.0, 2.0)]);
    raw(d, &["recipe", "apply", &scene(d, "c.json", 5.0, 2.0)]);
    let out = raw(
        d,
        &["export", "--force", "--reason", "rough cut for review"],
    );
    assert_eq!(out["forcedReason"], "rough cut for review", "{out}");
    assert!(
        out["forcedGates"]
            .as_array()
            .unwrap()
            .contains(&json!("gaps")),
        "{out}"
    );
    assert_eq!(out["silent"], true, "{out}");
    assert_eq!(out["loudnessPass"], false, "silence must fail: {out}");
    let path = d.join(out["path"].as_str().unwrap());
    let eval = raw(d, &["eval", path.to_str().unwrap()]);
    assert!(eval["score"].as_f64().unwrap() < 100.0, "{eval}");
}

#[test]
fn brief_rejects_unknown_fields_and_get_shows_the_schema() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    let b = input(
        d,
        "b.json",
        &json!({ "fields": { "ratio": "9:16" }, "questions": [] }),
    );
    let out = raw(d, &["brief", "propose", &b]);
    assert_eq!(out["code"], "E_REQUEST", "{out}");
    assert!(out["message"].as_str().unwrap().contains("aspect"), "{out}");
    let got = raw(d, &["brief", "get"]);
    let look = got["schema"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["name"] == "look");
    assert_eq!(look.unwrap()["default"], "clean-product", "{got}");
}

#[test]
fn export_cannot_write_over_project_media() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    raw(d, &["recipe", "apply", &scene(d, "a.json", 0.0, 3.0)]);
    let out = raw(d, &["export", "--out", "footage.mp4"]);
    assert_eq!(out["code"], "E_REQUEST", "{out}");
    assert!(
        out["message"].as_str().unwrap().contains("exports/"),
        "{out}"
    );
}

#[test]
fn voice_before_the_brief_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start_with(d, &[]);
    raw(d, &["pipeline", "detect"]);
    let v = input(d, "v.json", &json!({ "lines": [{ "text": "Hello" }] }));
    assert_eq!(raw(d, &["voice", "script", &v])["code"], "E_STAGE");
}

#[test]
fn json_inputs_share_one_convention_and_recipes_ripple_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    let inline =
        json!({ "recipe": "hook-title", "at": 0.0, "dur": 3.0, "slots": { "headline": "A" } });
    assert!(raw(d, &["recipe", "apply", &inline.to_string()])["rev"].is_u64());
    let f = scene(d, "b.json", 3.0, 3.0);
    assert!(raw(d, &["recipe", "apply", &format!("@{f}")])["rev"].is_u64());

    let insert = json!({ "recipe": "hook-title", "at": 3.0, "dur": 2.0, "place": "insert",
                         "slots": { "headline": "C" } });
    assert!(
        raw(d, &["recipe", "apply", &insert.to_string()])["rev"].is_u64(),
        "insert"
    );
    let p: Value =
        serde_json::from_str(&std::fs::read_to_string(d.join(".clipkit/project.json")).unwrap())
            .unwrap();
    for track in p["tracks"].as_array().unwrap() {
        let ats: Vec<f64> = track["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["at"].as_f64().unwrap())
            .collect();
        assert!(ats.windows(2).all(|w| w[0] <= w[1]), "{ats:?}");
        assert!(
            ats.iter().any(|a| *a >= 5.0),
            "the later scene moved by 2: {ats:?}"
        );
    }
    let summary = raw(d, &["project", "get"]).to_string();
    assert!(
        summary.contains("\"dur\"") && !summary.contains("\"end\""),
        "{summary}"
    );
}

#[test]
fn an_agent_token_cannot_act_as_the_user() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _s = Studio::start(d);
    let edit = json!({ "baseRev": 0, "author": "human", "ops": [
        { "op": "replace", "path": "/brief/status", "value": "confirmed" }] });
    let out = Command::new(env!("CARGO_BIN_EXE_ck"))
        .args(["project", "patch", &edit.to_string()])
        .current_dir(d)
        .env_remove("CK_STUDIO_TOKEN")
        .output()
        .unwrap();
    let v: Value = serde_json::from_slice(&[out.stdout, out.stderr].concat()).unwrap();
    assert_eq!(v["code"], "E_AUTH", "{v}");
    let info = std::fs::read_to_string(d.join(".clipkit/server.json")).unwrap();
    assert!(!info.contains("test-studio-token"));
}
