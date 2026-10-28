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
| Fixtures generator produces media and ground truth | PASS | 2026-10-28, ffmpeg 8.0.1: `just fixtures` -> "wrote 11 files ... (counter overlay: true)", ffprobe duration checks inside the script pass; frame at 1.5 s reads "frame 45" and "00:00:01:15". Fixed a pipefail bug that had silently disabled the overlay. |
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

## Phase 2: server and studio shell (2026-10-28 to 2026-10-29, done)

Command: `just check` (64 of 64 tests, hooks 41 of 41). ck-server suite run 3 more times in a row: 14 of 14 each before review fixes; 16 of 16 after. Test names below are in `crates/ck-server/src/tests/`.

| Item | Status | Evidence |
| --- | --- | --- |
| New error codes E_AUTH, E_NOT_FOUND, E_REQUEST, E_SERVER pinned; every code maps to an HTTP error status | PASS | `ck-core error::tests::codes_are_stable` (snapshot extended), `api::every_error_code_maps_to_an_error_status` |
| Server binds 127.0.0.1 on a free port; server.json has port + 64-char token, mode 0600, removed on shutdown | PASS | `live::serves_ws_events_and_cleans_up` |
| Missing/wrong token refused (API and /ws handshake); query token only on /ws | PASS | `api::missing_or_wrong_token_is_refused`, `api::token_in_query_only_counts_for_ws`, `live::serves_ws_events_and_cleans_up` |
| Foreign Host (DNS rebinding) and foreign Origin refused, even with the token, on every route | PASS | `api::foreign_host_is_refused_even_with_token`, `api::origin_must_be_this_server`, `api::unknown_route_still_checks_host` |
| Status, project summary, layer and full views; unknown layer E_NOT_FOUND | PASS | `api::status_with_token`, `api::project_views` |
| Agent patch needs its lock; result stored on disk with locks | PASS | `api::agent_patch_needs_its_lock_and_is_stored` |
| Human edits refused (E_LOCK_HELD, nothing applied) while the agent holds the layer; lock cannot be stolen; allowed after unlock | PASS | `api::human_edit_is_refused_while_agent_holds_the_layer` |
| Heartbeat keeps a lock; expiry after the timeout frees the layer | PASS | `api::expired_agent_lock_frees_the_layer` (fake clock) |
| Stale, protected-field, malformed and oversized patches refused | PASS | `api::stale_patch_is_a_conflict`, `api::protected_field_patch_is_invalid`, `api::malformed_and_oversized_bodies_are_bad_requests` |
| Failed save rolls memory back to disk | PASS | `api::failed_save_leaves_memory_at_the_stored_state` |
| WS pushes snapshot on connect, then revision + snapshot after a patch, in under 300 ms (server side) | PASS | `live::serves_ws_events_and_cleans_up` (asserts < 300 ms from patch to both events) |
| Step 3: shared ops over the loopback client: status, summary, layer, patch, lock, unlock; errors (E_LOCK_HELD, E_PATCH_STALE, E_NOT_FOUND, E_AUTH) survive the HTTP trip; missing or dead server is E_SERVER | PASS | `ops::reads_status_summary_and_layer`, `ops::patch_lock_unlock_round_trip`, `ops::wrong_token_is_e_auth`, `ops::missing_or_dead_server_is_e_server` |
| Step 3: output size caps (status 4 KB, summary 16 KB, layer 8 KB, patch 4 KB, lock 4 KB): reads over the cap refused with E_OUTPUT_CAP; write replies trimmed with an `omitted` count; replies over 64 KB refused; non-JSON error bodies map to E_SERVER | PASS | `ops::answers_over_the_cap_are_refused`, `ops::write_replies_over_the_cap_are_trimmed_not_refused`, `ops::non_json_errors_and_oversize_replies_are_mapped`; snapshot `codes_are_stable` extended with E_OUTPUT_CAP |
| Step 3: CLI `ck studio` (creates the project, prints url + port without the token, refuses a second studio), `ck status`, `ck project get [--summary\|--layer]`, `ck project patch <json\|->`, `ck lock`, `ck unlock`; one line of JSON under each cap; human patch refused while the agent holds the lock; errors as JSON on stderr, exit 1 | PASS | `crates/ck-cli/tests/cli_commands.rs`: `studio_creates_a_project_and_serves_every_command`, `patch_reads_stdin_and_rejects_bad_json`, `commands_without_a_studio_fail_with_e_server`, `bad_arguments_are_usage_errors`; `cli_smoke::no_args_exit_with_short_help` (help cap 2 KB; the Phase 1 256-byte cap still holds for `--version` and usage errors). `just check`: 76 of 76 tests, hooks 41 of 41 |
| Step 3 review | PASS | rust-reviewer on the step 3 diff; fixed: write replies trimmed instead of refused after commit, redirects off (token never follows), response limit 64 KB, server.json read cause kept, any non-E_SERVER answer counts as a running studio, only BrokenPipe ignored on stdout. Open: E_SERVER hint names `ck studio --ensure`, which arrives in Phase 3 |
| Step 4: MCP tool table pinned (names, CLI twins, caps, input schemas); names follow the CLI path; each tool's cap equals its CLI cap; rmcp list matches the table | PASS | `crates/ck-mcp/src/tests.rs`: `tool_table_snapshot` (insta), `every_tool_is_named_after_its_cli_twin`, `every_tool_has_an_object_schema_and_its_cli_cap`, `rmcp_tool_list_matches_the_table` |
| Step 4: `ck mcp` over stdio against a real `ck studio`: initialize, tools/list with the same schemas; every tool's output is byte-identical to the CLI's stdout (lock timestamps aside), errors (E_PATCH_STALE, E_NOT_FOUND, E_LOCK_HELD, E_SERVER) byte-identical to the CLI's stderr; human edit refused while the agent holds the layer; bad arguments E_REQUEST; unknown tool a protocol error | PASS | `crates/ck-cli/tests/mcp_parity.rs`: `tools_list_names_every_cli_twin_with_its_schema`, `every_tool_answers_like_its_cli_twin`, `bad_arguments_and_a_missing_studio_are_tool_errors` |
| Step 4: output caps through MCP: lock reply over 4 KB trimmed with the same `omitted` as the CLI; status over its cap refused with the same E_OUTPUT_CAP as the CLI | PASS | `mcp_parity::output_caps_hold_through_mcp`. `just check`: 84 of 84 tests, hooks 41 of 41; `cargo nextest run -p ck-cli -p ck-mcp --stress-count 3`: 3 of 3 iterations passed |
| Step 4 review | PASS | rust-reviewer on the step 4 diff: no unwrap in library code, blocking calls in spawn_blocking, nothing else on stdout. Fixed: added the over-cap MCP test and exact per-tool cap assertions |
| Step 5: TS bindings generated from the Rust types (ts-rs 12.0.1) into `studio/src/generated/types.ts`: model, API requests/replies, CkError/ErrorCode, Event, JsonValue; every referenced name declared; integers as `number`, never `bigint` | PASS | `crates/ck-server/src/ts_types.rs`: `every_referenced_type_is_declared`, `no_bigint_in_bindings` |
| Step 5: drift fails `just check` (and CI, which runs it) | PASS | `ts_types::generated_types_match_committed_file`; negative check on 2026-10-29: editing `Mode` in the committed file made it fail with "studio/src/generated/types.ts is stale or missing; run `just types`", and `just types` restored it. `just check`: 87 of 87 tests, hooks 41 of 41 |
| Step 6a: embedded studio served next to the API: index without a token, client routes fall back to index, bundle files typed and cached immutable, missing files and `..` traversal E_NOT_FOUND, unknown `/api/*` stays JSON E_NOT_FOUND, non-GET 405, API still needs the token, Host still checked, missing build explains `just studio-build` | PASS | `crates/ck-server/src/tests/studio.rs` (9 tests). `just check`: 96 of 96 tests, hooks 41 of 41, studio prettier check and `tsc --noEmit` clean |
| Step 6a: real `ck studio` (debug, reading `studio/dist` after `just studio-build`) serves the build; the WS sends a snapshot first | PASS | 2026-10-29, `node target/live-check/check.mjs target/live-studio` printed `{"index":[200,"text/html; charset=utf-8",true],"bundle":["/assets/index-D4aAr7Iv.js",200,"text/javascript; charset=utf-8",565637],"spaRoute":200,"apiNoToken":403,"firstEvent":"snapshot"}` |
| Step 6a review | PASS | rust-reviewer on the step 6a diff: traversal, guard order, no unwrap confirmed. Fixed: API/WS check on the slash-trimmed, lowercased first segment (`//api/x`, `/ws/`, `/API/x` now JSON E_NOT_FOUND), 405 carries E_REQUEST JSON and `Allow: GET, HEAD`, route-miss hint no longer points at layer ids. Tests extended in `studio::unknown_api_paths_stay_json_errors`, `studio::only_get_and_head_reach_static_files`; `just check` 96 of 96 |
| Step 6a: studio renders the live project in a browser | PASS | Step 7, `studio/e2e/phase2.spec.ts` "a CLI patch appears...": the layer shows in the timeline and in the Remotion Player preview (`.studio-preview [data-ck-layer]`) in headless Chromium; 2026-10-29 `just e2e` 2 of 2 |
| Step 6b: studio logic: snapshot/locks/revision reducer, capped activity feed, lock liveness matching `Lock::is_live`, only live agent locks disable edits, "agent stopped" when every agent lock lapsed, human field patch (one replace, baseRev + layerRevs, author human), unknown layers and fields refused | PASS | `studio/src/state.test.ts` (10 tests, `node --test`, run by `just studio-check`). 2026-10-29 `just check`: 96 of 96 nextest, hooks 41 of 41, prettier, `tsc --noEmit` and 10 of 10 studio tests clean |
| Step 6b: the studio's edit path against a real `ck studio`: a human edit built by `fieldPatch` is refused while the agent holds the layer and lands after unlock; revision events carry author and label for the feed | PASS | 2026-10-29, `node target/live-check/panels.mjs target/live-studio` printed `{"agentAdd":200,"humanWhileLocked":[423,"E_LOCK_HELD"],"humanAfterUnlock":200,"nameNow":"Mine","events":["snapshot","locks","revision:agent:Text pass: adding the hook title","snapshot","locks","revision:human:Edit name of Hook title","snapshot"]}`; `check.mjs` served the new bundle (`/assets/index-BpSVptQj.js`, 571987 bytes) |
| Step 6b: panels (banner, timeline, inspector, activity feed, ghost cursor, flash, disabled controls) render and behave in a browser | PASS | Step 7 Playwright, asserted via `data-ck-*`: timeline item with text, `data-ck-flash=true` after the patch, ghost cursor on the agent-locked layer, feed entry with author agent then human, inspector `data-ck-readonly` true then false, inputs disabled then enabled, a real fill + Enter edit landing. Banner checked in the idle state only: `/agent` is read-only to patches, so working/stopped need Phase 3 commands (stopped is covered by the unit tests) |
| Step 7: Playwright e2e (@playwright/test 1.64.1, Chromium) against a real `ck studio` per test on a fresh temp project, driven by the real `ck` CLI | PASS | `studio/e2e/fixture.ts`, `studio/e2e/phase2.spec.ts` (2 tests). 2026-10-29 `just e2e`: 2 passed (1.6s); `pnpm --dir studio e2e --repeat-each=10 --workers=4`: 20 of 20 passed, no flakes |
| Phase proof: CLI patch visible in the browser in < 300 ms | PASS | `phase2.spec.ts` "a CLI patch appears in the timeline and the preview in under 300 ms": a MutationObserver in the page timestamps the layer's arrival, measured from just before the `ck project patch` process is spawned (so it includes CLI startup, HTTP, WS push and React render). 2026-10-29: 11 to 12 ms single worker, 12 to 26 ms with 4 parallel workers; budget 300 ms |
| Phase proof: human edits refused while locked | PASS | `phase2.spec.ts` "human edits are refused...": after `ck lock t1 title` the DOM shows `data-ck-readonly=true` with every input disabled, and the studio's own `fieldPatch` request sent from the page (studio Origin, session token) gets 423 `E_LOCK_HELD`; `ck project get --layer title` confirms the name unchanged; after `ck unlock` a UI edit lands (name "Mine", feed author human) |
| Crate release dates at least two weeks old (direct dependencies) | PASS | 2026-10-29: crates.io API (`/api/v1/crates/<name>/<version>`, node fetch) for all 204 registry crates in the lock graph. tokio 1.54.1, ureq 3.5.0 and rmcp 3.8.0 were too new and are pinned back to =1.53.2, =3.4.2 and =3.5.1 (released 2026-10-03, 2026-09-13, 2026-10-05); `just check` green on the pins. Dates in DECISIONS.md, step 8 |
| Crate release dates at least two weeks old (transitive) | BLOCKED | cc 1.10.0, find-msvc-tools 0.1.15, libc 0.2.191, rustix 1.1.6, smallvec 1.16.4, uuid 1.29.0 and ureq-proto 0.6.5 were released 2026-10-16 to 2026-10-27. Pinning them needs `cargo update --precise`, which headless runs cannot use. Open question |
| Independent review | PASS | rust-reviewer on the steps 1-2 diff; fixed: 0600 before rename, query token limited to /ws, no session mutex on async workers, degraded mode on failed reload, snapshot only with subscribers, WS message cap. Recorded as open: role trust boundary, second-server check, 5xx path redaction (DECISIONS.md, Phase 2) |
| Step 8: phase review | PASS | rust-reviewer on the Phase 2 diff (4f9a064..HEAD): no high-severity findings. Fixed: heartbeat had no CLI/MCP twin; no CSP or anti-framing headers; the MCP schema fallback was `null`. spec-checker on Phase 2: fixed the idle shutdown (spec 5.1), the heartbeat command and the missing open-time check (spec 1.2). The rest are recorded as open or as deferred to later phases (DECISIONS.md step 8, PROGRESS.md) |
| Step 8: `ck heartbeat` / MCP `heartbeat` renews the holder's locks past their TTL; a lapsed lock is not revived; CLI and MCP outputs match; same 4 KB cap | PASS | `ck-server tests::ops::heartbeat_keeps_locks_past_their_ttl` (fake clock), `mcp_parity::every_tool_answers_like_its_cli_twin`, `bad_arguments_and_a_missing_studio_are_tool_errors`, `ck-mcp tests::every_tool_has_an_object_schema_and_its_cli_cap`, `tool_table_snapshot`, `cli_commands::commands_without_a_studio_fail_with_e_server` |
| Step 8: idle shutdown (spec 5.1): the count restarts on each authorized call; an open WebSocket holds it off; 0 never stops; refused and static requests do not count; a real `ck studio --idle 1` exits 0 by itself at least 1 s after the last call and removes server.json | PASS | `ck-server tests::idle::*` (5 tests, paused tokio time), `cli_commands::studio_stops_by_itself_after_the_idle_time` |
| Step 8: CSP and `X-Frame-Options: DENY` on studio files; no CSP violation while the Player preview, panels and edits run | PASS | `studio::index_is_served_without_a_token` header assertions (CSP constant, no `unsafe-eval`); `phase2.spec.ts` records `securitypolicyviolation` events and requires none in both tests. The first run caught `media-src data:` from the Player, now allowed. 2026-10-29 `just e2e` 2 of 2; `--repeat-each=10 --workers=4` 20 of 20 |
| Step 8: studio opens in under 3 s (spec 1.2) | PASS | `phase2.spec.ts`: `page.goto` to the connected timeline took 253 ms (2026-10-29, debug ck, warm machine); budget 3000 ms |
| Step 8: e2e in CI | BLOCKED | `e2e` job added to `.github/workflows/ci.yml` (Playwright Chromium with deps, `just e2e`); not pushed, so it has never run on GitHub |
| Phase 2 final run | PASS | 2026-10-29 `just check`: 103 of 103 nextest, hooks 41 of 41, studio prettier, `tsc --noEmit` and 10 of 10 unit tests; `just e2e` 2 of 2 |

