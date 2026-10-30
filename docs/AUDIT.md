# Phase audit (2026-10-30)

A recheck of phases 0 to 10 (spec 12 has no phases 11 or 12) against VERIFICATION.md,
PROGRESS.md open questions and DECISIONS.md: what was missed, and why. Status as of commit
`ed8cc21` plus this audit.

## 1. Process misses (things that should have been caught)

| What | Since | Why it was missed | Status |
| --- | --- | --- | --- |
| CI on GitHub was red | Phase 7 (25 of 39 recent runs failed) | Runs were pushed but never checked. Cause: the runners had no ffmpeg, so tone-speaker voice tests and the record-along e2e failed, and every fixture-backed media test skipped | Fixed: both jobs install ffmpeg and run `just fixtures`; e2e bundles the render worker. Run 41030811771 green |
| Fixture media tests skipped in CI | Phase 5 (open question) | Recorded as a question, not acted on | Fixed with the CI change above |
| Reviewer and spec-checker subagents never ran | Phases 5 to 10 | Those agent types were not available in the sessions; self reviews only | Open: run them in a session that has them |
| The agent was never shown as working | Phase 2 | Nothing in production set `agent.status`; a Phase 2 test even asserted `idle` "until the pipeline commands" set it, which never happened. The banner (spec 5.2 live view) could not show the agent's label, and lock changes never resent the project to the studio | Fixed: agent writes and agent locks set it working with the patch label and touched layers; releasing all locks or handoff sets it idle; lock changes resend the snapshot |
| Stale evidence rows (spec 14.7 rows, Phase 4 step 10b, CI rows) still said BLOCKED after later phases fixed them | Phases 5, 8, 9 | New rows were added per phase; old rows were not revisited | Fixed in this audit (section 4) |

## 2. Built later than planned (now done)

| Item | Planned | Done in |
| --- | --- | --- |
| Export of the generated video (Phase 4 step 10b) | Phase 4 | Phase 9 step 2 (render worker) |
| Critic pass (`verify --critic`) | Phase 4/9 | Phase 10 step 1 |
| Global structural lock (spec 5.3) | Phase 2 | Phase 6 step 3 |
| History and branches in the studio | Phase 9 | Follow-up after Phase 10 |
| Render against original media; music beds in the mix | Phase 9 | Follow-up after Phase 10 |
| Scene detection, loudness gates, restricted fixture blocked, unverified never searchable (spec 14.7) | Phases 5, 8 | Phases 5 and 8 |

## 3. Still open, and why

### Needs the owner
- **Fish Audio TTS, STT and cloning** (Phases 5, 7): stubs until `FISH_AUDIO_API_KEY` is set and the
  API is verified (spec 8.3 says verify before coding). Local whisper (C++ build) not added.
- **Benchmark results** (Phase 9): need real agent runs (tokens) and the owner's talking-head clip
  and voice sample. Harness and report are built.
- **Backup demo recording** (spec 12.1): needs a person.

### Needs an interactive agent session
- **Phase 3 proof in a real Claude Code and Codex session**: headless runs cannot start one.
- **`plugin/.mcp.json`**: refused as a sensitive file in headless runs; content is in DECISIONS.md.
- **Codex and Cursor config formats**: written from memory; verify against their docs.
- **MCP long-call timeouts** for `wait` and `ask` per agent (Phase 4 step 9c).

### Blocked on a dependency or license decision
- **Sound design placement**: no CC0 sound set has a confirmed source; vetting reads git repos and
  local folders, not site archive downloads (Kenney, Pixabay).
- **Silero VAD**: `ort` has no stable 2.0 and downloads ONNX Runtime at build time.
- **Demucs**: needs Python, which the default install must not add.
- **Wave 3 frame-clock adapter and card previews**: no GSAP/Motion-style source vetted yet.
- **remotion-qa, audio-bed-check, tesseract.js** (spec 14.7, Phase 9): licenses unchecked; OCR
  checks in `ck eval` are therefore "not measured".
- **tabler-icons**: over the quarantine size limits; icon, LUT and sfx ingest not built.

### Not built (scope or time), no external blocker
- Timeline drag, trim, split, snapping and zoom in the studio (spec 5.4).
- Loop range in the studio (reported as absent in `ck context`).
- Side-by-side variants view in the studio.
- Export queue with progress (exports block one request thread; spec 8.6).
- `ck repro`, per-layer render cache, project bundle and standalone Remotion project export.
- Audio ducking in the mix; caption words drawn in preview and export.
- `brief set` command (spec 6.2); stage gating of operations by pipeline stage.
- Ablation switches for the bench's C arms.
- Packaged installers (`cargo install` from source only).
- More agents beyond Claude Code, Codex, Cursor and generic MCP.

### Known weaknesses
- Any token holder can act as agent or human (author is self-declared).
- `tests::idle::only_authorized_calls_count` uses wall-clock margins and can flake under load.
- Seven transitive crates were under two weeks old at Phase 2 close; repin when they age.

## 4. Fixed during the audit
- CI red since Phase 7 (ffmpeg missing on runners); fixture tests skipping in CI.
- Agent working state never set; lock changes not pushed to the studio.
- Stop button with cooperative cancel (spec 5.2): the studio's Stop sets `agent.stopRequested`;
  ck-core refuses every agent patch with E_STAGE until `ck handoff` (which clears it).
- `tests::idle::only_authorized_calls_count` moved to paused tokio time (no wall-clock flake).

## 5. Optimization pass (2026-10-30)
Measured, then changed only what the numbers showed. Details and numbers in VERIFICATION.md
("Follow-ups after Phase 10") and DECISIONS.md ("Optimization pass").

| Area | Before | After |
| --- | --- | --- |
| `just check` wall time | 77 s | 49 s |
| Test CPU (whole suite) | 422 s | 274 s |
| Undo/redo property test | 47 s | 4.7 s |
| Import of a 720p H.264 clip | encode (12 to 16 s in tests) | stream copy (under 2 s) |
| Patch, 500 layers (release) | 28.8 ms | 11.9 ms |
| Patch, 2000 layers (release) | 87.9 ms | 66.1 ms |
| Release binary | 26.9 MB | 16.8 MB |
| Asset search, 62 cards | about 10 ms | unchanged (not worth an index on disk) |
