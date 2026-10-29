//! `ck init` / `ck uninstall` end to end against a fake home folder (spec 6.3): detect, plan
//! without writing unless confirmed, idempotent, user content kept, no residue.
#![allow(clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{Value, json};

/// The output is a fixed list of at most four paths.
const CAP: usize = 2048;

struct Home {
    tmp: tempfile::TempDir,
    /// An empty PATH folder, so the machine's real claude or codex is never detected.
    bin: PathBuf,
}

impl Home {
    fn new() -> Self {
        let tmp = tempfile::tempdir().unwrap();
        let bin = tmp.path().join("bin");
        std::fs::create_dir(&bin).unwrap();
        let home = tmp.path().join("home");
        std::fs::create_dir(&home).unwrap();
        Self { tmp, bin }
    }

    /// A home where both agents have their config folder.
    fn with_agents() -> Self {
        let h = Self::new();
        std::fs::create_dir(h.path(".claude")).unwrap();
        std::fs::create_dir(h.path(".codex")).unwrap();
        h
    }

    fn home(&self) -> PathBuf {
        self.tmp.path().join("home")
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.home().join(rel)
    }

    fn write(&self, rel: &str, text: &str) {
        let p = self.path(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, text).unwrap();
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.path(rel)).unwrap()
    }

    fn ck(&self, args: &[&str], env: &[(&str, &Path)]) -> Output {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_ck"));
        cmd.args(args)
            .env("HOME", self.home())
            .env("PATH", &self.bin)
            .env_remove("CLAUDE_CONFIG_DIR")
            .env_remove("CODEX_HOME")
            .stdin(Stdio::null());
        for (k, v) in env {
            cmd.env(k, v);
        }
        cmd.output().unwrap()
    }

    /// A command that must succeed: its one line of JSON, within the cap.
    fn ok(&self, args: &[&str]) -> Value {
        self.ok_env(args, &[])
    }

    fn ok_env(&self, args: &[&str], env: &[(&str, &Path)]) -> Value {
        let out = self.ck(args, env);
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(out.status.success(), "ck {args:?} failed: {stderr}");
        let stdout = String::from_utf8(out.stdout).unwrap();
        assert_eq!(stdout.lines().count(), 1);
        assert!(stdout.len() <= CAP + 1, "ck {args:?} over its cap");
        serde_json::from_str(&stdout).unwrap()
    }

    fn err(&self, args: &[&str]) -> Value {
        let out = self.ck(args, &[]);
        assert_eq!(out.status.code(), Some(1), "ck {args:?} should fail");
        assert!(out.stdout.is_empty());
        serde_json::from_slice(&out.stderr).unwrap()
    }

    /// Every folder and file under the home folder, with file bytes.
    fn tree(&self) -> BTreeMap<String, Option<Vec<u8>>> {
        let mut out = BTreeMap::new();
        walk(&self.home(), &self.home(), &mut out);
        out
    }
}

fn walk(root: &Path, dir: &Path, out: &mut BTreeMap<String, Option<Vec<u8>>>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let p = entry.unwrap().path();
        let rel = p.strip_prefix(root).unwrap().display().to_string();
        if p.is_dir() {
            out.insert(rel, None);
            walk(root, &p, out);
        } else {
            out.insert(rel, Some(std::fs::read(&p).unwrap()));
        }
    }
}

/// `[(agent, [(path relative to home, change)])]` from a report.
fn changes(h: &Home, report: &Value) -> Vec<(String, Vec<(String, String)>)> {
    let home = h.home().display().to_string();
    report["agents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| {
            let steps = a["steps"].as_array().unwrap().iter().map(|s| {
                let p = s["path"].as_str().unwrap();
                let rel = p.strip_prefix(&home).unwrap().trim_start_matches('/');
                (rel.to_owned(), s["change"].as_str().unwrap().to_owned())
            });
            (a["agent"].as_str().unwrap().to_owned(), steps.collect())
        })
        .collect()
}

