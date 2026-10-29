//! Codex: registers `ck mcp` in `$CODEX_HOME/config.toml` and adds the ClipKit section to
//! `$CODEX_HOME/AGENTS.md`, each between marker lines (spec 6.3). Codex has no hooks; the first ck
//! call starts the studio.

use std::path::Path;

use ck_core::{CkError, CkResult, ErrorCode};

use super::marked::{self, Markers};
use super::{Homes, Mode, Step, file_step, read_opt};

/// The texts in adapters/codex/, embedded so the binary carries them.
const CONFIG_BLOCK: &str = include_str!("../../../../adapters/codex/config.toml");
const AGENTS_SECTION: &str = include_str!("../../../../adapters/codex/AGENTS.md");

const TOML: Markers = Markers {
    begin: "# clipkit:begin (added by ck init; ck uninstall removes it)",
    end: "# clipkit:end",
};
const MARKDOWN: Markers = Markers {
    begin: "<!-- clipkit:begin (added by ck init; ck uninstall removes it) -->",
    end: "<!-- clipkit:end -->",
};
/// The table the block defines; one outside the block is the user's and is never duplicated.
const TABLE: &str = "[mcp_servers.clipkit]";

pub fn plan(mode: Mode, homes: &Homes) -> CkResult<Vec<Step>> {
    Ok(vec![
        marked_step(
            mode,
            &homes.codex.join("config.toml"),
            &TOML,
            CONFIG_BLOCK,
            true,
        )?,
        marked_step(
            mode,
            &homes.codex.join("AGENTS.md"),
            &MARKDOWN,
            AGENTS_SECTION,
            false,
        )?,
    ])
}

/// Add (init) or cut out (uninstall) the block in `path`. A file left empty is removed.
fn marked_step(mode: Mode, path: &Path, m: &Markers, body: &str, conflict: bool) -> CkResult<Step> {
    let have = read_opt(path)?.unwrap_or_default();
    let want = match mode {
        Mode::Init => {
            let outside = marked::remove(&have, m)?;
            if conflict && defines_clipkit(&outside) {
                return Err(CkError::new(
                    ErrorCode::Request,
                    format!("{} already defines {TABLE}", path.display()),
                )
                .with_hint("Remove that table or leave Codex out (--agent claude-code)."));
            }
            marked::upsert(&have, m, body)?
        }
        Mode::Uninstall if !marked::contains(&have, m)? => return Ok(Step::unchanged(path)),
        Mode::Uninstall => marked::remove(&have, m)?,
    };
    file_step(path, (!want.is_empty()).then_some(want))
}

/// Whether the user's TOML already defines `mcp_servers.clipkit` in any spelling: a table
/// header (quoted, spaced, commented or a sub-table), a dotted key, or a key of `[mcp_servers]`.
/// A line scan, not a parser; it errs towards refusing.
fn defines_clipkit(toml: &str) -> bool {
    let mut table = String::new();
    for line in toml.lines() {
        let code = line.split('#').next().unwrap_or_default();
        let bare: String = code
            .chars()
            .filter(|c| !c.is_whitespace() && *c != '"' && *c != '\'')
            .collect();
        if bare.starts_with('[') {
            table = bare.trim_matches(|c| c == '[' || c == ']').to_owned();
            if table == "mcp_servers.clipkit" || table.starts_with("mcp_servers.clipkit.") {
                return true;
            }
            continue;
        }
        let key = bare.split('=').next().unwrap_or_default();
        let full = if table.is_empty() {
            key.to_owned()
        } else {
            format!("{table}.{key}")
        };
        if full == "mcp_servers.clipkit"
            || full.starts_with("mcp_servers.clipkit.")
            || (full == "mcp_servers" && bare.contains("clipkit"))
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::defines_clipkit;

    #[test]
    fn finds_the_table_in_any_spelling() {
        for toml in [
            "[mcp_servers.clipkit]\n",
            "[ mcp_servers.clipkit ] # mine\n",
            "[mcp_servers.\"clipkit\"]\n",
            "[mcp_servers.clipkit.env]\n",
            "mcp_servers.clipkit.command = \"x\"\n",
            "[mcp_servers]\nclipkit = { command = \"x\" }\n",
            "mcp_servers = { clipkit = { command = \"x\" } }\n",
        ] {
            assert!(defines_clipkit(toml), "{toml}");
        }
        for toml in [
            "",
            "[mcp_servers.other]\ncommand = \"clipkit\"\n",
            "# [mcp_servers.clipkit]\n",
            "[mcp_servers.clipkit2]\n",
        ] {
            assert!(!defines_clipkit(toml), "{toml}");
        }
    }
}
