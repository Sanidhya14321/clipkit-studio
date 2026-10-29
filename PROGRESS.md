# Progress

STATUS: RUNNING
PHASE: 3

## Handoff (latest)
1. Done: Phase 3 step 4 except `.mcp.json`. `plugin/` has the manifest, hooks/hooks.json (four `ck hook` commands on `$CLAUDE_PROJECT_DIR`), the clipkit-video skill (spec 9.4 text plus pipeline.md, guardrails.md and voice.md), the clipkit-review skill and the read-only clipkit-critic agent. The format was checked against the current Claude Code reference.
2. Proof: `crates/ck-cli/tests/plugin_structure.rs`, 8 tests that run the real hook command lines through `sh` and compare the skill against the spec 9.4 block. `just check`: 137 of 137 nextest (1 ignored), hooks 41 of 41, studio OK.
3. Open: writing `plugin/.mcp.json` was refused as a sensitive file (BLOCKED). In an interactive session, write it with the content in DECISIONS.md (Phase 3 step 4), then remove the `#[ignore]` on `mcp_json_registers_ck_mcp_on_the_project_folder`. The reviewer has not yet run on the Phase 3 diff (step 6).
4. Next: step 5, `ck init` / `ck uninstall` for Claude Code and Codex, plus adapters/ texts. Check the Codex MCP config file and format, and the Claude Code plugin install path, with the researcher first.
5. Start: `just check`. Plugin tests: `cargo nextest run -p ck-cli -E 'binary(plugin_structure)' --run-ignored all`.

## Open questions
- Phase 3 step 4: `plugin/.mcp.json` must be written in an interactive session; headless runs refuse it as a sensitive file. The content is in DECISIONS.md, and the test is ignored until then.
- Phase 5: `ort` has no stable 2.0 release and downloads ONNX Runtime at build time; confirm static linking into `ck` or evaluate a pure-Rust runtime (DECISIONS.md, asset expansion).
- Phase 10: runtime for the optional Demucs sidecar, given the default install must not add Python.
- Pixabay is wave 1 but placed in Phase 8 because ingesting it needs the vetting pipeline.
- `@remotion/whisper-web` and `@remotion/effects` declare UNLICENSED and `@remotion/animation-utils` declares no license; vetting must read the real terms.
- Lock ids for non-layer content: Phase 1 uses `@project` for project-level fields and track ids for track fields and item order (DECISIONS.md, Phase 1). Revisit if the studio needs finer locks.
- Phase 2 step 8: direct crate dates are verified on the crates.io API, and tokio, ureq and rmcp are pinned back. The transitive crates cc 1.10.0, find-msvc-tools 0.1.15, libc 0.2.191, rustix 1.1.6, smallvec 1.16.4, uuid 1.29.0 and ureq-proto 0.6.5 are still under two weeks old. In an interactive session run `cargo update -p <name> --precise <older>`, or wait until they age past two weeks.
- Phase 2/3: any token holder can act as agent or human (author/holder are self-declared). Decide whether the studio and the agent get separate credentials.
- Phase 2 step 4: MCP `lock`/`unlock` require `holder` while the CLI defaults `--holder agent`; revisit if strict input parity is wanted (DECISIONS.md, step 4).
- Phase 2 studio (step 6): `pnpm add` is denied in headless runs; dependencies are written into package.json by hand and installed with `pnpm install`. Step 6a found the npm registry reachable: versions and dates checked with `pnpm view` (DECISIONS.md, step 6a).
- Phase 2 step 6a: the CI action majors (pnpm/action-setup@v4, actions/setup-node@v5) are unverified; confirm on the first CI run. CSP was added in step 8.
- Phase 2 step 6b: spec 5.2's Stop button needs cooperative cancel through the event bus (no route yet). Spec 5.3's global lock for structural changes has no lock id in ck-core. Both are deferred (DECISIONS.md, step 6b); confirm Phase 3 and Phase 6 pick them up.
- Phase 2 step 8: the CI `e2e` job (Playwright Chromium, no cache) is unverified until the first push. Reviewer leftovers: status/summary reads over the cap are refused instead of paged; a no-op heartbeat or unlock still fsyncs and publishes; no cap on concurrent WebSockets; the second-studio check has a check-then-bind gap.

## Roadmap additions: asset supply expansion (2026-10-28)

