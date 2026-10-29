//! The ck binary: clap adapters over the shared ops in ck-server. Output is one line of compact
//! JSON on stdout; failures print the error as JSON on stderr and exit 1.

mod args;
mod hook;
mod install;
mod studio;

use std::io::{Read, Write};
use std::process::ExitCode;

use ck_core::model::Anchor;
use ck_core::patch::PatchRequest;
use ck_core::{CkError, CkResult, ErrorCode};
use ck_server::api::BODY_LIMIT;
use ck_server::idle::DEFAULT_IDLE_SECS;
use ck_server::launch::{self, EnsureRequest, Launcher};
use ck_server::session::brief::{BriefDiffRequest, BriefWaitRequest};
use ck_server::session::comments::{
    CommentAddRequest, CommentListRequest, CommentReplyRequest, CommentResolveRequest,
};
use ck_server::session::edit::{CutApplyRequest, SourceRequest};
use ck_server::session::export::{ExportRequest, Preset};
use ck_server::session::handoff::{HandoffRequest, WaitRequest};
use ck_server::session::import::ImportRequest;
use ck_server::session::look::LookApplyRequest;
use ck_server::session::{HeartbeatRequest, LockRequest, UnlockRequest};
use clap::Parser;
use serde::Serialize;

use args::{
    BriefCommand, Cli, Command, CommentsCommand, CutCommand, LayoutCommand, LookCommand, PresetArg,
    ProjectCommand, RecipeCommand, SoundCommand, TasteCommand, TranscriptCommand,
};
use ck_assets::library::{
    self, IdRequest, LookListRequest, RecipeSearchRequest, TasteSearchRequest,
};

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
    // Hooks speak Claude Code's exit-code protocol instead of printing errors.
    if let Command::Hook(hook) = &cli.command {
        return hook::run(hook, &cli.dir);
    }
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
    // The first call of a session starts the studio (spec 5.1).
    let ops = || launch::connect(&cli.dir, &Launcher::current(DEFAULT_IDLE_SECS)?);
    match cli.command {
        Command::Studio {
            port,
            idle,
            ensure: true,
            open,
            quiet,
        } => {
            let launcher = Launcher {
                port,
                ..Launcher::current(idle)?
            };
            let ensured = launch::ensure(&cli.dir, &EnsureRequest { open }, &launcher)?;
            if quiet { Ok(()) } else { print(&ensured) }
        }
        Command::Studio { port, idle, .. } => studio::run(&cli.dir, port, idle),
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
        Command::Brief(BriefCommand::Propose { file }) => {
            let req = serde_json::from_str(&read_input(&file, "proposal")?)
                .map_err(|e| request_err(format!("proposal JSON: {e}")))?;
            print(&ops()?.brief_propose(&req)?)
        }
        Command::Brief(BriefCommand::Wait { timeout }) => {
            print(&ops()?.brief_wait(&BriefWaitRequest {
                timeout_secs: timeout,
            })?)
        }
        Command::Brief(BriefCommand::Diff { since }) => {
            print(&ops()?.brief_diff(&BriefDiffRequest { since })?)
        }
        Command::Brief(BriefCommand::Get) => print(&ops()?.brief_get()?),
        // Built-in asset library: no studio needed.
        Command::Taste(TasteCommand::Search { query, limit }) => {
            print(&library::taste_search(&TasteSearchRequest {
                query,
                limit,
            })?)
        }
        Command::Taste(TasteCommand::Read { id }) => {
            print(&library::taste_read(&IdRequest { id })?)
        }
        Command::Look(LookCommand::List) => print(&library::look_list(&LookListRequest {})?),
        Command::Look(LookCommand::Get { id }) => print(&library::look_get(&IdRequest { id })?),
        Command::Look(LookCommand::Apply { id, dry_run }) => {
            print(&ops()?.look_apply(&LookApplyRequest { look: id, dry_run })?)
        }
        Command::Recipe(RecipeCommand::Search { query, limit }) => {
            print(&library::recipe_search(&RecipeSearchRequest {
                query,
                limit,
            })?)
        }
        Command::Recipe(RecipeCommand::Card { id }) => {
            print(&library::recipe_card(&IdRequest { id })?)
        }
        Command::Recipe(RecipeCommand::Apply { file }) => {
            let req = serde_json::from_str(&read_input(&file, "recipe apply")?)
                .map_err(|e| request_err(format!("recipe apply JSON: {e}")))?;
            print(&ops()?.recipe_apply(&req)?)
        }
        Command::Layout(LayoutCommand::Check) => print(&ops()?.layout_check()?),
        Command::Verify => print(&ops()?.verify()?),
        Command::Sound(SoundCommand::Plan) => print(&ops()?.sound_plan()?),
        Command::Import { path } => {
            let path = std::path::absolute(&path)
                .map_err(|e| CkError::new(ErrorCode::Request, format!("import path: {e}")))?;
            print(&ops()?.import(&ImportRequest {
                path: path.to_string_lossy().into_owned(),
            })?)
        }
        Command::Transcribe { source } => print(&ops()?.transcribe(&SourceRequest { source })?),
        Command::Transcript(TranscriptCommand::Get { source }) => {
            print(&ops()?.transcript_get(&SourceRequest { source })?)
        }
        Command::Cut(CutCommand::Suggest { source }) => {
            print(&ops()?.cut_suggest(&SourceRequest { source })?)
        }
        Command::Cut(CutCommand::Apply { source, ids }) => {
            print(&ops()?.cut_apply(&CutApplyRequest { source, ids })?)
        }
        Command::Context => print(&ops()?.context()?),
        Command::Comments(CommentsCommand::List { all }) => {
            print(&ops()?.comments_list(&CommentListRequest { all })?)
        }
        Command::Comments(CommentsCommand::Add {
            text,
            layer,
            at,
            to,
            path,
        }) => print(&ops()?.comments_add(&CommentAddRequest {
            text,
            anchor: Anchor {
                t_start: at,
                t_end: to,
                layer_id: layer,
                prop_path: path,
                point: None,
            },
            author: None,
        })?),
        Command::Comments(CommentsCommand::Reply { id, text, scope }) => {
            print(&ops()?.comments_reply(&CommentReplyRequest {
                id,
                text,
                scope,
                author: None,
            })?)
        }
        Command::Comments(CommentsCommand::Resolve { id, text }) => {
            print(&ops()?.comments_resolve(&CommentResolveRequest {
                id,
                text,
                author: None,
            })?)
        }
        Command::Export { preset, out } => {
            let preset = match preset {
                PresetArg::Source => Preset::Source,
                PresetArg::Youtube => Preset::Youtube,
                PresetArg::Tiktok => Preset::Tiktok,
                PresetArg::Reels => Preset::Reels,
                PresetArg::Shorts => Preset::Shorts,
            };
            print(&ops()?.export(&ExportRequest { preset, out })?)
        }
        Command::Handoff { label } => print(&ops()?.handoff(&HandoffRequest { label })?),
        Command::Wait { since, timeout } => print(&ops()?.wait(&WaitRequest {
            since,
            timeout_secs: timeout,
        })?),
        Command::Init(args) => install::run(install::Mode::Init, &args, &cli.dir),
        Command::Uninstall(args) => install::run(install::Mode::Uninstall, &args, &cli.dir),
        // Handled in main before run.
        Command::Hook(_) => Ok(()),
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
        read_limited(std::io::stdin(), "patch")?
    } else {
        arg.to_owned()
    };
    serde_json::from_str(&text).map_err(|e| request_err(format!("patch JSON: {e}")))
}

/// A file argument (`ck brief propose FILE`), or `-` for stdin.
fn read_input(arg: &str, what: &str) -> CkResult<String> {
    if arg == "-" {
        return read_limited(std::io::stdin(), what);
    }
    let file = std::fs::File::open(arg)
        .map_err(|e| request_err(format!("opening {what} file {arg:?}: {e}")))?;
    read_limited(file, what)
}

fn read_limited(input: impl Read, what: &str) -> CkResult<String> {
    let mut s = String::new();
    input
        .take(BODY_LIMIT as u64 + 1)
        .read_to_string(&mut s)
        .map_err(|e| request_err(format!("reading {what}: {e}")))?;
    if s.len() > BODY_LIMIT {
        return Err(request_err(format!("{what} is over {BODY_LIMIT} bytes")));
    }
    Ok(s)
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
