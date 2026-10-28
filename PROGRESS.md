# Progress

STATUS: RUNNING
PHASE: 2

## Handoff (latest)
1. Done: Phase 1 core. ck-core (error codes, model + validation, timeline, patch engine, history with undo/redo, locks, atomic storage, migrations, logging) and ck-pipeline (stage machine, stage and retry gates).
2. Proof: `just check` green; 48 tests pass 5 of 5 runs; review findings fixed with regression tests. Evidence in VERIFICATION.md.
3. Open: see Open questions below. ffmpeg is not installed locally, so the fixtures generator has not run here; CI has not run (nothing pushed).
4. Next: Phase 2 (server and studio shell). Read spec 00, 01, 02, 03, 05, 06 (6.2) and plan in PROGRESS.md.
5. Start: run `scripts/autobuild.sh` for unattended runs, or `/clear` and paste the Phase 2 message from docs/spec/13-kickoff.md.

## Open questions
- Phase 5: `ort` has no stable 2.0 release and downloads ONNX Runtime at build time; confirm static linking into `ck` or evaluate a pure-Rust runtime (DECISIONS.md, asset expansion).
- Phase 10: runtime for the optional Demucs sidecar, given the default install must not add Python.
- Pixabay is wave 1 but placed in Phase 8 because ingesting it needs the vetting pipeline.
- `@remotion/whisper-web` and `@remotion/effects` declare UNLICENSED and `@remotion/animation-utils` declares no license; vetting must read the real terms.
- Lock ids for non-layer content: Phase 1 uses `@project` for project-level fields and track ids for track fields and item order (DECISIONS.md, Phase 1). Revisit if the studio needs finer locks.
- ffmpeg is not installed on the dev machine (`sudo apt install ffmpeg`), so `just fixtures` is unverified locally.

## Roadmap additions: asset supply expansion (2026-10-28)

Spec 14. Implementation stays in the phases below; no finished phase is reopened (only Phase 0 is finished, and it has no affected items).
- Phase 5 (edit slice): local whisper transcription (whisper-rs), Silero VAD via ONNX, ffmpeg scene detection as a second cut-candidate signal, ffmpeg ebur128/loudnorm loudness gates and export presets.
- Phase 8 (assets at scale): wave 2 ingestion (RenderComp Free, cliphou.se, RemotionUI, Remotion Bits, Kenney, Tabler, Simple Icons, IWLTBAP free LUTs) and Pixabay; frame-clock adapter with determinism tests BEFORE any wrappable card (GSAP, Motion, Origin UI, Launch UI); LYGIA and BBC-style restricted fixtures for license mode; site-license snapshots, custom-license and trademark notes.
- Phase 9 (polish and proof): remotion-qa, audio-bed-check and tesseract.js in the evaluator and CI after license checks; ffmpeg loudness in the evaluator; wavesurfer.js, culori, opentype.js in the studio; revisit librosa replacement.
- Phase 10 (stretch): Demucs opt-in sidecar.

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
- [ ] Run the fixtures generator locally (BLOCKED: ffmpeg not installed).
