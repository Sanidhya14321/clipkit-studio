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

#[test]
fn unknown_args_exit_with_usage() {
    let out = Command::new(env!("CARGO_BIN_EXE_ck")).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stderr.len() <= MAX_OUTPUT_BYTES);
}
