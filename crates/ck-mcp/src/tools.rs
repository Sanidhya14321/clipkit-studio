//! The MCP tool table (spec 6.2). Each tool is the twin of a ck subcommand: the name is the
//! command path joined with `_` (`ck project get` is `project_get`), the input is the JSON the
//! command takes, and the output is the one line of compact JSON the command prints.

use std::path::Path;

use ck_assets::hub::Hub;
use ck_assets::ingest::{self, IngestRequest};
use ck_assets::library::{
    self, IdRequest, LookListRequest, RecipeSearchRequest, TasteSearchRequest,
};
use ck_assets::sources::{self, SourcesAddRequest, SourcesGapsRequest, SourcesListRequest};
use ck_assets::vet::{self, VetRequest};
use ck_core::patch::PatchRequest;
use ck_core::{CkError, CkResult, ErrorCode};
use ck_server::idle::DEFAULT_IDLE_SECS;
use ck_server::launch::{self, EnsureRequest, Launcher};
use ck_server::ops::{Ops, caps};
use ck_server::session::ask::AskRequest;
use ck_server::session::brief::{BriefDiffRequest, BriefProposeRequest, BriefWaitRequest};
use ck_server::session::comments::{
    CommentAddRequest, CommentListRequest, CommentReplyRequest, CommentResolveRequest,
};
use ck_server::session::context::ContextRequest;
use ck_server::session::edit::{CutApplyRequest, SourceRequest};
use ck_server::session::export::ExportRequest;
use ck_server::session::handoff::{HandoffRequest, WaitRequest};
use ck_server::session::import::ImportRequest;
use ck_server::session::look::LookApplyRequest;
use ck_server::session::recipe::RecipeApplyRequest;
use ck_server::session::voice::{
    CaptionsRequest, SlotRequest, VoiceCloneRequest, VoiceModeRequest, VoiceScriptRequest,
};
use ck_server::session::{HeartbeatRequest, LockRequest, UnlockRequest};
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `status` takes no input.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StatusArgs {}

/// `project get`: the summary (default) or one layer in full.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProjectGetArgs {
    /// Ids, kinds and spans per track (the default).
    #[serde(default)]
    pub summary: bool,
    /// One layer by id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layer: Option<String>,
}

/// One MCP tool: its CLI twin, description, output cap and input schema.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolSpec {
    pub name: &'static str,
    pub command: &'static str,
    pub description: &'static str,
    /// Output cap in bytes of compact JSON (the same cap as the CLI command).
    pub cap: usize,
    pub input_schema: Value,
}

