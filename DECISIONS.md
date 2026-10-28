# Decisions

Newest first within each section. Each entry: what, why, and the source when it is a verified fact.

## Phase 1 (2026-10-28)

### Crate versions (crates.io release dates checked 2026-10-28; all at least two weeks old)
| Crate | Version | Released | Notes |
| --- | --- | --- | --- |
| serde | 1.0.229 | 2026-07-18 | `derive` |
| serde_json | 1.0.154 | 2026-10-11 | `float_roundtrip`: without it a proptest found f64 values that change by one ULP across save and load |
| json-patch | 4.2.0 | 2026-04-30 | `schemars` feature (depends on schemars 1.x) so patch ops derive JsonSchema |
| schemars | 1.2.3 | 2026-10-12 | |
| thiserror | 2.0.21 | 2026-09-23 | |
| tracing | 0.1.44 | 2025-12-18 | |
| tracing-subscriber | 0.3.23 | 2026-03-13 | `json`, `env-filter`; ck-cli only |
| proptest | 1.12.0 | 2026-10-10 | dev |
| insta | 1.49.0 | 2026-10-03 | dev, `json` |
| tempfile | 3.27.0 | 2026-03-11 | dev |

### Model choices where spec 3.2 is silent
- Items are a common struct plus a body tagged by `type` (footage, clip, voice, audio, caption); `rev` per item is engine-set.
- Footage and audio `src` are source ids and are checked against `sources`. Caption `ref` names a source or voice slot.
- `plan`, `derived` and `branches` stay untyped JSON until Phases 4, 8 and 9 define them.
- Voice slot status values: empty, pending, ready, failed. Brief `effort` and `licenseMode` are strings until their phases.
- Locks live in the project (spec 3.2) and carry holder (agent or human), heartbeat and ttl.
- Undo and redo are new revisions (approved by the owner); default lock ttl 30 s (approved).

## Phase 0 (2026-10-28)

### Repository
- **Reuse the existing GitHub repo** (`Sanidhya14321/clipkit-studio`, local folder `Hackcbs-Project`)
  at the owner's request, to keep contributors. The previous Codex-built TypeScript implementation is
  preserved on the local branch `codex-archive` (commit `1cc3aff`, including all of its uncommitted
  work) and removed from `main`. Its untracked, gitignored leftovers (`.clipkit/`, `node_modules/`,
  `artifacts/`, build output, about 5 GB) were moved to `~/Desktop/clipkit-codex-leftovers/`, not deleted.
- Kept `LICENSE` (MIT, "ClipKit Studio contributors").
- The repo root is the workspace root (spec 3.1 shows a `clipkit/` folder; that folder is this repo).

### Claude Code formats (verified 2026-10-28 in the official docs)
Sources: code.claude.com/docs/en/memory, /skills, /sub-agents, /hooks, /hooks-guide, /permissions, /settings.
- **Memory**: project memory is `./CLAUDE.md` (or `./.claude/CLAUDE.md`); `CLAUDE.local.md` is personal
  and gitignored. `@path` imports exist (max depth four) but do not reduce context cost, so CLAUDE.md
  uses plain pointers instead. Docs target under 200 lines; this project keeps it under 120 (spec 11.1).
- **Skills**: `.claude/skills/<name>/SKILL.md`, YAML frontmatter then Markdown. All fields optional;
  we set `name` and `description` (description + `when_to_use` are truncated at 1,536 chars in the
  listing). Supporting files sit in the skill folder and are linked with relative Markdown links.
- **Subagents**: `.claude/agents/<name>.md`; required `name` and `description`; optional `tools`
  (comma-separated string or YAML list), `disallowedTools`, `model` (`haiku`, `sonnet`, `opus`,
  `fable`, a full model id, or `inherit`), and more. Unknown fields are ignored.
  - Pattern-scoped entries such as `Bash(git diff *)` inside an agent's `tools` were **not verified**,
    so agents list plain tool names and use `disallowedTools` (Edit, Write, NotebookEdit) for read-only
    roles; the prompt restricts Bash to read-only git commands for rust-reviewer.
  - Models: `haiku` for test-runner and researcher (routine), `sonnet` for spec-checker and rust-reviewer.
