//! The ck binary: clap adapters over the shared ops in ck-server. Output is one line of compact
//! JSON on stdout; failures print the error as JSON on stderr and exit 1.

mod args;
mod hook;
mod install;
mod studio;

use std::io::{Read, Write};
use std::process::ExitCode;

use ck_core::model::{Anchor, VoiceMode};
use ck_core::patch::PatchRequest;
use ck_core::{CkError, CkResult, ErrorCode};
use ck_server::api::BODY_LIMIT;
use ck_server::idle::DEFAULT_IDLE_SECS;
use ck_server::launch::{self, EnsureRequest, Launcher};
use ck_server::session::ask::AskRequest;
use ck_server::session::branch::{
    BranchRequest, HistoryRequest as RevHistoryRequest, VariantsRequest,
};
use ck_server::session::brief::{BriefDiffRequest, BriefWaitRequest};
use ck_server::session::comments::{
    CommentAddRequest, CommentListRequest, CommentReplyRequest, CommentResolveRequest,
};
use ck_server::session::derive::DeriveRequest;
use ck_server::session::edit::{CutApplyRequest, SourceRequest};
use ck_server::session::edl::{GridBuildRequest, PlanExpandRequest, PlanSetRequest};
use ck_server::session::export::{ExportRequest, Preset};
use ck_server::session::handoff::{HandoffRequest, WaitRequest};
use ck_server::session::import::ImportRequest;
use ck_server::session::look::LookApplyRequest;
use ck_server::session::music::MusicAnalyzeRequest;
use ck_server::session::reference::ReferenceRequest;
use ck_server::session::render::RenderCheckRequest;
use ck_server::session::storyboard::StoryboardRequest;
use ck_server::session::verify::VerifyRequest;
use ck_server::session::voice::{
    CaptionsRequest, SlotRequest, VoiceCloneRequest, VoiceModeRequest,
};
use ck_server::session::{HeartbeatRequest, LockRequest, UnlockRequest};
use clap::Parser;
use serde::Serialize;

