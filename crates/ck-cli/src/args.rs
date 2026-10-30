//! Command-line shape (clap). Names, inputs and outputs match the MCP tools (spec 6.2).

use std::path::PathBuf;

use ck_core::model::Holder;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

#[derive(Debug, Parser)]
#[command(
    name = "ck",
    version,
    about = "ClipKit Studio",
    arg_required_else_help = true
)]
pub struct Cli {
    /// Project folder (holds .clipkit/).
    #[arg(long, global = true, default_value = ".")]
    pub dir: PathBuf,
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Serve the project and studio on 127.0.0.1 (--ensure: in the background).
    Studio {
        /// Port to bind; 0 picks a free one.
        #[arg(long, default_value_t = 0)]
        port: u16,
        /// Stop after this many seconds with no API call and no open studio tab; 0 never stops.
        #[arg(long, default_value_t = ck_server::idle::DEFAULT_IDLE_SECS)]
        idle: u64,
        /// Return at once: start the studio in the background unless it is already running.
        #[arg(long)]
        ensure: bool,
        /// With --ensure: open the studio in the browser, once per server.
        #[arg(long, requires = "ensure")]
        open: bool,
        /// With --ensure: print nothing on success.
        #[arg(long, requires = "ensure")]
        quiet: bool,
    },
    /// Serve the same operations as MCP tools on stdio (spec 6.2).
    Mcp,
    /// Project rev, mode, pipeline stage, agent state and live locks.
    Status,
    /// Read or change the project.
    #[command(subcommand)]
    Project(ProjectCommand),
    /// Lock layers before touching them.
    Lock {
        #[arg(required = true)]
        layer_ids: Vec<String>,
        #[command(flatten)]
        holder: HolderArg,
    },
    /// Release layers (all of the holder's locks by default).
    Unlock {
        layer_ids: Vec<String>,
        #[command(flatten)]
        holder: HolderArg,
    },
    /// Renew the holder's live locks so they do not expire.
    Heartbeat {
        #[command(flatten)]
        holder: HolderArg,
    },
    /// Brief Studio: propose, wait, read the brief or its diff.
    #[command(subcommand)]
    Brief(BriefCommand),
    /// Search and read the taste files.
    #[command(subcommand)]
    Taste(TasteCommand),
    /// List, read and apply the looks (design tokens).
    #[command(subcommand)]
    Look(LookCommand),
    /// Search, read and apply recipes (scenes with typed slots).
    #[command(subcommand)]
    Recipe(RecipeCommand),
    /// Check text layout (safe areas, collisions, sizes).
    #[command(subcommand)]
    Layout(LayoutCommand),
    /// Run the quality gates (schema, timing, layout, contrast, motion, provenance).
    Verify {
        /// Also render the critic's contact sheet (6 frames at cuts and motion peaks).
        #[arg(long)]
        critic: bool,
    },
    /// Plan sound cues: whooshes, hits, risers, room tone, ducking.
    #[command(subcommand)]
    Sound(SoundCommand),
    /// Import a media file as a project source.
    Import { path: std::path::PathBuf },
    /// Transcribe a source with word timestamps.
    Transcribe { source: String },
    /// Read a packed transcript.
    #[command(subcommand)]
    Transcript(TranscriptCommand),
    /// Suggest and apply cuts.
    #[command(subcommand)]
    Cut(CutCommand),
    /// Voice slots: script, mode.
    #[command(subcommand)]
    Voice(VoiceCommand),
    /// Ask the user a question in the studio; prints the answer.
    Ask {
        /// The question (omit with --id to keep waiting).
        question: Option<String>,
        /// An option to offer; repeat for more.
        #[arg(long = "option")]
        options: Vec<String>,
        /// Keep waiting on an earlier question.
        #[arg(long)]
        id: Option<String>,
        /// Seconds to block (default 20, at most 25).
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// What the user points at in the studio, plus open comments.
    Context,
    /// List, add, reply to and resolve comments.
    #[command(subcommand)]
    Comments(CommentsCommand),
    /// Retrieval ladder over vetted assets: search, card, skeleton, slice, source.
    #[command(subcommand)]
    Assets(AssetsCommand),
    /// License and attribution report for the assets the project uses.
    #[command(subcommand)]
    License(LicenseCommand),
    /// Source Hub: list, add and report gaps.
    #[command(subcommand)]
    Sources(SourcesCommand),
    /// Score an exported video from the file alone.
    Eval {
        video: String,
        /// Expectations JSON {durationSec?, aspect?, loudness?, cutsAtSec?, sources?}, or -.
        #[arg(long)]
        expect: Option<String>,
    },
    /// Rebuild the project from its patches (no agent).
    Replay {
        /// Also rebuild this earlier revision.
        #[arg(long)]
        to: Option<u64>,
    },
    /// First revision where the quality gates fail.
    Bisect {
        #[arg(long)]
        gate: Option<String>,
    },
    /// Reference video statistics and suggested brief fields.
    #[command(subcommand)]
    Reference(ReferenceCommand),
    /// Recent revisions in short.
    History {
        #[arg(long)]
        limit: Option<usize>,
    },
    /// Save, list and switch branches.
    #[command(subcommand)]
    Branch(BranchCommand),
    /// One still per look at a frame (2 or 3 looks).
    Variants {
        #[arg(long = "look")]
        looks: Vec<String>,
        #[arg(long)]
        frame: Option<u64>,
    },
    /// Benchmark report from bench run lines.
    #[command(subcommand)]
    Bench(BenchCommand),
    /// The local taste profile: show, clear, disable, enable.
    Profile {
        #[arg(value_enum, default_value_t = ProfileArg::Show)]
        action: ProfileArg,
    },
    /// The seeded demo project.
    #[command(subcommand)]
    Demo(DemoCommand),
    /// Redacted trace bundle.
    #[command(subcommand)]
    Trace(TraceCommand),
    /// Render checks (determinism).
    #[command(subcommand)]
    Render(RenderCommand),
    /// Export the project to MP4 (fast path or render worker).
    Export {
        /// source, youtube, tiktok, reels or shorts.
        #[arg(long, value_enum, default_value_t = PresetArg::Source)]
        preset: PresetArg,
        /// Output path relative to the project folder.
        #[arg(long)]
        out: Option<String>,
    },
    /// Hand the project to the user: release agent locks, go idle.
    Handoff {
        /// What the studio shows for the agent while idle.
        #[arg(long)]
        label: Option<String>,
    },
    /// Block until the user changes something after a rev; prints the events.
    Wait {
        /// Report revisions after this rev (from handoff or the last wait); default the current rev.
        #[arg(long)]
        since: Option<u64>,
        /// Seconds to block (default 20, at most 25); 0 checks once.
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// Claude Code plugin hooks: read the hook JSON on stdin; exit 2 blocks.
    #[command(subcommand)]
    Hook(HookCommand),
    /// Set ClipKit up in Claude Code, Codex, Cursor or any MCP client.
    Init(InstallArgs),
    /// Remove everything ck init added.
    Uninstall(InstallArgs),
}

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Agent to set up (repeatable); default: the agents found on this machine.
    #[arg(long = "agent", value_enum)]
    pub agents: Vec<AgentKind>,
    /// Write without asking.
    #[arg(long)]
    pub yes: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AgentKind {
    ClaudeCode,
    Codex,
    Cursor,
    /// Any MCP client: mcp.json and RULES.md under ~/.clipkit/mcp to point it at.
    Mcp,
}

#[derive(Debug, Subcommand)]
pub enum HookCommand {
    /// UserPromptSubmit: start and open the studio for video prompts; one context line.
    Prompt,
    /// PreToolUse: refuse direct edits to .clipkit/, assets/, media and ck-bypassing commands.
    Guard,
    /// Stop: block once while the agent holds locks or is mid-stage.
    Stop,
    /// SessionEnd: release the agent's locks.
    SessionEnd,
}

#[derive(Debug, Subcommand)]
pub enum ProjectCommand {
    /// Summary (default) or one layer in full.
    Get {
        /// Ids, kinds and spans per track.
        #[arg(long, conflicts_with = "layer")]
        summary: bool,
        /// One layer by id.
        #[arg(long)]
        layer: Option<String>,
    },
    /// Apply a patch: a JSON object {baseRev, ops, author, label?, layerRevs?}, or - for stdin.
    Patch { patch: String },
}

#[derive(Debug, Subcommand)]
pub enum BriefCommand {
    /// Send inferred values: a JSON file {fields, confidence?, questions?, auto?}, or - for stdin.
    Propose { file: String },
    /// Block until the user confirms the brief (or it is auto); call again while ready is false.
    Wait {
        /// Seconds to block (default 20, at most 25); 0 checks once.
        #[arg(long)]
        timeout: Option<u64>,
    },
    /// What changed in the brief since a rev you read.
    Diff {
        #[arg(long)]
        since: u64,
    },
    /// The brief in compact form with its rev.
    Get,
}

#[derive(Debug, Subcommand)]
pub enum TasteCommand {
    /// Rank the taste files by query words (id, title, tags); read at most 3 of them.
    Search {
        /// Words to look for; empty lists every file.
        #[arg(default_value = "")]
        query: String,
        /// Most hits to return (default 5, at most 12).
        #[arg(long)]
        limit: Option<usize>,
    },
    /// One taste file in full.
    Read { id: String },
}

#[derive(Debug, Subcommand)]
pub enum LookCommand {
    /// The looks in short.
    List,
    /// One look with all its design tokens.
    Get { id: String },
    /// Re-theme the project with a look; prints the layers that change.
    Apply {
        id: String,
        /// Only preview the impact; write nothing.
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum RecipeCommand {
    /// Rank the recipes by query words; each hit lists its slots (`name*:kind<=chars`).
    Search {
        /// Words to look for; empty lists every recipe.
        #[arg(default_value = "")]
        query: String,
        /// Most hits to return (default 5, at most 12).
        #[arg(long)]
        limit: Option<usize>,
    },
    /// One recipe in full: slots, layers and props.
    Card { id: String },
    /// Expand a recipe into named layers: a JSON file {recipe, at?, duration?, slots, prefix?,
    /// look?}, or - for stdin.
    Apply { file: String },
}

#[derive(Debug, Subcommand)]
pub enum TranscriptCommand {
    /// The packed transcript: phrases with times, silences and (fillers).
    Get { source: String },
}

#[derive(Debug, Subcommand)]
pub enum CutCommand {
    /// Cut candidates on word boundaries, with confidence, plus scene changes.
    Suggest { source: String },
    /// Apply suggested cuts as footage segments on the video track.
    Apply {
        source: String,
        /// Cut ids from `cut suggest`, comma separated; default every suggested cut.
        #[arg(long, value_delimiter = ',')]
        ids: Vec<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum SoundCommand {
    /// Timed sound cues for the audio pass (read-only).
    Plan,
}

#[derive(Debug, Subcommand)]
pub enum LayoutCommand {
    /// Check the text layers against the layout rules for the brief's platform.
    Check,
}

#[derive(Debug, Args)]
pub struct HolderArg {
    /// Who holds the locks: agent or human.
    #[arg(long, default_value = "agent", value_parser = parse_holder)]
    pub holder: Holder,
}

fn parse_holder(s: &str) -> Result<Holder, String> {
    serde_json::from_value(serde_json::Value::String(s.to_owned()))
        .map_err(|_| format!("{s:?} is not agent or human"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum PresetArg {
    Source,
    Youtube,
    Tiktok,
    Reels,
    Shorts,
}

#[derive(Debug, Subcommand)]
pub enum CommentsCommand {
    /// The send bundle: open and working comments with their layers.
    List {
        /// Include resolved comments.
        #[arg(long)]
        all: bool,
    },
    /// Add a comment anchored to a moment, range, layer or element.
    Add {
        text: String,
        #[arg(long)]
        layer: Option<String>,
        /// Moment, or range start, in seconds.
        #[arg(long)]
        at: Option<f64>,
        /// Range end in seconds.
        #[arg(long)]
        to: Option<f64>,
        /// Element path inside the layer (data-ck-path).
        #[arg(long)]
        path: Option<String>,
    },
    /// Reply; an agent reply on an open comment starts work on it.
    Reply {
        id: String,
        text: String,
        /// Layers you will change besides the anchored one, comma separated.
        #[arg(long, value_delimiter = ',')]
        scope: Option<Vec<String>>,
    },
    /// Resolve a comment; prints the layers changed since it was made.
    Resolve {
        id: String,
        /// Closing reply.
        #[arg(long)]
        text: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum VoiceCommand {
    /// Split a script into voice slots: a JSON file {lines: [{text, window?}], mode?}, or -.
    Script { file: String },
    /// Generate speech for an ai or cloned slot as its new take.
    Speak { slot: String },
    /// Clone the voice from a consented studio recording of a slot.
    Clone {
        slot: String,
        /// Take id; default the active take.
        #[arg(long)]
        take: Option<String>,
    },
    /// Captions from each slot's active take timing.
    Captions,
    /// Switch a slot between ai, recorded and cloned.
    Mode {
        slot: String,
        #[arg(value_enum)]
        mode: ModeArg,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ModeArg {
    Ai,
    Recorded,
    Cloned,
}

#[derive(Debug, Subcommand)]
pub enum SourcesCommand {
    /// One line per source: id, status, enabled, license class, kind, repo.
    List {
        /// unverified, vetted or rejected.
        #[arg(long)]
        status: Option<String>,
    },
    /// Register a source (unverified and disabled until vetted).
    Add {
        id: String,
        #[arg(long)]
        kind: String,
        /// owner/name on GitHub; leave out when unknown.
        #[arg(long)]
        repo: Option<String>,
        #[arg(long, default_value = "")]
        name: String,
        /// The license the listing claims (recorded as a claim).
        #[arg(long, default_value = "")]
        license: String,
    },
    /// Vet a source without running its code; pins the result in sources.lock.
    Vet {
        id: String,
        /// Vet a local folder instead of cloning the repo.
        #[arg(long)]
        from: Option<String>,
    },
    /// Turn a vetted source into cards, ast.json files and catalog lines.
    Ingest { id: String },
    /// What the catalog is missing.
    Gaps,
}

#[derive(Debug, Subcommand)]
pub enum AssetsCommand {
    /// Top 5 compact lines from vetted sources.
    Search {
        #[arg(default_value = "")]
        query: String,
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        tag: Option<String>,
        #[arg(long)]
        aspect: Option<String>,
        /// commercial-safe, personal or any (default: the brief's).
        #[arg(long)]
        mode: Option<String>,
        /// Lowest grade accepted: A, B or C.
        #[arg(long)]
        grade: Option<String>,
        #[arg(long)]
        collection: Option<String>,
        #[arg(long)]
        limit: Option<usize>,
    },
    /// One asset card (at most 2 per decision).
    Card { id: String },
    /// Props, primitives, imports and outline.
    Skeleton { id: String },
    /// One symbol's source.
    Slice { id: String, symbol: String },
    /// A project-local codemod copy (needs the studio running).
    Derive {
        id: String,
        /// Folder name for the copy.
        #[arg(long = "as")]
        name: String,
    },
    /// The full source, with a reason (once per decision).
    Source {
        id: String,
        #[arg(long)]
        reason: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum LicenseCommand {
    /// Every asset used, with license and attribution, checked against the license mode.
    Report {
        /// commercial-safe, personal or any (default: the brief's).
        #[arg(long)]
        mode: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum RenderCommand {
    /// Render frames twice and out of order; report frames that differ.
    Check {
        /// Frame to check; repeat for more (default first, middle, last).
        #[arg(long = "frame")]
        frames: Vec<u64>,
    },
}

#[derive(Debug, Subcommand)]
pub enum TraceCommand {
    /// Write one redacted, shareable JSON trace under .clipkit/traces/.
    Bundle,
}

#[derive(Debug, Subcommand)]
pub enum DemoCommand {
    /// Restore the seeded demo project in this folder.
    Reset {
        /// Replace a project that is not the demo.
        #[arg(long)]
        force: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum BenchCommand {
    /// Medians, ranges and pass rates per arm and scenario.
    Report {
        #[arg(default_value = "bench/results.jsonl")]
        results: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum BranchCommand {
    /// Save the current content under a name.
    Save { name: String },
    /// Saved branches.
    List,
    /// Switch to a saved branch (one undoable revision).
    Switch { name: String },
}

#[derive(Debug, Subcommand)]
pub enum ReferenceCommand {
    /// Statistics only; --apply writes suggested brief fields (source reference).
    Analyze {
        path: String,
        #[arg(long)]
        apply: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ProfileArg {
    Show,
    Clear,
    Disable,
    Enable,
}
