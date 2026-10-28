# Decisions

Newest first within each section. Each entry: what, why, and the source when it is a verified fact.

## Asset supply expansion (2026-10-28, owner update)

Source: docs/sources/asset-expansion-catalog.md, treated as third-party data. Spec: docs/spec/14-asset-expansion.md.

### Routes for the catalog's Python tools (single Rust binary, no Python runtime)
Crates checked on crates.io on 2026-10-28. Each is re-verified when its phase starts; nothing is added to Cargo.toml yet.
- **Transcription (faster-whisper)**: `whisper-rs` 0.16.0 (2026-03-12, Unlicense; repo moved to codeberg.org/tazz4843/whisper-rs) binding whisper.cpp (MIT). Builds whisper.cpp from C++ source, so the build needs a C++ toolchain and cmake. The binary stays single; the ggml model file is downloaded on first use, pinned per project and cached. Local whisper is the default, Fish Audio STT the cloud option. Model weight license to be read at Phase 5.
- **VAD (Silero)**: `ort` has no stable 2.0 release; candidates are 2.0.0-rc.14 (2026-10-12) or rc.13 (2026-07-28); rc.15 (2026-10-19) is too new. MIT OR Apache-2.0, MSRV 1.92. Open point for Phase 5: ort downloads ONNX Runtime binaries at build time; confirm it links statically into `ck`, or evaluate a pure-Rust ONNX runtime (for example tract) instead. `voice_activity_detector` 0.2.1 was rejected: non-standard license and pinned to ort rc.10.
- **Scene detection (PySceneDetect)**: ffmpeg scene score (`scdet` filter or `select='gt(scene,T)'` with `showinfo`) from ck-media. No new crate.
- **Loudness (pyloudnorm)**: ffmpeg `ebur128` to measure and two-pass `loudnorm` to normalize, for every gate and export preset, so measurement and normalization share one meter. The `ebur128` crate (0.1.10, 2024-10-26, MIT; 0.1.11 from 2026-10-26 is too new) is kept as an optional in-process cross-check in tests only.
- **Onset and beat (librosa)**: deferred. `aubio` and `aubio-rs` were last released in 2021; `beat-detector` 0.1.2 is tiny and unmaintained. Revisit in Phase 9; until then pacing uses taste rules.
- **Stems (Demucs)**: deferred to Phase 10 as an optional opt-in sidecar, off by default, never in the interactive path, stems cached by source hash. The sidecar's runtime is an open question, because it must not add Python to the default install.
- JavaScript tools stay on the TypeScript side: tesseract.js, remotion-qa and audio-bed-check in the evaluator and CI (Phase 9, after license checks); wavesurfer.js, culori and opentype.js in the studio (Phase 9).

### sources.yaml
- Restored unchanged from `codex-archive` (38 entries; JSON is valid YAML), then extended. Existing entries are untouched except the `remotion` line, which gains a `packages` list.
- 14 `@remotion/*` packages confirmed on npm (4.0.539) were added to the `remotion` line. `@remotion/claude-code-plugin` and `@remotion/codex-plugin` were named by the catalog but return 404 on npm, so they were not added.
- 25 new entries: `status: unverified`, `enabled: false`, `licenseClass: unknown`, plus the claimed license in `license`, `wave`, `phase`, `homepage` for site sources and `licenseNote` for custom, site, CC BY, trademark and paid-tier notes.
- Repos left null where the catalog gives none or hedges: remotionui, remotion-bits, cliphouse, remotion-qa, audio-bed-check, origin-ui (two candidate orgs), kenney-audio, iwltbap-luts, pixabay-media.
- Kind and tier come from the catalog's prose because its YAML companion was not supplied.
- Pixabay is in wave 1, but ingesting it needs the Phase 8 vetting pipeline, so its phase is 8.

### Spec folder
- `docs/spec/14-asset-expansion.md` is an owner-requested addition. The guard-edits hook blocks Write and Edit under `docs/spec/` by design, so it was written through the shell at the owner's explicit request. `split-spec.py` only rewrites files 00 to 13.

## Phase 2 (2026-10-28)

