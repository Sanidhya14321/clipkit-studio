//! Claude Code: writes the plugin as a local marketplace under `~/.clipkit/claude-plugin` and adds
//! the marketplace and the enabled plugin to the user's Claude Code settings.json (spec 6.1, 6.3).

use std::path::Path;

use ck_core::{CkError, CkResult, ErrorCode};
use serde_json::{Map, Value, json};

use super::{Homes, Mode, Step, file_step, owned_dir_step, read_opt};

/// Marketplace and plugin name; the plugin id in settings is `clipkit@clipkit`.
const NAME: &str = "clipkit";
const PLUGIN_ID: &str = "clipkit@clipkit";

/// The shipped plugin (plugin/, README excluded), embedded so the binary carries it.
pub const PLUGIN_FILES: &[(&str, &str)] = &[
    (
        ".claude-plugin/plugin.json",
        include_str!("../../../../plugin/.claude-plugin/plugin.json"),
    ),
    (
        "hooks/hooks.json",
        include_str!("../../../../plugin/hooks/hooks.json"),
    ),
    (
        "skills/clipkit-video/SKILL.md",
        include_str!("../../../../plugin/skills/clipkit-video/SKILL.md"),
    ),
    (
        "skills/clipkit-video/pipeline.md",
        include_str!("../../../../plugin/skills/clipkit-video/pipeline.md"),
    ),
    (
        "skills/clipkit-video/guardrails.md",
        include_str!("../../../../plugin/skills/clipkit-video/guardrails.md"),
    ),
    (
        "skills/clipkit-video/voice.md",
        include_str!("../../../../plugin/skills/clipkit-video/voice.md"),
    ),
    (
        "skills/clipkit-review/SKILL.md",
        include_str!("../../../../plugin/skills/clipkit-review/SKILL.md"),
    ),
    (
        "agents/clipkit-critic.md",
        include_str!("../../../../plugin/agents/clipkit-critic.md"),
    ),
];

/// The plugin's `.mcp.json`. It is not in plugin/ yet (a headless run may not write it; see
/// DECISIONS.md, Phase 3 step 4), so the installed plugin gets it from here.
pub const MCP_JSON: &str = r#"{
  "mcpServers": {
    "clipkit": {
      "command": "ck",
      "args": ["mcp", "--dir", "${CLAUDE_PROJECT_DIR}"]
    }
  }
}
"#;

pub fn plan(mode: Mode, homes: &Homes) -> CkResult<Vec<Step>> {
    let root = homes.clipkit.join("claude-plugin");
    Ok(vec![
        // The marketplace folder is ClipKit's own.
        owned_dir_step(mode, &root, marketplace_files(&root))?,
        settings_step(mode, &root, &homes.claude.join("settings.json"))?,
    ])
}

fn marketplace_json() -> String {
    let manifest = json!({
        "name": NAME,
        "owner": { "name": "ClipKit Studio" },
        "plugins": [{
            "name": NAME,
            "source": format!("./{NAME}"),
            "description": "Edit and generate video in a live ClipKit studio.",
        }],
    });
    format!("{manifest:#}\n")
}

/// Every file of the marketplace folder `root`, with its content.
fn marketplace_files(root: &Path) -> Vec<(std::path::PathBuf, String)> {
    let plugin = root.join(NAME);
    let mut files = vec![(
        root.join(".claude-plugin").join("marketplace.json"),
        marketplace_json(),
    )];
    files.extend(
        PLUGIN_FILES
            .iter()
            .map(|(rel, text)| (plugin.join(rel), (*text).to_owned())),
    );
    files.push((plugin.join(".mcp.json"), MCP_JSON.to_owned()));
    files
}

/// Add or remove `extraKnownMarketplaces.clipkit` and `enabledPlugins["clipkit@clipkit"]`. Other
/// values stay as they are; the file is rewritten (pretty, keys sorted) only when one of the two
/// keys changes. A `clipkit` marketplace pointing anywhere else is the user's: init refuses it and
/// uninstall leaves it, with its enabled flag, alone.
fn settings_step(mode: Mode, root: &Path, path: &Path) -> CkResult<Step> {
    let root = root.to_str().ok_or_else(|| {
        CkError::new(
            ErrorCode::Request,
            format!("{} is not valid UTF-8", root.display()),
        )
        .with_hint("Set HOME to a folder whose path is valid UTF-8.")
    })?;
    let ours = json!({ "source": { "source": "directory", "path": root } });
    let have = read_opt(path)?;
    let parsed: Result<Value, _> = have.as_deref().map_or(Ok(json!({})), serde_json::from_str);
    let mut value = match (parsed, mode) {
        (Ok(v), _) if v.is_object() => v,
        // Uninstall cannot have written a file it cannot read: nothing of ours is in it.
        (_, Mode::Uninstall) => return Ok(Step::unchanged(path)),
        (Err(e), Mode::Init) => {
            return Err(bad_settings(path, &format!("is not valid JSON: {e}")));
        }
        (Ok(_), Mode::Init) => return Err(bad_settings(path, "is not a JSON object")),
    };
    let Some(obj) = value.as_object_mut() else {
        return Ok(Step::unchanged(path));
    };
    let before = obj.clone();
    let market = obj.get("extraKnownMarketplaces").and_then(|m| m.get(NAME));
    match mode {
        Mode::Init if market.is_some_and(|m| *m != ours) => {
            return Err(bad_settings(
                path,
                "already has another marketplace named clipkit",
            ));
        }
        Mode::Init => {
            object(obj, "extraKnownMarketplaces", path)?.insert(NAME.to_owned(), ours);
            object(obj, "enabledPlugins", path)?.insert(PLUGIN_ID.to_owned(), json!(true));
        }
        Mode::Uninstall if market.is_some_and(|m| *m != ours) => {}
        Mode::Uninstall => {
            remove_key(obj, "extraKnownMarketplaces", NAME);
            remove_key(obj, "enabledPlugins", PLUGIN_ID);
        }
    }
    if *obj == before {
        return Ok(Step::unchanged(path));
    }
    // Uninstall of a file that held only ClipKit's keys removes the file.
    let want = (!obj.is_empty()).then(|| format!("{value:#}\n"));
    file_step(path, want)
}

fn object<'a>(
    obj: &'a mut Map<String, Value>,
    key: &str,
    path: &Path,
) -> CkResult<&'a mut Map<String, Value>> {
    obj.entry(key)
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| bad_settings(path, &format!("has a {key} that is not an object")))
}

/// Remove `outer.inner`, and `outer` too when that leaves it empty.
fn remove_key(obj: &mut Map<String, Value>, outer: &str, inner: &str) {
    if let Some(map) = obj.get_mut(outer).and_then(Value::as_object_mut) {
        map.remove(inner);
        if map.is_empty() {
            obj.remove(outer);
        }
    }
}

fn bad_settings(path: &Path, what: &str) -> CkError {
    CkError::new(ErrorCode::Request, format!("{} {what}", path.display()))
        .with_hint("Fix the settings file or leave Claude Code out (--agent codex).")
}
