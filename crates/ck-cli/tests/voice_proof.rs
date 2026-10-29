//! Phase 7 proof (spec 12): a script becomes voice slots; without a Fish Audio key `voice speak`
//! falls back to the labeled placeholder; a studio recording becomes a recorded, trimmed,
//! normalized and fit-checked take; cloning waits for consent; captions need the transcriber;
//! `ck ask` round-trips. Through the real `ck` binary against a real `ck studio`. The recording
//! is posted over HTTP exactly as the studio does; the studio side is in studio/e2e/phase7.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

use ck_server::info::ServerInfo;
use ck_server::ops::caps;
use common::{Studio, err, ok};
use serde_json::{Value, json};

/// POST `body` to the studio as the browser would, with the session token.
fn post_bytes(d: &Path, path: &str, kind: &str, body: &[u8]) -> (u16, Value) {
    let info = ServerInfo::read(&d.join(".clipkit")).unwrap();
    let mut s = TcpStream::connect(("127.0.0.1", info.port)).unwrap();
    let head = format!(
        "POST {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nx-ck-token: {}\r\n\
         content-type: {kind}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n",
        info.port,
        info.token,
        body.len()
    );
    s.write_all(head.as_bytes()).unwrap();
    s.write_all(body).unwrap();
    let mut out = String::new();
    s.read_to_string(&mut out).unwrap();
    let status: u16 = out[9..12].parse().unwrap();
    let json = out.split("\r\n\r\n").nth(1).unwrap_or("null");
    (status, serde_json::from_str(json).unwrap_or(Value::Null))
}

#[test]
fn a_script_becomes_voiced_slots_with_studio_takes() {
    let tones =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated/tones.wav");
    if !tones.is_file() {
        eprintln!("skipped: run `just fixtures` first");
        return;
    }
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);

    let script = d.join("script.json");
    std::fs::write(
        &script,
        json!({ "lines": [
            { "text": "Meet the editor you talk to." },
            { "text": "Record this one yourself.", "window": { "start": 3.0, "end": 13.0 } }
        ] })
        .to_string(),
    )
    .unwrap();
    let slots = ok(
        d,
        &["voice", "script", script.to_str().unwrap()],
        caps::VOICE_SCRIPT,
    );
    assert_eq!(slots["slots"].as_array().unwrap().len(), 2);

    // No Fish Audio key: the AI slot falls back to the labeled browser placeholder.
    let spoken = ok(d, &["voice", "speak", "v1"], caps::VOICE_WRITE);
    assert_eq!(spoken["placeholder"], true, "{spoken}");
    assert_eq!(spoken["slot"]["status"], "failed");

    // The user records v2 in the studio: a recorded take, trimmed (10 s of tone) and fit to
    // its 10 s window.
    ok(d, &["voice", "mode", "v2", "recorded"], caps::VOICE_WRITE);
    let (status, take) = post_bytes(
        d,
        "/api/voice/take?slot=v2",
        "audio/wav",
        &std::fs::read(&tones).unwrap(),
    );
    assert_eq!(status, 200, "{take}");
    assert_eq!(take["take"], "t1");
    assert_eq!(take["info"]["fit"], "fits");
    assert!((take["info"]["durationSec"].as_f64().unwrap() - 10.0).abs() < 0.2);

    // Cloning waits for the user's consent; captions need the transcriber (no key here).
    assert_eq!(err(d, &["voice", "clone", "v2"])["code"], "E_REQUEST");
    assert_eq!(err(d, &["voice", "captions"])["code"], "E_VOICE");

    // Lane C: an unanswered question returns its id to resume with.
    let asked = ok(
        d,
        &["ask", "Use my take?", "--option", "Yes", "--timeout", "0"],
        caps::ASK,
    );
    assert_eq!(asked, json!({ "id": "q1", "answered": false }));
}
