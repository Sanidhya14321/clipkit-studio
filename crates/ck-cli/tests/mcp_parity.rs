//! `ck mcp` end to end: a real MCP session over stdio against a real `ck studio`, checked
//! against the CLI twin of every tool (spec 6.2: same names, inputs and outputs).
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

mod common;

use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use ck_server::ops::caps;
use common::{Background, NO_BROWSER, Studio, add_track, err_line, ok_line};
use serde_json::{Value, json};

/// A `ck mcp` process with an initialized session; killed on drop.
struct Mcp {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl Mcp {
    fn start(dir: &Path) -> Self {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ck"))
            .args(["mcp", "--dir"])
            .arg(dir)
            .env("BROWSER", NO_BROWSER)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .unwrap();
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut mcp = Self {
            child,
            stdin,
            stdout,
            next_id: 1,
        };
        let init = mcp.request(
            "initialize",
            json!({
                "protocolVersion": "2025-06-18",
                "capabilities": {},
                "clientInfo": { "name": "ck-tests", "version": "0" }
            }),
        );
        assert_eq!(init["result"]["serverInfo"]["name"], "ck", "{init}");
        assert!(init["result"]["capabilities"]["tools"].is_object());
        mcp.send(&json!({ "jsonrpc": "2.0", "method": "notifications/initialized" }));
        mcp
    }

    fn send(&mut self, msg: &Value) {
        writeln!(self.stdin, "{msg}").unwrap();
        self.stdin.flush().unwrap();
    }

    /// One JSON-RPC request; returns the whole response (`result` or `error`).
    fn request(&mut self, method: &str, params: Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }));
        loop {
            let mut line = String::new();
            assert!(
                self.stdout.read_line(&mut line).unwrap() > 0,
                "ck mcp closed"
            );
            let msg: Value = serde_json::from_str(&line).unwrap();
            if msg["id"] == id {
                return msg;
            }
        }
    }

    /// Call a tool; returns (isError, the one text block).
    fn call(&mut self, name: &str, args: Value) -> (bool, String) {
        let res = self.request("tools/call", json!({ "name": name, "arguments": args }));
        let result = &res["result"];
        let content = result["content"].as_array().unwrap();
        assert_eq!(content.len(), 1, "{res}");
        assert_eq!(content[0]["type"], "text");
        let text = content[0]["text"].as_str().unwrap().to_owned();
        (result["isError"].as_bool().unwrap_or(false), text)
    }

    /// A call that must succeed, within `cap`.
    fn ok(&mut self, name: &str, args: Value, cap: usize) -> String {
        let (is_error, text) = self.call(name, args);
        assert!(!is_error, "{name} failed: {text}");
        assert!(text.len() <= cap, "{name} over its {cap} byte cap");
        text
    }

    /// A call that must fail as a tool error; returns the CkError JSON text.
    fn err(&mut self, name: &str, args: Value) -> String {
        let (is_error, text) = self.call(name, args);
        assert!(is_error, "{name} should fail: {text}");
        text
    }
}