/// Every tool, in CLI help order.
pub fn specs() -> Vec<ToolSpec> {
    vec![
        spec::<EnsureRequest>(
            "studio",
            "ck studio",
            "Start the studio in the background unless it is running (ck studio --ensure); \
             `open` opens its browser tab once.",
            caps::STUDIO,
        ),
        spec::<StatusArgs>(
            "status",
            "ck status",
            "Project rev, mode, pipeline stage, agent state and live locks.",
            caps::STATUS,
        ),
        spec::<ProjectGetArgs>(
            "project_get",
            "ck project get",
            "The project summary (ids, kinds and spans per track), or one layer in full with \
             `layer`.",
            caps::SUMMARY.max(caps::LAYER),
        ),
        spec::<PatchRequest>(
            "project_patch",
            "ck project patch",
            "Apply an RFC 6902 patch at baseRev. Lock the layers first.",
            caps::PATCH,
        ),
        spec::<LockRequest>(
            "lock",
            "ck lock",
            "Lock layers before touching them; returns the holder's live locks.",
            caps::LOCK,
        ),
        spec::<UnlockRequest>(
            "unlock",
            "ck unlock",
            "Release layers (all of the holder's locks when layerIds is omitted).",
            caps::LOCK,
        ),
        spec::<HeartbeatRequest>(
            "heartbeat",
            "ck heartbeat",
            "Renew the holder's live locks so they do not expire; returns them.",
            caps::LOCK,
        ),
        spec::<BriefProposeRequest>(
            "brief_propose",
            "ck brief propose",
            "Send inferred brief values only (and at most 3 questions, each naming what it \
             affects); the studio shows the Brief page. Then call brief_wait.",
            caps::BRIEF_PROPOSE,
        ),
        spec::<BriefWaitRequest>(
            "brief_wait",
            "ck brief wait",
            "Block until the user confirms the brief (or it is auto), at most 25 s; call again \
             while ready is false. Returns the confirmed brief.",
            caps::BRIEF,
        ),
        spec::<BriefDiffRequest>(
            "brief_diff",
            "ck brief diff",
            "Only what changed in the brief since a rev you read; patch only layers whose \
             briefDeps changed.",
            caps::BRIEF,
        ),
        spec::<StatusArgs>(
            "brief_get",
            "ck brief get",
            "The brief in compact form (value and source per field) with its rev.",
            caps::BRIEF,
        ),
        spec::<TasteSearchRequest>(
            "taste_search",
            "ck taste search",
            "Rank the taste files by query words (id, title, tags); read at most 3 of them.",
            library::caps::TASTE_SEARCH,
        ),
        spec::<IdRequest>(
            "taste_read",
            "ck taste read",
            "One taste file in full: direction with concrete numbers.",
            library::caps::TASTE_READ,
        ),
        spec::<LookListRequest>(
            "look_list",
            "ck look list",
            "The looks in short: id, name, description and motion personality.",
            library::caps::LOOK_LIST,
        ),
        spec::<IdRequest>(
            "look_get",
            "ck look get",
            "One look with all its design tokens (colors, fonts, type scale, motion).",
            library::caps::LOOK_GET,
        ),
        spec::<RecipeSearchRequest>(
            "recipe_search",
            "ck recipe search",
            "Rank the recipes by query words; each hit lists its slots (name*:kind<=chars). \
             Prefer a recipe over hand-built layers when one fits.",
            library::caps::RECIPE_SEARCH,
        ),
        spec::<IdRequest>(
            "recipe_card",
            "ck recipe card",
            "One recipe in full: slots, layers and props.",
            library::caps::RECIPE_CARD,
        ),
        spec::<SourcesListRequest>(
            "sources_list",
            "ck sources list",
            "The Source Hub registry, one line per source: id, status, enabled, license class, \
             kind, repo. Only vetted and enabled sources are searchable.",
            sources::caps::SOURCES_LIST,
        ),
        spec::<SourcesAddRequest>(
            "sources_add",
            "ck sources add",
            "Register a source (unverified, disabled, license unknown until vetted). Never \
             guess a repo: leave it out when unknown.",
            sources::caps::SOURCES_WRITE,
        ),
        spec::<VetRequest>(
            "sources_vet",
            "ck sources vet",
            "Vet a source without running its code: shallow read-only fetch into quarantine \
             (or `from` a local folder), license read from its text, malware-lure and \
             dependency checks. Pass: vetted and enabled; blocker: rejected; unknown license: \
             stays unverified. Pins the result in sources.lock.",
            vet::caps::VET,
        ),
        spec::<IngestRequest>(
            "sources_ingest",
            "ck sources ingest",
            "Turn a vetted source into cards: each exported component gets asset.md, ast.json \
             and a read-only copy of its file (parsing only, nothing runs); rebuilds the \
             catalog. Owner notes are kept.",
            ingest::caps::INGEST,
        ),
        spec::<SourcesGapsRequest>(
            "sources_gaps",
            "ck sources gaps",
            "What the catalog is missing: usable and unverified source counts, sources without \
             a repo, asset kinds with no searchable card.",
            sources::caps::SOURCES_GAPS,
        ),
        spec::<LookApplyRequest>(
            "look_apply",
            "ck look apply",
            "Re-theme the project with a look: one write, layers keep their token refs. Returns \
             the previous look and the layers that change; dryRun only previews.",
            caps::LOOK_APPLY,
        ),
        spec::<RecipeApplyRequest>(
            "recipe_apply",
            "ck recipe apply",
            "Fill a recipe's slots: adds its named layers (tokens, provenance, briefDeps) as \
             one revision. Returns the layer ids.",
            caps::RECIPE_APPLY,
        ),
        spec::<StatusArgs>(
            "layout_check",
            "ck layout check",
            "Check the text layers against the layout rules (safe area, overlap, gap, grid, \
             size, cut text, at most 3 blocks) for the brief's platform. Returns the issues.",
            caps::LAYOUT_CHECK,
        ),
        spec::<StatusArgs>(
            "verify",
            "ck verify",
            "Run the quality gates (schema, gaps, overlaps, duration, layout, contrast, fonts, \
             motion, reuse, provenance, placeholders). Returns passed and failed gates and the \
             issues; fix failures before the next pass.",
            caps::VERIFY,
        ),
        spec::<StatusArgs>(
            "sound_plan",
            "ck sound plan",
            "Plan sound design for the audio pass: whooshes just before cuts, hits on text \
             landings, risers before hooks, room tone under cuts, duck cues for music under \
             voice. Read-only; returns timed cues.",
            caps::SOUND_PLAN,
        ),
        spec::<HandoffRequest>(
            "handoff",
            "ck handoff",
            "Hand the project to the user: release every agent lock and set the agent idle. \
             Then reply in five lines or fewer and loop on wait with since = the returned rev.",
            caps::HANDOFF,
        ),
        spec::<ImportRequest>(
            "import",
            "ck import",
            "Import a media file by absolute path: probe, hash, 720p proxy and 48 kHz audio into \
             the cache, then register it as a project source. Same content twice returns the \
             existing source.",
            caps::IMPORT,
        ),
        spec::<SourceRequest>(
            "transcribe",
            "ck transcribe",
            "Transcribe an imported source with word timestamps; the transcript stays on disk \
             (read it with transcript_get).",
            caps::TRANSCRIBE,
        ),
        spec::<SourceRequest>(
            "transcript_get",
            "ck transcript get",
            "The packed transcript of a source: one line per phrase (#firstWord start-end text, \
             fillers in parentheses) or silence (~ start-end silence).",
            caps::TRANSCRIPT,
        ),
        spec::<SourceRequest>(
            "cut_suggest",
            "ck cut suggest",
            "Cut candidates for a source (silence, filler, retake) on word boundaries with \
             confidence, plus scene changes. Apply with cut_apply.",
            caps::CUT_SUGGEST,
        ),
        spec::<CutApplyRequest>(
            "cut_apply",
            "ck cut apply",
            "Apply suggested cuts (ids, or all when empty): the source's footage on the video \
             track becomes the kept segments, end to end, as one revision.",
            caps::CUT_APPLY,
        ),
        spec::<ExportRequest>(
            "export",
            "ck export",
            "Export a footage-only project to MP4 (fast ffmpeg path): preset source, youtube, \
             tiktok, reels or shorts; normalized to -14 LUFS, -1 dBTP and measured.",
            caps::EXPORT,
        ),
        spec::<VoiceScriptRequest>(
            "voice_script",
            "ck voice script",
            "Split the script into voice slots v1..vn, one per line, with target windows \
             (estimated at 2.5 words/s when absent); replaces the slots and the voice track, \
             keeping takes of unchanged lines.",
            caps::VOICE_SCRIPT,
        ),
        spec::<VoiceModeRequest>(
            "voice_mode",
            "ck voice mode",
            "Switch a voice slot between ai, recorded and cloned; takes are kept.",
            caps::VOICE_WRITE,
        ),
        spec::<SlotRequest>(
            "voice_speak",
            "ck voice speak",
            "Generate speech for an ai or cloned slot (cached by text and voice); the take is \
             trimmed, normalized and fit to the window. placeholder=true means no audio could \
             be made: the studio plays a labeled browser voice.",
            caps::VOICE_WRITE,
        ),
        spec::<VoiceCloneRequest>(
            "voice_clone",
            "ck voice clone",
            "Clone the speaker's voice from a take recorded in the studio that the user \
             consented to (default the active take); the slot becomes cloned. Uploaded or \
             generated audio is refused.",
            caps::VOICE_WRITE,
        ),
        spec::<CaptionsRequest>(
            "voice_captions",
            "ck voice captions",
            "Captions from each slot's active take with its real word timing (transcribed once \
             per take); one caption item per slot follows its voice item. Slots without a \
             take are listed in skipped.",
            caps::VOICE_SCRIPT,
        ),
        spec::<AskRequest>(
            "ask",
            "ck ask",
            "Ask the user a question in the studio (up to 6 options; they may also type or \
             speak an answer) and block up to 25 s. answered=false: call again with the id.",
            caps::ASK,
        ),
        spec::<ContextRequest>(
            "context",
            "ck context",
            "What the user points at in the studio: selected layer, hovered element, playhead, \
             loop range and open comments. Call it first when the user says this, here or that.",
            caps::CONTEXT,
        ),
        spec::<CommentListRequest>(
            "comments_list",
            "ck comments list",
            "The send bundle: open and working comments (all with all=true) with anchors, \
             declared scope and last reply, plus the anchored layers in short.",
            caps::COMMENTS,
        ),
        spec::<CommentAddRequest>(
            "comments_add",
            "ck comments add",
            "Add a comment anchored to a moment (tStart), range (tStart, tEnd), layer (layerId) \
             or element (layerId, propPath); use it to flag an uncertainty for the user.",
            caps::COMMENT_WRITE,
        ),
        spec::<CommentReplyRequest>(
            "comments_reply",
            "ck comments reply",
            "Reply to a comment. An agent reply on an open comment starts work on it; scope \
             lists the layers you will change besides the anchored one (others are refused \
             with E_SCOPE until you resolve).",
            caps::COMMENT_WRITE,
        ),
        spec::<CommentResolveRequest>(
            "comments_resolve",
            "ck comments resolve",
            "Resolve a comment, with an optional closing reply; returns the layers changed \
             since it was made.",
            caps::COMMENT_WRITE,
        ),
        spec::<WaitRequest>(
            "wait",
            "ck wait",
            "Block until the user changes something after `since` (comments, brief edits, \
             confirmations, layer edits), at most 25 s. Returns events oldest first; empty on \
             timeout. Call again with since = the returned rev.",
            caps::WAIT,
        ),
    ]
}

