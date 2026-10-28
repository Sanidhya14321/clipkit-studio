//! Shared helpers for the end-to-end tests: a real `ck studio` and one-shot ck commands.
#![allow(clippy::unwrap_used, dead_code)] // test helpers outside #[test] functions

use std::io::{BufRead, BufReader};
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};

use serde_json::{Value, json};

/// A running `ck studio`; killed on drop.
pub struct Studio {
    child: Child,
    pub started: Value,
}

impl Studio {
    pub fn start(dir: &Path) -> Self {
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

pub fn ck(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ck"))
        .args(args)
        .arg("--dir")
        .arg(dir)
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
