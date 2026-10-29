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
fn brief_tools_answer_like_their_cli_twins() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);
    let mut mcp = Mcp::start(d);

    let proposal = json!({
        "fields": { "platform": "youtube", "length": 15 },
        "questions": [{ "id": "price", "text": "Show the price?", "affects": ["cta"] }]
    });
    let file = d.join("brief.json");
    std::fs::write(&file, proposal.to_string()).unwrap();
    let path = file.to_str().unwrap();
    let cli = ok_line(d, &["brief", "propose", path], caps::BRIEF_PROPOSE);
    assert_eq!(serde_json::from_str::<Value>(&cli).unwrap()["rev"], 1);
    // The same values again change nothing, through either door.
    assert_eq!(mcp.ok("brief_propose", proposal, caps::BRIEF_PROPOSE), cli);

    let get = ok_line(d, &["brief", "get"], caps::BRIEF);
    assert_eq!(mcp.ok("brief_get", Value::Null, caps::BRIEF), get);
    let waited = ok_line(d, &["brief", "wait", "--timeout", "0"], caps::BRIEF);
    assert_eq!(
        mcp.ok("brief_wait", json!({ "timeoutSecs": 0 }), caps::BRIEF),
        waited
    );
    assert_eq!(
        serde_json::from_str::<Value>(&waited).unwrap()["ready"],
        false
    );

    let confirm = json!({
        "baseRev": 1, "author": "human",
        "ops": [{ "op": "replace", "path": "/brief/status", "value": "confirmed" }]
    });
    ok_line(d, &["project", "patch", &confirm.to_string()], caps::PATCH);
    let diff = ok_line(d, &["brief", "diff", "--since", "1"], caps::BRIEF);
    assert_eq!(
        mcp.ok("brief_diff", json!({ "since": 1 }), caps::BRIEF),
        diff
    );
    assert_eq!(
        serde_json::from_str::<Value>(&diff).unwrap()["changes"][0]["field"],
        "status"
    );

    let missing = mcp.err("brief_diff", json!({ "since": 99 }));
    assert_eq!(code(&missing), "E_NOT_FOUND");
    assert_eq!(missing, err_line(d, &["brief", "diff", "--since", "99"]));
    let too_many = json!({ "fields": {}, "questions": [
        { "id": "a", "text": "?", "affects": ["x"] }, { "id": "b", "text": "?", "affects": ["x"] },
        { "id": "c", "text": "?", "affects": ["x"] }, { "id": "d", "text": "?", "affects": ["x"] }
    ]});
    std::fs::write(&file, too_many.to_string()).unwrap();
    let refused = mcp.err("brief_propose", too_many);
    assert_eq!(code(&refused), "E_REQUEST");
    assert_eq!(refused, err_line(d, &["brief", "propose", path]));
}

