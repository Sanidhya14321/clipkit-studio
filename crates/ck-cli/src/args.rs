//! Command-line shape (clap). Names, inputs and outputs match the MCP tools (spec 6.2).

use std::path::PathBuf;

use ck_core::model::Holder;
use clap::{Args, Parser, Subcommand};

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
    /// Serve the project and the studio on 127.0.0.1 until interrupted.
    Studio {
        /// Port to bind; 0 picks a free one.
        #[arg(long, default_value_t = 0)]
        port: u16,
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