/// Whether `name` runs over the built-in asset library, without a studio.
pub fn is_offline(name: &str) -> bool {
    matches!(
        name,
        "taste_search"
            | "taste_read"
            | "look_list"
            | "look_get"
            | "recipe_search"
            | "recipe_card"
            | "sources_list"
            | "sources_add"
            | "sources_gaps"
            | "sources_vet"
            | "sources_ingest"
    )
}

/// Tools over the built-in asset library and the Source Hub: they need no studio. `None` when
/// `name` is not one. `root` is the project folder the hub is found from.
pub fn offline(root: &std::path::Path, name: &str, args: Value) -> Option<CkResult<String>> {
    let hub = || Hub::locate(root);
    let out = match name {
        "taste_search" => parse(args).and_then(|r| json(&library::taste_search(&r)?)),
        "taste_read" => parse(args).and_then(|r| json(&library::taste_read(&r)?)),
        "look_list" => parse(args).and_then(|r| json(&library::look_list(&r)?)),
        "look_get" => parse(args).and_then(|r| json(&library::look_get(&r)?)),
        "recipe_search" => parse(args).and_then(|r| json(&library::recipe_search(&r)?)),
        "recipe_card" => parse(args).and_then(|r| json(&library::recipe_card(&r)?)),
        "sources_list" => parse::<SourcesListRequest>(args).and_then(|r| {
            let out = sources::sources_list(&hub()?, &r)?;
            json(&ck_server::ops::trimmed(
                "sources list",
                sources::caps::SOURCES_LIST,
                out,
            )?)
        }),
        "sources_add" => {
            parse::<SourcesAddRequest>(args).and_then(|r| json(&sources::sources_add(&hub()?, &r)?))
        }
        "sources_vet" => parse::<VetRequest>(args).and_then(|r| {
            let out = vet::vet(&hub()?, &r)?;
            json(&ck_server::ops::trimmed(
                "sources vet",
                vet::caps::VET,
                out,
            )?)
        }),
        "sources_ingest" => parse::<IngestRequest>(args).and_then(|r| {
            let out = ingest::ingest(&hub()?, &r)?;
            json(&ck_server::ops::trimmed(
                "sources ingest",
                ingest::caps::INGEST,
                out,
            )?)
        }),
        "sources_gaps" => parse::<SourcesGapsRequest>(args)
            .and_then(|r| json(&sources::sources_gaps(&hub()?, &r)?)),
        _ => return None,
    };
    Some(out)
}