Spec 14. Implementation stays in the phases below; no finished phase is reopened (only Phase 0 is finished, and it has no affected items).
- Phase 5 (edit slice): local whisper transcription (whisper-rs), Silero VAD via ONNX, ffmpeg scene detection as a second cut-candidate signal, ffmpeg ebur128/loudnorm loudness gates and export presets.
- Phase 8 (assets at scale): wave 2 ingestion (RenderComp Free, cliphou.se, RemotionUI, Remotion Bits, Kenney, Tabler, Simple Icons, IWLTBAP free LUTs) and Pixabay; frame-clock adapter with determinism tests BEFORE any wrappable card (GSAP, Motion, Origin UI, Launch UI); LYGIA and BBC-style restricted fixtures for license mode; site-license snapshots, custom-license and trademark notes.
- Phase 9 (polish and proof): remotion-qa, audio-bed-check and tesseract.js in the evaluator and CI after license checks; ffmpeg loudness in the evaluator; wavesurfer.js, culori, opentype.js in the studio; revisit librosa replacement.
- Phase 10 (stretch): Demucs opt-in sidecar.

## Phase 3: plugin and auto-launch (2026-10-29)

Spec: 00, 05 (5.1), 06, 09. Proof: typing a video prompt in Claude Code opens the studio and the agent can patch a layer while the user watches.

Steps:
- [x] 1. `ck studio --ensure [--open] [--quiet]` and its MCP twin `studio`: idempotent, starts a detached server in the background, opens the browser tab once per server.
- [x] 2. First ck or MCP call ensures the studio is running (spec 5.1, agents without hooks).
- [x] 3. `ck hook prompt|guard|stop|session-end` (spec 6.1 hook behavior, fast, never blocks a non-video session).
- [x] 4. (`.mcp.json` BLOCKED: write refused headless; see handoff) `plugin/`: manifest, skills (clipkit-video with spec 9.4 text, clipkit-review), clipkit-critic agent, .mcp.json, hooks/hooks.json; format checked against the current plugins reference; structure tests.
- [ ] 5. `ck init` / `ck uninstall` for Claude Code and Codex (detect, ask, idempotent, no residue); adapters/ texts.
- [ ] 6. Proof run, reviewer and spec-checker, evidence, phase done.

Run plan (autonomous run 4): step 4 only.
- Researcher checks the current plugins/hooks/skills/agents reference; differences recorded in DECISIONS.md (and hook field names re-checked).
- plugin/: .claude-plugin/plugin.json, .mcp.json (`ck mcp`), hooks/hooks.json (prompt, guard on Write|Edit|Bash, stop, session-end via `ck hook <name> --dir "$CLAUDE_PROJECT_DIR"`).
- skills/clipkit-video/SKILL.md with the spec 9.4 text verbatim plus pipeline.md, guardrails.md, voice.md; skills/clipkit-review/SKILL.md; agents/clipkit-critic.md (read-only, at most ten lines).
- Structure tests (Rust, in ck-cli tests): files exist, JSON parses, hook commands name real `ck hook` subcommands, SKILL.md under 100 lines and contains the 9.4 text, frontmatter present, no timestamps.

Run plan (autonomous run 3): step 3 only.
- ck-cli `hook` module: reads the Claude Code hook JSON from stdin (capped), project folder from `--dir` (hooks.json will pass `$CLAUDE_PROJECT_DIR`).
- `prompt`: acts only for a video-looking prompt or an existing .clipkit; ensure + open, one-line note (url, mode, stage, agent). `guard`: only in a .clipkit project; exit 2 with a reason for Write/Edit on .clipkit/, assets/, media files and Bash running ffmpeg/ffprobe/sox, touching .clipkit or mutating assets/ or media.
- `stop`: never starts a studio; respects stop_hook_active; blocks once on agent locks or agent status working. `session-end`: releases agent locks; always exit 0.
- Tests: unit tests for the classifiers; CLI e2e per hook (non-video session silent and exit 0, block exit 2, stop loop guard, session-end unlock).

Run plan (autonomous run 2): step 2 only.
- ck-server `launch::connect(dir, launcher)`: Ops for the running studio, or ensure one first (the start also opens the tab once, best effort, so hookless agents see it).
- CLI ops commands and every MCP tool except `studio` go through it; E_SERVER now only means the start failed.
- Tests: each CLI command and the first MCP call start a studio; a start failure (corrupt project.json) is the same error in both doors; existing E_SERVER tests updated. Test helpers set BROWSER=true so no real browser opens.