fn all(change: &str) -> Vec<(String, Vec<(String, String)>)> {
    vec![
        (
            "claude-code".into(),
            vec![
                (".clipkit/claude-plugin".into(), change.into()),
                (".claude/settings.json".into(), change.into()),
            ],
        ),
        (
            "codex".into(),
            vec![
                (".codex/config.toml".into(), change.into()),
                (".codex/AGENTS.md".into(), change.into()),
            ],
        ),
    ]
}

#[test]
fn no_agent_found_is_refused() {
    let h = Home::new();
    let e = h.err(&["init", "--yes"]);
    assert_eq!(e["code"], "E_REQUEST");
    assert!(e["hint"].as_str().unwrap().contains("--agent"));
    assert_eq!(h.tree(), BTreeMap::new());
}

#[test]
fn without_yes_and_a_terminal_only_the_plan_is_printed() {
    let h = Home::with_agents();
    let before = h.tree();
    let r = h.ok(&["init"]);
    assert_eq!(r["command"], "init");
    assert_eq!(r["applied"], false);
    assert!(r["hint"].as_str().unwrap().contains("--yes"));
    assert_eq!(changes(&h, &r), all("create"));
    assert_eq!(h.tree(), before, "nothing written without consent");
}

#[test]
fn init_writes_both_agents_and_is_idempotent() {
    let h = Home::with_agents();
    let r = h.ok(&["init", "--yes"]);
    assert_eq!(r["applied"], true);
    assert!(r.get("hint").is_none());
    assert_eq!(changes(&h, &r), all("create"));

    // Claude Code: a local marketplace holding the plugin, known and enabled in settings.
    let root = h.path(".clipkit/claude-plugin");
    let settings: Value = serde_json::from_str(&h.read(".claude/settings.json")).unwrap();
    assert_eq!(
        settings,
        json!({
            "extraKnownMarketplaces": { "clipkit": { "source": {
                "source": "directory", "path": root.display().to_string() } } },
            "enabledPlugins": { "clipkit@clipkit": true },
        })
    );
    let market: Value =
        serde_json::from_str(&h.read(".clipkit/claude-plugin/.claude-plugin/marketplace.json"))
            .unwrap();
    assert_eq!(market["name"], "clipkit");
    assert_eq!(market["plugins"][0]["source"], "./clipkit");
    let mcp: Value =
        serde_json::from_str(&h.read(".clipkit/claude-plugin/clipkit/.mcp.json")).unwrap();
    assert_eq!(
        mcp["mcpServers"]["clipkit"],
        json!({ "command": "ck", "args": ["mcp", "--dir", "${CLAUDE_PROJECT_DIR}"] })
    );

    // Codex: the MCP table and the AGENTS.md section, each between markers.
    let config = h.read(".codex/config.toml");
    assert!(config.starts_with("# clipkit:begin"));
    assert!(config.contains("[mcp_servers.clipkit]\ncommand = \"ck\"\nargs = [\"mcp\"]\n"));
    assert!(config.ends_with("# clipkit:end\n"));
    let agents = h.read(".codex/AGENTS.md");
    assert!(agents.starts_with("<!-- clipkit:begin"));
    assert!(agents.contains("## ClipKit video"));
    assert!(agents.ends_with("<!-- clipkit:end -->\n"));

    let after = h.tree();
    let again = h.ok(&["init", "--yes"]);
    assert_eq!(again["applied"], false);
    assert!(
        again.get("hint").is_none(),
        "nothing pending, nothing to confirm"
    );
    assert_eq!(changes(&h, &again), all("unchanged"));
    assert_eq!(h.tree(), after, "a second init writes nothing");
}

#[test]
fn the_installed_plugin_is_the_shipped_plugin_plus_mcp_json() {
    let h = Home::with_agents();
    h.ok(&["init", "--yes", "--agent", "claude-code"]);
    let shipped = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../plugin");
    let mut want = BTreeMap::new();
    walk(&shipped, &shipped, &mut want);
    want.remove("README.md");
    let installed = h.path(".clipkit/claude-plugin/clipkit");
    let mut got = BTreeMap::new();
    walk(&installed, &installed, &mut got);
    assert!(got.remove(".mcp.json").is_some());
    // plugin/.mcp.json may exist once an interactive session writes it (DECISIONS.md).
    want.remove(".mcp.json");
    assert_eq!(
        got, want,
        "add new plugin files to PLUGIN_FILES in install/claude.rs"
    );
}