#[test]
fn recipe_apply_answers_like_its_cli_twin() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);
    let mut mcp = Mcp::start(d);

    let req = json!({ "recipe": "hook-title", "slots": { "headline": "Ship faster" } });
    let file = d.join("apply.json");
    std::fs::write(&file, req.to_string()).unwrap();
    let path = file.to_str().unwrap();
    let cli: Value =
        serde_json::from_str(&ok_line(d, &["recipe", "apply", path], caps::RECIPE_APPLY)).unwrap();
    let tool: Value =
        serde_json::from_str(&mcp.ok("recipe_apply", req, caps::RECIPE_APPLY)).unwrap();
    // The second apply gets the next prefix and rev; otherwise the same answer.
    assert_eq!((&cli["rev"], &tool["rev"]), (&json!(1), &json!(2)));
    assert_eq!(cli["layers"][0], "hook-title-backdrop");
    assert_eq!(tool["layers"][0], "hook-title-2-backdrop");
    assert_eq!(cli["look"], tool["look"]);

    // Two hook titles on screen together, each solved alone: the checker sees them overlap.
    let checked = ok_line(d, &["layout", "check"], caps::LAYOUT_CHECK);
    assert_eq!(
        mcp.ok("layout_check", Value::Null, caps::LAYOUT_CHECK),
        checked
    );
    let checked: Value = serde_json::from_str(&checked).unwrap();
    assert_eq!(
        (&checked["checked"], &checked["ok"]),
        (&json!(2), &json!(false))
    );
    assert_eq!(checked["issues"][0]["rule"], "overlap");

    // The same collision fails the layout gate of verify.
    let verified = ok_line(d, &["verify"], caps::VERIFY);
    assert_eq!(mcp.ok("verify", Value::Null, caps::VERIFY), verified);

    let planned = ok_line(d, &["sound", "plan"], caps::SOUND_PLAN);
    assert_eq!(mcp.ok("sound_plan", Value::Null, caps::SOUND_PLAN), planned);
    let planned: Value = serde_json::from_str(&planned).unwrap();
    assert!(planned["cues"].is_array());

    let handed = ok_line(d, &["handoff"], caps::HANDOFF);
    assert_eq!(mcp.ok("handoff", Value::Null, caps::HANDOFF), handed);
    let handed: Value = serde_json::from_str(&handed).unwrap();
    assert_eq!(handed["rev"], 2);
    let waited = ok_line(d, &["wait", "--since", "0", "--timeout", "0"], caps::WAIT);
    assert_eq!(
        mcp.ok("wait", json!({ "since": 0, "timeoutSecs": 0 }), caps::WAIT),
        waited
    );
    assert_eq!(
        serde_json::from_str::<Value>(&waited).unwrap()["events"],
        json!([]),
        "agent revisions are not user events"
    );
    let verified: Value = serde_json::from_str(&verified).unwrap();
    assert_eq!(verified["ok"], false);
    assert!(
        verified["failed"]
            .as_array()
            .unwrap()
            .contains(&json!("layout"))
    );

    // Re-theme: the preview answers the same on both; then each applies once.
    let preview = ok_line(
        d,
        &["look", "apply", "cinematic", "--dry-run"],
        caps::LOOK_APPLY,
    );
    let req = json!({ "look": "cinematic", "dryRun": true });
    assert_eq!(mcp.ok("look_apply", req, caps::LOOK_APPLY), preview);
    let cli: Value = serde_json::from_str(&ok_line(
        d,
        &["look", "apply", "cinematic"],
        caps::LOOK_APPLY,
    ))
    .unwrap();
    assert_eq!((&cli["applied"], &cli["rev"]), (&json!(true), &json!(3)));
    let tool: Value = serde_json::from_str(&mcp.ok(
        "look_apply",
        json!({ "look": "cinematic" }),
        caps::LOOK_APPLY,
    ))
    .unwrap();
    assert_eq!(tool["applied"], false, "already the project look");
    let refused = mcp.err("look_apply", json!({ "look": "nope" }));
    assert_eq!(code(&refused), "E_NOT_FOUND");
    assert_eq!(refused, err_line(d, &["look", "apply", "nope"]));

    let bad = json!({ "recipe": "hook-title", "slots": { "nope": 1 } });
    std::fs::write(&file, bad.to_string()).unwrap();
    let refused = mcp.err("recipe_apply", bad);
    assert_eq!(code(&refused), "E_REQUEST");
    assert_eq!(refused, err_line(d, &["recipe", "apply", path]));
}

#[test]
fn import_answers_like_its_cli_twin() {
    let media = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/generated/tones.wav");
    if !media.exists() {
        eprintln!("skipped: run `just fixtures` first");
        return;
    }
    let media = media.canonicalize().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);
    let mut mcp = Mcp::start(d);

    let path = media.to_str().unwrap();
    let cli: Value = serde_json::from_str(&ok_line(d, &["import", path], caps::IMPORT)).unwrap();
    assert_eq!((&cli["added"], &cli["rev"]), (&json!(true), &json!(1)));
    assert_eq!(cli["source"]["kind"], "audio");
    // The same content again: both answer with the existing source and no new revision.
    let again = ok_line(d, &["import", path], caps::IMPORT);
    assert_eq!(
        mcp.ok("import", json!({ "path": path }), caps::IMPORT),
        again
    );
    let refused = mcp.err("import", json!({ "path": "/no/such/clip.mp4" }));
    assert_eq!(code(&refused), "E_MEDIA");
    assert_eq!(refused, err_line(d, &["import", "/no/such/clip.mp4"]));

    // Without a transcriber or a transcript, the edit tools refuse alike on both sides.
    let id = cli["source"]["id"].as_str().unwrap();
    let req = json!({ "source": id });
    let refused = mcp.err("transcribe", req.clone());
    assert_eq!(code(&refused), "E_VOICE");
    assert_eq!(refused, err_line(d, &["transcribe", id]));
    for (tool, args) in [
        ("transcript_get", vec!["transcript", "get", id]),
        ("cut_suggest", vec!["cut", "suggest", id]),
        ("cut_apply", vec!["cut", "apply", id]),
    ] {
        let refused = mcp.err(tool, req.clone());
        assert_eq!(code(&refused), "E_NOT_FOUND", "{tool}");
        assert_eq!(refused, err_line(d, &args), "{tool}");
    }

    // An audio-only project has no footage: export refuses alike.
    let refused = mcp.err("export", json!({ "preset": "tiktok" }));
    assert_eq!(code(&refused), "E_REQUEST");
    assert_eq!(refused, err_line(d, &["export", "--preset", "tiktok"]));
}