Run plan (autonomous run 1): step 1 only.
- ck-server `launch`: running-server probe (moved from `ck studio`), `ensure` under a file lock: spawn `ck studio` detached (new process group, stderr to .clipkit/studio.log), wait until the new pid answers status, report early exits with the log tail.
- `--open`: open `url#token=` with `$BROWSER` or the platform opener once per server (marker .clipkit/opened); `--quiet` prints nothing.
- MCP `studio` tool ({open}) runs the same ensure; tool table, caps and snapshot updated.
- Tests: CLI e2e (start, idempotent second call, open once via a fake `$BROWSER`, quiet, stale server.json, early-exit error), MCP parity for `studio`.

## Phase 2: server and studio shell (2026-10-28) - DONE

Spec: 00, 01, 02, 03, 05, 06 (6.2). Proof: a CLI patch appears in the browser in under 300 ms and human edits are refused while locked.

Run plan (autonomous run 1): steps 1 and 2 only.
- Verify tokio/axum/tower versions (researcher), record in DECISIONS.md.
- ck-server: `Session` (Store + Document behind a mutex, injectable clock), event bus (broadcast), server.json (port + token, 127.0.0.1 free port).
- HTTP API (status, project summary/layer, patch, lock, unlock, heartbeat) and `/ws` push; token, Host and Origin checks; body limit.
- Tests: oneshot router tests for every route and refusal, a real-socket WS test that a patch event arrives.

Run plan (autonomous run 2): step 3 only.
- Deps: clap (derive) for ck-cli, ureq (no TLS, no proxy) as the loopback HTTP client; record in DECISIONS.md.
- ck-server `client` (server.json discovery, token header, CkError from error bodies) and `ops` (status, summary, layer, patch, lock, unlock) with documented output size caps (new E_OUTPUT_CAP).
- ck-cli (clap): `ck studio [--port]` (creates an empty project if none), `ck status`, `ck project get --summary|--layer`, `ck project patch <json|->`, `ck lock`, `ck unlock`; compact JSON out, CkError JSON + exit 1 on failure.
- Tests: ops against a live server (each op, error mapping, missing server), cap tests, CLI end-to-end against `ck studio` in a temp dir.

Run plan (autonomous run 3): step 4 only.
- Dep: rmcp (server, transport-io) in ck-mcp; version from `cargo info`, recorded in DECISIONS.md.
- ck-mcp: tool table (status, project_get, project_patch, lock, unlock) with JSON Schema inputs from the Rust request types; each call runs `ck_server::ops` in spawn_blocking and returns the same compact JSON the CLI prints (errors: CkError JSON, isError).
- `ck mcp` subcommand serving it on stdio.
- Tests: insta snapshot of the tool list, per-tool cap table test, e2e parity test driving `ck mcp` over stdio against a real `ck studio` and comparing with CLI output.

Run plan (autonomous run 4): step 5 only.
- Dep: ts-rs (serde-compat, serde-json-impl); version from `cargo info`, recorded in DECISIONS.md.
- Derive `TS` on every wire type: ck-core model, PatchRequest, CkError/ErrorCode; ck-server Event, Status, Summary, request/result types. u64/usize map to `number` (JSON numbers, all under 2^53).
- ck-server `ts_types::render()` joins every declaration into one file, `studio/src/generated/types.ts`.
- Drift test in nextest (so `just check` and CI fail on drift); `just types` regenerates (CK_WRITE_TYPES=1).

Run plan (autonomous run 5): step 6a only (step 6 split: 6a scaffold + embed, 6b panels).
- npm deps (react, vite, remotion, @remotion/player, typescript, prettier) written into studio/package.json by hand; versions and release dates from `pnpm view`, recorded in DECISIONS.md.
- studio: token from the URL fragment, API/WS client, live project state from `snapshot` events, Remotion Player preview whose composition is a pure function of the project (items carry `data-ck-layer`).
- ck-server: rust-embed serves `studio/dist` (SPA fallback to index.html, no token for static files, site guard still applies); a clear "studio not built" page when dist is empty.
- just: `studio-build`, `studio-dev`; tests: static serving via a fake asset table, guard still applies, real build served by `ck studio` checked once.