fn spec<T: JsonSchema>(
    name: &'static str,
    command: &'static str,
    description: &'static str,
    cap: usize,
) -> ToolSpec {
    // Serializing a schemars schema does not fail; fall back to the widest valid shape anyway.
    let mut input_schema = serde_json::to_value(schemars::schema_for!(T))
        .unwrap_or_else(|_| serde_json::json!({ "type": "object" }));
    if let Value::Object(map) = &mut input_schema {
        // MCP clients only need the shape; the draft URI and title are noise in the tool list.
        map.remove("$schema");
        map.remove("title");
    }
    ToolSpec {
        name,
        command,
        description,
        cap,
        input_schema,
    }
}

/// Whether `name` is a tool.
pub fn exists(name: &str) -> bool {
    specs().iter().any(|s| s.name == name)
}

/// Run tool `name` with `args` against the studio and return its compact JSON output.
pub fn call(ops: &Ops, name: &str, args: Value) -> CkResult<String> {
    match name {
        "status" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.status()?)
        }
        "project_get" => match parse::<ProjectGetArgs>(args)? {
            ProjectGetArgs {
                summary: true,
                layer: Some(_),
            } => Err(request_err("give summary or layer, not both".to_owned())),
            ProjectGetArgs {
                layer: Some(id), ..
            } => json(&ops.layer(&id)?),
            ProjectGetArgs { layer: None, .. } => json(&ops.summary()?),
        },
        "project_patch" => json(&ops.patch(&parse(args)?)?),
        "lock" => json(&ops.lock(&parse(args)?)?),
        "unlock" => json(&ops.unlock(&parse(args)?)?),
        "heartbeat" => json(&ops.heartbeat(&parse(args)?)?),
        "brief_propose" => json(&ops.brief_propose(&parse(args)?)?),
        "brief_wait" => json(&ops.brief_wait(&parse(args)?)?),
        "brief_diff" => json(&ops.brief_diff(&parse(args)?)?),
        "look_apply" => json(&ops.look_apply(&parse(args)?)?),
        "import" => json(&ops.import(&parse(args)?)?),
        "export" => json(&ops.export(&parse(args)?)?),
        "comments_list" => json(&ops.comments_list(&parse(args)?)?),
        "ask" => json(&ops.ask(&parse::<AskRequest>(args)?)?),
        "voice_script" => json(&ops.voice_script(&parse(args)?)?),
        "voice_mode" => json(&ops.voice_mode(&parse(args)?)?),
        "voice_speak" => json(&ops.voice_speak(&parse(args)?)?),
        "voice_clone" => json(&ops.voice_clone(&parse(args)?)?),
        "voice_captions" => json(&ops.voice_captions(&parse::<CaptionsRequest>(args)?)?),
        "context" => {
            let _: ContextRequest = parse(args)?;
            json(&ops.context()?)
        }
        "comments_add" => json(&ops.comments_add(&parse(args)?)?),
        "comments_reply" => json(&ops.comments_reply(&parse(args)?)?),
        "comments_resolve" => json(&ops.comments_resolve(&parse(args)?)?),
        "transcribe" => json(&ops.transcribe(&parse(args)?)?),
        "transcript_get" => json(&ops.transcript_get(&parse(args)?)?),
        "cut_suggest" => json(&ops.cut_suggest(&parse(args)?)?),
        "cut_apply" => json(&ops.cut_apply(&parse(args)?)?),
        "recipe_apply" => json(&ops.recipe_apply(&parse(args)?)?),
        "brief_get" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.brief_get()?)
        }
        "layout_check" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.layout_check()?)
        }
        "verify" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.verify()?)
        }
        "sound_plan" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.sound_plan()?)
        }
        "handoff" => json(&ops.handoff(&parse(args)?)?),
        "wait" => json(&ops.wait(&parse(args)?)?),
        other => Err(request_err(format!("no tool named {other:?}"))),
    }
}

