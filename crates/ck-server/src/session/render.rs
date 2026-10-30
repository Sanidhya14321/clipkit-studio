//! The full render path (spec 2.3): component layers are rendered by the Node render worker
//! (Remotion) into muted video; the result is cached by a content hash of everything that
//! draws (project content, looks, size), so an unchanged project never renders twice.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use ck_core::model::SourceKind;
use ck_core::{CkError, CkResult, ErrorCode, Project};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use super::Session;
use crate::info::ServerInfo;

/// Environment variable naming the render worker folder (with `src/render.mjs` and `bundle/`).
pub const WORKER_ENV: &str = "CK_RENDER_WORKER";

fn render_err(msg: impl Into<String>) -> CkError {
    CkError::new(ErrorCode::Render, msg.into())
}

/// The worker folder: `CK_RENDER_WORKER`, else `render-worker/` above the ck binary or `start`.
pub fn find_worker(start: &Path) -> CkResult<PathBuf> {
    if let Some(dir) = std::env::var_os(WORKER_ENV).filter(|v| !v.is_empty()) {
        return Ok(PathBuf::from(dir));
    }
    let exe = std::env::current_exe().ok();
    let roots = exe
        .iter()
        .flat_map(|e| e.ancestors().map(Path::to_path_buf).collect::<Vec<_>>())
        .chain(start.ancestors().map(Path::to_path_buf));
    for root in roots {
        let dir = root.join("render-worker");
        if dir.join("src/render.mjs").is_file() {
            return Ok(dir);
        }
    }
    Err(render_err("no render worker found")
        .with_hint("Install it with `just worker`, or set CK_RENDER_WORKER to its folder."))
}

/// Content that draws: the project without engine state, plus the look set and frame size.
fn render_key(project: &Project, looks: &Value) -> CkResult<String> {
    let full = serde_json::to_value(project)
        .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
    let v = ck_core::patch::content(&full);
    let mut h = Sha256::new();
    h.update(v.to_string().as_bytes());
    h.update(looks.to_string().as_bytes());
    Ok(h.finalize().iter().map(|b| format!("{b:02x}")).collect())
}

impl Session {
    /// Muted video of the whole project at its own size, from the cache or the worker.
    pub(super) fn render_video(&self, project: &Project) -> CkResult<PathBuf> {
        let looks = serde_json::to_value(ck_assets::library::looks()?)
            .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
        let key = render_key(project, &looks)?;
        let out = self.cache_dir().join("renders").join(format!("{key}.mp4"));
        if out.is_file() {
            return Ok(out);
        }
        let worker = find_worker(&self.workdir())?;
        let bundle = worker.join("bundle");
        if !bundle.join("index.html").is_file() {
            return Err(
                render_err("the render worker is not bundled").with_hint("Run `just worker` once.")
            );
        }
        // Footage plays through the studio's own media route (token in the query).
        let info = ServerInfo::read(self.store.root())
            .map_err(|e| render_err(format!("no running studio: {}", e.message)))?;
        let media: serde_json::Map<String, Value> = project
            .sources
            .iter()
            .filter(|s| s.kind == SourceKind::Video && s.proxy_path.is_some())
            .map(|s| {
                (
                    s.id.clone(),
                    Value::String(format!(
                        "http://127.0.0.1:{}/media/{}.mp4?token={}",
                        info.port, s.hash, info.token
                    )),
                )
            })
            .collect();
        let dir = out.parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir)
            .map_err(|e| CkError::new(ErrorCode::Storage, e.to_string()))?;
        let tmp = dir.join(format!(".tmp-{key}.mp4"));
        let job = json!({
            "bundle": bundle,
            "out": tmp,
            "project": project,
            "looks": looks,
            "media": media,
        });
        let mut child = Command::new("node")
            .arg(worker.join("src/render.mjs"))
            .current_dir(&worker)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| render_err(format!("node could not start: {e}")))?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(job.to_string().as_bytes())
                .map_err(|e| render_err(format!("job not sent: {e}")))?;
        }
        let mut result: Option<Result<(), String>> = None;
        if let Some(stdout) = child.stdout.take() {
            for line in BufReader::new(stdout).lines().map_while(Result::ok) {
                let Ok(v) = serde_json::from_str::<Value>(&line) else {
                    continue;
                };
                if let Some(p) = v.get("progress").and_then(Value::as_f64) {
                    tracing::info!(progress = p, "render progress");
                } else if v.get("done").is_some() {
                    result = Some(Ok(()));
                } else if let Some(e) = v.get("error").and_then(Value::as_str) {
                    result = Some(Err(e.to_owned()));
                }
            }
        }
        let status = child.wait().map_err(|e| render_err(e.to_string()))?;
        match result {
            Some(Ok(())) if status.success() => {
                std::fs::rename(&tmp, &out).map_err(|e| render_err(e.to_string()))?;
                Ok(out)
            }
            other => {
                let _ = std::fs::remove_file(&tmp);
                Err(render_err(match other {
                    Some(Err(e)) => format!("render failed: {e}"),
                    _ => format!("render worker exited with {status}"),
                }))
            }
        }
    }
}
