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

## Phase 1: core (2026-10-28, Rust 1.99.1, cargo-nextest 0.9.150)

Command for every test row: `cargo nextest run --workspace` (part of `just check`). Five consecutive full runs: 48 of 48 passed each time (logs in `target/evidence/phase1-run-1..5.txt`, local only).

| Item | Status | Evidence |
| --- | --- | --- |
| Stable error codes with hints | PASS | `error::tests::codes_are_stable` (insta snapshot), `display_matches_serde` |
| Project model (spec 3.2) serializes camelCase, round-trips, schema pinned | PASS | `model::tests::sample_is_valid_and_round_trips`, `item_body_is_tagged_and_camel_case`, `json_schema_is_stable`, proptest `clip_items_round_trip` |
| Schema gate rejects bad projects | PASS | `model::tests::validation_rejects_bad_projects`, `patch::tests::protected_fields_and_bad_patches_are_refused` |
| Timeline math (frames, spans, overlaps) | PASS | `timeline::tests::*` incl. proptests `frames_round_trip`, `back_to_back_items_never_overlap`, `overlap_set_ignores_item_order` (checked against brute force) |
| Revision gate: stale patches refused naming only changed layers | PASS | `patch::tests::stale_patch_lists_only_changed_layers` |
| Engine-owned fields cannot be patched; item revs cannot be forged | PASS | `patch::tests::protected_fields_and_bad_patches_are_refused`, `whole_item_replace_cannot_forge_revs` |
| Undo/redo are revisioned and match linear-undo semantics | PASS | proptest `document::tests::edits_undo_redo_match_reference` (random edit/undo/redo sequences vs reference model; revs +1 each; refused patches change nothing) |
| Lock gate: agent needs locks, human refused while agent holds, expiry | PASS | `document::tests::agent_needs_locks_and_human_is_refused_while_agent_holds`, `agent_cannot_edit_project_or_track_content_without_a_lock`, `locks::tests::*`, proptest `lock_invariants_hold` |
| Atomic storage, crash leftovers, corruption, replay | PASS | `storage::tests::save_load_round_trip_and_replay`, `leftover_temp_files_are_removed_and_ignored`, `corruption_is_reported_with_storage_code`, `revision_file_must_match_its_name_and_never_be_overwritten`, `missing_history_is_detected`, `project_behind_history_is_detected`, `only_our_temp_files_are_removed` |
| Migrations: older projects keep loading | PASS | `ck-core::migrations every_fixture_version_loads` (tests/fixtures/projects/v0-unversioned.json), `migrate::tests::*` |
| Stage machine and stage gate (E_STAGE with next-step hint, brief-auto exception) | PASS | `stage::tests::every_transition_is_allowed_or_refused_with_hint` (exhaustive), `unconfirmed_brief_blocks_planning_unless_auto`, `patch_before_base_is_refused_with_next_step` |
| Retry gate | PASS | `retry::tests::third_identical_failing_call_is_refused`, `success_resets_the_count` |
| JSON logs carry standard fields; secrets redacted | PASS | `log::tests::json_lines_carry_standard_fields_and_redact_secrets`, `ck-cli::cli_smoke logging_is_off_by_default_and_json_when_enabled` |
| CLI output under size cap | PASS | `ck-cli::cli_smoke version_prints_and_stays_under_cap` |
| Independent review | PASS | rust-reviewer and spec-checker subagents run on the Phase 1 diff; all med/low correctness findings fixed with regression tests (see PROGRESS.md Phase 1) |
| Flight recorder folders `.clipkit/trace/`, `.clipkit/cache/` | BLOCKED | Deferred by design to Phases 2+, 5 and 9 (DECISIONS.md, Phase 1) |

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