## Phase 3: plugin and auto-launch (2026-10-29, in progress)

Tests in `crates/ck-cli/tests/cli_ensure.rs` and `mcp_parity.rs` unless named otherwise.

| Item | Status | Evidence |
| --- | --- | --- |
| Step 1: `ck studio --ensure` starts a detached studio, then is idempotent | PASS | `ensure_starts_once_then_reuses_the_running_studio`: started true then false on the same port, `ck status` answers, server stops on SIGTERM and removes server.json |
| Step 1: ensure returns within about 200 ms (spec 5.1) | PASS | Same test with `--no-capture`, three runs on 2026-10-29 (debug ck): cold 16.0, 15.5, 15.6 ms; warm 4.9, 4.9, 5.1 ms. Test bounds are 3 s and 1 s for loaded CI |
| Step 1: concurrent ensures start one server | PASS | `concurrent_ensures_start_exactly_one_studio` (4 parallel calls, one `started: true`, one port) |
| Step 1: `--open` opens the tab once per server, token only in the opener argument | PASS | `open_opens_the_tab_once_per_server` (fake `$BROWSER` records `url#token=`; second call opens nothing; a restarted server opens once with a new token); ensure output has no token |
| Step 1: `--quiet` prints nothing; `--open`/`--quiet` need `--ensure` | PASS | `ensure_starts_once_then_reuses_the_running_studio`, `open_and_quiet_need_ensure` |
| Step 1: stale server.json and start failures | PASS | `ensure_replaces_a_stale_server_json`; `a_studio_that_cannot_start_reports_its_own_error` (taken port: the child's E_SERVER "cannot bind" is relayed, no server.json left) |
| Step 1: MCP twin `studio`, schema, cap, parity | PASS | `studio_tool_ensures_like_ck_studio_ensure` (MCP starts the server; then MCP and CLI answer byte for byte the same), `bad_arguments_and_a_missing_studio_are_tool_errors`, ck-mcp `tool_table_snapshot` and `every_tool_has_an_object_schema_and_its_cli_cap` |
| Step 1 run | PASS | 2026-10-29 `just check`: 110 of 110 nextest, hooks 41 of 41, studio prettier, typecheck and unit tests |

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
