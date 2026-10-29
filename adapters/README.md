# adapters

Texts that `ck init` writes for agents other than Claude Code (spec 6.3). The binary embeds them;
editing a file here changes what the next `ck init` writes.

- `codex/AGENTS.md`: the ClipKit section added between markers to `$CODEX_HOME/AGENTS.md`. The
  same text is the Cursor rule body and the generic `RULES.md`.
- `codex/config.toml`: the `ck mcp` registration added between markers to `$CODEX_HOME/config.toml`.
- `cursor/frontmatter.mdc`: the frontmatter of `.cursor/rules/clipkit.mdc`, written in the
  project folder. The MCP entry goes into `~/.cursor/mcp.json`.
- `mcp/mcp.json`: the registration for any MCP client, written to `~/.clipkit/mcp/mcp.json` with
  `~/.clipkit/mcp/RULES.md` by `ck init --agent mcp`.
