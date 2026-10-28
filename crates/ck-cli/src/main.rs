//! The ck binary: clap adapters over the shared ops in ck-server. Output is one line of compact
//! JSON on stdout; failures print the error as JSON on stderr and exit 1.

mod args;
mod studio;

use std::io::{Read, Write};
use std::process::ExitCode;

use ck_core::patch::PatchRequest;
use ck_core::{CkError, CkResult, ErrorCode};
use ck_server::api::BODY_LIMIT;
use ck_server::ops::Ops;
use ck_server::session::{HeartbeatRequest, LockRequest, UnlockRequest};
use clap::Parser;
use serde::Serialize;

use args::{Cli, Command, ProjectCommand};

/// JSON log lines to stderr when `CK_LOG` is set (for example `CK_LOG=debug`). Off by default so
/// command output stays small; fields follow ck_core::log.
fn init_logging() {
    let Ok(filter) = std::env::var("CK_LOG") else {
        return;
    };
    let _ = tracing_subscriber::fmt()
        .json()
        .flatten_event(true)
        .with_current_span(true)
        .with_span_list(false)
        .with_env_filter(tracing_subscriber::EnvFilter::new(filter))
        .with_writer(std::io::stderr)
        .try_init();
}

fn main() -> ExitCode {
    init_logging();
    tracing::debug!(version = env!("CARGO_PKG_VERSION"), "ck start");
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let line = serde_json::to_string(&e).unwrap_or_else(|_| e.to_string());
            eprintln!("{line}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> CkResult<()> {
    let root = cli.dir.join(".clipkit");
    let ops = || Ops::connect(&root);
    match cli.command {
        Command::Studio { port, idle } => studio::run(&cli.dir, port, idle),
        Command::Mcp => mcp(root),
        Command::Status => print(&ops()?.status()?),
        Command::Project(ProjectCommand::Get {
            layer: Some(id), ..
        }) => print(&ops()?.layer(&id)?),
        Command::Project(ProjectCommand::Get { layer: None, .. }) => print(&ops()?.summary()?),
        Command::Project(ProjectCommand::Patch { patch }) => {
            let req = read_patch(&patch)?;
            print(&ops()?.patch(&req)?)
        }
        Command::Lock { layer_ids, holder } => print(&ops()?.lock(&LockRequest {
            holder: holder.holder,
            layer_ids,
        })?),
        Command::Unlock { layer_ids, holder } => print(&ops()?.unlock(&UnlockRequest {
            holder: holder.holder,
            layer_ids: (!layer_ids.is_empty()).then_some(layer_ids),
        })?),
        Command::Heartbeat { holder } => print(&ops()?.heartbeat(&HeartbeatRequest {
            holder: holder.holder,
        })?),
    }
}

/// `ck mcp`: stdout carries the protocol, so nothing else is printed there.
fn mcp(root: std::path::PathBuf) -> CkResult<()> {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| CkError::new(ErrorCode::Server, format!("no async runtime: {e}")))?;
    rt.block_on(ck_mcp::CkMcp::new(root).serve_stdio())
}

/// The patch argument is the JSON itself, or `-` to read it from stdin.
fn read_patch(arg: &str) -> CkResult<PatchRequest> {
    let text = if arg == "-" {
        let mut s = String::new();
        std::io::stdin()
            .take(BODY_LIMIT as u64 + 1)
            .read_to_string(&mut s)
            .map_err(|e| request_err(format!("reading stdin: {e}")))?;
        if s.len() > BODY_LIMIT {
            return Err(request_err(format!("patch is over {BODY_LIMIT} bytes")));
        }
        s
    } else {
        arg.to_owned()
    };
    serde_json::from_str(&text).map_err(|e| request_err(format!("patch JSON: {e}")))
}

fn request_err(message: String) -> CkError {
    CkError::new(ErrorCode::Request, message)
}

/// One line of compact JSON on stdout.
pub(crate) fn print<T: Serialize>(value: &T) -> CkResult<()> {
    let line =
        serde_json::to_string(value).map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))?;
    let mut out = std::io::stdout().lock();
    match writeln!(out, "{line}").and_then(|()| out.flush()) {
        // A closed pipe (for example `| head`) is not an error worth reporting.
        Err(e) if e.kind() != std::io::ErrorKind::BrokenPipe => Err(CkError::new(
            ErrorCode::Server,
            format!("writing output: {e}"),
        )),
        _ => Ok(()),
    }
}
