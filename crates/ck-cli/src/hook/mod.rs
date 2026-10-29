//! `ck hook prompt|guard|stop|session-end`: the Claude Code plugin hooks (spec 6.1). Each reads
//! the hook's JSON from stdin, finishes quickly and never blocks a session without a ClipKit
//! project. Exit 0 lets the agent go on; exit 2 blocks with the reason on stderr. A hook never
//! fails a session with exit 1: errors are logged (CK_LOG) and the hook steps aside.

mod detect;
mod guard;

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ck_core::model::{AgentStatus, Holder};
use ck_core::{CkError, CkResult};
use ck_server::client::Client;
use ck_server::idle::DEFAULT_IDLE_SECS;
use ck_server::launch::{self, EnsureRequest, Launcher};
use ck_server::ops::Ops;
use ck_server::session::UnlockRequest;
use ck_server::view::Status;
use serde::Deserialize;
use serde_json::Value;

use crate::args::HookCommand;

/// Largest hook input read from stdin; a bigger one is treated as empty.
const INPUT_LIMIT: usize = 1024 * 1024;
/// The prompt note joins the agent's context, so it stays one short line (spec 6.1).
pub const NOTE_CAP: usize = 300;
/// Most lock ids named in the stop reason.
const STOP_IDS: usize = 5;

/// The fields of Claude Code's hook input that the hooks read; the rest is ignored.
#[derive(Debug, Default, Deserialize)]
struct HookInput {
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    cwd: Option<PathBuf>,
    #[serde(default)]
    tool_name: Option<String>,
    #[serde(default)]
    tool_input: Option<Value>,
    #[serde(default)]
    stop_hook_active: bool,
}

pub fn run(cmd: &HookCommand, dir: &Path) -> ExitCode {
    let input = read_input();
    let verdict = match cmd {
        HookCommand::Prompt => {
            prompt(&input, dir);
            None
        }
        HookCommand::Guard => guard(&input, dir).map(str::to_owned),
        HookCommand::Stop => stop(&input, dir),
        HookCommand::SessionEnd => {
            session_end(dir);
            None
        }
    };
    match verdict {
        Some(reason) => {
            eprintln!("{reason}");
            ExitCode::from(2)
        }
        None => ExitCode::SUCCESS,
    }
}

fn read_input() -> HookInput {
    let mut text = String::new();
    let read = std::io::stdin()
        .take(INPUT_LIMIT as u64 + 1)
        .read_to_string(&mut text);
    if read.is_err() || text.len() > INPUT_LIMIT {
        return HookInput::default();
    }
    serde_json::from_str(&text).unwrap_or_default()
}

fn has_project(dir: &Path) -> bool {
    dir.join(".clipkit").join("project.json").is_file()
}

/// UserPromptSubmit: for a video prompt or an existing project, start the studio, open its tab
/// once and print one context line. Anything else prints nothing.
fn prompt(input: &HookInput, dir: &Path) {
    let video = input.prompt.as_deref().is_some_and(detect::is_video_prompt);
    if !video && !has_project(dir) {
        return;
    }
    let note = match ensure(dir) {
        Ok((url, status)) => note(&url, status.as_ref()),
        Err(e) => format!("ClipKit studio did not start ({}): {}", e.code, e.message),
    };
    println!("{}", clip(&note, NOTE_CAP));
}

/// The studio url and, when it answers, its status. A failing browser is not an error here.
fn ensure(dir: &Path) -> CkResult<(String, Option<Status>)> {
    let launcher = Launcher::current(DEFAULT_IDLE_SECS)?;
    let ensured = launch::ensure(dir, &EnsureRequest { open: true }, &launcher).or_else(|e| {
        tracing::warn!(error = %e.message, "studio tab not opened");
        launch::ensure(dir, &EnsureRequest { open: false }, &launcher)
    })?;
    let status = Ops::connect(&dir.join(".clipkit"))
        .and_then(|ops| ops.status())
        .ok();
    Ok((ensured.url, status))
}

fn note(url: &str, status: Option<&Status>) -> String {
    let Some(s) = status else {
        return format!("ClipKit studio: {url}. Use ck commands; never edit .clipkit/ directly.");
    };
    format!(
        "ClipKit studio: {url} (mode {}, stage {}, agent {}). Use ck commands; never edit \
         .clipkit/ directly.",
        word(&s.mode),
        word(&s.pipeline.stage),
        word(&s.agent.status)
    )
}

/// An enum's serialized name, for example `generate`.
fn word<T: serde::Serialize>(value: &T) -> String {
    match serde_json::to_value(value) {
        Ok(Value::String(s)) => s.to_lowercase(),
        _ => "unknown".to_owned(),
    }
}

