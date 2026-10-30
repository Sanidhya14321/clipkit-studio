# Update 5 audit (U5-0, 2026-10-30)

Maps every requirement in `docs/updates/UPDATE_5.md` to exists, partial or missing, at commit
`af6c2d5`. Method: read PROGRESS.md, spec index and the skills, then searched `crates/`,
`studio/src/`, `render-worker/`, `plugin/`, `adapters/` and `assets/` for each feature. No code
changed in this step.

## Findings that change the plan

1. **The prompt assumes Phase 2 is current; every phase through 10 is done.** PROGRESS.md says
   `STATUS: PHASES DONE`. Rule 0.5 forbids reopening them, so each "lands in Phase N" in section
   13 of the update becomes a follow-up task under a new Update 5 block in PROGRESS.md.
2. **U5-8 is "end of Phase 4", which has passed.** It should run after U5-4 and U5-5 exist, since
   S16 needs them. Its stop condition now guards U5-6 and U5-7 instead of Phases 5 to 9.
3. **Step 3 tells the agent to edit `docs/spec/`; CLAUDE.md and a hook forbid that.** The new
   `15-learnings.md` and edits to files 02, 04, 05, 08, 10 and INDEX.md need an owner decision at
   the U5-2 gate (lift the rule for that step, or carry the changes in `docs/updates/`).
4. **The `.autobuild` loop is not running** (no `.autobuild/`, no process), and the tree is clean.
5. **Existing thresholds differ from section 4.2.** `assets/taste/pacing.md` has five levels
   (calm to rapid) and the brief uses them; the update has four and names a file
   `taste/pacing-and-structure.md`. U5-4 must map one to the other, not add a second scale.
6. **A dev machine without a toolchain.** This Mac had no Rust, ffmpeg, pnpm or Homebrew at audit
   time; see VERIFICATION.md for how the U5-1 runs were done.

## Requirements

Status: E exists, P partial, M missing. "Phase" is the update phase that delivers it.

| Section | Requirement | Status | Evidence or gap | Phase |
| --- | --- | --- | --- | --- |
| 2.1 | EngineAdapter (TS) and Rust job-control trait | M | Worker calls Remotion directly (`render-worker/src/render.mjs`) | U5-1 |
| 2.1 | Engine-neutral project; clips declare runtime | P | Project JSON has no Remotion types; no `runtime` field | U5-1 |
| 2.1 | Compiler adds data-ck attributes for both engines | P | `studio/src/preview/HoverSelect.tsx`, Remotion only | U5-1 |
| 2.2 | Three spike fixtures and measurements | M | None | U5-1 |
| 2.4 | Spec, worker and clip migration after approval | M | Blocked on approval | After U5-1 |
| 3.1 | `ck music analyze`, cached analysis | M | No matches | U5-5 |
| 3.1 | project.beatGrid, nudge UI, atBeat and durBeats | M | No matches | U5-5 |
| 3.1 | Snap to beats in the studio | P | Timeline snapping exists; no beat targets | U5-5 |
| 3.2 | brief.rhythm, plan.beatSheet, storyboard strip | P | Storyboard checkpoint exists (Phase 10); no beat sheet or rhythm | U5-5 |
| 3.3 | Beat-aware `ck sound plan`, sound kit | P | `ck sound plan` exists; no beat mode; no vetted CC0 sound set, so placement is an open item | U5-5 |
| 3.4 | music, grid and beatsheet commands | M | Not in `ck-cli/src/args.rs` | U5-5 |
| 4.1 | Still ratio, longest hold, cut density, motion curve, beat alignment | M | No matches; `ck verify` has only `--critic` | U5-4 |
| 4.2 | Thresholds in taste | P | `assets/taste/pacing.md` holds shot lengths only (finding 5) | U5-4 |
| 4.3 | `verify --energy`, blocks handoff, studio curve | M | | U5-4 |
| 4.4 | Playback check and self-fix | P | Export already writes yuv420p and faststart; nothing checks them or runs a decode test | U5-4 |
| 4.5 | `ck eval energy`, `ck export --check-playback` | M | | U5-4 |
| 5.1 | Drafts, review bar, partial accept, stale rebase | P | Branches, undo and revisions exist to build on; no draft concept | U5-3 |
| 5.1 | Review-first settings, autopilot auto-accept | M | | U5-3 |
| 5.2 | Agent locks scoped to the draft | P | Per-layer and global locks exist, on main only | U5-3 |
| 5.2 | Stop leaves the draft for review | P | Stop with cooperative cancel exists (`af6c2d5`) | U5-3 |
| 5.3 | Interrupt and redirect queue | M | | U5-3 |
| 5.4 | draft and review commands; agent cannot accept | M | Open question: "any token holder can act as agent or human"; the guardrail needs separate studio credentials first | U5-3 |
| 6 | Style folder, `ck style` group, Brief Studio picker | M | Local taste profile (`ck profile`) is related, not reused | U5-7 |
| 7.1 | taste/moods.md, 12 moods, dangling check | M | Only 4 looks exist, so the 12 moods must share them or U5-4 adds looks | U5-4 |
| 7.2 | Mood field, match, apply, blend | M | | U5-4 |
| 7.3 | mood commands, brief.mood | M | | U5-4 |
| 8.1 | 13 kinetic blocks | P | Recipe `kinetic-list` and third-party `kinetic-word-stack` exist; none of the 13 as specified | U5-6 |
| 8.2 | Caption presets and visual picker | P | Caption item has a style id; `assets/taste/captions.md`; no presets | U5-4 |
| 8.3 | `recipe search --family` | M | | U5-4, U5-6 |
| 9 | Face and person tracks, keep-out, auto-reframe, overlay | M | "face" matches are typefaces and Rust trait wording. `ort` is an open question from Phase 5 | U5-7 |
| 10 | Router skill and 8 workflow skills | P | Two skills exist: clipkit-video (27 lines) and clipkit-review. Already small, so the token comparison in 10.2 is not a given | U5-4 |
| 10 | Codex and Cursor adapters split | M | | U5-4 |
| 11 | Early benchmark, S16 | P | Harness exists (`bench/`, `ck bench report`); no real agent runs yet | U5-8 |
| 12 | New error codes | M | None of the eight exist | With each feature |
| 12 | Migrations, TS types, CLI and MCP parity | E | The machinery exists; each addition must use it | With each feature |
| 14 | Checks L1 to L25 | M | L1 and L2 are addressed by U5-1 | Each phase |
