# adapters

Texts that `ck init` writes for agents other than Claude Code (spec 6.3). The binary embeds them;
editing a file here changes what the next `ck init` writes.

- `codex/AGENTS.md`: the ClipKit section added between markers to `$CODEX_HOME/AGENTS.md`.
- `codex/config.toml`: the `ck mcp` registration added between markers to `$CODEX_HOME/config.toml`.

Cursor rules and a generic MCP config are not written yet (Phase 3 covers Claude Code and Codex).