/// PreToolUse on Write, Edit and Bash: refuse direct changes in a ClipKit project.
fn guard(input: &HookInput, dir: &Path) -> Option<&'static str> {
    if !dir.join(".clipkit").is_dir() {
        return None;
    }
    let tool_input = input.tool_input.as_ref()?;
    match input.tool_name.as_deref()? {
        "Bash" => guard::bash(tool_input.get("command")?.as_str()?),
        "Write" | "Edit" | "MultiEdit" | "NotebookEdit" => {
            let path = ["file_path", "notebook_path"]
                .iter()
                .find_map(|k| tool_input.get(*k)?.as_str())?;
            let cwd = input.cwd.clone().unwrap_or_else(|| dir.to_path_buf());
            let cwd = std::path::absolute(&cwd).unwrap_or(cwd);
            let dir = std::path::absolute(dir).unwrap_or_else(|_| dir.to_path_buf());
            guard::file(Path::new(path), &cwd, &dir)
        }
        _ => None,
    }
}

/// Stop: block once while the agent still holds locks or is mid-stage. Never starts a studio,
/// and steps aside when Claude Code is already continuing because of a stop hook.
fn stop(input: &HookInput, dir: &Path) -> Option<String> {
    if input.stop_hook_active {
        return None;
    }
    let info = launch::running(&dir.join(".clipkit"))?;
    let status = Ops::new(Client::new(&info))
        .status()
        .map_err(|e| log("stop", &e))
        .ok()?;
    stop_reason(&status)
}

fn stop_reason(status: &Status) -> Option<String> {
    let ids: Vec<&str> = status
        .locks
        .iter()
        .filter(|l| l.holder == Holder::Agent)
        .map(|l| l.layer_id.as_str())
        .collect();
    let working = status.agent.status == AgentStatus::Working;
    if ids.is_empty() && !working {
        return None;
    }
    let mut parts = Vec::new();
    if !ids.is_empty() {
        let mut named = ids
            .iter()
            .take(STOP_IDS)
            .copied()
            .collect::<Vec<_>>()
            .join(", ");
        if ids.len() > STOP_IDS {
            named.push_str(&format!(" and {} more", ids.len() - STOP_IDS));
        }
        parts.push(format!("you still hold locks on {named}"));
    }
    if working {
        parts.push(format!(
            "stage {} is not finished (agent status is working)",
            word(&status.pipeline.stage)
        ));
    }
    Some(format!(
        "ClipKit: {}. Finish the current step, then run ck unlock before you stop.",
        parts.join(" and ")
    ))
}

/// SessionEnd: release the agent's locks so the user can edit at once. Logs go to stderr only
/// under CK_LOG; there is no trace store to flush yet.
fn session_end(dir: &Path) {
    let Some(info) = launch::running(&dir.join(".clipkit")) else {
        return;
    };
    let req = UnlockRequest {
        holder: Holder::Agent,
        layer_ids: None,
    };
    if let Err(e) = Ops::new(Client::new(&info)).unlock(&req) {
        log("session-end", &e);
    }
}

fn log(hook: &str, e: &CkError) {
    tracing::warn!(hook, code = %e.code, error = %e.message, "hook skipped");
}

/// At most `cap` bytes, cut on a char boundary.
fn clip(s: &str, cap: usize) -> &str {
    let mut end = s.len().min(cap);
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ck_core::model::{AgentState, Lock, Mode, PipelineState, Stage};

    fn status(locks: &[(&str, Holder)], agent: AgentStatus) -> Status {
        Status {
            project_id: "p".into(),
            rev: 1,
            mode: Mode::Generate,
            pipeline: PipelineState {
                stage: Stage::Passes,
                pass: None,
                last_completed_pass: None,
            },
            agent: AgentState {
                status: agent,
                ..AgentState::default()
            },
            locks: locks
                .iter()
                .map(|(id, holder)| Lock {
                    layer_id: (*id).into(),
                    holder: *holder,
                    acquired_at_ms: 0,
                    heartbeat_at_ms: 0,
                    ttl_ms: 1,
                })
                .collect(),
        }
    }

    #[test]
    fn stop_passes_when_the_agent_is_done() {
        let s = status(&[("a", Holder::Human)], AgentStatus::Idle);
        assert_eq!(stop_reason(&s), None);
    }

    #[test]
    fn stop_names_held_locks_and_unfinished_work() {
        let ids: Vec<String> = (0..7).map(|i| format!("l{i}")).collect();
        let locks: Vec<(&str, Holder)> = ids.iter().map(|i| (i.as_str(), Holder::Agent)).collect();
        let reason = stop_reason(&status(&locks, AgentStatus::Working)).unwrap();
        assert!(reason.contains("l0, l1, l2, l3, l4 and 2 more"), "{reason}");
        assert!(reason.contains("stage passes is not finished"), "{reason}");
        assert!(reason.contains("ck unlock"));
    }

    #[test]
    fn note_is_one_short_line() {
        let s = status(&[], AgentStatus::Idle);
        let n = note("http://127.0.0.1:4000", Some(&s));
        assert!(n.contains("mode generate, stage passes, agent idle"), "{n}");
        assert!(n.len() <= NOTE_CAP && !n.contains('\n'));
        assert_eq!(clip("héllo", 2), "h");
    }
}
