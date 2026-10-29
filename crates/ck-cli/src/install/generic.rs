//! Any MCP client: writes `~/.clipkit/mcp/mcp.json` (the `ck mcp` registration in the common
//! `mcpServers` shape) and `~/.clipkit/mcp/RULES.md` for the user to point their client at
//! (spec 6.3). Never detected; only `--agent mcp` writes it.

use ck_core::CkResult;

use super::{Homes, Mode, Step, owned_dir_step};

const MCP_JSON: &str = include_str!("../../../../adapters/mcp/mcp.json");
const RULES: &str = include_str!("../../../../adapters/codex/AGENTS.md");

pub fn plan(mode: Mode, homes: &Homes) -> CkResult<Vec<Step>> {
    let root = homes.clipkit.join("mcp");
    let files = vec![
        (root.join("mcp.json"), MCP_JSON.to_owned()),
        (root.join("RULES.md"), RULES.to_owned()),
    ];
    Ok(vec![owned_dir_step(mode, &root, files)?])
}
