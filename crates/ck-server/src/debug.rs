//! Debuggability (spec 10.4): `ck replay` rebuilds the project from its patches without an
//! agent, `ck bisect` finds the first revision where the quality gates fail, and `ck trace
//! bundle` writes one redacted, shareable JSON file. All read the `.clipkit` folder directly;
//! no studio is needed.

use std::path::{Path, PathBuf};

use ck_assets::library;
use ck_core::storage::Store;
use ck_core::{CkError, CkResult, Document, ErrorCode, Project};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::session::look::project_look_id;
use crate::session::verify::verify;

pub mod caps {
    pub const REPLAY: usize = 1024;
    pub const BISECT: usize = 2 * 1024;
    pub const TRACE: usize = 1024;
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReplayRequest {
    /// Also rebuild the content at this earlier revision.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ReplayResult {
    pub rev: u64,
    /// Replaying every stored patch over the base reproduces the stored project.
    pub reproduces: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub problem: Option<String>,
    /// At `to`: layers and duration of the rebuilt content.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<RevSummary>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RevSummary {
    pub rev: u64,
    pub layers: usize,
    pub duration_sec: f64,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BisectRequest {
    /// Only this gate (for example `overlaps`); default any gate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gate: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BisectResult {
    pub rev: u64,
    /// The first revision whose content fails, when the current one fails.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_failing: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Gates failing at that revision.
    pub gates: Vec<String>,
    /// Revisions checked.
    pub checked: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TraceRequest {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TraceResult {
    /// The bundle, relative to the project folder.
    pub path: String,
    pub bytes: usize,
    /// Values replaced with `[redacted]`.
    pub redacted: usize,
}

fn load(root: &Path) -> CkResult<Document> {
    Store::new(root).load()
}

/// The project content at `rev`, as a project.
fn project_at(doc: &Document, rev: u64) -> CkResult<Project> {
    let mut content = doc.content_at(rev)?;
    if let Some(o) = content.as_object_mut() {
        o.insert("rev".into(), json!(rev));
    }
    serde_json::from_value(content)
        .map_err(|e| CkError::new(ErrorCode::Schema, format!("rev {rev}: {e}")))
}

pub fn replay(root: &Path, req: &ReplayRequest) -> CkResult<ReplayResult> {
    let doc = load(root)?;
    let (reproduces, problem) = match Store::new(root).verify_replay() {
        Ok(()) => (true, None),
        Err(e) => (false, Some(e.message)),
    };
    let at = match req.to {
        Some(rev) => {
            let p = project_at(&doc, rev)?;
            Some(RevSummary {
                rev,
                layers: p.items().count(),
                duration_sec: ck_core::timeline::duration(&p),
            })
        }
        None => None,
    };
    Ok(ReplayResult {
        rev: doc.project().rev,
        reproduces,
        problem,
        at,
    })
}

fn failing_gates(p: &Project, gate: Option<&str>) -> CkResult<Vec<String>> {
    let look = library::look_get(&library::IdRequest {
        id: project_look_id(p),
    })?;
    Ok(verify(p, &look)
        .failed
        .into_iter()
        .filter(|g| gate.is_none_or(|w| w == g))
        .collect())
}

/// Binary search over revisions (oldest reachable to current) for the first failing one.
/// Assumes once it breaks it stays broken, as bisect does.
pub fn bisect(root: &Path, req: &BisectRequest) -> CkResult<BisectResult> {
    let doc = load(root)?;
    let current = doc.project().rev;
    let oldest = doc
        .history()
        .first()
        .map_or(current, |r| r.rev.saturating_sub(1));
    let gate = req.gate.as_deref();
    let mut checked = Vec::new();
    let mut fails = |rev: u64| -> CkResult<Vec<String>> {
        checked.push(rev);
        failing_gates(&project_at(&doc, rev)?, gate)
    };
    if fails(current)?.is_empty() {
        return Ok(BisectResult {
            rev: current,
            first_failing: None,
            label: None,
            gates: Vec::new(),
            checked,
        });
    }
    let (mut lo, mut hi) = (oldest, current);
    if !fails(lo)?.is_empty() {
        hi = lo;
    }
    // Invariant: `hi` fails; everything at or below `lo` passes (unless lo == hi).
    while hi > lo + 1 {
        let mid = lo + (hi - lo) / 2;
        if fails(mid)?.is_empty() {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    let gates = failing_gates(&project_at(&doc, hi)?, gate)?;
    let label = doc
        .history()
        .iter()
        .find(|r| r.rev == hi)
        .map(|r| r.label.clone());
    Ok(BisectResult {
        rev: current,
        first_failing: Some(hi),
        label,
        gates,
        checked,
    })
}

/// Key names whose values never leave the machine.
const SECRET_KEYS: &[&str] = &[
    "token",
    "secret",
    "password",
    "apikey",
    "api_key",
    "authorization",
];

/// Replaces secret-looking values, and the home folder in strings, in place.
fn redact(v: &mut Value, home: Option<&str>, count: &mut usize) {
    match v {
        Value::Object(o) => {
            for (k, val) in o.iter_mut() {
                let lower = k.to_lowercase();
                if SECRET_KEYS.iter().any(|s| lower.contains(s)) {
                    *val = Value::String("[redacted]".into());
                    *count += 1;
                } else {
                    redact(val, home, count);
                }
            }
        }
        Value::Array(a) => a.iter_mut().for_each(|x| redact(x, home, count)),
        Value::String(s) => {
            if let Some(h) = home.filter(|h| h.len() > 1 && s.contains(*h)) {
                *s = s.replace(h, "~");
                *count += 1;
            }
            if let Some(i) = s.find("?token=") {
                s.truncate(i);
                s.push_str("?token=[redacted]");
                *count += 1;
            }
        }
        _ => {}
    }
}

fn first_line(cmd: &str, arg: &str) -> String {
    std::process::Command::new(cmd)
        .arg(arg)
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .and_then(|s| s.lines().next().map(str::to_owned))
        .unwrap_or_else(|| "not found".into())
}

/// Writes `.clipkit/traces/trace-<rev>.json`: environment, project, history (labels, authors,
/// touched layers and patches), gate results and replay status, redacted.
pub fn trace_bundle(root: &Path, _req: &TraceRequest) -> CkResult<TraceResult> {
    let doc = load(root)?;
    let project = doc.project().clone();
    let gates = failing_gates(&project, None).unwrap_or_default();
    let replayed = replay(root, &ReplayRequest::default())?;
    let mut bundle = json!({
        "kind": "clipkit-trace",
        "environment": {
            "ck": env!("CARGO_PKG_VERSION"),
            "os": std::env::consts::OS,
            "arch": std::env::consts::ARCH,
            "ffmpeg": first_line("ffmpeg", "-version"),
            "node": first_line("node", "--version"),
        },
        "project": project,
        "history": doc.history(),
        "failingGates": gates,
        "replay": replayed,
    });
    let home = std::env::var("HOME").ok();
    let mut redacted = 0;
    redact(&mut bundle, home.as_deref(), &mut redacted);
    let text = serde_json::to_vec_pretty(&bundle)
        .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
    let dir: PathBuf = root.join("traces");
    std::fs::create_dir_all(&dir).map_err(|e| CkError::new(ErrorCode::Storage, e.to_string()))?;
    let path = dir.join(format!("trace-{}.json", doc.project().rev));
    ck_core::storage::atomic_write(&path, &text)?;
    Ok(TraceResult {
        path: format!(".clipkit/traces/trace-{}.json", doc.project().rev),
        bytes: text.len(),
        redacted,
    })
}

#[cfg(test)]
mod tests;
