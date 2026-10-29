//! The ck commands end to end: a real `ck studio` in a temp folder, driven by other ck processes.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use ck_server::ops::caps;
use common::{Background, Studio, add_track, ck, err, ok};
use serde_json::{Value, json};

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
fn every_command_starts_a_missing_studio() {
    for (args, cap) in [
        (&["status"][..], caps::STATUS),
        (&["project", "get"], caps::SUMMARY),
        (
            &["project", "patch", &add_track(0, "human", "a")],
            caps::PATCH,
        ),
        (&["lock", "a"], caps::LOCK),
        (&["unlock"], caps::LOCK),
        (&["heartbeat"], caps::LOCK),
        (&["brief", "get"], caps::BRIEF),
    ] {
        let dir = tempfile::tempdir().unwrap();
        let d = dir.path();
        let bg = Background::new(d);
        ok(d, args, cap);
        assert!(bg.pid().is_some(), "{args:?} left a running studio");
        assert_eq!(
            ok(d, &["status"], caps::STATUS)["projectId"],
            "project",
            "{args:?}"
        );
    }
}

#[test]
fn a_studio_that_cannot_start_fails_the_command_with_its_own_error() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _bg = Background::new(d);
    std::fs::create_dir_all(d.join(".clipkit")).unwrap();
    std::fs::write(d.join(".clipkit/project.json"), "{not json").unwrap();

    let e = err(d, &["status"]);
    let code = e["code"].as_str().unwrap();
    assert!(code.starts_with("E_"), "{e}");
    assert!(e["hint"].as_str().is_some_and(|h| !h.is_empty()), "{e}");
    assert!(!d.join(".clipkit/server.json").exists());
}

#[test]
fn studio_stops_by_itself_after_the_idle_time() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let t0 = Instant::now();
    let mut studio = Studio::start_with(d, &["--idle", "1"]);
    let server_json = d.join(".clipkit/server.json");
    assert!(server_json.exists());

    // A call half-way through restarts the count.
    std::thread::sleep(Duration::from_millis(600));
    ok(d, &["status"], caps::STATUS);
    let called = t0.elapsed();

    let status = studio.exit_within(Duration::from_secs(10));
    assert!(status.is_some_and(|s| s.success()), "{status:?}");
    assert!(
        t0.elapsed() >= called + Duration::from_secs(1),
        "stopped {:?} after start, call at {called:?}",
        t0.elapsed()
    );
    assert!(!server_json.exists(), "server.json is removed on idle stop");

    // The next call starts a fresh studio in the background.
    let bg = Background::new(d);
    assert_eq!(ok(d, &["status"], caps::STATUS)["rev"], 0);
    assert!(bg.pid().is_some_and(|pid| pid != studio.pid()));
}

#[test]
fn bad_arguments_are_usage_errors() {
    let dir = tempfile::tempdir().unwrap();
    let out = ck(dir.path(), &["lock"]);
    assert_eq!(out.status.code(), Some(2), "lock needs at least one id");
    let out = ck(dir.path(), &["lock", "a", "--holder", "robot"]);
    assert_eq!(out.status.code(), Some(2));
}