#[test]
fn user_content_is_kept_in_place_and_uninstall_restores_it_byte_for_byte() {
    let h = Home::with_agents();
    // A rewritten settings.json is pretty with sorted keys; this one already is, so the bytes
    // come back exactly (other layouts come back equal as JSON).
    let settings = json!({
        "enabledPlugins": { "other@market": true },
        "model": "opus",
        "permissions": { "allow": ["Bash(ls)"] },
    });
    h.write(".claude/settings.json", &format!("{settings:#}\n"));
    h.write(
        ".codex/config.toml",
        "model = \"o4\"\n\n[mcp_servers.other]\ncommand = \"other\"\n",
    );
    h.write(".codex/AGENTS.md", "# My rules\n\nBe brief.\n");
    let before = h.tree();

    h.ok(&["init", "--yes"]);
    let s: Value = serde_json::from_str(&h.read(".claude/settings.json")).unwrap();
    assert_eq!(s["model"], "opus");
    assert_eq!(s["permissions"], settings["permissions"]);
    assert_eq!(s["enabledPlugins"]["other@market"], true);
    assert_eq!(s["enabledPlugins"]["clipkit@clipkit"], true);
    assert!(h.read(".codex/config.toml").starts_with(
        "model = \"o4\"\n\n[mcp_servers.other]\ncommand = \"other\"\n# clipkit:begin"
    ));
    assert!(
        h.read(".codex/AGENTS.md")
            .starts_with("# My rules\n\nBe brief.\n<!-- clipkit:begin")
    );

    let r = h.ok(&["uninstall", "--yes"]);
    assert_eq!(r["command"], "uninstall");
    assert_eq!(r["applied"], true);
    let mut want = all("update");
    want[0].1[0].1 = "remove".into();
    assert_eq!(changes(&h, &r), want);
    assert_eq!(h.tree(), before, "uninstall leaves no residue");

    let again = h.ok(&["uninstall", "--yes"]);
    assert_eq!(changes(&h, &again), all("unchanged"));
}

#[test]
fn uninstall_on_a_clean_machine_leaves_no_residue() {
    let h = Home::new();
    // Both agents found only on PATH: their config folders do not exist yet.
    for name in ["claude", "codex"] {
        use std::os::unix::fs::PermissionsExt;
        let p = h.bin.join(name);
        std::fs::write(&p, "#!/bin/sh\n").unwrap();
        std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o755)).unwrap();
    }
    let r = h.ok(&["init", "--yes"]);
    assert_eq!(changes(&h, &r), all("create"));
    let r = h.ok(&["uninstall"]);
    assert_eq!(r["applied"], false, "uninstall asks too");
    assert!(h.path(".clipkit").exists());
    h.ok(&["uninstall", "--yes"]);
    assert_eq!(h.tree(), BTreeMap::new());
}

#[test]
fn agent_flag_limits_the_work_to_that_agent() {
    let h = Home::with_agents();
    let r = h.ok(&["init", "--yes", "--agent", "codex"]);
    assert_eq!(changes(&h, &r), vec![all("create").remove(1)]);
    assert!(!h.path(".clipkit").exists());
    assert!(!h.path(".claude/settings.json").exists());
}

#[test]
fn claude_config_dir_and_codex_home_are_honoured() {
    let h = Home::new();
    let claude = h.tmp.path().join("cc");
    let codex = h.tmp.path().join("cx");
    std::fs::create_dir(&claude).unwrap();
    std::fs::create_dir(&codex).unwrap();
    let env = [
        ("CLAUDE_CONFIG_DIR", claude.as_path()),
        ("CODEX_HOME", codex.as_path()),
    ];
    h.ok_env(&["init", "--yes"], &env);
    assert!(claude.join("settings.json").is_file());
    assert!(codex.join("config.toml").is_file());
    assert!(codex.join("AGENTS.md").is_file());
    assert!(!h.path(".claude").exists());
    h.ok_env(&["uninstall", "--yes"], &env);
    assert!(!claude.join("settings.json").exists());
    assert!(!codex.join("config.toml").exists());
}

