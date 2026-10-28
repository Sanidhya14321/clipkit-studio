//! The ck commands end to end: a real `ck studio` in a temp folder, driven by other ck processes.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};

use ck_server::ops::caps;
use serde_json::{Value, json};

/// A running `ck studio`; killed on drop.
struct Studio {
    child: Child,
    started: Value,
}

impl Studio {
    fn start(dir: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ck"))
            .args(["studio", "--dir"])
            .arg(dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let mut line = String::new();
        BufReader::new(child.stdout.take().unwrap())
            .read_line(&mut line)
            .unwrap();
        let started = serde_json::from_str(&line).unwrap();
        Self { child, started }
    }
}

impl Drop for Studio {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn ck(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ck"))
        .args(args)
        .arg("--dir")
        .arg(dir)
        .output()
        .unwrap()
}

/// Run a command that must succeed; check its one-line output against `cap` and parse it.
fn ok(dir: &Path, args: &[&str], cap: usize) -> Value {
    let out = ck(dir, args);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "ck {args:?} failed: {stderr}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1, "one line of JSON");
    assert!(
        stdout.len() <= cap + 1,
        "ck {args:?} over its {cap} byte cap"
    );
    serde_json::from_str(&stdout).unwrap()
}

/// Run a command that must fail; return the error JSON from stderr.
fn err(dir: &Path, args: &[&str]) -> Value {
    let out = ck(dir, args);
    assert_eq!(out.status.code(), Some(1), "ck {args:?} should fail");
    assert!(out.stdout.is_empty());
    serde_json::from_slice(&out.stderr).unwrap()
}

/// A patch adding track `t-<name>` holding one clip `<name>`.
fn add_track(base_rev: u64, author: &str, name: &str) -> String {
    json!({
        "baseRev": base_rev,
        "author": author,
        "label": "add track",
        "ops": [{ "op": "add", "path": "/tracks/-", "value": {
            "id": format!("t-{name}"), "kind": "overlay",
            "items": [{ "id": name, "name": name, "type": "clip", "at": 0.0, "dur": 1.0 }]
        }}]
    })
    .to_string()
}

#[test]
fn studio_creates_a_project_and_serves_every_command() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let studio = Studio::start(d);
    let port = studio.started["port"].as_u64().unwrap();
    assert_eq!(
        studio.started["url"],
        format!("http://127.0.0.1:{port}/"),
        "the start line never carries the token"
    );
    assert!(studio.started.get("token").is_none());

    let status = ok(d, &["status"], caps::STATUS);
    assert_eq!(
        (status["projectId"].as_str(), status["rev"].as_u64()),
        (Some("project"), Some(0))
    );

    let summary = ok(d, &["project", "get"], caps::SUMMARY);
    assert_eq!(
        summary,
        ok(d, &["project", "get", "--summary"], caps::SUMMARY)
    );
    assert_eq!(summary["tracks"], json!([]));

    let locked = ok(d, &["lock", "intro", "t-intro"], caps::LOCK);
    assert_eq!(locked["locks"].as_array().unwrap().len(), 2);

    let patched = ok(
        d,
        &["project", "patch", &add_track(0, "agent", "intro")],
        caps::PATCH,
    );
    assert_eq!(patched["rev"], 1);
    let layer = ok(d, &["project", "get", "--layer", "intro"], caps::LAYER);
    assert_eq!(layer["name"], "intro");

    // While the agent holds the track, a human edit is refused with the lock error.
    let human = json!({
        "baseRev": 1, "author": "human",
        "ops": [{ "op": "replace", "path": "/tracks/0/items/0/name", "value": "mine" }]
    })
    .to_string();
    let refused = err(d, &["project", "patch", &human]);
    assert_eq!(refused["code"], "E_LOCK_HELD");
    assert!(refused["hint"].as_str().is_some_and(|h| !h.is_empty()));

    let stale = err(d, &["project", "patch", &add_track(0, "agent", "late")]);
    assert_eq!(stale["code"], "E_PATCH_STALE");

    let unlocked = ok(d, &["unlock"], caps::LOCK);
    assert_eq!(unlocked["locks"], json!([]));
    assert_eq!(ok(d, &["status"], caps::STATUS)["locks"], json!([]));

    // A second studio for the same project is refused.
    let twice = err(d, &["studio"]);
    assert_eq!(twice["code"], "E_SERVER");
    drop(studio);
}

#[test]
fn patch_reads_stdin_and_rejects_bad_json() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);

    let mut child = Command::new(env!("CARGO_BIN_EXE_ck"))
        .args(["project", "patch", "-", "--dir"])
        .arg(d)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    {
        use std::io::Write;
        let mut stdin = child.stdin.take().unwrap();
        stdin
            .write_all(add_track(0, "human", "a").as_bytes())
            .unwrap();
    }
    let out = child.wait_with_output().unwrap();
    assert!(out.status.success());
    let patched: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(patched["rev"], 1);

    assert_eq!(
        err(d, &["project", "patch", "{not json"])["code"],
        "E_REQUEST"
    );
    assert_eq!(
        err(d, &["project", "get", "--layer", "nope"])["code"],
        "E_NOT_FOUND"
    );
}

#[test]
fn commands_without_a_studio_fail_with_e_server() {
    let dir = tempfile::tempdir().unwrap();
    for args in [
        &["status"][..],
        &["project", "get"],
        &["lock", "a"],
        &["unlock"],
    ] {
        let e = err(dir.path(), args);
        assert_eq!(e["code"], "E_SERVER", "{args:?}");
        assert!(e["hint"].as_str().is_some_and(|h| h.contains("ck studio")));
    }
}

#[test]
fn bad_arguments_are_usage_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = ck(dir.path(), &["lock"]);
    assert_eq!(out.status.code(), Some(2), "lock needs at least one id");
    let out = ck(dir.path(), &["lock", "a", "--holder", "robot"]);
    assert_eq!(out.status.code(), Some(2));
}
