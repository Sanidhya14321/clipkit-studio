//! `ck studio --ensure` end to end (spec 5.1): starts a detached studio once, is idempotent under
//! concurrent calls, opens the browser tab once per server, and reports start failures.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{Duration, Instant};

use ck_server::ops::caps;
use common::{Background, err, ok};
use serde_json::{Value, json};

fn ensure_cmd(dir: &Path, extra: &[&str]) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_ck"));
    cmd.args(["studio", "--ensure", "--idle", "60", "--dir"])
        .arg(dir)
        .args(extra);
    cmd
}

/// A fake browser that appends the URL it was given to `opened.txt` next to it.
fn fake_browser(dir: &Path) -> (PathBuf, PathBuf) {
    use std::os::unix::fs::PermissionsExt;
    let log = dir.join("opened.txt");
    let script = dir.join("browser.sh");
    std::fs::write(
        &script,
        format!("#!/bin/sh\nprintf '%s\\n' \"$1\" >> '{}'\n", log.display()),
    )
    .unwrap();
    std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
    (script, log)
}

fn parse_ok(out: &Output) -> Value {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "ensure failed: {stderr}");
    let stdout = String::from_utf8(out.stdout.clone()).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.len() <= caps::STUDIO + 1, "over the studio cap");
    serde_json::from_str(&stdout).unwrap()
}

/// Wait until `path` has `lines` lines (the fake browser runs detached).
fn wait_lines(path: &Path, lines: usize) -> Vec<String> {
    let t0 = Instant::now();
    loop {
        let text = std::fs::read_to_string(path).unwrap_or_default();
        let got: Vec<String> = text.lines().map(str::to_owned).collect();
        if got.len() >= lines || t0.elapsed() > Duration::from_secs(5) {
            return got;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn ensure_starts_once_then_reuses_the_running_studio() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let bg = Background::new(d);

    let t0 = Instant::now();
    let first = parse_ok(&ensure_cmd(d, &[]).output().unwrap());
    let cold = t0.elapsed();
    assert_eq!(first["started"], true);
    assert_eq!(first["opened"], false);
    let port = first["port"].as_u64().unwrap();
    assert_eq!(first["url"], format!("http://127.0.0.1:{port}/"));
    assert!(
        first.get("token").is_none(),
        "the token stays in server.json"
    );
    assert!(bg.pid().is_some());

    // The detached server serves the project the ensure created.
    let status = ok(d, &["status"], caps::STATUS);
    assert_eq!(status["rev"], 0);

    let t1 = Instant::now();
    let second = parse_ok(&ensure_cmd(d, &[]).output().unwrap());
    let warm = t1.elapsed();
    assert_eq!(
        second,
        json!({ "url": first["url"], "port": port, "started": false, "opened": false })
    );
    eprintln!("ensure timings: cold {cold:?}, warm {warm:?}");
    // Spec 5.1: about 200 ms. Generous bounds for a loaded CI box; measured values are in
    // VERIFICATION.md.
    assert!(cold < Duration::from_secs(3), "cold ensure took {cold:?}");
    assert!(warm < Duration::from_secs(1), "warm ensure took {warm:?}");

    // --quiet prints nothing.
    let quiet = ensure_cmd(d, &["--quiet"]).output().unwrap();
    assert!(quiet.status.success());
    assert!(quiet.stdout.is_empty() && quiet.stderr.is_empty());

    // A second foreground studio is still refused while the background one runs.
    let refused = err(d, &["studio"]);
    assert_eq!(refused["code"], "E_SERVER");

    // The server outlives the ensure process and stops on SIGTERM like `ck studio`.
    bg.stop();
    assert!(!d.join(".clipkit/server.json").exists());
}

#[test]
fn concurrent_ensures_start_exactly_one_studio() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _bg = Background::new(d);
    let children: Vec<_> = (0..4)
        .map(|_| {
            ensure_cmd(d, &[])
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        })
        .collect();
    let outs: Vec<Value> = children
        .into_iter()
        .map(|c| parse_ok(&c.wait_with_output().unwrap()))
        .collect();
    let started = outs.iter().filter(|o| o["started"] == true).count();
    assert_eq!(started, 1, "{outs:?}");
    assert!(
        outs.iter().all(|o| o["port"] == outs[0]["port"]),
        "{outs:?}"
    );
}

#[test]
fn open_opens_the_tab_once_per_server() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let bg = Background::new(d);
    let (browser, log) = fake_browser(d);

    let first = parse_ok(
        &ensure_cmd(d, &["--open"])
            .env("BROWSER", &browser)
            .output()
            .unwrap(),
    );
    assert_eq!(
        (&first["started"], &first["opened"]),
        (&json!(true), &json!(true))
    );
    let token: Value =
        serde_json::from_slice(&std::fs::read(d.join(".clipkit/server.json")).unwrap()).unwrap();
    let expected = format!(
        "{}#token={}",
        first["url"].as_str().unwrap(),
        token["token"].as_str().unwrap()
    );
    assert_eq!(wait_lines(&log, 1), vec![expected.clone()]);

    let again = parse_ok(
        &ensure_cmd(d, &["--open"])
            .env("BROWSER", &browser)
            .output()
            .unwrap(),
    );
    assert_eq!(
        (&again["started"], &again["opened"]),
        (&json!(false), &json!(false))
    );

    // A new server is a new session: its tab opens once more.
    bg.stop();
    let restarted = parse_ok(
        &ensure_cmd(d, &["--open"])
            .env("BROWSER", &browser)
            .output()
            .unwrap(),
    );
    assert_eq!(
        (&restarted["started"], &restarted["opened"]),
        (&json!(true), &json!(true))
    );
    let lines = wait_lines(&log, 2);
    assert_eq!(lines.len(), 2, "{lines:?}");
    assert_ne!(lines[1], expected, "a fresh token for the new server");
}

#[test]
fn ensure_replaces_a_stale_server_json() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _bg = Background::new(d);
    std::fs::create_dir_all(d.join(".clipkit")).unwrap();
    let dead_port = std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port();
    let stale = json!({
        "port": dead_port, "token": "00", "pid": 1,
        "url": format!("http://127.0.0.1:{dead_port}/")
    });
    std::fs::write(d.join(".clipkit/server.json"), stale.to_string()).unwrap();

    let out = parse_ok(&ensure_cmd(d, &[]).output().unwrap());
    assert_eq!(out["started"], true);
    assert_eq!(ok(d, &["status"], caps::STATUS)["rev"], 0);
}

#[test]
fn a_studio_that_cannot_start_reports_its_own_error() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let taken = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = taken.local_addr().unwrap().port().to_string();

    let out = ensure_cmd(d, &["--port", &port]).output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let e: Value = serde_json::from_slice(&out.stderr).unwrap();
    assert_eq!(e["code"], "E_SERVER");
    assert!(
        e["message"].as_str().unwrap().contains("cannot bind"),
        "{e}"
    );
    assert!(!d.join(".clipkit/server.json").exists());
}

#[test]
fn open_and_quiet_need_ensure() {
    let dir = tempfile::tempdir().unwrap();
    for flag in ["--open", "--quiet"] {
        let out = Command::new(env!("CARGO_BIN_EXE_ck"))
            .args(["studio", flag, "--dir"])
            .arg(dir.path())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2), "{flag} is a usage error");
    }
}
