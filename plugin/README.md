# plugin

The shipped Claude Code plugin (spec 6.1). It needs `ck` on PATH.

- `.claude-plugin/plugin.json`: manifest (only file in that folder).
- `hooks/hooks.json`: `ck hook prompt|guard|stop|session-end --dir "$CLAUDE_PROJECT_DIR"`.
- `skills/clipkit-video/`: runtime skill (spec 9.4) with pipeline.md, guardrails.md, voice.md.
- `skills/clipkit-review/`: ITERATE stage (comments, brief edits, voice requests).
- `agents/clipkit-critic.md`: read-only contact sheet critic, ten lines at most.
- `.mcp.json`: registers `ck mcp` (pending, see DECISIONS.md, Phase 3 step 4).

Structure tests: `crates/ck-cli/tests/plugin_structure.rs`.
