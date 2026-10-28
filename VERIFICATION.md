# Verification

Every item has a status (PASS, FAIL, BLOCKED) and evidence. No PASS without evidence (spec 10.5).

## Phase 0: setup (2026-10-28, local machine, Rust 1.99.1, just 1.59.0, cargo-nextest 0.9.150)

| Item | Status | Evidence |
| --- | --- | --- |
| Workspace builds; fmt and clippy clean with `-D warnings` | PASS | `just check` (cargo fmt --check, cargo clippy --workspace --all-targets) |
| ck binary skeleton prints version under the output cap | PASS | `ck-cli` tests `version_prints_and_stays_under_cap`, `unknown_args_exit_with_usage` via `just check` (2 passed) |
| Spec split: 14 section files + INDEX.md | PASS | `python3 -I scripts/split-spec.py` output lists docs/spec/00 to 13; INDEX.md hand-written |
| guard-edits blocks docs/spec/, MASTER_PROMPT.md, .env files, assets/quarantine/; allows others | PASS | `tests/hooks/run.sh` (part of `just check`): 41 passed, 0 failed |
| guard-bash blocks deletes outside the repo and secret dumps; allows normal commands and heredocs | PASS | `tests/hooks/run.sh` |
| format-file is silent and never blocks | PASS | `tests/hooks/run.sh` |
| Stop hook never loops (`stop_hook_active: true` exits 0 silently) | PASS | `tests/hooks/run.sh` |
| Stop hook blocks once with a short reason when check-fast fails | PASS | Manual run: appended mis-formatted code to crates/ck-core/src/lib.rs, piped `{"stop_hook_active":false}` to stop-check.sh -> `{"decision":"block","reason":"check-fast FAILED at cargo fmt --check ..."}`; file restored |
| Every hook finishes in under 2 s | PASS | `tests/hooks/run.sh` timing assertion |
| settings.json is valid JSON | PASS | `jq -e . .claude/settings.json` |
| Every skill and agent has frontmatter with a description | PASS | frontmatter check loop over .claude/skills/*/SKILL.md and .claude/agents/*.md |
| CI workflow parses | PASS | `python3 -c "yaml.safe_load(...)"` on .github/workflows/ci.yml |
| CI runs green on GitHub | BLOCKED | Not pushed yet; runs on the first push or PR |
| Fixtures generator produces media and ground truth | BLOCKED | ffmpeg not installed locally (`just fixtures` exits 1 with "ffmpeg not found"); CI `fixtures` job covers it once pushed |
| Hooks fire inside a live Claude Code session | PASS | During Phase 0 the live guard-bash hook blocked a heredoc writing `.gitignore` (a false positive, fixed and covered by a regression case in tests/hooks/run.sh) |

## Asset supply expansion checks (spec 14.7, added 2026-10-28)

| Item | Status | Evidence |
| --- | --- | --- |
| Local transcription works with no network | BLOCKED | Not implemented; Phase 5 |
| VAD candidates match fixture ground truth within tolerance | BLOCKED | Not implemented; Phase 5 (fixtures/generated/ground-truth.json tones.silences) |
| Scene detection finds the fixture scene changes | BLOCKED | Not implemented; Phase 5 (ground-truth.json scenes.changesAtSec) |
| Loudness gates use the ffmpeg meter | BLOCKED | Not implemented; Phase 5 and 9 |
| A restricted fixture (LYGIA or BBC style) is blocked in commercial-safe mode | BLOCKED | Not implemented; Phase 8 |
| An unresolved or unverified source never appears in search | BLOCKED | Not implemented; Phase 8. Today all 63 sources.yaml entries are unverified and disabled: `jq '[.[]|select(.enabled or .status!="unverified")]|length' sources.yaml` -> 0 |
| No Python dependency added to the install | BLOCKED | Install does not exist yet (Phase 3). Today: no Python package manifest in the repo; Python appears only as the dev-time `scripts/split-spec.py` |
