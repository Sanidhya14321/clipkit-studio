//! The MCP tool table (spec 6.2). Each tool is the twin of a ck subcommand: the name is the
//! command path joined with `_` (`ck project get` is `project_get`), the input is the JSON the
//! command takes, and the output is the one line of compact JSON the command prints.

use std::path::Path;

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
use ck_core::patch::PatchRequest;
use ck_core::{CkError, CkResult, ErrorCode};
use ck_eval::EvalRequest;
use ck_eval::bench::{self, BenchReportRequest};
use ck_server::debug::{self, BisectRequest, ReplayRequest, TraceRequest};
use ck_server::demo::{self, DemoRequest};
use ck_server::idle::DEFAULT_IDLE_SECS;
use ck_server::launch::{self, EnsureRequest, Launcher};
use ck_server::ops::{Ops, caps};
use ck_server::profile::{self as taste, ProfileCommandRequest};
use ck_server::session::ask::AskRequest;
use ck_server::session::branch::{
    BranchRequest, HistoryRequest as RevHistoryRequest, VariantsRequest,
};
use ck_server::session::brief::{BriefDiffRequest, BriefProposeRequest, BriefWaitRequest};
use ck_server::session::comments::{
    CommentAddRequest, CommentListRequest, CommentReplyRequest, CommentResolveRequest,
};
use ck_server::session::context::ContextRequest;
use ck_server::session::derive::DeriveRequest;
use ck_server::session::edit::{CutApplyRequest, SourceRequest};
use ck_server::session::edl::{GridBuildRequest, PlanExpandRequest, PlanSetRequest};
use ck_server::session::export::{ExportFormat, ExportRequest};
use ck_server::session::formats::{TimelineExportRequest, TimelineFormat};
use ck_server::session::handoff::{HandoffRequest, WaitRequest};
use ck_server::session::import::ImportRequest;
use ck_server::session::look::LookApplyRequest;
use ck_server::session::music::MusicAnalyzeRequest;
use ck_server::session::recipe::RecipeApplyRequest;
use ck_server::session::reference::ReferenceRequest;
use ck_server::session::render::RenderCheckRequest;
use ck_server::session::storyboard::StoryboardRequest;
use ck_server::session::verify::VerifyRequest;
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
        spec::<AssetSearchRequest>(
            "assets_search",
            "ck assets search",
            "Retrieval ladder step 1: top 5 compact lines (id kind grade licenseClass: use when) \
             from vetted sources, filtered by kind, tag, aspect, grade, collection and the \
             license mode (default the brief's, else commercial-safe).",
            retrieve::caps::SEARCH,
        ),
        spec::<AssetIdRequest>(
            "assets_card",
            "ck assets card",
            "Step 2: one asset card. At most 2 distinct cards per decision (E_BUDGET after).",
            retrieve::caps::CARD,
        ),
        spec::<AssetIdRequest>(
            "assets_skeleton",
            "ck assets skeleton",
            "Step 3: props, animation primitives, imports and the outline with line ranges \
             (about 600 tokens).",
            retrieve::caps::SKELETON,
        ),
        spec::<AssetSliceRequest>(
            "assets_slice",
            "ck assets slice",
            "Step 4: one symbol's source from the outline (about 800 tokens).",
            retrieve::caps::SLICE,
        ),
        spec::<AssetSourceRequest>(
            "assets_source",
            "ck assets source",
            "Step 5: the full source, only with a one-line reason; once per decision.",
            retrieve::caps::SOURCE,
        ),
        spec::<LicenseReportRequest>(
            "license_report",
            "ck license report",
            "Every asset the project uses with license, SPDX, class, attribution line and notes, \
             checked against the license mode; ClipKit originals and the Remotion runtime \
             license listed apart. ok=false means a blocked or unresolved asset.",
            report::caps::REPORT,
        ),
        spec::<EvalRequest>(
            "eval",
            "ck eval",
            "Score an exported video from the file alone: duration, aspect, loudness, clipping, \
             black and frozen frames, cut accuracy against expected cuts, sources unchanged. \
             OCR checks are listed as not measured.",
            ck_eval::caps::EVAL,
        ),
        spec::<ReplayRequest>(
            "replay",
            "ck replay",
            "Rebuild the project from its stored patches without an agent; reproduces=false \
             means history and project disagree. `to` also rebuilds an earlier revision.",
            debug::caps::REPLAY,
        ),
        spec::<BisectRequest>(
            "bisect",
            "ck bisect",
            "Find the first revision where the quality gates (or one `gate`) fail, by binary \
             search over history; returns its label and failing gates.",
            debug::caps::BISECT,
        ),
        spec::<TraceRequest>(
            "trace_bundle",
            "ck trace bundle",
            "Write one redacted, shareable JSON trace (environment, project, history, failing \
             gates, replay status) under .clipkit/traces/.",
            debug::caps::TRACE,
        ),
        spec::<DemoRequest>(
            "demo_reset",
            "ck demo reset",
            "Restore the seeded demo project (brief, three scenes, voice script, a comment) in \
             this folder; refuses a non-demo project without force and while a studio runs.",
            demo::caps::DEMO,
        ),
        spec::<BenchReportRequest>(
            "bench_report",
            "ck bench report",
            "Medians, ranges and pass rates per arm and scenario from bench run lines; tokens \
             only from the agents' own usage (otherwise not measured).",
            bench::caps::REPORT,
        ),
        spec::<ProfileCommandRequest>(
            "profile",
            "ck profile",
            "The local taste profile learned from the user's picks: show its defaults, or \
             clear, disable or enable it. It fills brief fields nobody set; never overrides.",
            ck_server::profile::caps::PROFILE,
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
        spec::<VerifyRequest>(
            "verify",
            "ck verify",
            "Run the quality gates (schema, gaps, overlaps, duration, layout, contrast, fonts, \
             motion, reuse, provenance, placeholders). Returns passed and failed gates and the \
             issues; fix failures before the next pass. critic=true also renders the contact \
             sheet for the clipkit-critic subagent (once per pass, when gates are uncertain).",
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
        spec::<StatusArgs>(
            "doctor",
            "ck doctor",
            "Check this machine: tools, render worker and keys set (values never shown), with \
             a fix line for each problem.",
            ck_server::doctor::caps::DOCTOR,
        ),
        spec::<SourceRequest>(
            "content_map",
            "ck content map",
            "Compact content map of a transcribed source: numbered phrases, hook, fillers, \
             retakes and pauses. Read this, not the transcript; phrase i spans anchors i to i+1.",
            caps::CONTENT_MAP,
        ),
        spec::<GridBuildRequest>(
            "grid_build",
            "ck grid build",
            "Build the rhythm grid in plan.edl: speech from source, beats from music, else a \
             generated grid from the brief's pace (fast by default).",
            caps::GRID,
        ),
        spec::<StatusArgs>(
            "plan_get",
            "ck plan get",
            "Read plan.edl: stored segments and a grid summary.",
            caps::PLAN,
        ),
        spec::<PlanSetRequest>(
            "plan_set",
            "ck plan set",
            "Replace plan.edl segments. Segments use grid anchor indices, never raw times.",
            caps::PLAN_WRITE,
        ),
        spec::<StatusArgs>(
            "plan_candidates",
            "ck plan candidates",
            "Top 3 recipes per segment, one line each; pick one and set it in the segment.",
            caps::PLAN_CANDIDATES,
        ),
        spec::<PlanExpandRequest>(
            "plan_expand",
            "ck plan expand",
            "Expand segments into layers on exact frames, with captions from kept words. Name \
             a segment to change only that segment.",
            caps::PLAN_WRITE,
        ),
        spec::<StatusArgs>(
            "pipeline_detect",
            "ck pipeline detect",
            "Read the mode from the project's sources and open the brief stage. First step of \
             every job.",
            caps::PIPELINE,
        ),
        spec::<StatusArgs>(
            "pipeline_next",
            "ck pipeline next",
            "Finish the current pass or step and move to the next. Passes run in order: \
             structure, audio, text, graphics, transitions, look, polish, then verify.",
            caps::PIPELINE,
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
        spec::<DeriveRequest>(
            "assets_derive",
            "ck assets derive",
            "Adaptation rung 3: a project-local copy of an asset with colours lifted into \
             tokens, data-ck-layer instrumentation and a props schema; provenance recorded and \
             the asset added to assetsUsed. Refused when the license mode does not allow it.",
            caps::DERIVE,
        ),
        spec::<RenderCheckRequest>(
            "render_check",
            "ck render check",
            "Determinism test: renders frames (default first, middle, last) in order and again \
             in reverse through the render worker and compares them. deterministic=false lists \
             the unstable frames (time- or random-dependent layers).",
            caps::RENDER_CHECK,
        ),
        spec::<MusicAnalyzeRequest>(
            "music_analyze",
            "ck music analyze",
            "Bounded audio onset/tempo candidates; full data cached; no timing or license approval.",
            caps::MUSIC,
        ),
        spec::<ReferenceRequest>(
            "reference_analyze",
            "ck reference analyze",
            "Statistics of a reference video only (duration, aspect, shot lengths, cuts per \
             15 s, loudness) and suggested brief fields; apply=true writes them with source \
             reference, keeping fields the user set.",
            caps::REFERENCE,
        ),
        spec::<StoryboardRequest>(
            "storyboard_get",
            "ck storyboard get",
            "Storyboard checkpoint: one still per scene as a sheet shown in the studio. When \
             the brief's checkpoints include storyboard, wait (ck wait) for the user's confirm \
             before building the passes.",
            caps::STORYBOARD,
        ),
        spec::<RevHistoryRequest>(
            "history",
            "ck history",
            "Recent revisions in short, newest first: rev, author, label, layers touched.",
            ck_server::session::branch::caps::HISTORY,
        ),
        spec::<BranchRequest>(
            "branch_save",
            "ck branch save",
            "Save the current project content as a named branch (replaces a save of that name).",
            ck_server::session::branch::caps::BRANCH,
        ),
        spec::<StatusArgs>(
            "branch_list",
            "ck branch list",
            "Saved branches with the revision each was saved at.",
            ck_server::session::branch::caps::BRANCH,
        ),
        spec::<BranchRequest>(
            "branch_switch",
            "ck branch switch",
            "Make the project equal a saved branch, as one undoable revision.",
            ck_server::session::branch::caps::BRANCH,
        ),
        spec::<VariantsRequest>(
            "variants",
            "ck variants",
            "Token-only variants: one still per look (2 or 3) at a frame, rendered by the \
             worker; pick one with look_apply.",
            ck_server::session::branch::caps::VARIANTS,
        ),
        spec::<ExportRequest>(
            "export",
            "ck export",
            "Export the project to MP4 (fast ffmpeg path or render worker): preset source, \
             youtube, tiktok, reels or shorts; normalized to -14 LUFS, -1 dBTP and measured. queue: true returns a job at once; follow it with jobs.",
            caps::EXPORT,
        ),
        spec::<StatusArgs>(
            "jobs",
            "ck jobs",
            "Queued, running and finished exports with progress, newest first.",
            caps::EXPORT_JOBS,
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
            | "assets_search"
            | "assets_card"
            | "assets_skeleton"
            | "assets_slice"
            | "assets_source"
            | "license_report"
            | "eval"
            | "replay"
            | "bisect"
            | "trace_bundle"
            | "demo_reset"
            | "bench_report"
            | "profile"
    )
}

fn capped_json<T: serde::Serialize>(what: &str, cap: usize, v: &T) -> CkResult<String> {
    json(&ck_server::ops::capped(what, cap, v)?)
}

/// Tools over the built-in asset library and the Source Hub: they need no studio. `None` when
/// `name` is not one. `root` is the project's `.clipkit` folder; the hub is found from the
/// project folder.
pub fn offline(root: &std::path::Path, name: &str, args: Value) -> Option<CkResult<String>> {
    let project = project_dir(root);
    let hub = || Hub::locate(project);
    let ladder = || Hub::locate(project).map(|h| Retrieval::new(h, project));
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
            let out = ingest::ingest_with_media(
                &hub()?,
                &r,
                Some(&ck_server::session::music::asset_analysis),
            )?;
            json(&ck_server::ops::trimmed(
                "sources ingest",
                ingest::caps::INGEST,
                out,
            )?)
        }),
        "assets_search" => parse::<AssetSearchRequest>(args).and_then(|r| {
            capped_json(
                "assets search",
                retrieve::caps::SEARCH,
                &ladder()?.search(&r)?,
            )
        }),
        "assets_card" => parse::<AssetIdRequest>(args)
            .and_then(|r| capped_json("assets card", retrieve::caps::CARD, &ladder()?.card(&r)?)),
        "assets_skeleton" => parse::<AssetIdRequest>(args).and_then(|r| {
            capped_json(
                "assets skeleton",
                retrieve::caps::SKELETON,
                &ladder()?.skeleton(&r)?,
            )
        }),
        "assets_slice" => parse::<AssetSliceRequest>(args).and_then(|r| {
            capped_json("assets slice", retrieve::caps::SLICE, &ladder()?.slice(&r)?)
        }),
        "assets_source" => parse::<AssetSourceRequest>(args).and_then(|r| {
            capped_json(
                "assets source",
                retrieve::caps::SOURCE,
                &ladder()?.source(&r)?,
            )
        }),
        "license_report" => parse::<LicenseReportRequest>(args).and_then(|r| {
            let out = report::license_report(&hub()?, &project.join(".clipkit"), &r)?;
            json(&ck_server::ops::trimmed(
                "license report",
                report::caps::REPORT,
                out,
            )?)
        }),
        "eval" => parse::<EvalRequest>(args).and_then(|r| {
            capped_json(
                "eval",
                ck_eval::caps::EVAL,
                &ck_eval::evaluate(project, &r)?,
            )
        }),
        "replay" => parse::<ReplayRequest>(args)
            .and_then(|r| capped_json("replay", debug::caps::REPLAY, &debug::replay(root, &r)?)),
        "bisect" => parse::<BisectRequest>(args)
            .and_then(|r| capped_json("bisect", debug::caps::BISECT, &debug::bisect(root, &r)?)),
        "trace_bundle" => parse::<TraceRequest>(args).and_then(|r| {
            capped_json(
                "trace bundle",
                debug::caps::TRACE,
                &debug::trace_bundle(root, &r)?,
            )
        }),
        "demo_reset" => parse::<DemoRequest>(args)
            .and_then(|r| capped_json("demo reset", demo::caps::DEMO, &demo::reset(project, &r)?)),
        "bench_report" => parse::<BenchReportRequest>(args).and_then(|r| {
            capped_json(
                "bench report",
                bench::caps::REPORT,
                &bench::report(project, &r)?,
            )
        }),
        "profile" => parse::<ProfileCommandRequest>(args).and_then(|r| {
            let path = taste::default_path();
            capped_json(
                "profile",
                taste::caps::PROFILE,
                &taste::run(path.as_deref(), &r)?,
            )
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
        "jobs" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.export_jobs()?)
        }
        "export" => {
            let req: ExportRequest = parse(args)?;
            match req.format {
                ExportFormat::Mp4 if req.queue => json(&ops.export_enqueue(&req)?),
                ExportFormat::Mp4 => json(&ops.export(&req)?),
                ExportFormat::Otio | ExportFormat::Srt => {
                    json(&ops.export_timeline(&TimelineExportRequest {
                        format: if req.format == ExportFormat::Otio {
                            TimelineFormat::Otio
                        } else {
                            TimelineFormat::Srt
                        },
                        out: req.out,
                    })?)
                }
            }
        }
        "history" => json(&ops.history(&parse::<RevHistoryRequest>(args)?)?),
        "storyboard_get" => json(&ops.storyboard_get(&parse::<StoryboardRequest>(args)?)?),
        "music_analyze" => json(&ops.music_analyze(&parse::<MusicAnalyzeRequest>(args)?)?),
        "reference_analyze" => json(&ops.reference_analyze(&parse::<ReferenceRequest>(args)?)?),
        "branch_save" => json(&ops.branch_save(&parse::<BranchRequest>(args)?)?),
        "branch_list" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.branch_list()?)
        }
        "branch_switch" => json(&ops.branch_switch(&parse::<BranchRequest>(args)?)?),
        "variants" => json(&ops.variants(&parse::<VariantsRequest>(args)?)?),
        "render_check" => json(&ops.render_check(&parse::<RenderCheckRequest>(args)?)?),
        "assets_derive" => json(&ops.derive(&parse::<DeriveRequest>(args)?)?),
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
        "verify" => json(&ops.verify(&parse::<VerifyRequest>(args)?)?),
        "sound_plan" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.sound_plan()?)
        }
        "doctor" => {
            let StatusArgs {} = parse(args)?;
            json(&ck_server::doctor::run(
                &std::env::current_dir().unwrap_or_default(),
            ))
        }
        "content_map" => json(&ops.content_map(&parse(args)?)?),
        "grid_build" => json(&ops.grid_build(&parse(args)?)?),
        "plan_get" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.plan_get()?)
        }
        "plan_set" => json(&ops.plan_set(&parse(args)?)?),
        "plan_candidates" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.plan_candidates()?)
        }
        "plan_expand" => json(&ops.plan_expand(&parse(args)?)?),
        "pipeline_detect" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.pipeline_detect()?)
        }
        "pipeline_next" => {
            let StatusArgs {} = parse(args)?;
            json(&ops.pipeline_next()?)
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
