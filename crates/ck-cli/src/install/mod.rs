//! `ck init` / `ck uninstall`: set ClipKit up in Claude Code and Codex (spec 6.3). Init finds the
//! installed agents, shows its plan and asks before writing anything; it is idempotent, and
//! uninstall removes everything init added. Without `--yes` and without a terminal to ask on,
//! nothing is written and the plan is the output.

mod claude;
mod codex;
mod marked;

use std::fs;
use std::io::{BufRead, IsTerminal, Write};
use std::path::{Path, PathBuf};

use ck_core::storage::atomic_write;
use ck_core::{CkError, CkResult, ErrorCode};
use serde::Serialize;

use crate::args::{AgentKind, InstallArgs};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Init,
    Uninstall,
}

/// What a step does to its path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Change {
    Create,
    Update,
    Remove,
    Unchanged,
}

enum Op {
    Write(PathBuf, String),
    Remove(PathBuf),
    RemoveDir(PathBuf),
}

/// One reported path and the file operations behind it.
pub struct Step {
    path: PathBuf,
    change: Change,
    ops: Vec<Op>,
}

impl Step {
    fn unchanged(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            change: Change::Unchanged,
            ops: Vec::new(),
        }
    }
}

/// Where each agent and ClipKit keep their user-level files.
pub struct Homes {
    clipkit: PathBuf,
    claude: PathBuf,
    codex: PathBuf,
}

impl Homes {
    /// `~/.clipkit`, `$CLAUDE_CONFIG_DIR` or `~/.claude`, `$CODEX_HOME` or `~/.codex`.
    fn from_env() -> CkResult<Self> {
        let home = env_path("HOME")
            .or_else(|| env_path("USERPROFILE"))
            .ok_or_else(|| {
                CkError::new(ErrorCode::Request, "no home folder: HOME is not set")
                    .with_hint("Set HOME to your home folder and run the command again.")
            })?;
        Ok(Self {
            clipkit: home.join(".clipkit"),
            claude: env_path("CLAUDE_CONFIG_DIR").unwrap_or_else(|| home.join(".claude")),
            codex: env_path("CODEX_HOME").unwrap_or_else(|| home.join(".codex")),
        })
    }
}

fn env_path(key: &str) -> Option<PathBuf> {
    std::env::var_os(key)
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
}

#[derive(Serialize)]
struct StepView {
    path: String,
    change: Change,
}

#[derive(Serialize)]
struct AgentView {
    agent: AgentKind,
    steps: Vec<StepView>,
}

#[derive(Serialize)]
struct Report {
    command: &'static str,
    applied: bool,
    agents: Vec<AgentView>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<&'static str>,
}

const ALL: [AgentKind; 2] = [AgentKind::ClaudeCode, AgentKind::Codex];

pub fn run(mode: Mode, args: &InstallArgs) -> CkResult<()> {
    let homes = Homes::from_env()?;
    let agents: Vec<AgentKind> = if !args.agents.is_empty() {
        ALL.into_iter()
            .filter(|a| args.agents.contains(a))
            .collect()
    } else if mode == Mode::Uninstall {
        // Uninstall looks only for what init added, so it checks every agent.
        ALL.to_vec()
    } else {
        ALL.into_iter().filter(|a| detected(*a, &homes)).collect()
    };
    if agents.is_empty() {
        return Err(CkError::new(
            ErrorCode::Request,
            "found no supported agent (Claude Code or Codex)",
        )
        .with_hint("Install one, or name it with --agent claude-code or --agent codex."));
    }
    let mut plan = Vec::new();
    for agent in agents {
        let steps = match agent {
            AgentKind::ClaudeCode => claude::plan(mode, &homes)?,
            AgentKind::Codex => codex::plan(mode, &homes)?,
        };
        plan.push((agent, steps));
    }
    let pending = plan
        .iter()
        .flat_map(|(_, steps)| steps)
        .any(|s| s.change != Change::Unchanged);
    let applied = pending && (args.yes || confirm(&plan));
    if applied {
        for step in plan.iter().flat_map(|(_, steps)| steps) {
            for op in &step.ops {
                apply(op)?;
            }
        }
        // A folder whose last file uninstall removed goes too (remove_dir refuses a non-empty
        // one), so init's folders leave no residue and no other folder is touched.
        let removed = plan.iter().flat_map(|(_, steps)| steps);
        for step in removed.filter(|s| s.change == Change::Remove) {
            if let Some(dir) = step.path.parent() {
                let _ = fs::remove_dir(dir);
            }
        }
    }
    crate::print(&Report {
        command: match mode {
            Mode::Init => "init",
            Mode::Uninstall => "uninstall",
        },
        applied,
        agents: plan
            .into_iter()
            .map(|(agent, steps)| AgentView {
                agent,
                steps: steps
                    .into_iter()
                    .map(|s| StepView {
                        path: s.path.display().to_string(),
                        change: s.change,
                    })
                    .collect(),
            })
            .collect(),
        hint: (pending && !applied)
            .then_some("Nothing was written. Run again with --yes to apply."),
    })
}