### Crate versions (latest on crates.io per `cargo search` on 2026-10-28)
Release dates are PLACEHOLDER: this headless run could not reach the web (WebFetch and the researcher's tools were denied), so the two-week-age rule is unverified. Open question: re-check dates and pin back if any is too new.
| Crate | Version | Notes |
| --- | --- | --- |
| tokio | 1.54.1 | rt-multi-thread, macros, net, sync, time, signal |
| axum | 0.8.9 | default features off; http1, json, query, tokio, ws |
| getrandom | 0.4.3 | session token from the OS generator (`getrandom::fill`) |
| tower | 0.5.3 | dev, `util` for `oneshot` |
| tokio-tungstenite | 0.29.0 | dev; same major as axum's ws dependency (0.30.0 is newest) |
| futures-util | 0.3.34 | dev |

### Server choices (steps 1 and 2)
- **New error codes**: E_AUTH (token, Host or Origin refused), E_NOT_FOUND, E_REQUEST (malformed or oversized body), E_SERVER (bind, task or state failures). Added to the pinned list; existing codes unchanged.
- **Guards**: every route checks Host is `127.0.0.1:<port>` or `localhost:<port>` (DNS rebinding) and Origin, when present, is this server (cross-site). `/api/*` and `/ws` need the token: header `x-ck-token`; the `token` query parameter counts only on `/ws` (browsers cannot set WebSocket headers), so API tokens stay out of URLs.
- **server.json** holds port, token, pid and URL; written via a 0600 temp file and rename, removed on shutdown only if it still describes this server.
- **Events**: `snapshot` (whole project, on connect and after each change, only built when a subscriber exists), `revision` (rev, author, label, touched) and `locks`. The studio renders from snapshots; diffs over the wire can come later if size demands it.
- **Writes** run on `spawn_blocking` (they fsync). A failed save reloads the document from disk; if the reload also fails, the session refuses all calls with E_STORAGE.
- **Stage gate not wired yet**: agent patches are allowed only in BASE, PASSES and ITERATE, but no pipeline commands exist before Phase 4, so wiring it now would make every Phase 2 agent patch fail. It is wired together with the pipeline commands.
- **Trust boundary (open)**: callers state `author`/`holder` themselves; any token holder can patch or unlock as either role. Acceptable for one local user; per-role credentials (agent token vs studio token) are an open question for Phase 3, when the plugin and studio get distinct launch paths.
- Deferred: idle shutdown, refusing a second server while server.json's pid is alive (`ck studio --ensure`, step 3 / Phase 3); redacting paths in 5xx bodies (local, same-user only).

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

### Engine choices (Phase 1)
- **Content vs engine state**: diffs and undo work on project content with `rev`, item `rev`, `locks`, `pipeline` and `agent` removed. Patches may not touch those fields (E_PATCH_INVALID); the engine sets them.
- **Lock ids beyond layers**: the spec locks "touched layers" but is silent on non-layer content. Simplest safe rule: an item id when the item is added, removed, changed or moved to another track; a track id when the track's fields or the relative order of its items change; `@project` for any other content (settings, brief, sources, voice slots, comments, tokens, plan). Ids may not start with `@`. The agent must hold every touched id; humans are refused on agent-held ids. (Found by the Phase 1 rust-reviewer: without this, agent edits outside items skipped the lock gate.)
- **Locks on ids that do not exist yet** are allowed, so the agent can lock a layer before adding it. Lock changes are not revisions; they are saved inside project.json.
- **A patch that changes nothing** is refused (E_PATCH_INVALID) so history holds no empty revisions.
- **Stale patches** are refused whenever `baseRev` is behind (no automatic rebase); the error lists items changed since `baseRev` plus any `layerRevs` mismatches.
- **Storage**: `history/base.json` is the replay starting point; history must run contiguously from base rev + 1 to the project rev, and each file name must equal the rev inside. Temp files are `.<name>.tmp-<pid>` and only those are removed on load.
- **Log field names** are camelCase as spec 10.4 writes them (traceId, runId, turnId, layerId, plus layerIds for multi-layer events). `ck` logs JSON to stderr only when `CK_LOG` is set.
- **Stage machine**: VERIFY may send work back to any pass; ITERATE always returns to VERIFY; leaving BRIEF needs a confirmed or auto brief. Operation-to-stage table in `ck-pipeline/src/stage.rs`.
- **Retry gate**: a call is the op name plus its arguments with keys sorted; refused after 2 failures, reset by a success.

### Autonomous build loop (2026-10-28)
- Commits and PRs carry no Claude attribution (owner request): `.claude/settings.json` sets `attribution.commit` and `attribution.pr` to "" (the `attribution` key replaces the deprecated `includeCoAuthoredBy`; code.claude.com settings reference).
- `scripts/autobuild.sh` pushes `main` to origin after a run that made a commit and passed `just check` (plain push, never forced; `PUSH=0` turns it off). The agent never pushes.
- Dry run (MAX_RUNS=1, 2026-10-28): success, 1 commit (Phase 2 steps 1-2), `just check` green, 30 permission denials: 18 WebFetch + 1 WebSearch (crate version lookups; no allow rule), 11 Bash (paths outside the repo, curl, shell loops, python3/sed edits, cd-prefixed compounds). The run reported 102 turns with `--max-turns 80`, so the flag may not cap turns as expected.
- `scripts/autobuild.sh` (owner-supplied) runs `claude -p` sessions with `scripts/autobuild-prompt.md`. It reads `STATUS:` and `PHASE:` from the top of PROGRESS.md.
- Checked against the installed CLI (2.1.312): `-p`, `--output-format`, `--permission-mode` are documented in `claude --help`; `--max-turns` is not listed there but is accepted (an unknown flag errors with "unknown option", this one does not).
- The prompt file is the owner's text plus a short "Loop contract" section: keep the STATUS/PHASE lines current, skip the phase-runner approval wait in autonomous runs, and recover an interrupted run's uncommitted changes first.
- `--permission-mode acceptEdits` approves file edits only; Bash runs only what `.claude/settings.json` allows, and anything else is denied in `-p` mode rather than prompting. The owner approved adding git log/show/rev-parse, ls, wc and jq to the allow list (git status/diff were already there) and the autonomous-mode note in CLAUDE.md and the phase-runner skill. git push, dependency installs (`cargo add/install/update`, `pnpm add`, `npm install`, `npx`) stay "ask", which headless runs treat as denied; `.env` reads stay denied.

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
