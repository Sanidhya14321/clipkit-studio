//! `ck studio`: open (or create) the project under `<dir>/.clipkit` and serve it until stopped.

use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use ck_core::model::Mode;
use ck_core::storage::Store;
use ck_core::{CkError, CkResult, ErrorCode, Project};
use ck_server::ops::Ops;
use ck_server::{Server, Session, SystemClock};
use serde::Serialize;

/// The one line `ck studio` prints once it listens. The token stays in server.json.
#[derive(Debug, Serialize)]
struct Started {
    url: String,
    port: u16,
}

pub fn run(dir: &Path, port: u16, idle_secs: u64) -> CkResult<()> {
    let root = dir.join(".clipkit");
    // Anything that answers on the recorded port, even with an error, counts as running; only
    // E_SERVER (no server.json, nothing listening) means the way is clear.
    if let Ok(ops) = Ops::connect(&root)
        && ops
            .status()
            .map_or_else(|e| e.code != ErrorCode::Server, |_| true)
    {
        return Err(CkError::new(
            ErrorCode::Server,
            "a studio is already serving this project",
        )
        .with_hint("Use the running studio; its port is in .clipkit/server.json."));
    }
    let store = Store::new(&root);
    if !store.project_path().exists() {
        store.init(&new_project(dir))?;
    }
    let session = Arc::new(Session::open(&root, Arc::new(SystemClock))?);
    let idle = session
        .activity()
        .clone()
        .until_idle(Duration::from_secs(idle_secs));
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| CkError::new(ErrorCode::Server, format!("no async runtime: {e}")))?;
    rt.block_on(async move {
        let server = Server::bind(&root, session, port).await?;
        let info = server.info();
        crate::print(&Started {
            url: info.url.clone(),
            port: info.port,
        })?;
        server.run(shutdown(idle)).await
    })
}

/// An empty project named after its folder. Mode stays GENERATE until mode detection (Phase 4)
/// sets it.
fn new_project(dir: &Path) -> Project {
    let name = std::fs::canonicalize(dir)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .unwrap_or_else(|| "Untitled".to_owned());
    Project::new("project", name, Mode::Generate)
}

/// Idle timeout, Ctrl-C, or SIGTERM on unix.
async fn shutdown(idle: impl Future<Output = ()>) {
    tokio::select! {
        _ = idle => tracing::info!("idle timeout, stopping"),
        _ = signal() => {}
    }
}

async fn signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        if let Ok(mut term) = signal(SignalKind::terminate()) {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {}
                _ = term.recv() => {}
            }
            return;
        }
    }
    let _ = tokio::signal::ctrl_c().await;
}