/// Agent present: its config folder exists or its command is on PATH.
fn detected(agent: AgentKind, homes: &Homes) -> bool {
    let (dir, bin) = match agent {
        AgentKind::ClaudeCode => (&homes.claude, "claude"),
        AgentKind::Codex => (&homes.codex, "codex"),
    };
    dir.is_dir() || on_path(bin)
}

fn on_path(bin: &str) -> bool {
    let Some(path) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&path).any(|d| {
        let p = d.join(bin);
        p.is_file() || p.with_extension("exe").is_file()
    })
}

/// Show the plan on stderr and ask; no terminal means no.
fn confirm(plan: &[(AgentKind, Vec<Step>)]) -> bool {
    let stdin = std::io::stdin();
    if !stdin.is_terminal() {
        return false;
    }
    let mut err = std::io::stderr().lock();
    for (agent, steps) in plan {
        for s in steps.iter().filter(|s| s.change != Change::Unchanged) {
            let _ = writeln!(err, "{agent:?}: {:?} {}", s.change, s.path.display());
        }
    }
    let _ = write!(err, "Apply these changes? [y/N] ");
    let _ = err.flush();
    let mut answer = String::new();
    if stdin.lock().read_line(&mut answer).is_err() {
        return false;
    }
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn apply(op: &Op) -> CkResult<()> {
    match op {
        Op::Write(path, text) => {
            // Write through a symlink (dotfile managers) and keep the file's permissions.
            let path = fs::canonicalize(path).unwrap_or_else(|_| path.clone());
            let perms = fs::metadata(&path).ok().map(|m| m.permissions());
            if let Some(dir) = path.parent() {
                fs::create_dir_all(dir).map_err(|e| storage(dir, &e))?;
            }
            atomic_write(&path, text.as_bytes())?;
            match perms {
                Some(p) => fs::set_permissions(&path, p).map_err(|e| storage(&path, &e)),
                None => Ok(()),
            }
        }
        Op::Remove(path) => ignore_missing(fs::remove_file(path), path),
        Op::RemoveDir(path) => ignore_missing(fs::remove_dir_all(path), path),
    }
}

fn ignore_missing(result: std::io::Result<()>, path: &Path) -> CkResult<()> {
    match result {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(storage(path, &e)),
        _ => Ok(()),
    }
}

fn storage(path: &Path, e: &std::io::Error) -> CkError {
    CkError::new(ErrorCode::Storage, format!("{}: {e}", path.display())).with_hint(
        "Fix the permissions or free space for that path; running the command again is safe.",
    )
}

/// The file's text, or None when it does not exist.
fn read_opt(path: &Path) -> CkResult<Option<String>> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(storage(path, &e)),
    }
}

/// A step that makes `path` hold `want` (None: no file).
fn file_step(path: &Path, want: Option<String>) -> CkResult<Step> {
    let have = read_opt(path)?;
    let p = path.to_path_buf();
    let (change, ops) = match (have, want) {
        (None, None) => (Change::Unchanged, Vec::new()),
        (Some(h), Some(w)) if h == w => (Change::Unchanged, Vec::new()),
        (None, Some(w)) => (Change::Create, vec![Op::Write(p.clone(), w)]),
        (Some(_), Some(w)) => (Change::Update, vec![Op::Write(p.clone(), w)]),
        (Some(_), None) => (Change::Remove, vec![Op::Remove(p.clone())]),
    };
    Ok(Step {
        path: p,
        change,
        ops,
    })
}