#[test]
fn comment_tools_answer_like_their_cli_twins() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let _studio = Studio::start(d);
    let mut mcp = Mcp::start(d);

    let cli: Value = serde_json::from_str(&ok_line(
        d,
        &["comments", "add", "Is this the right logo?", "--at", "2.5"],
        caps::COMMENT_WRITE,
    ))
    .unwrap();
    assert_eq!((&cli["id"], &cli["status"]), (&json!("c1"), &json!("open")));
    let tool: Value = serde_json::from_str(&mcp.ok(
        "comments_add",
        json!({ "text": "Check the pacing", "anchor": { "tStart": 1.0, "tEnd": 3.0 } }),
        caps::COMMENT_WRITE,
    ))
    .unwrap();
    assert_eq!(tool["id"], "c2");

    let listed = ok_line(d, &["comments", "list"], caps::COMMENTS);
    assert_eq!(mcp.ok("comments_list", json!({}), caps::COMMENTS), listed);

    let replied: Value = serde_json::from_str(&mcp.ok(
        "comments_reply",
        json!({ "id": "c1", "text": "Looking", "scope": [] }),
        caps::COMMENT_WRITE,
    ))
    .unwrap();
    assert_eq!(replied["status"], "working");
    let resolved: Value = serde_json::from_str(&ok_line(
        d,
        &["comments", "resolve", "c1", "--text", "Kept it"],
        caps::COMMENT_WRITE,
    ))
    .unwrap();
    assert_eq!(resolved["status"], "resolved");

    let context = ok_line(d, &["context"], caps::CONTEXT);
    assert_eq!(mcp.ok("context", json!({}), caps::CONTEXT), context);
    assert!(context.contains("\"c2\""), "{context}");

    // Voice slots: the script from a file on the CLI, the mode through MCP.
    let script = json!({ "lines": [{ "text": "Hello there." }, { "text": "Bye now." }] });
    let file = d.join("script.json");
    std::fs::write(&file, script.to_string()).unwrap();
    let cli: Value = serde_json::from_str(&ok_line(
        d,
        &["voice", "script", file.to_str().unwrap()],
        caps::VOICE_SCRIPT,
    ))
    .unwrap();
    let tool: Value =
        serde_json::from_str(&mcp.ok("voice_script", script, caps::VOICE_SCRIPT)).unwrap();
    assert_eq!(cli["slots"], tool["slots"]);
    let moded = ok_line(d, &["voice", "mode", "v1", "recorded"], caps::VOICE_WRITE);
    assert!(moded.contains("\"recorded\""), "{moded}");
    let refused = mcp.err("voice_mode", json!({ "slot": "v9", "mode": "ai" }));
    assert_eq!(refused, err_line(d, &["voice", "mode", "v9", "ai"]));

    // Ask: unanswered after a 0 s wait on both sides; a resume of an unknown id refuses alike.
    let cli: Value = serde_json::from_str(&ok_line(
        d,
        &["ask", "Keep it?", "--option", "Yes", "--timeout", "0"],
        caps::ASK,
    ))
    .unwrap();
    assert_eq!(cli, json!({ "id": "q1", "answered": false }));
    let tool: Value = serde_json::from_str(&mcp.ok(
        "ask",
        json!({ "question": "Sure?", "timeoutSecs": 0 }),
        caps::ASK,
    ))
    .unwrap();
    assert_eq!(tool, json!({ "id": "q2", "answered": false }));
    let refused = mcp.err("ask", json!({ "id": "q9", "timeoutSecs": 0 }));
    assert_eq!(
        refused,
        err_line(d, &["ask", "--id", "q9", "--timeout", "0"])
    );

    let refused = mcp.err("comments_resolve", json!({ "id": "c9" }));
    assert_eq!(code(&refused), "E_NOT_FOUND");
    assert_eq!(refused, err_line(d, &["comments", "resolve", "c9"]));
}