Run plan (autonomous run 6): step 6b only.
- Pure studio logic in `studio/src/state.ts` (event reducer, activity feed, flash, live agent locks, editability, human patch builder) with `node --test` unit tests (Node 24 strips types; `@types/node` devDep), wired into `just studio-check`.
- Panels: banner (agent status, stage/pass, label; "agent stopped" when its locks expired), activity feed (one line per revision), timeline (tracks, items by span, select, flash on touch, ghost cursor on agent-active/locked layers), inspector (name, purpose, timing; edits sent as JSON patches with baseRev/layerRevs, author human).
- Read-only: inspector controls disabled on agent-locked layers; server refusal (E_LOCK_HELD) shown inline. Canvas outline + flash via Player inputProps (composition stays pure).
- Out of scope (recorded): Stop button (needs cooperative cancel, Phase 3), token usage per step (meter, later), global structural lock (core has none yet).

Run plan (autonomous run 7): step 7 only.
- Dep: @playwright/test 1.64.1 (released 2026-10-14, checked with `pnpm view`) as a studio devDep; Chromium only, installed with `pnpm exec playwright install chromium`.
- Fixture per test: a fresh temp dir, `target/debug/ck studio --dir <tmp>` spawned, server.json read for port + token, the page opened at `url#token=`.
- Test 1: `ck project patch` (the real CLI) adds a track and layer; a MutationObserver in the page timestamps the layer's arrival; assert under 300 ms from the CLI spawn.
- Test 2: `ck lock` as agent; the inspector shows `data-ck-readonly=true` with disabled inputs; the studio's own patch request is refused with 423 E_LOCK_HELD; after `ck unlock` the edit lands.
- `just e2e` builds ck and the studio, then runs Playwright; evidence in VERIFICATION.md.

Run plan (autonomous run 8): step 8 only (phase wrap-up).
- Release dates for the unverified Phase 2 crates from the crates.io API (node fetch); pin back any under two weeks old.
- rust-reviewer on the Phase 2 diff (4f9a064..HEAD) and spec-checker on Phase 2; fix real gaps, record the rest as open.
- Decide CSP and the CI e2e job (simplest option that fits the spec; DECISIONS.md).
- VERIFICATION.md Phase 2 final rows, `just check` + `just e2e` green, tick step 8, `PHASE: 3`, handoff, commit.

Steps (each ends with `just check`):
- [x] 1. Deps + ck-server core: session state, event bus, server.json, bind to 127.0.0.1 on a free port.
- [x] 2. HTTP + WS API with token/Host/Origin guards; refusals tested (E_AUTH, E_LOCK_HELD over HTTP).
- [x] 3. Shared ops layer + CLI basics: `ck studio` (serve), `ck status`, `ck project get --summary|--layer`, `ck project patch` (spec 6.2 naming), `ck lock`, `ck unlock`; output size caps.
- [x] 4. MCP twin (rmcp, stdio) for the same ops; schema, size cap and CLI/MCP parity tests.
- [x] 5. TS type generation from the Rust types (ts-rs or specta, verified), checked in CI for drift.
- [x] 6. Studio shell: Vite + React + Remotion Player preview, timeline, inspector, read-only mode, banner, activity feed, ghost cursor; embedded with rust-embed.
  - [x] 6a. Scaffold, live project state over WS, Player preview, rust-embed serving, `just studio-build|studio-dev|studio-check`.
  - [x] 6b. Panels: timeline, inspector, read-only mode, banner, activity feed, ghost cursor.
- [x] 7. Playwright e2e: CLI patch visible in under 300 ms; human edit refused while the agent holds the lock.
- [x] 8. Evidence, reviewers, DECISIONS.md, handoff, commit; mark phase done. Review follow-ups: `ck heartbeat` + MCP twin, idle shutdown (spec 5.1), CSP + X-Frame-Options, CI e2e job, open-time check (spec 1.2), crate pins.

## Phase 1: core (2026-10-28) - DONE

Spec: 02 (2.2), 03 (3.2, 3.3), 04 (4.1, 4.2), 10 (10.1, 10.4). Proof: heavy unit + property tests green.

