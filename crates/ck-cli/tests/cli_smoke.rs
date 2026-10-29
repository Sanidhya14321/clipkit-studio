//! Smoke test for the ck binary skeleton, including the output size cap every CLI command gets.

use std::process::Command;

const MAX_OUTPUT_BYTES: usize = 256;

#[test]
fn version_prints_and_stays_under_cap() {
    let out = Command::new(env!("CARGO_BIN_EXE_ck"))
        .arg("--version")
        .output()
        .unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.starts_with("ck "));
    assert!(stdout.len() <= MAX_OUTPUT_BYTES);
}

/// Help text lists every command, so it gets a larger cap than command output (3 KB since
/// Phase 8: spec 6.2's full command set does not fit 2 KB with one-line summaries).
const MAX_HELP_BYTES: usize = 3 * 1024;

#[test]
fn no_args_exit_with_short_help() {
    let out = Command::new(env!("CARGO_BIN_EXE_ck")).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8(out.stderr).unwrap();
    assert!(stderr.contains("Usage: ck"));
    for command in ["studio", "status", "project", "lock", "unlock"] {
        assert!(stderr.contains(command), "help lists {command}");
    }
    assert!(stderr.len() <= MAX_HELP_BYTES, "{} bytes", stderr.len());
}

#[test]
fn unknown_command_is_a_short_usage_error() {
    let out = Command::new(env!("CARGO_BIN_EXE_ck"))
        .arg("frobnicate")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stderr.len() <= MAX_OUTPUT_BYTES);
}

#[test]
fn logging_is_off_by_default_and_json_when_enabled() {
    let quiet = Command::new(env!("CARGO_BIN_EXE_ck"))
        .arg("--version")
        .env_remove("CK_LOG")
        .output()
        .unwrap();
    assert!(quiet.stderr.is_empty());

    let out = Command::new(env!("CARGO_BIN_EXE_ck"))
        .arg("--version")
        .env("CK_LOG", "debug")
        .output()
        .unwrap();
    let stderr = String::from_utf8(out.stderr).unwrap();
    let line: serde_json::Value = serde_json::from_str(stderr.lines().next().unwrap()).unwrap();
    assert_eq!(line["message"], "ck start");
    assert_eq!(line["level"], "DEBUG");
}