use args::{
    AssetsCommand, BenchCommand, BranchCommand, BriefCommand, Cli, Command, CommentsCommand,
    ContentCommand, CutCommand, DemoCommand, FormatArg, GridCommand, LayoutCommand, LicenseCommand,
    LookCommand, ModeArg, MusicCommand, PipelineCommand, PlanCommand, PresetArg, ProfileArg,
    ProjectCommand, RecipeCommand, ReferenceCommand, RenderCommand, SoundCommand, SourcesCommand,
    StoryboardCommand, TasteCommand, TraceCommand, TranscriptCommand, VoiceCommand,
};
use ck_assets::card::{AssetKind, Grade, LicenseMode};
use ck_assets::hub::Hub;
use ck_assets::ingest::{self, IngestRequest};
use ck_assets::library::{
    self, IdRequest, LookListRequest, RecipeSearchRequest, TasteSearchRequest,
};
use ck_assets::report::{self, LicenseReportRequest};
use ck_assets::retrieve::{
    self, AssetIdRequest, AssetSearchRequest, AssetSliceRequest, AssetSourceRequest, Retrieval,
};
use ck_assets::sources::{self, SourcesAddRequest, SourcesGapsRequest, SourcesListRequest};
use ck_assets::vet::{self, VetRequest};

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
            autopilot: _,
        } => {
            let launcher = Launcher {
                port,
                ..Launcher::current(idle)?
            };
            let ensured = launch::ensure(&cli.dir, &EnsureRequest { open }, &launcher)?;
            if quiet { Ok(()) } else { print(&ensured) }
        }
        Command::Studio {
            port,
            idle,
            autopilot,
            ..
        } => studio::run(&cli.dir, port, idle, autopilot),
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
        Command::Verify { critic } => print(&ops()?.verify(&VerifyRequest { critic })?),
        Command::Content(ContentCommand::Map { source }) => {
            print(&ops()?.content_map(&SourceRequest { source })?)
        }
        Command::Grid(GridCommand::Build {
            source,
            music,
            bpm,
            dur,
        }) => print(&ops()?.grid_build(&GridBuildRequest {
            source,
            music,
            bpm,
            dur,
        })?),
        Command::Plan(PlanCommand::Get) => print(&ops()?.plan_get()?),
        Command::Plan(PlanCommand::Set { input }) => {
            let text = read_input(&input, "plan")?;
            let req: PlanSetRequest = serde_json::from_str(&text)
                .map_err(|e| CkError::new(ErrorCode::Request, format!("bad plan: {e}")))?;
            print(&ops()?.plan_set(&req)?)
        }
        Command::Plan(PlanCommand::Candidates) => print(&ops()?.plan_candidates()?),
        Command::Plan(PlanCommand::Expand { segment }) => {
            print(&ops()?.plan_expand(&PlanExpandRequest { segment })?)
        }
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
        Command::Ask {
            question,
            options,
            id,
            timeout,
        } => print(&ops()?.ask(&AskRequest {
            question,
            options,
            id,
            timeout_secs: timeout,
        })?),
        Command::Voice(VoiceCommand::Script { file }) => {
            let req = serde_json::from_str(&read_input(&file, "voice script")?)
                .map_err(|e| request_err(format!("voice script JSON: {e}")))?;
            print(&ops()?.voice_script(&req)?)
        }
        Command::Voice(VoiceCommand::Speak { slot }) => {
            print(&ops()?.voice_speak(&SlotRequest { slot })?)
        }
        Command::Voice(VoiceCommand::Clone { slot, take }) => {
            print(&ops()?.voice_clone(&VoiceCloneRequest { slot, take })?)
        }
        Command::Voice(VoiceCommand::Captions) => {
            print(&ops()?.voice_captions(&CaptionsRequest {})?)
        }
        Command::Voice(VoiceCommand::Mode { slot, mode }) => {
            let mode = match mode {
                ModeArg::Ai => VoiceMode::Ai,
                ModeArg::Recorded => VoiceMode::Recorded,
                ModeArg::Cloned => VoiceMode::Cloned,
            };
            print(&ops()?.voice_mode(&VoiceModeRequest {
                slot,
                mode,
                author: None,
            })?)
        }
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
            // `--scope ""` declares an empty scope (only the anchored layer).
            let scope = scope.map(|ids| ids.into_iter().filter(|i| !i.is_empty()).collect());
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
        Command::Assets(AssetsCommand::Derive { id, name }) => {
            print(&ops()?.derive(&DeriveRequest { id, name })?)
        }
        Command::Assets(cmd) => {
            let ladder = Retrieval::new(Hub::locate(&cli.dir)?, &cli.dir);
            match cmd {
                AssetsCommand::Search {
                    query,
                    kind,
                    tag,
                    aspect,
                    mode,
                    grade,
                    collection,
                    limit,
                } => {
                    let bad =
                        |what: &str, v: &str| request_err(format!("--{what} {v:?} is not known"));
                    let req = AssetSearchRequest {
                        query,
                        kind: kind
                            .map(|k| AssetKind::parse(&k).ok_or_else(|| bad("kind", &k)))
                            .transpose()?,
                        tag,
                        aspect,
                        mode: mode
                            .map(|m| LicenseMode::parse(&m).ok_or_else(|| bad("mode", &m)))
                            .transpose()?,
                        grade: grade
                            .map(|g| Grade::parse(&g).ok_or_else(|| bad("grade", &g)))
                            .transpose()?,
                        collection,
                        limit,
                    };
                    capped_print(
                        "assets search",
                        retrieve::caps::SEARCH,
                        ladder.search(&req)?,
                    )
                }
                // Handled above: it needs the studio.
                AssetsCommand::Derive { .. } => Ok(()),
                AssetsCommand::Card { id } => capped_print(
                    "assets card",
                    retrieve::caps::CARD,
                    ladder.card(&AssetIdRequest { id })?,
                ),
                AssetsCommand::Skeleton { id } => capped_print(
                    "assets skeleton",
                    retrieve::caps::SKELETON,
                    ladder.skeleton(&AssetIdRequest { id })?,
                ),
                AssetsCommand::Slice { id, symbol } => capped_print(
                    "assets slice",
                    retrieve::caps::SLICE,
                    ladder.slice(&AssetSliceRequest { id, symbol })?,
                ),
                AssetsCommand::Source { id, reason } => capped_print(
                    "assets source",
                    retrieve::caps::SOURCE,
                    ladder.source(&AssetSourceRequest { id, reason })?,
                ),
            }
        }
        Command::License(LicenseCommand::Report { mode }) => {
            let mode = mode
                .map(|m| {
                    LicenseMode::parse(&m)
                        .ok_or_else(|| request_err(format!("--mode {m:?} is not known")))
                })
                .transpose()?;
            let out = report::license_report(
                &Hub::locate(&cli.dir)?,
                &cli.dir.join(".clipkit"),
                &LicenseReportRequest { mode },
            )?;
            print(&ck_server::ops::trimmed(
                "license report",
                report::caps::REPORT,
                out,
            )?)
        }
        Command::Sources(cmd) => {
            let hub = Hub::locate(&cli.dir)?;
            match cmd {
                SourcesCommand::List { status } => {
                    let status = status
                        .map(|s| serde_json::from_value(serde_json::Value::String(s)))
                        .transpose()
                        .map_err(|e| request_err(format!("--status: {e}")))?;
                    let out = sources::sources_list(&hub, &SourcesListRequest { status })?;
                    print(&ck_server::ops::trimmed(
                        "sources list",
                        sources::caps::SOURCES_LIST,
                        out,
                    )?)
                }
                SourcesCommand::Add {
                    id,
                    kind,
                    repo,
                    name,
                    license,
                } => print(&sources::sources_add(
                    &hub,
                    &SourcesAddRequest {
                        id,
                        name,
                        kind,
                        repo,
                        license,
                    },
                )?),
                SourcesCommand::Vet { id, from } => {
                    let from = from
                        .map(|f| std::path::absolute(&f).map(|p| p.to_string_lossy().into_owned()))
                        .transpose()
                        .map_err(|e| request_err(format!("--from: {e}")))?;
                    let out = vet::vet(&hub, &VetRequest { id, from })?;
                    print(&ck_server::ops::trimmed(
                        "sources vet",
                        vet::caps::VET,
                        out,
                    )?)
                }
                SourcesCommand::Ingest { id } => {
                    let out = ingest::ingest_with_media(
                        &hub,
                        &IngestRequest { id },
                        Some(&ck_server::session::music::asset_analysis),
                    )?;
                    print(&ck_server::ops::trimmed(
                        "sources ingest",
                        ingest::caps::INGEST,
                        out,
                    )?)
                }
                SourcesCommand::Gaps => {
                    print(&sources::sources_gaps(&hub, &SourcesGapsRequest {})?)
                }
            }
        }
        Command::Eval { video, expect } => {
            let expect = match expect {
                Some(f) => serde_json::from_str(&read_input(&f, "eval")?)
                    .map_err(|e| request_err(format!("eval expectations JSON: {e}")))?,
                None => Default::default(),
            };
            let report = ck_eval::evaluate(&cli.dir, &ck_eval::EvalRequest { video, expect })?;
            capped_print("eval", ck_eval::caps::EVAL, report)
        }
        Command::Music(MusicCommand::Analyze { path }) => {
            let path = std::path::absolute(&path)
                .map_err(|e| request_err(e.to_string()))?
                .to_string_lossy()
                .into_owned();
            print(&ops()?.music_analyze(&MusicAnalyzeRequest { path })?)
        }
        Command::Reference(ReferenceCommand::Analyze { path, apply }) => {
            let path = std::path::absolute(&path)
                .map_err(|e| request_err(format!("reference path: {e}")))?
                .to_string_lossy()
                .into_owned();
            print(&ops()?.reference_analyze(&ReferenceRequest { path, apply })?)
        }
        Command::Storyboard(StoryboardCommand::Get) => {
            print(&ops()?.storyboard_get(&StoryboardRequest {})?)
        }
        Command::History { limit } => print(&ops()?.history(&RevHistoryRequest { limit })?),
        Command::Branch(BranchCommand::Save { name }) => {
            print(&ops()?.branch_save(&BranchRequest { name, author: None })?)
        }
        Command::Branch(BranchCommand::List) => print(&ops()?.branch_list()?),
        Command::Branch(BranchCommand::Switch { name }) => {
            print(&ops()?.branch_switch(&BranchRequest { name, author: None })?)
        }
        Command::Variants { looks, frame } => {
            print(&ops()?.variants(&VariantsRequest { looks, frame })?)
        }
        Command::Bench(BenchCommand::Report { results }) => {
            let out =
                ck_eval::bench::report(&cli.dir, &ck_eval::bench::BenchReportRequest { results })?;
            capped_print("bench report", ck_eval::bench::caps::REPORT, out)
        }
        Command::Profile { action } => {
            use ck_server::profile::{self as taste, ProfileAction, ProfileCommandRequest};
            let action = match action {
                ProfileArg::Show => ProfileAction::Show,
                ProfileArg::Clear => ProfileAction::Clear,
                ProfileArg::Disable => ProfileAction::Disable,
                ProfileArg::Enable => ProfileAction::Enable,
            };
            let path = taste::default_path();
            let out = taste::run(path.as_deref(), &ProfileCommandRequest { action })?;
            capped_print("profile", taste::caps::PROFILE, out)
        }
        Command::Demo(DemoCommand::Reset { force }) => {
            let out = ck_server::demo::reset(&cli.dir, &ck_server::demo::DemoRequest { force })?;
            capped_print("demo reset", ck_server::demo::caps::DEMO, out)
        }
        Command::Replay { to } => {
            let out = ck_server::debug::replay(&root, &ck_server::debug::ReplayRequest { to })?;
            capped_print("replay", ck_server::debug::caps::REPLAY, out)
        }
        Command::Bisect { gate } => {
            let out = ck_server::debug::bisect(&root, &ck_server::debug::BisectRequest { gate })?;
            capped_print("bisect", ck_server::debug::caps::BISECT, out)
        }
        Command::Trace(TraceCommand::Bundle) => {
            let out = ck_server::debug::trace_bundle(&root, &ck_server::debug::TraceRequest {})?;
            capped_print("trace bundle", ck_server::debug::caps::TRACE, out)
        }
        Command::Render(RenderCommand::Check { frames }) => {
            print(&ops()?.render_check(&RenderCheckRequest { frames })?)
        }
        Command::Export {
            format: format @ (FormatArg::Otio | FormatArg::Srt),
            out,
            ..
        } => {
            use ck_server::session::formats::{TimelineExportRequest, TimelineFormat};
            let format = if format == FormatArg::Otio {
                TimelineFormat::Otio
            } else {
                TimelineFormat::Srt
            };
            print(&ops()?.export_timeline(&TimelineExportRequest { format, out })?)
        }
        Command::Doctor => {
            let found = ck_server::doctor::run(&std::env::current_dir().unwrap_or_default());
            print(&found)?;
            if !found.ok {
                std::process::exit(1);
            }
            Ok(())
        }
        Command::Jobs => print(&ops()?.export_jobs()?),
        Command::Pipeline(PipelineCommand::Detect) => print(&ops()?.pipeline_detect()?),
        Command::Pipeline(PipelineCommand::Next) => print(&ops()?.pipeline_next()?),
        Command::Export {
            preset,
            out,
            force,
            reason,
            queue,
            ..
        } => {
            if force && reason.as_deref().is_none_or(|r| r.trim().is_empty()) {
                return Err(request_err("--force needs --reason".into()));
            }
            let preset = match preset {
                PresetArg::Source => Preset::Source,
                PresetArg::Youtube => Preset::Youtube,
                PresetArg::Tiktok => Preset::Tiktok,
                PresetArg::Reels => Preset::Reels,
                PresetArg::Shorts => Preset::Shorts,
            };
            let req = ExportRequest {
                queue,
                force_reason: reason.filter(|_| force),
                preset,
                out,
                ..Default::default()
            };
            if queue {
                print(&ops()?.export_enqueue(&req)?)
            } else {
                print(&ops()?.export(&req)?)
            }
        }
        Command::Handoff {
            label,
            override_reason,
        } => print(&ops()?.handoff(&HandoffRequest {
            label,
            override_reason,
        })?),
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

/// Print `v` unless its compact JSON is over `cap` (E_OUTPUT_CAP).
fn capped_print<T: Serialize>(what: &str, cap: usize, v: T) -> CkResult<()> {
    print(&ck_server::ops::capped(what, cap, v)?)
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
    serde_json::from_str(&read_input(arg, "patch")?)
        .map_err(|e| request_err(format!("patch JSON: {e}")))
}

/// The one JSON input convention: inline JSON, `@file` (or a bare path), or `-` for stdin.
fn read_input(arg: &str, what: &str) -> CkResult<String> {
    if arg == "-" {
        return read_limited(std::io::stdin(), what);
    }
    if arg.trim_start().starts_with(['{', '[']) {
        return Ok(arg.to_owned());
    }
    let path = arg.strip_prefix('@').unwrap_or(arg);
    let file = std::fs::File::open(path)
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