impl Drop for Mcp {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Lock timestamps differ between two acquisitions; everything else must match.
fn without_times(text: &str) -> Value {
    fn strip(v: &mut Value) {
        match v {
            Value::Object(map) => {
                map.retain(|k, _| !k.ends_with("AtMs"));
                map.values_mut().for_each(strip);
            }
            Value::Array(items) => items.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut v = serde_json::from_str(text).unwrap();
    strip(&mut v);
    v
}

fn code(text: &str) -> Value {
    serde_json::from_str::<Value>(text).unwrap()["code"].clone()
}

#[test]
fn tools_list_names_every_cli_twin_with_its_schema() {
    let dir = tempfile::tempdir().unwrap();
    let mut mcp = Mcp::start(dir.path());
    let listed = mcp.request("tools/list", json!({}));
    let tools = listed["result"]["tools"].as_array().unwrap();
    let specs = ck_mcp::tools::specs();
    assert_eq!(tools.len(), specs.len());
    for (tool, spec) in tools.iter().zip(&specs) {
        assert_eq!(tool["name"], spec.name);
        assert_eq!(tool["description"], spec.description);
        assert_eq!(tool["inputSchema"], spec.input_schema, "{}", spec.name);
    }
    // Unknown tools are a protocol error, not a tool result.
    let unknown = mcp.request("tools/call", json!({ "name": "nope", "arguments": {} }));
    assert!(unknown["error"].is_object(), "{unknown}");
}

#[test]
fn every_tool_answers_like_its_cli_twin() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);
    let mut mcp = Mcp::start(d);

    // Reads: byte for byte the CLI's stdout line.
    assert_eq!(
        mcp.ok("status", json!({}), caps::STATUS),
        ok_line(d, &["status"], caps::STATUS)
    );
    assert_eq!(
        mcp.ok("status", Value::Null, caps::STATUS),
        ok_line(d, &["status"], caps::STATUS),
        "status can be called without arguments"
    );
    let summary = ok_line(d, &["project", "get", "--summary"], caps::SUMMARY);
    assert_eq!(mcp.ok("project_get", json!({}), caps::SUMMARY), summary);
    assert_eq!(
        mcp.ok("project_get", json!({ "summary": true }), caps::SUMMARY),
        summary
    );

    // Writes: the same request through either door gives the same answer.
    let cli_lock = ok_line(d, &["lock", "intro", "t-intro"], caps::LOCK);
    let mcp_lock = mcp.ok(
        "lock",
        json!({ "holder": "agent", "layerIds": ["intro", "t-intro"] }),
        caps::LOCK,
    );
    assert_eq!(without_times(&mcp_lock), without_times(&cli_lock));

    let patch = add_track(0, "agent", "intro");
    let patched = mcp.ok(
        "project_patch",
        serde_json::from_str(&patch).unwrap(),
        caps::PATCH,
    );
    assert_eq!(serde_json::from_str::<Value>(&patched).unwrap()["rev"], 1);
    // The same patch again is stale through both doors, with the same error.
    let stale = mcp.err("project_patch", serde_json::from_str(&patch).unwrap());
    assert_eq!(code(&stale), "E_PATCH_STALE");
    assert_eq!(stale, err_line(d, &["project", "patch", &patch]));

    assert_eq!(
        mcp.ok("project_get", json!({ "layer": "intro" }), caps::LAYER),
        ok_line(d, &["project", "get", "--layer", "intro"], caps::LAYER)
    );
    let missing = mcp.err("project_get", json!({ "layer": "nope" }));
    assert_eq!(code(&missing), "E_NOT_FOUND");
    assert_eq!(missing, err_line(d, &["project", "get", "--layer", "nope"]));

    // Turn-taking holds through MCP: a human edit is refused while the agent holds the layer.
    let human = json!({
        "baseRev": 1, "author": "human",
        "ops": [{ "op": "replace", "path": "/tracks/0/items/0/name", "value": "mine" }]
    });
    let refused = mcp.err("project_patch", human.clone());
    assert_eq!(code(&refused), "E_LOCK_HELD");
    assert_eq!(
        refused,
        err_line(d, &["project", "patch", &human.to_string()])
    );

    let mcp_beat = mcp.ok("heartbeat", json!({ "holder": "agent" }), caps::LOCK);
    let cli_beat = ok_line(d, &["heartbeat"], caps::LOCK);
    assert_eq!(without_times(&mcp_beat), without_times(&cli_beat));
    assert_eq!(
        serde_json::from_str::<Value>(&cli_beat).unwrap()["locks"]
            .as_array()
            .unwrap()
            .len(),
        2
    );

    let mcp_unlock = mcp.ok(
        "unlock",
        json!({ "holder": "agent", "layerIds": ["intro"] }),
        caps::LOCK,
    );
    let cli_unlock = ok_line(d, &["unlock", "intro"], caps::LOCK);
    assert_eq!(without_times(&mcp_unlock), without_times(&cli_unlock));
    assert_eq!(
        mcp.ok("unlock", json!({ "holder": "agent" }), caps::LOCK),
        ok_line(d, &["unlock"], caps::LOCK)
    );
    assert_eq!(
        mcp.ok("status", json!({}), caps::STATUS),
        ok_line(d, &["status"], caps::STATUS)
    );
}

#[test]
fn output_caps_hold_through_mcp() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);
    let mut mcp = Mcp::start(d);

    // Enough locks that the lock reply must be trimmed and status goes over its cap.
    let ids: Vec<String> = (0..120)
        .map(|i| format!("layer-with-a-long-id-{i:03}"))
        .collect();
    let locked = mcp.ok(
        "lock",
        json!({ "holder": "agent", "layerIds": ids }),
        caps::LOCK,
    );
    let locked: Value = serde_json::from_str(&locked).unwrap();
    let kept = locked["locks"].as_array().unwrap().len();
    assert!(kept > 0 && kept < ids.len());
    assert_eq!(locked["omitted"], ids.len() - kept);

    let mut args = vec!["lock"];
    args.extend(ids.iter().map(String::as_str));
    let cli = ok_line(d, &args, caps::LOCK);
    assert_eq!(
        without_times(&cli)["omitted"],
        locked["omitted"],
        "the CLI trims the same way"
    );

    let over = mcp.err("status", json!({}));
    assert_eq!(code(&over), "E_OUTPUT_CAP");
    assert_eq!(over, err_line(d, &["status"]));
}

