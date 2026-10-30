//! The full render path (spec 2.3): component layers are rendered by the Node render worker
//! (Remotion) into muted video; the result is cached by a content hash of everything that
//! draws (project content, looks, size), so an unchanged project never renders twice.

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use ck_core::model::SourceKind;
use ck_core::{CkError, CkResult, ErrorCode, Project};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
        let dir = out.parent().unwrap_or(Path::new("."));
        std::fs::create_dir_all(dir)
            .map_err(|e| CkError::new(ErrorCode::Storage, e.to_string()))?;
        let tmp = dir.join(format!(".tmp-{key}.mp4"));
        let mut job = self.job(project, &looks)?;
        job["out"] = json!(tmp);
        let result = self.run_worker(&job);
        match result {
            Ok(()) => {
                std::fs::rename(&tmp, &out).map_err(|e| render_err(e.to_string()))?;
                Ok(out)
            }
            Err(e) => {
                let _ = std::fs::remove_file(&tmp);
                Err(e)
            }
        }
    }

    /// The job fields every render shares: bundle, project, looks and footage URLs.
    pub(super) fn job(&self, project: &Project, looks: &Value) -> CkResult<Value> {
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
        Ok(json!({
            "worker": worker,
            "bundle": bundle,
            "project": project,
            "looks": looks,
            "media": media,
        }))
    }

    /// Runs the worker on `job` and waits for its `done` or `error` line.
    pub(super) fn run_worker(&self, job: &Value) -> CkResult<()> {
        let worker = PathBuf::from(job["worker"].as_str().unwrap_or("render-worker"));
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
            Some(Ok(())) if status.success() => Ok(()),
            Some(Err(e)) => Err(render_err(format!("render failed: {e}"))),
            _ => Err(render_err(format!("render worker exited with {status}"))),
        }
    }

    /// `render check` (spec 7.6): renders `frames` in order, then again in reverse (each frame
    /// twice, out of order) and compares the PNGs. Any difference means the composition depends
    /// on more than the frame number.
    pub fn render_check(&self, req: &RenderCheckRequest) -> CkResult<RenderCheckResult> {
        let project = self.project()?;
        let fps = f64::from(project.settings.fps);
        let last = (ck_core::timeline::duration(&project) * fps)
            .ceil()
            .max(1.0) as u64
            - 1;
        let frames: Vec<u64> = if req.frames.is_empty() {
            vec![0, last / 2, last]
        } else {
            req.frames.iter().map(|f| (*f).min(last)).collect()
        };
        let looks = serde_json::to_value(ck_assets::library::looks()?)
            .map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
        let dir = self.cache_dir().join("renders").join("check");
        std::fs::create_dir_all(&dir)
            .map_err(|e| CkError::new(ErrorCode::Storage, e.to_string()))?;
        let order: Vec<(u64, PathBuf)> = frames
            .iter()
            .chain(frames.iter().rev())
            .enumerate()
            .map(|(i, f)| (*f, dir.join(format!("{i}-{f}.png"))))
            .collect();
        let mut job = self.job(&project, &looks)?;
        job["out"] = json!(dir);
        job["stills"] = json!(
            order
                .iter()
                .map(|(f, p)| json!({ "frame": f, "out": p }))
                .collect::<Vec<_>>()
        );
        self.run_worker(&job)?;
        let mut hashes: std::collections::BTreeMap<u64, Vec<String>> = Default::default();
        for (f, p) in &order {
            let bytes =
                std::fs::read(p).map_err(|e| render_err(format!("{}: {e}", p.display())))?;
            let h: String = Sha256::digest(&bytes)[..8]
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect();
            hashes.entry(*f).or_default().push(h);
            let _ = std::fs::remove_file(p);
        }
        let unstable = unstable_frames(&hashes);
        Ok(RenderCheckResult {
            frames,
            deterministic: unstable.is_empty(),
            unstable,
        })
    }
}

/// Frames whose renders do not all hash the same.
pub fn unstable_frames(hashes: &std::collections::BTreeMap<u64, Vec<String>>) -> Vec<u64> {
    hashes
        .iter()
        .filter(|(_, hs)| hs.windows(2).any(|w| w[0] != w[1]))
        .map(|(f, _)| *f)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_frame_whose_renders_differ_is_unstable() {
        let h = |s: &[&str]| s.iter().map(|x| (*x).to_owned()).collect::<Vec<_>>();
        let map = [
            (0, h(&["a", "a"])),
            (15, h(&["b", "c"])),
            (30, h(&["d", "d"])),
        ]
        .into_iter()
        .collect();
        assert_eq!(unstable_frames(&map), [15]);
    }

    #[test]
    fn the_worker_is_found_from_the_repo_or_the_environment() {
        let here = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(find_worker(here).unwrap().join("src/render.mjs").is_file());
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RenderCheckRequest {
    /// Frames to check; default first, middle and last.
    #[serde(default)]
    pub frames: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RenderCheckResult {
    pub frames: Vec<u64>,
    pub deterministic: bool,
    /// Frames whose two renders differ.
    pub unstable: Vec<u64>,
}
