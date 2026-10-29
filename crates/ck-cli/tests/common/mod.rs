//! Shared helpers for the end-to-end tests: a real `ck studio` and one-shot ck commands.
#![allow(clippy::unwrap_used, dead_code)] // test helpers outside #[test] functions

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

/// A running `ck studio`; killed on drop.
pub struct Studio {
    child: Child,
    pub started: Value,
}

impl Studio {
    pub fn start(dir: &Path) -> Self {
        Self::start_with(dir, &[])
    }

    /// `ck studio` with extra arguments (for example `--idle 1`).
    pub fn start_with(dir: &Path, extra: &[&str]) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ck"))
            .args(["studio", "--dir"])
            .arg(dir)
            .args(extra)
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

    pub fn pid(&self) -> u32 {
        self.child.id()
    }

    /// Wait up to `limit` for the studio to exit by itself.
    pub fn exit_within(&mut self, limit: Duration) -> Option<ExitStatus> {
        let t0 = Instant::now();
        while t0.elapsed() < limit {
            if let Some(status) = self.child.try_wait().unwrap() {
                return Some(status);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        None
    }
}

impl Drop for Studio {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// A fake browser that appends the URL it was given to `opened.txt` next to it.
pub fn fake_browser(dir: &Path) -> (PathBuf, PathBuf) {
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

/// Wait until `path` has `lines` lines (the fake browser runs detached).
pub fn wait_lines(path: &Path, lines: usize) -> Vec<String> {
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

/// A no-op browser, so a command that starts a studio (spec 5.1) never opens a real tab.
pub const NO_BROWSER: &str = "true";

pub fn ck(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ck"))
        .args(args)
        .arg("--dir")
        .arg(dir)
        .env("BROWSER", NO_BROWSER)
        .output()
        .unwrap()
}

/// Run a command that must succeed and return its one stdout line (without the newline),
/// checked against `cap`.
pub fn ok_line(dir: &Path, args: &[&str], cap: usize) -> String {
    let out = ck(dir, args);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "ck {args:?} failed: {stderr}");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1, "one line of JSON");
    assert!(
        stdout.len() <= cap + 1,
        "ck {args:?} over its {cap} byte cap"
    );
    stdout.trim_end_matches('\n').to_owned()
}

/// Run a command that must succeed; check its one-line output against `cap` and parse it.
pub fn ok(dir: &Path, args: &[&str], cap: usize) -> Value {
    serde_json::from_str(&ok_line(dir, args, cap)).unwrap()
}

/// Run a command that must fail; return its stderr line (the error JSON).
pub fn err_line(dir: &Path, args: &[&str]) -> String {
    let out = ck(dir, args);
    assert_eq!(out.status.code(), Some(1), "ck {args:?} should fail");
    assert!(out.stdout.is_empty());
    String::from_utf8(out.stderr)
        .unwrap()
        .trim_end_matches('\n')
        .to_owned()
}

/// Run a command that must fail; return the error JSON from stderr.
pub fn err(dir: &Path, args: &[&str]) -> Value {
    serde_json::from_str(&err_line(dir, args)).unwrap()
}

/// A patch adding track `t-<name>` holding one clip `<name>`.
pub fn add_track(base_rev: u64, author: &str, name: &str) -> String {
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

/// A studio started in the background by `ck studio --ensure`; stopped on drop through the pid
/// in its server.json (SIGTERM, the same path as Ctrl-C).
pub struct Background {
    root: std::path::PathBuf,
}

impl Background {
    pub fn new(dir: &Path) -> Self {
        Self {
            root: dir.join(".clipkit"),
        }
    }

    pub fn pid(&self) -> Option<u32> {
        let text = std::fs::read_to_string(self.root.join("server.json")).ok()?;
        let info: Value = serde_json::from_str(&text).ok()?;
        info["pid"].as_u64().and_then(|p| u32::try_from(p).ok())
    }

    /// Stop the server and wait until it removed its server.json.
    pub fn stop(&self) {
        let Some(pid) = self.pid() else { return };
        let _ = Command::new("kill").arg(pid.to_string()).status();
        let t0 = Instant::now();
        while self.root.join("server.json").exists() && t0.elapsed() < Duration::from_secs(10) {
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Background {
    fn drop(&mut self) {
        self.stop();
    }
}
