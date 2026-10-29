//! `ck hook prompt|guard|stop|session-end` end to end (spec 6.1): hook JSON on stdin, exit 0 to
//! let the agent go on, exit 2 with a reason on stderr to block, and nothing at all in a session
//! without a ClipKit project.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use std::io::Write;
use std::path::Path;
use std::process::{Command, Output, Stdio};

use ck_server::ops::caps;
use common::{Background, NO_BROWSER, Studio, fake_browser, ok, wait_lines};
use serde_json::{Value, json};

/// The prompt note's cap (`hook::NOTE_CAP` in the binary).
const NOTE_CAP: usize = 300;

fn hook_with(dir: &Path, name: &str, input: &str, browser: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_ck"))
        .args(["hook", name, "--dir"])
        .arg(dir)
        .env("BROWSER", browser)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn hook(dir: &Path, name: &str, input: &Value) -> Output {
    hook_with(dir, name, &input.to_string(), NO_BROWSER)
}

fn passes(out: &Output) {
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(out.status.code(), Some(0), "hook should pass: {stderr}");
    assert!(out.stdout.is_empty(), "no output when passing");
}

/// Exit 2 with a one-line reason on stderr, nothing on stdout.
fn blocks(out: &Output) -> String {
    assert_eq!(out.status.code(), Some(2), "hook should block");
    assert!(out.stdout.is_empty());
    let reason = String::from_utf8(out.stderr.clone()).unwrap();
    assert_eq!(reason.lines().count(), 1, "one line: {reason}");
    reason
}

fn bash(cmd: &str) -> Value {
    json!({ "hook_event_name": "PreToolUse", "tool_name": "Bash", "tool_input": { "command": cmd } })
}

fn write(path: &str, cwd: &Path) -> Value {
    json!({
        "hook_event_name": "PreToolUse", "cwd": cwd, "tool_name": "Write",
        "tool_input": { "file_path": path, "content": "x" }
    })
}

#[test]
fn a_non_video_session_is_never_touched() {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path();
    let prompt = json!({ "hook_event_name": "UserPromptSubmit", "prompt": "fix the parser test" });
    passes(&hook(d, "prompt", &prompt));
    passes(&hook(d, "guard", &bash("ffmpeg -i a.mp4 b.mp4")));
    passes(&hook(d, "guard", &write(".clipkit/project.json", d)));
    passes(&hook(d, "stop", &json!({ "stop_hook_active": false })));
    passes(&hook(d, "session-end", &json!({ "reason": "exit" })));
    // Malformed or empty input is stepped over, not an error.
    passes(&hook_with(d, "prompt", "not json", NO_BROWSER));
    passes(&hook_with(d, "guard", "", NO_BROWSER));
    assert!(!d.join(".clipkit").exists(), "no project, no studio");
}

#[test]
fn a_video_prompt_starts_and_opens_the_studio_with_one_line() {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path();
    let (browser, log) = fake_browser(d);
    let browser = browser.to_str().unwrap();
    let _studio = Background::new(d);
    let prompt = json!({ "prompt": "Make a 20 second launch video" });
    let out = hook_with(d, "prompt", &prompt.to_string(), browser);
    assert!(out.status.success());
    let note = String::from_utf8(out.stdout).unwrap();
    assert_eq!(note.lines().count(), 1, "{note}");
    assert!(note.len() <= NOTE_CAP + 1, "over the note cap: {note}");
    assert!(
        note.starts_with("ClipKit studio: http://127.0.0.1:"),
        "{note}"
    );
    assert!(
        note.contains("mode generate, stage boot, agent idle"),
        "{note}"
    );
    assert!(!note.contains("token"), "the token never reaches the agent");
    let opened = wait_lines(&log, 1);
    assert_eq!(opened.len(), 1);

    // Later prompts in the project, video or not, reuse the studio and do not open a second tab.
    let status = ok(d, &["status"], caps::STATUS);
    let next = json!({ "prompt": "make the title bigger" });
    let out = hook_with(d, "prompt", &next.to_string(), browser);
    assert!(
        String::from_utf8(out.stdout)
            .unwrap()
            .starts_with("ClipKit studio: ")
    );
    assert_eq!(wait_lines(&log, 2), opened, "the tab opens once per server");
    assert_eq!(ok(d, &["status"], caps::STATUS)["rev"], status["rev"]);
}

#[test]
fn guard_blocks_direct_edits_and_bypassing_commands_in_a_project() {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path();
    std::fs::create_dir_all(d.join(".clipkit")).unwrap();

    let reason = blocks(&hook(d, "guard", &write(".clipkit/project.json", d)));
    assert!(reason.contains("ck project patch"), "{reason}");
    let abs = d.join("assets/cards/a.json");
    let reason = blocks(&hook(d, "guard", &write(abs.to_str().unwrap(), d)));
    assert!(reason.contains("assets/"), "{reason}");
    blocks(&hook(d, "guard", &write("footage/take.mov", d)));
    let edit = json!({
        "tool_name": "Edit", "cwd": d,
        "tool_input": { "file_path": "x/../.clipkit/server.json", "old_string": "a", "new_string": "b" }
    });
    blocks(&hook(d, "guard", &edit));
    let reason = blocks(&hook(
        d,
        "guard",
        &bash("ffmpeg -i footage/a.mp4 -t 2 cut.mp4"),
    ));
    assert!(reason.contains("ck export"), "{reason}");
    blocks(&hook(d, "guard", &bash("cat .clipkit/server.json")));

    passes(&hook(d, "guard", &write("src/main.rs", d)));
    passes(&hook(d, "guard", &bash("ck status")));
    passes(&hook(d, "guard", &bash("cargo test")));
    let read =
        json!({ "tool_name": "Read", "tool_input": { "file_path": ".clipkit/project.json" } });
    passes(&hook(d, "guard", &read));
}

#[test]
fn stop_blocks_once_while_the_agent_holds_locks() {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path();
    let stop = json!({ "hook_event_name": "Stop", "stop_hook_active": false });
    let looping = json!({ "hook_event_name": "Stop", "stop_hook_active": true });

    // No studio running: pass, and do not start one.
    std::fs::create_dir_all(d.join(".clipkit")).unwrap();
    passes(&hook(d, "stop", &stop));
    assert!(!d.join(".clipkit/server.json").exists());

    let _studio = Studio::start(d);
    passes(&hook(d, "stop", &stop));
    ok(d, &["lock", "intro", "t-intro"], caps::LOCK);
    ok(d, &["lock", "--holder", "human", "outro"], caps::LOCK);
    let reason = blocks(&hook(d, "stop", &stop));
    assert!(reason.contains("intro, t-intro"), "{reason}");
    assert!(!reason.contains("outro"), "human locks are not the agent's");
    assert!(reason.contains("ck unlock"), "{reason}");
    // Claude Code is already continuing because of this hook: never block again.
    passes(&hook(d, "stop", &looping));

    ok(d, &["unlock"], caps::LOCK);
    passes(&hook(d, "stop", &stop));
}

#[test]
fn session_end_releases_the_agents_locks_only() {
    let tmp = tempfile::tempdir().unwrap();
    let d = tmp.path();
    let _studio = Studio::start(d);
    ok(d, &["lock", "intro"], caps::LOCK);
    ok(d, &["lock", "--holder", "human", "outro"], caps::LOCK);
    passes(&hook(d, "session-end", &json!({ "reason": "logout" })));
    let locks = ok(d, &["status"], caps::STATUS)["locks"].clone();
    let held: Vec<(&str, &str)> = locks
        .as_array()
        .unwrap()
        .iter()
        .map(|l| {
            (
                l["layerId"].as_str().unwrap(),
                l["holder"].as_str().unwrap(),
            )
        })
        .collect();
    assert_eq!(held, vec![("outro", "human")]);
}
