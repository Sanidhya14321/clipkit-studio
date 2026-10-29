//! Cursor: registers `ck mcp` under `mcpServers.clipkit` in `~/.cursor/mcp.json` and writes the
//! rules file `.cursor/rules/clipkit.mdc` in the project folder (`--dir`), since Cursor keeps user
//! rules in its settings, not in a file (spec 6.3; format UNVERIFIED, DECISIONS.md).

use std::path::Path;

use ck_core::{CkError, CkResult, ErrorCode};
use serde_json::{Value, json};

use super::{Homes, Mode, Step, file_step, read_opt};

const FRONTMATTER: &str = include_str!("../../../../adapters/cursor/frontmatter.mdc");
const SECTION: &str = include_str!("../../../../adapters/codex/AGENTS.md");
const NAME: &str = "clipkit";

pub fn rules() -> String {
    format!("{FRONTMATTER}{SECTION}")
}

pub fn plan(mode: Mode, homes: &Homes, dir: &Path) -> CkResult<Vec<Step>> {
    // Rules first: uninstall then removes rules/ before .cursor/ is tried.
    let rules_path = dir.join(".cursor").join("rules").join("clipkit.mdc");
    let rules_step = match mode {
        Mode::Init => file_step(&rules_path, Some(rules()))?,
        // Only our own text is removed; a changed file is the user's now.
        Mode::Uninstall if read_opt(&rules_path)?.as_deref() == Some(rules().as_str()) => {
            file_step(&rules_path, None)?
        }
        Mode::Uninstall => Step::unchanged(&rules_path),
    };
    Ok(vec![
        rules_step,
        mcp_step(mode, &homes.cursor.join("mcp.json"))?,
    ])
}

/// Add or remove `mcpServers.clipkit`. A `clipkit` server that is not ours is the user's: init
/// refuses it and uninstall leaves it alone. A file left empty is removed.
fn mcp_step(mode: Mode, path: &Path) -> CkResult<Step> {
    let ours = json!({ "command": "ck", "args": ["mcp"] });
    let have = read_opt(path)?;
    let parsed: Result<Value, _> = have.as_deref().map_or(Ok(json!({})), serde_json::from_str);
    let mut value = match (parsed, mode) {
        (Ok(v), _) if v.is_object() => v,
        (_, Mode::Uninstall) => return Ok(Step::unchanged(path)),
        (Err(e), Mode::Init) => return Err(bad(path, &format!("is not valid JSON: {e}"))),
        (Ok(_), Mode::Init) => return Err(bad(path, "is not a JSON object")),
    };
    let Some(obj) = value.as_object_mut() else {
        return Ok(Step::unchanged(path));
    };
    let before = obj.clone();
    let current = obj.get("mcpServers").and_then(|m| m.get(NAME)).cloned();
    match mode {
        Mode::Init if current.as_ref().is_some_and(|c| *c != ours) => {
            return Err(bad(path, "already has another MCP server named clipkit"));
        }
        Mode::Init => {
            obj.entry("mcpServers")
                .or_insert_with(|| json!({}))
                .as_object_mut()
                .ok_or_else(|| bad(path, "has an mcpServers that is not an object"))?
                .insert(NAME.to_owned(), ours);
        }
        Mode::Uninstall if current.as_ref() != Some(&ours) => {}
        Mode::Uninstall => {
            if let Some(map) = obj.get_mut("mcpServers").and_then(Value::as_object_mut) {
                map.remove(NAME);
                if map.is_empty() {
                    obj.remove("mcpServers");
                }
            }
        }
    }
    if *obj == before {
        return Ok(Step::unchanged(path));
    }
    file_step(path, (!obj.is_empty()).then(|| format!("{value:#}\n")))
}

fn bad(path: &Path, what: &str) -> CkError {
    CkError::new(ErrorCode::Request, format!("{} {what}", path.display()))
        .with_hint("Fix that file or leave Cursor out (--agent claude-code, codex or mcp).")
}
