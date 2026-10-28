# Progress

STATUS: RUNNING
PHASE: 2

## Handoff (latest)
1. Done: Phase 2 step 7. Playwright e2e (`studio/e2e/`, @playwright/test 1.64.1, Chromium) runs a real `ck studio` per test on a fresh temp project and drives it with the real `ck` CLI. `just e2e` builds ck and the studio, then runs the tests.
2. Proof: both phase proofs PASS. A CLI patch shows in the timeline and preview 11 to 26 ms after the CLI spawn (budget 300 ms). While the agent holds the lock the inspector is read-only, its inputs are disabled, and the studio's own patch gets 423 E_LOCK_HELD; after unlock a UI edit lands. Results: `just e2e` 2 of 2, 20 of 20 on repeat, `just check` green (96 nextest, hooks 41, studio 10).
3. Open: e2e is not in CI yet. The banner is checked only when idle, because `/agent` is read-only until Phase 3. Stop button, the global structural lock and the CSP are still deferred. Plus the earlier items.
4. Next: Phase 2 step 8. Run rust-reviewer on the phase diff and spec-checker on Phase 2, decide on CSP and the CI e2e job, finish VERIFICATION.md (version-date rows), tick the phase done and set `PHASE: 3`.
5. Start: `just check`, `just e2e` (one-time `pnpm --dir studio exec playwright install chromium`). Manual: `just studio-build`, `cargo run -p ck-cli -- studio --dir <folder>`, open `url#token=<server.json token>`.

## Open questions
- Phase 5: `ort` has no stable 2.0 release and downloads ONNX Runtime at build time; confirm static linking into `ck` or evaluate a pure-Rust runtime (DECISIONS.md, asset expansion).
- Phase 10: runtime for the optional Demucs sidecar, given the default install must not add Python.
- Pixabay is wave 1 but placed in Phase 8 because ingesting it needs the vetting pipeline.
- `@remotion/whisper-web` and `@remotion/effects` declare UNLICENSED and `@remotion/animation-utils` declares no license; vetting must read the real terms.
- Lock ids for non-layer content: Phase 1 uses `@project` for project-level fields and track ids for track fields and item order (DECISIONS.md, Phase 1). Revisit if the studio needs finer locks.
- Phase 2: release dates of tokio 1.54.1, axum 0.8.9, getrandom 0.4.3, tower 0.5.3, tokio-tungstenite 0.29.0, futures-util 0.3.34 are unverified (headless runs cannot reach the web; the researcher and web-fetch subagents were denied). Re-check in an interactive session and pin back any release under two weeks old.
- Phase 2/3: any token holder can act as agent or human (author/holder are self-declared). Decide whether the studio and the agent get separate credentials.
- Phase 2 step 3: release dates of clap 4.6.7 and ureq 3.5.0 are unverified (same headless limit); re-check and pin back if under two weeks old.
- Phase 2 step 3: the E_SERVER hint says `ck studio --ensure`, which Phase 3 adds; until then an agent following it gets a usage error. Keep the hint (it is the shipped wording) or reword before Phase 3 if that slips.
- Phase 2 step 4: rmcp 3.8.0 release date unverified (same headless limit); re-check and pin back if under two weeks old. MCP `lock`/`unlock` require `holder` while the CLI defaults `--holder agent`; revisit if strict input parity is wanted (DECISIONS.md, step 4).
- Phase 2 step 5: ts-rs 12.0.1 release date unverified (same headless limit); re-check and pin back if under two weeks old.
- Phase 2 studio (step 6): `pnpm add` is denied in headless runs; dependencies are written into package.json by hand and installed with `pnpm install`. Step 6a found the npm registry reachable: versions and dates checked with `pnpm view` (DECISIONS.md, step 6a).
- Phase 2 step 6a: rust-embed 8.13.0 release date and the CI action majors (pnpm/action-setup@v4, actions/setup-node@v5) unverified; confirm on the first CI run. No CSP on the studio page yet (Remotion inline styles); still open after 6b, decide in step 8 or Phase 3.
- Phase 2 step 6b: spec 5.2's Stop button needs cooperative cancel through the event bus (no route yet). Spec 5.3's global lock for structural changes has no lock id in ck-core. Both are deferred (DECISIONS.md, step 6b); confirm Phase 3 and Phase 6 pick them up.
- Phase 2 step 7: the Playwright e2e suite is not in CI yet (it needs a Chromium download or cache in the workflow). Decide in step 8.

## Roadmap additions: asset supply expansion (2026-10-28)

Spec 14. Implementation stays in the phases below; no finished phase is reopened (only Phase 0 is finished, and it has no affected items).
- Phase 5 (edit slice): local whisper transcription (whisper-rs), Silero VAD via ONNX, ffmpeg scene detection as a second cut-candidate signal, ffmpeg ebur128/loudnorm loudness gates and export presets.
- Phase 8 (assets at scale): wave 2 ingestion (RenderComp Free, cliphou.se, RemotionUI, Remotion Bits, Kenney, Tabler, Simple Icons, IWLTBAP free LUTs) and Pixabay; frame-clock adapter with determinism tests BEFORE any wrappable card (GSAP, Motion, Origin UI, Launch UI); LYGIA and BBC-style restricted fixtures for license mode; site-license snapshots, custom-license and trademark notes.
- Phase 9 (polish and proof): remotion-qa, audio-bed-check and tesseract.js in the evaluator and CI after license checks; ffmpeg loudness in the evaluator; wavesurfer.js, culori, opentype.js in the studio; revisit librosa replacement.
- Phase 10 (stretch): Demucs opt-in sidecar.

## Phase 2: server and studio shell (2026-10-28) - IN PROGRESS

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
- [ ] 8. Evidence, reviewers, DECISIONS.md, handoff, commit; mark phase done.

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
