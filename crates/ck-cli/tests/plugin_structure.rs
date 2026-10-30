//! Structure of the shipped Claude Code plugin in `plugin/` (spec 6.1, format checked against the
//! plugins reference on 2026-10-29): manifest, hooks wired to real `ck hook` subcommands, the
//! runtime skill carrying the spec 9.4 text, the review skill and the read-only critic agent.
#![allow(clippy::unwrap_used)] // test helpers outside #[test] functions

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::Value;

/// The hook events spec 6.1 asks for, with the `ck hook` subcommand each one runs.
const HOOKS: [(&str, &str); 4] = [
    ("UserPromptSubmit", "prompt"),
    ("PreToolUse", "guard"),
    ("Stop", "stop"),
    ("SessionEnd", "session-end"),
];
/// Spec 6.1: the runtime skill stays under about 100 lines.
const SKILL_LINES: usize = 100;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn plugin(rel: &str) -> PathBuf {
    root().join("plugin").join(rel)
}

fn read(rel: &str) -> String {
    let path = plugin(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

fn json(rel: &str) -> Value {
    serde_json::from_str(&read(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// The YAML frontmatter's `key: value` lines and the body after it.
fn frontmatter(text: &str) -> (Vec<(String, String)>, String) {
    let rest = text.strip_prefix("---\n").expect("starts with ---");
    let (head, body) = rest.split_once("\n---\n").expect("closing ---");
    let fields = head
        .lines()
        .filter_map(|l| l.split_once(": "))
        .map(|(k, v)| (k.trim().to_owned(), v.trim().to_owned()))
        .collect();
    (fields, body.to_owned())
}

fn field<'a>(fields: &'a [(String, String)], key: &str) -> &'a str {
    fields
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.as_str())
        .unwrap_or_else(|| panic!("frontmatter has no {key}"))
}

/// Text that changes between runs would break prompt caching (spec 9.1): no ISO dates.
fn has_date(text: &str) -> bool {
    let b = text.as_bytes();
    b.windows(10).any(|w| {
        w.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            _ => c.is_ascii_digit(),
        })
    })
}

/// The lines of the runtime skill text in spec 9.4 (the fenced block).
fn spec_skill_lines() -> Vec<String> {
    let spec = std::fs::read_to_string(root().join("docs/spec/09-token-efficiency.md")).unwrap();
    let (_, after) = spec.split_once("### 9.4").unwrap();
    let (_, block) = after.split_once("```\n").unwrap();
    let (block, _) = block.split_once("```").unwrap();
    block
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(str::to_owned)
        .collect()
}

#[test]
fn manifest_names_the_plugin_and_sits_alone() {
    let manifest = json(".claude-plugin/plugin.json");
    assert_eq!(manifest["name"], "clipkit");
    assert!(
        manifest["description"]
            .as_str()
            .is_some_and(|d| !d.is_empty())
    );
    // Only the manifest goes in .claude-plugin/; components sit at the plugin root.
    let entries: Vec<_> = std::fs::read_dir(plugin(".claude-plugin"))
        .unwrap()
        .map(|e| e.unwrap().file_name().into_string().unwrap())
        .collect();
    assert_eq!(entries, ["plugin.json"]);
    for component in [
        "skills/clipkit-video/SKILL.md",
        "skills/clipkit-review/SKILL.md",
        "agents/clipkit-critic.md",
        "hooks/hooks.json",
    ] {
        assert!(plugin(component).is_file(), "missing {component}");
    }
}

#[test]
fn hooks_run_the_ck_hook_subcommands_on_the_project_folder() {
    let file = json("hooks/hooks.json");
    let events = file["hooks"].as_object().expect("top-level hooks object");
    assert_eq!(events.len(), HOOKS.len(), "{events:?}");
    for (event, sub) in HOOKS {
        let groups = events[event].as_array().unwrap();
        assert_eq!(groups.len(), 1, "{event}");
        let handlers = groups[0]["hooks"].as_array().unwrap();
        assert_eq!(handlers.len(), 1, "{event}");
        assert_eq!(handlers[0]["type"], "command");
        assert_eq!(
            handlers[0]["command"],
            format!("ck hook {sub} --dir \"$CLAUDE_PROJECT_DIR\""),
            "{event}"
        );
        // Every hook finishes quickly (spec 6.1); timeouts are in seconds.
        let timeout = handlers[0]["timeout"].as_u64().unwrap();
        assert!((1..=10).contains(&timeout), "{event} timeout {timeout}");
    }
    let matcher = events["PreToolUse"][0]["matcher"].as_str().unwrap();
    let tools: Vec<_> = matcher.split('|').collect();
    assert_eq!(tools, ["Write", "Edit", "Bash"]);
}