#[test]
fn sources_tools_answer_like_their_cli_twins_without_a_studio() {
    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    std::fs::write(d.join("sources.yaml"), "[]").unwrap();
    let mut mcp = Mcp::start(d);
    let added = ok_line(
        d,
        &[
            "sources",
            "add",
            "pack-a",
            "--kind",
            "remotion-components",
            "--repo",
            "me/pack-a",
        ],
        ck_assets::sources::caps::SOURCES_WRITE,
    );
    assert!(added.contains("\"unverified\""), "{added}");
    let list = ok_line(
        d,
        &["sources", "list"],
        ck_assets::sources::caps::SOURCES_LIST,
    );
    assert_eq!(
        mcp.ok(
            "sources_list",
            json!({}),
            ck_assets::sources::caps::SOURCES_LIST
        ),
        list
    );
    let gaps = ok_line(
        d,
        &["sources", "gaps"],
        ck_assets::sources::caps::SOURCES_GAPS,
    );
    assert_eq!(
        mcp.ok(
            "sources_gaps",
            json!({}),
            ck_assets::sources::caps::SOURCES_GAPS
        ),
        gaps
    );
    // Vetting a fixture pack from a local folder: CLI and MCP pin the same result.
    let pack = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/sources/pack-mit");
    let pack = pack.canonicalize().unwrap();
    let cli = ok_line(
        d,
        &["sources", "vet", "pack-a", "--from", pack.to_str().unwrap()],
        ck_assets::vet::caps::VET,
    );
    let tool = mcp.ok(
        "sources_vet",
        json!({ "id": "pack-a", "from": pack }),
        ck_assets::vet::caps::VET,
    );
    assert_eq!(cli, tool);
    assert!(cli.contains("\"vetted\""), "{cli}");
    let cli = ok_line(
        d,
        &["sources", "ingest", "pack-a"],
        ck_assets::ingest::caps::INGEST,
    );
    let tool = mcp.ok(
        "sources_ingest",
        json!({ "id": "pack-a" }),
        ck_assets::ingest::caps::INGEST,
    );
    assert_eq!(cli, tool);
    assert!(cli.contains("pack-a/glow-title"), "{cli}");

    // The retrieval ladder answers alike (no project here, so no ledger is kept).
    use ck_assets::retrieve::caps as rc;
    let found = ok_line(d, &["assets", "search", "glow"], rc::SEARCH);
    assert!(
        found.contains("pack-a/glow-title remotion-clip A permissive"),
        "{found}"
    );
    assert_eq!(
        mcp.ok("assets_search", json!({ "query": "glow" }), rc::SEARCH),
        found
    );
    let skeleton = ok_line(
        d,
        &["assets", "skeleton", "pack-a/glow-title"],
        rc::SKELETON,
    );
    let tool = mcp.ok(
        "assets_skeleton",
        json!({ "id": "pack-a/glow-title" }),
        rc::SKELETON,
    );
    assert_eq!(tool, skeleton);
    let refused = mcp.err("assets_card", json!({ "id": "pack-a/nope" }));
    assert_eq!(code(&refused), "E_NOT_FOUND");
    assert_eq!(refused, err_line(d, &["assets", "card", "pack-a/nope"]));

    let refused = mcp.err("sources_add", json!({ "id": "pack-a", "kind": "x" }));
    assert_eq!(code(&refused), "E_REQUEST");
    assert_eq!(
        refused,
        err_line(d, &["sources", "add", "pack-a", "--kind", "x"])
    );
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
    assert_eq!(
        mcp.ok(
            "recipe_search",
            json!({ "query": "hook", "limit": 2 }),
            lib::RECIPE_SEARCH
        ),
        ok_line(
            d,
            &["recipe", "search", "hook", "--limit", "2"],
            lib::RECIPE_SEARCH
        )
    );
    assert_eq!(
        mcp.ok("recipe_search", Value::Null, lib::RECIPE_SEARCH),
        ok_line(d, &["recipe", "search", "--limit", "5"], lib::RECIPE_SEARCH)
    );
    assert_eq!(
        mcp.ok(
            "recipe_card",
            json!({ "id": "outro-cta" }),
            lib::RECIPE_CARD
        ),
        ok_line(d, &["recipe", "card", "outro-cta"], lib::RECIPE_CARD)
    );
    let missing = mcp.err("recipe_card", json!({ "id": "nope" }));
    assert_eq!(code(&missing), "E_NOT_FOUND");
    assert_eq!(missing, err_line(d, &["recipe", "card", "nope"]));
    // None of these started a studio.
    assert!(!d.join(".clipkit/server.json").exists());
}