/// The `studio` tool: `ck studio --ensure` for the project whose `.clipkit` folder is `root`.
/// It needs no running server, so it does not go through `Ops`.
pub fn studio(root: &Path, args: Value) -> CkResult<String> {
    let req: EnsureRequest = parse(args)?;
    json(&launch::ensure(
        project_dir(root),
        &req,
        &Launcher::current(DEFAULT_IDLE_SECS)?,
    )?)
}

/// Ops for every other tool; the first call of a session starts the studio (spec 5.1), as the
/// CLI does.
pub fn connect(root: &Path) -> CkResult<Ops> {
    launch::connect(project_dir(root), &Launcher::current(DEFAULT_IDLE_SECS)?)
}

fn project_dir(root: &Path) -> &Path {
    root.parent().unwrap_or(Path::new("."))
}

/// Missing arguments count as an empty object, so `status` can be called bare.
fn parse<T: DeserializeOwned>(args: Value) -> CkResult<T> {
    let args = if args.is_null() {
        Value::Object(Default::default())
    } else {
        args
    };
    serde_json::from_value(args).map_err(|e| request_err(format!("arguments: {e}")))
}

fn json<T: Serialize>(value: &T) -> CkResult<String> {
    serde_json::to_string(value).map_err(|e| CkError::new(ErrorCode::Server, e.to_string()))
}

fn request_err(message: String) -> CkError {
    CkError::new(ErrorCode::Request, message)
}
