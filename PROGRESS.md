# Progress

## Handoff (latest)
1. Done: Phase 0 setup. Workspace skeleton (9 crates, no logic), CLAUDE.md, spec split + INDEX.md, 8 skills, 4 subagents, 4 hooks + settings, justfile, CI, fixtures generator.
2. Proof: `just check` green locally (fmt, clippy, 2 tests, 41 hook payload tests); see VERIFICATION.md.
3. Open: fixtures generator not run locally (ffmpeg missing; `sudo apt install ffmpeg`, then `just fixtures`); CI not yet run (nothing pushed).
4. Next: Phase 1, core (project model, patches, revisions, undo, storage, migrations, locks, stage machine, error codes, logging).
5. Start: `/clear`, then the Phase 1 message from docs/spec/13-kickoff.md ("Use the phase-runner skill for Phase 1 ...").

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