/// Runs a hooks.json command line through the shell the way Claude Code does, with `ck` resolved
/// to the binary under test and `CLAUDE_PROJECT_DIR` pointing at `dir`.
#[cfg(unix)]
fn run_hook_command(event: &str, dir: &Path, input: &str) -> Output {
    use std::io::Write;
    use std::process::Stdio;

    let file = json("hooks/hooks.json");
    let command = file["hooks"][event][0]["hooks"][0]["command"]
        .as_str()
        .unwrap();
    let args = command.strip_prefix("ck ").unwrap();
    let line = format!("\"$CK_BIN\" {args}");
    let mut child = Command::new("sh")
        .args(["-c", &line])
        .env("CK_BIN", env!("CARGO_BIN_EXE_ck"))
        .env("CLAUDE_PROJECT_DIR", dir)
        .env("BROWSER", "true")
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

#[cfg(unix)]
#[test]
fn every_hook_command_passes_silently_outside_a_clipkit_project() {
    let dir = tempfile::tempdir().unwrap();
    for (event, _) in HOOKS {
        let out = run_hook_command(event, dir.path(), "{\"prompt\":\"fix the parser bug\"}");
        assert_eq!(out.status.code(), Some(0), "{event}: {out:?}");
        assert!(out.stdout.is_empty(), "{event}: {out:?}");
    }
    assert!(!dir.path().join(".clipkit").exists());
}

#[cfg(unix)]
#[test]
fn guard_command_uses_the_project_folder_from_the_environment() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir(dir.path().join(".clipkit")).unwrap();
    let target = dir.path().join(".clipkit/project.json");
    let input = serde_json::json!({
        "hook_event_name": "PreToolUse",
        "tool_name": "Write",
        "cwd": dir.path(),
        "tool_input": {"file_path": target, "content": "{}"},
    });
    let out = run_hook_command("PreToolUse", dir.path(), &input.to_string());
    assert_eq!(out.status.code(), Some(2), "{out:?}");
    assert!(!out.stderr.is_empty());
}

#[test]
fn runtime_skill_carries_the_spec_text_and_stays_short() {
    let text = read("skills/clipkit-video/SKILL.md");
    let (fields, body) = frontmatter(&text);
    assert_eq!(field(&fields, "name"), "clipkit-video");
    assert!(!field(&fields, "description").is_empty());
    assert!(
        text.lines().count() < SKILL_LINES,
        "{} lines",
        text.lines().count()
    );
    let lines = spec_skill_lines();
    assert!(lines.len() > 10, "spec 9.4 block not found");
    // Spec 9.4 names commands that were never built; the shipped lines use the real ones
    // (DECISIONS.md, end-to-end pipeline audit).
    const CORRECTED: &[&str] = &[
        "ck pipeline detect",
        "ck footage",
        "ck see",
        "ck status idle",
        "ck doctor",
    ];
    for line in lines
        .iter()
        .filter(|l| !CORRECTED.iter().any(|c| l.contains(c)))
    {
        assert!(body.contains(line.as_str()), "SKILL.md lacks: {line}");
    }
    assert!(!has_date(&text));
}

#[test]
fn runtime_skill_references_exist_and_are_small() {
    let text = read("skills/clipkit-video/SKILL.md");
    for name in ["pipeline.md", "guardrails.md", "voice.md"] {
        assert!(
            text.contains(&format!("]({name})")),
            "SKILL.md links {name}"
        );
        let reference = read(&format!("skills/clipkit-video/{name}"));
        assert!(reference.lines().count() < SKILL_LINES, "{name}");
        assert!(!has_date(&reference), "{name}");
    }
}

#[test]
fn review_skill_has_frontmatter() {
    let text = read("skills/clipkit-review/SKILL.md");
    let (fields, body) = frontmatter(&text);
    assert_eq!(field(&fields, "name"), "clipkit-review");
    assert!(field(&fields, "description").contains("ck wait"));
    assert!(body.contains("ck wait") && body.contains("ck brief diff"));
    assert!(text.lines().count() < SKILL_LINES);
    assert!(!has_date(&text));
}

#[test]
fn critic_agent_is_read_only_and_answers_in_ten_lines() {
    let text = read("agents/clipkit-critic.md");
    let (fields, body) = frontmatter(&text);
    assert_eq!(field(&fields, "name"), "clipkit-critic");
    assert!(!field(&fields, "description").is_empty());
    assert_eq!(field(&fields, "tools"), "Read");
    assert!(body.contains("at most ten lines"));
}

#[test]
fn mcp_json_registers_ck_mcp_on_the_project_folder() {
    let file = json(".mcp.json");
    let server = &file["mcpServers"]["clipkit"];
    assert_eq!(server["command"], "ck");
    assert_eq!(
        server["args"],
        serde_json::json!(["mcp", "--dir", "${CLAUDE_PROJECT_DIR}"])
    );
}