- **Hooks** (in `.claude/settings.json`): `hooks.<Event>[] = {matcher, hooks: [{type: "command",
  command, timeout}]}`. A matcher of only letters/digits/`|` is an exact list (`Write|Edit`).
  stdin JSON carries `tool_name`, `tool_input.file_path` / `tool_input.command`, `cwd`; Stop carries
  `stop_hook_active`. Exit 0 = ok; exit 2 = block, stderr goes to Claude; other = non-blocking error.
  Stop and PostToolUse can return `{"decision":"block","reason":...}`. `$CLAUDE_PROJECT_DIR` is the
  project root. Default command timeout is 600 s, so every hook sets its own.
  - The harness also caps consecutive Stop continuations at 8 (`CLAUDE_CODE_STOP_HOOK_BLOCK_CAP`);
    our Stop hook additionally exits 0 whenever `stop_hook_active` is true, so it blocks at most once.
- **Permissions**: evaluated deny, then ask, then allow; first match wins; list keys merge across
  settings files. `Bash(cargo *)` (space form) matches `cargo` and `cargo build` but not `cargofoo`.
  `Read(./.env)` style path rules are documented; `**` globs in Read rules were **not verified** with
  an exact example, so the guard-edits hook and `.gitignore` back them up.
  `$schema`: `https://json.schemastore.org/claude-code-settings.json`.

### Build setup choices
- **Hook scripts** live in `.claude/hooks/*.sh` (bash + jq) and are tested by `tests/hooks/run.sh`
  with recorded payloads (exit code, output, under 2 s). CI runs it inside `just check`.
- **Stop hook** skips instantly when no Rust-related file changed, so docs-only turns stay fast.
- **guard-bash** is a heuristic net, not a sandbox; permissions are the real boundary. It checks line
  by line so heredoc bodies (for example a `.gitignore` listing `.env`) do not trigger it.
- **Studio formatter hook** uses the package's own `node_modules/.bin/prettier` once Phase 2 installs it;
  until then it does nothing.
- **`.env.example`** stays editable; every other `.env*` file is blocked from edits and reads.
- **Asset quarantine folder** is `assets/quarantine/` (spec 3.1 names no folder).
- **Ask** (not allow) for `cargo add/install/update`, `pnpm add/update/dlx`, `npm install`, `npx` and
  `git push`: these are the "network installs of new dependencies" from spec 11.6. Reads and edits
  outside the repo already prompt by default, so no extra rule is needed.
- **Workspace lints**: `unsafe_code = forbid`, `clippy::unwrap_used = deny` (tests allowed via
  `clippy.toml`), `dbg_macro = deny`.
- **No dependencies in Phase 0**: crates are empty skeletons. Each phase adds the crates it needs,
  checks the current version, and records it below.
- **Fixtures** are generated by `fixtures/generate.sh` (bash, ffmpeg argument arrays) into the gitignored
  `fixtures/generated/`, with `ground-truth.json`. The frame-counter overlay needs ffmpeg built with
  `drawtext`; without it the video is still generated and `ground-truth.json` says `"overlay": false`.

### Tool versions (checked 2026-10-28)
| Tool | Version | Note |
| --- | --- | --- |
| Rust toolchain | 1.99.1 stable (2026-10-13) | pinned in `rust-toolchain.toml`; edition 2024, resolver 3 |
| just | installed via `cargo install --locked` | version in VERIFICATION.md |
| cargo-nextest | installed via `cargo install --locked` | version in VERIFICATION.md |
| actions/checkout | v7 (latest v7.0.1) | `gh api .../releases/latest` |
| Swatinem/rust-cache | v2 (latest v2.9.3) | |
| taiki-e/install-action | v2 (latest v2.88.0) | installs just and cargo-nextest in CI |
| jq | 1.8.1 local | hooks depend on it |
| ffmpeg | not installed locally; Ubuntu candidate 8.0.1 | CI installs it from apt |
| Node | 24.21.0 local | studio and render worker arrive in Phases 2 and 9 |
