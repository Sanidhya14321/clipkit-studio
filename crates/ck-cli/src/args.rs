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
    /// Serve the project and the studio on 127.0.0.1 until interrupted, or with --ensure start
    /// it in the background if it is not running.
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
    /// Release layers (all of the holder's locks when no ids are given).
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
    /// Brief Studio: propose inferred values, wait for the user, read the brief or its diff.
    #[command(subcommand)]
    Brief(BriefCommand),
    /// Search and read the taste files (direction with concrete numbers).
    #[command(subcommand)]
    Taste(TasteCommand),
    /// List and read the looks (design tokens plus motion personality).
    #[command(subcommand)]
    Look(LookCommand),
    /// Search, read and apply recipes (pre-wired scenes with typed slots).
    #[command(subcommand)]
    Recipe(RecipeCommand),
    /// Check text layout: safe areas, collisions, grid, sizes, cut text.
    #[command(subcommand)]
    Layout(LayoutCommand),
    /// Run the quality gates: schema, gaps, overlaps, duration, layout, contrast, fonts,
    /// motion, reuse, provenance, placeholders.
    Verify,
    /// Plan sound design: whooshes, hits, risers, room tone and ducking.
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
    /// Hand the project to the user: release every agent lock and set the agent idle.
    Handoff {
        /// What the studio shows for the agent while idle.
        #[arg(long)]
        label: Option<String>,
    },
    /// Block until the user changes something after a rev; prints the events (empty on timeout).
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
    /// Set ClipKit up in Claude Code, Codex, Cursor or any MCP client (spec 6.3).
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
