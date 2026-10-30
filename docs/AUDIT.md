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

## 5. Reviewer findings, phases 5 to 10 (2026-10-30)

rust-reviewer (R) and spec-checker (S), first run. Found by reading code; each fixed item has its test or a passing suite, and R10-class items were not reproduced.

### Fixed in the sprint

| Id | Severity | Finding |
| --- | --- | --- |
| R1 | guardrail, high | `export --out` could overwrite source media; output now stays in `exports/` |
| R2 | wrong-result, high | Empty transcript cut the whole clip; no words now gives no cuts, and a cut set keeping nothing is refused |
| R3, S4, S5 | wrong-result, high | Footage volume ignored in export, speed out of sync; volume now applies, speed other than 1 is refused on the render path |
| R4 | guardrail | Clone stayed usable after consent withdrawal |
| R5 | guardrail | Voice, derive and other agent writes skipped the stage gate; import, transcribe and reference stay open as evidence |
| R6 | guardrail | Retry gate evaded by changing the label or base rev |
| R7 | wrong-result | Recut reset volume and speed |
| S1 | guardrail | Agent could swap a consented take's file |
| S2, S3 | guardrail | Retrieval budget never refused, and reset on every revision; now refuses and lasts one turn (cleared by handoff) |

### Open

| Id | Severity | Finding |
| --- | --- | --- |
| R11 | wrong-result, low | Second split of a layer refused |
| S6 | wrong-result, medium | Comment resolve lists touched layers, no before and after |
| R8, R9, R10 | robustness, medium | Loudness pipe cap on long audio; worker has no timeout or stderr; `/media` buffers whole files |
| R12 to R16 | robustness, low | Eval aborts on ffmpeg errors; mix accepts empty clips; fixed temp names; take id race; hub errors swallowed |
| S7 to S9 | missing feature | Single-revision revert; branch merge and compare; time in history lines |
| S10 to S12 | missing feature | Derived components not type-checked or rendered once; root-only instrumentation; profile learns brief picks only |
| S13 to S16 | missing feature | Fast path footage only; no aspect variants (presets of another shape are now refused); first audio track only; wavesurfer, opentype, culori absent |
| S17, S18 | minor | Retake detection adjacent only; send bundle includes Working comments |

## 6. E2E pipeline audit (2026-10-30, docs/E2E_PIPELINE.md section 6)

Baseline at 2d0cc96: `cargo nextest` 376 passed, 1 skipped (with Node on PATH; without it 2 render tests fail on `node could not start`).

| Item | Result | Evidence |
| --- | --- | --- |
| 9:16 brief made a 1920x1080 project | Fixed | `sprint_proof::brief_sets_the_canvas_and_edits_follow`, `verify_fails_when_size_and_brief_disagree` |
| Stage stayed at boot; out-of-order calls accepted | Fixed | `sprint_proof::voice_before_the_brief_is_refused`, `pipeline_proof` |
| Export ran with failing gates | Fixed | `sprint_proof::export_refuses_failing_gates_and_a_reframing_preset` |
| Silent audio passed loudness | Fixed | `sprint_proof::forced_export_records_the_reason_and_silence_fails_loudness` |
| Skill named missing commands; unknown brief fields accepted | Fixed | `docs_agree`, `brief_rejects_unknown_fields_and_get_shows_the_schema` |
| Plugin MCP registration pending | Fixed | `plugin/.mcp.json` present |
| JSON forms, at and dur, @project, recipe ripple and insert, sorted items | Fixed | `json_inputs_share_one_convention_and_recipes_ripple_in_order` |
| Code review and spec check never run on phases 5 to 10 | Fixed | Section 5 above; open findings stay listed there |
| Single test from a 9:16 brief to an exported file | Confirmed missing | Added in step 2 |
| Ducking in the mix; caption words drawn; export queue; `ck repro`; timeline drag, trim, split, loop | Confirmed not built | No code in `ck-media`, `studio/src`, `api.rs` |
| Content map, plan.edl, speech and generated grids, per-segment selection | Confirmed not built | `Project.plan` is an untyped, unused value |
| Professional editor UI | In progress elsewhere | `codex/ui-pro` worktree with uncommitted work; `docs/updates/UI_UPDATE.md` exists only there |