#[test]
fn the_first_tool_call_starts_the_studio() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let bg = Background::new(d);
    let mut mcp = Mcp::start(d);

    let status = mcp.ok("status", json!({}), caps::STATUS);
    assert!(bg.pid().is_some(), "the call left a running studio");
    assert_eq!(status, ok_line(d, &["status"], caps::STATUS));
}

#[test]
fn a_studio_that_cannot_start_is_the_same_tool_error() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _bg = Background::new(d);
    std::fs::create_dir_all(d.join(".clipkit")).unwrap();
    std::fs::write(d.join(".clipkit/project.json"), "{not json").unwrap();
    let mut mcp = Mcp::start(d);

    let down = mcp.err("status", json!({}));
    assert!(
        code(&down).as_str().is_some_and(|c| c.starts_with("E_")),
        "{down}"
    );
    assert_eq!(down, err_line(d, &["status"]));
}

#[test]
fn bad_arguments_are_tool_errors() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);
    let mut mcp = Mcp::start(d);
    for (name, args) in [
        ("status", json!({ "extra": 1 })),
        ("project_get", json!({ "summary": true, "layer": "a" })),
        ("project_patch", json!({ "baseRev": "zero" })),
        ("lock", json!({ "holder": "robot", "layerIds": ["a"] })),
        ("unlock", json!({})),
        ("heartbeat", json!({ "holder": "robot" })),
        ("studio", json!({ "open": "yes" })),
    ] {
        let e = mcp.err(name, args);
        assert_eq!(code(&e), "E_REQUEST", "{name}: {e}");
    }
}

#[test]
fn studio_tool_ensures_like_ck_studio_ensure() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _bg = Background::new(d);
    let mut mcp = Mcp::start(d);

    // No studio: the tool starts one in the background.
    let started: Value = serde_json::from_str(&mcp.ok("studio", json!({}), caps::STUDIO)).unwrap();
    assert_eq!(
        (&started["started"], &started["opened"]),
        (&json!(true), &json!(false))
    );

    // Running: both doors answer byte for byte the same.
    let cli = ok_line(d, &["studio", "--ensure"], caps::STUDIO);
    assert_eq!(mcp.ok("studio", Value::Null, caps::STUDIO), cli);
    let cli: Value = serde_json::from_str(&cli).unwrap();
    assert_eq!(
        (&cli["started"], &cli["port"]),
        (&json!(false), &started["port"])
    );
}

#[test]
fn library_tools_answer_like_their_cli_twins_without_a_studio() {
    use ck_assets::library::caps as lib;
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let mut mcp = Mcp::start(d);
    assert_eq!(
        mcp.ok(
            "taste_search",
            json!({ "query": "pacing", "limit": 3 }),
            lib::TASTE_SEARCH
        ),
        ok_line(
            d,
            &["taste", "search", "pacing", "--limit", "3"],
            lib::TASTE_SEARCH
        )
    );
    assert_eq!(
        mcp.ok("taste_search", Value::Null, lib::TASTE_SEARCH),
        ok_line(d, &["taste", "search"], lib::TASTE_SEARCH)
    );
    assert_eq!(
        mcp.ok("taste_read", json!({ "id": "pacing" }), lib::TASTE_READ),
        ok_line(d, &["taste", "read", "pacing"], lib::TASTE_READ)
    );
    assert_eq!(
        mcp.ok("look_list", json!({}), lib::LOOK_LIST),
        ok_line(d, &["look", "list"], lib::LOOK_LIST)
    );
    assert_eq!(
        mcp.ok("look_get", json!({ "id": "cinematic" }), lib::LOOK_GET),
        ok_line(d, &["look", "get", "cinematic"], lib::LOOK_GET)
    );
    let missing = mcp.err("look_get", json!({ "id": "nope" }));
    assert_eq!(code(&missing), "E_NOT_FOUND");
    assert_eq!(missing, err_line(d, &["look", "get", "nope"]));
    assert_eq!(code(&mcp.err("taste_read", json!({}))), "E_REQUEST");
    // None of these started a studio.
    assert!(!d.join(".clipkit/server.json").exists());
}
