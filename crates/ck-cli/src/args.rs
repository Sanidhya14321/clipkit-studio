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
    /// Claude Code plugin hooks: read the hook JSON on stdin; exit 2 blocks (spec 6.1).
    #[command(subcommand)]
    Hook(HookCommand),
    /// Set ClipKit up in Claude Code, Codex, Cursor or any MCP client: show the plan, ask, then
    /// write (spec 6.3).
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