#[test]
fn a_codex_table_of_the_users_own_is_refused_and_nothing_is_written() {
    let h = Home::with_agents();
    h.write(
        ".codex/config.toml",
        "[mcp_servers.clipkit]\ncommand = \"mine\"\n",
    );
    let before = h.tree();
    let e = h.err(&["init", "--yes"]);
    assert_eq!(e["code"], "E_REQUEST");
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("[mcp_servers.clipkit]")
    );
    assert_eq!(h.tree(), before);
}

#[test]
fn malformed_settings_json_is_refused_and_nothing_is_written() {
    let h = Home::with_agents();
    h.write(".claude/settings.json", "{ not json");
    let before = h.tree();
    let e = h.err(&["init", "--yes"]);
    assert_eq!(e["code"], "E_REQUEST");
    assert!(e["hint"].as_str().unwrap().contains("--agent codex"));
    assert_eq!(h.tree(), before);
}

#[test]
fn init_never_starts_a_studio_or_creates_a_project() {
    let h = Home::with_agents();
    let project = h.tmp.path().join("project");
    std::fs::create_dir(&project).unwrap();
    let dir = project.display().to_string();
    h.ok(&["init", "--yes", "--dir", &dir]);
    h.ok(&["uninstall", "--yes", "--dir", &dir]);
    assert_eq!(std::fs::read_dir(&project).unwrap().count(), 0);
}

#[test]
fn a_clipkit_marketplace_of_the_users_own_is_refused_and_kept_by_uninstall() {
    let h = Home::with_agents();
    let settings = json!({
        "enabledPlugins": { "clipkit@clipkit": true },
        "extraKnownMarketplaces": {
            "clipkit": { "source": { "source": "github", "repo": "me/ck" } }
        },
    });
    h.write(".claude/settings.json", &format!("{settings:#}\n"));
    let before = h.tree();
    let e = h.err(&["init", "--yes"]);
    assert_eq!(e["code"], "E_REQUEST");
    assert!(
        e["message"]
            .as_str()
            .unwrap()
            .contains("another marketplace")
    );
    let r = h.ok(&["uninstall", "--yes", "--agent", "claude-code"]);
    assert_eq!(r["applied"], false);
    assert_eq!(h.tree(), before);
}

#[test]
fn unreadable_settings_do_not_block_uninstall() {
    let h = Home::with_agents();
    h.ok(&["init", "--yes", "--agent", "codex"]);
    let jsonc = "{ // a comment\n}\n";
    h.write(".claude/settings.json", jsonc);
    let r = h.ok(&["uninstall", "--yes"]);
    assert_eq!(r["applied"], true);
    assert_eq!(h.read(".claude/settings.json"), jsonc);
    assert!(!h.path(".codex/config.toml").exists());
}

#[test]
fn symlinked_files_stay_symlinks_and_keep_their_mode() {
    use std::os::unix::fs::PermissionsExt;
    let h = Home::with_agents();
    let real = h.tmp.path().join("dotfiles-config.toml");
    std::fs::write(&real, "model = \"o4\"\n").unwrap();
    std::fs::set_permissions(&real, std::fs::Permissions::from_mode(0o600)).unwrap();
    let link = h.path(".codex/config.toml");
    std::os::unix::fs::symlink(&real, &link).unwrap();
    h.ok(&["init", "--yes", "--agent", "codex"]);
    let meta = std::fs::symlink_metadata(&link).unwrap();
    assert!(meta.file_type().is_symlink());
    let text = std::fs::read_to_string(&real).unwrap();
    assert!(text.contains("[mcp_servers.clipkit]"));
    let mode = std::fs::metadata(&real).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    h.ok(&["uninstall", "--yes", "--agent", "codex"]);
    assert_eq!(std::fs::read_to_string(&real).unwrap(), "model = \"o4\"\n");
}