Steps (each ends with `just check`):
- [x] 1. Deps: serde, serde_json, json-patch, schemars, thiserror, tracing (+ tracing-subscriber in ck-cli), dev: proptest, insta, tempfile. Versions verified >= 2 weeks old, recorded in DECISIONS.md.
- [x] 2. `ck-core::error`: `ErrorCode` enum (E_STAGE, E_LOCK_HELD, E_PATCH_STALE, E_PATCH_INVALID, E_SCHEMA, E_STORAGE, E_MIGRATION, E_RETRY, ...) with stable string codes + hints; `CkError`. Snapshot test pins every code.
- [x] 3. `ck-core::model`: Project, Source, Track, Item (footage/clip/voice/audio/caption + common fields), VoiceSlot, Comment, Brief, Pipeline, Agent, Locks (spec 3.2), `schemaVersion`. Serde camelCase + JsonSchema; insta snapshot of the JSON Schema; round-trip proptest.
- [x] 4. `ck-core::timeline`: seconds <-> frames, item spans, overlap detection. Proptests.
- [x] 5. `ck-core::patch`: apply RFC 6902 patch with `baseRev` + per-layer revs; validate result against the model; stale -> E_PATCH_STALE listing only changed layer ids; bump project rev and touched item revs. Inverse patch computed by diff.
- [x] 6. `ck-core::history`: append-only revision log (author, label, timestamp, touched layer ids, forward + inverse patch); undo/redo as NEW revisions (history never rewritten). Proptest: apply then undo == original; undo then redo == applied; random sequences keep rev monotonic.
- [x] 7. `ck-core::storage`: `.clipkit/` layout, atomic write (temp, fsync, rename, dir fsync), one history file per rev, load + replay check. Tests with tempfile, including a crash leftover temp file.
- [x] 8. `ck-core::migrate`: versioned migration chain; v0 fixture project (no schemaVersion) loads as v1. Test that every older fixture loads.
- [x] 9. `ck-core::locks`: per-layer locks for agent or human, heartbeat, expiry via injectable clock; human edits refused while agent holds the layer (E_LOCK_HELD). Proptests on random lock/heartbeat/expire sequences.
- [x] 10. `ck-pipeline::stage`: stage machine for spec 4.1 (BOOT..ITERATE, passes in order), operation -> allowed-stage table, E_STAGE with hint naming the next allowed step, brief-auto exception; retry gate (same failing call + args refused after two attempts, E_RETRY). Exhaustive transition tests.
- [x] 11. Logging: standard field names (traceId, runId, turnId, stage, pass, layerId, rev; camelCase per spec 10.4) and secret redaction in `ck-core::log`; JSON subscriber init in `ck-cli`. Test captures a log line and checks fields + redaction.
- [x] 12. VERIFICATION.md evidence, rust-reviewer + spec-checker subagents, DECISIONS.md, handoff, commit.

Review follow-ups (fixed in the wrap-up commit): agent edits to project-level content, track fields, item moves across tracks and item reorders now go through the lock gate; storage checks revision file names against their rev, history continuity from the base rev, strict temp-file names, overflow-safe rev arithmetic; log fields renamed to spec camelCase.
Deferred by design: `.clipkit/trace/` and `.clipkit/cache/` folders (flight recorder Phase 2+/9, cache Phase 5); wiring the stage gate to patches happens in the Phase 2 server.

Out of scope for Phase 1 (later phases): scope, budget, destructive, quality and honesty gates (Phases 4, 6, 8); props-schema validation for clips (Phase 4); TS type generation (Phase 2); CLI/MCP commands (Phase 2).

## Phase 0: setup (2026-10-28)

Plan (approved by the owner before building):
- [x] Archive the Codex implementation to branch `codex-archive`; clean `main`; move gitignored leftovers out of the repo.
- [x] Check current Claude Code docs for memory, skills, subagents, hooks, settings; record in DECISIONS.md.
- [x] Copy the prompt to docs/MASTER_PROMPT.md; split it into docs/spec/ with `scripts/split-spec.py`; write INDEX.md.
- [x] Cargo workspace skeleton: 9 crates from spec 3.1, pinned toolchain, workspace lints, nextest config.
- [x] Placeholder folders: studio/, render-worker/, plugin/, adapters/, assets/quarantine/, fixtures/.
- [x] CLAUDE.md (55 lines).
- [x] Skills: spec-lookup, phase-runner, rust-conventions, studio-frontend, media-ffmpeg, asset-pipeline, testing-and-evidence, debug-triage.
- [x] Subagents: test-runner, spec-checker, rust-reviewer, researcher.
- [x] Hooks: format-file (PostToolUse), guard-edits and guard-bash (PreToolUse), stop-check (Stop); permissions in .claude/settings.json.
- [x] Hook payload tests: tests/hooks/run.sh.
- [x] justfile: check, check-fast, test, fmt, studio-dev, e2e, fixtures, bench, split-spec.
- [x] CI: .github/workflows/ci.yml (fast suite + fixtures job).
- [x] Fixtures generator: fixtures/generate.sh -> fixtures/generated/ with ground-truth.json.
- [x] Run the fixtures generator locally (ffmpeg 8.0.1, 2026-10-28).
