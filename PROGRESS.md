# Progress

STATUS: RUNNING
PHASE: 2

## Handoff (latest)
1. Done: Phase 2 steps 1-2. ck-server: Session (document + store + event bus), server.json (port, token, 0600), Host/Origin/token guards, HTTP API (status, project views, patch, lock, unlock, heartbeat) and `/ws` push.
2. Proof: `just check` green (64 tests, hooks 41); 16 ck-server tests, including human edit refused while locked and WS events < 300 ms server side. Evidence in VERIFICATION.md (Phase 2).
3. Open: crate release dates unverified (no web in headless runs); role trust boundary (callers state author/holder); stage gate wiring waits for Phase 4 pipeline commands. See DECISIONS.md, Phase 2.
4. Next: Phase 2 step 3: shared ops + CLI basics (`ck studio`, `ck status`, `ck project get`, `ck patch`, `ck lock`, `ck unlock`) talking to the server via server.json; needs clap and an HTTP client (verify versions).
5. Start: run `scripts/autobuild.sh`, or open a session and continue Phase 2 at step 3 in PROGRESS.md.

## Open questions
- Phase 5: `ort` has no stable 2.0 release and downloads ONNX Runtime at build time; confirm static linking into `ck` or evaluate a pure-Rust runtime (DECISIONS.md, asset expansion).
- Phase 10: runtime for the optional Demucs sidecar, given the default install must not add Python.
- Pixabay is wave 1 but placed in Phase 8 because ingesting it needs the vetting pipeline.
- `@remotion/whisper-web` and `@remotion/effects` declare UNLICENSED and `@remotion/animation-utils` declares no license; vetting must read the real terms.
- Lock ids for non-layer content: Phase 1 uses `@project` for project-level fields and track ids for track fields and item order (DECISIONS.md, Phase 1). Revisit if the studio needs finer locks.
- Phase 2: release dates of tokio 1.54.1, axum 0.8.9, getrandom 0.4.3, tower 0.5.3, tokio-tungstenite 0.29.0, futures-util 0.3.34 are unverified (headless runs cannot reach the web; the researcher and web-fetch subagents were denied). Re-check in an interactive session and pin back any release under two weeks old.
- Phase 2/3: any token holder can act as agent or human (author/holder are self-declared). Decide whether the studio and the agent get separate credentials.
- Phase 2 studio (step 6): `pnpm add` is denied in headless runs; dependencies must be written into package.json by hand and installed with `pnpm install`, with versions that cannot be verified without web access.

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

Steps (each ends with `just check`):
- [x] 1. Deps + ck-server core: session state, event bus, server.json, bind to 127.0.0.1 on a free port.
- [x] 2. HTTP + WS API with token/Host/Origin guards; refusals tested (E_AUTH, E_LOCK_HELD over HTTP).
- [ ] 3. Shared ops layer + CLI basics: `ck studio` (serve), `ck status`, `ck project get --summary|--layer`, `ck patch`, `ck lock`, `ck unlock`; output size caps.
- [ ] 4. MCP twin (rmcp, stdio) for the same ops; schema, size cap and CLI/MCP parity tests.
- [ ] 5. TS type generation from the Rust types (ts-rs or specta, verified), checked in CI for drift.
- [ ] 6. Studio shell: Vite + React + Remotion Player preview, timeline, inspector, read-only mode, banner, activity feed, ghost cursor; embedded with rust-embed.
- [ ] 7. Playwright e2e: CLI patch visible in under 300 ms; human edit refused while the agent holds the lock.
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
