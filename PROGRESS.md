# Progress

STATUS: RUNNING
PHASE: 4

## Handoff (latest)
1. Done: Phase 4 step 7: 8 native core clips in `studio/src/clips/` (`clips.tsx`, pure `motion.ts`), rendered by `ProjectComposition` with look tokens resolved; instrumented with `data-ck-layer`/`data-ck-clip`/`data-ck-part`.
2. Proof: `just check` exit 0; `pnpm --dir studio e2e` 6 of 6 incl. `e2e/phase4-clips.spec.ts`; `motion.test.ts` 7 tests. Evidence in VERIFICATION.md, step 7.
3. Open: the project look is not stored (preview uses the brief look, else clean-product); media sources draw as named panels until proxies; list/quote/stat text not solver-placed; text width is a 0.55 em estimate.
4. Next: step 8: GENERATE quality gates of spec 4.3 in `ck verify` (schema, gaps, overlaps, duration, safe area, size, contrast, fonts, motion limits, reuse, provenance, no placeholders).
5. Start: `just check`; read spec 04 section 4.3 and spec 06 for `ck verify`; DECISIONS.md steps 6b and 7.

## Open questions
- Phase 4 step 7: no stored project look; a recipe applied with a request `look` that differs from the brief previews in the brief's look. Store the look with step 9 re-theme (DECISIONS.md, step 7).
- Phase 4 step 6b: text-layer role defaults to `title` (recipes carry no `role`), and recipe apply does not re-solve layers of other scenes on screen at the same time; revisit with step 7 rendering (DECISIONS.md, step 6b).
- Phase 4 step 5b: recipe apply falls back to the `clean-product` look when neither the request nor the brief names one; revisit if the plan stage should pick it.
- Phase 4 step 3: spec 6.2 lists `brief set`; not built (the studio confirms through a human patch). Add it if the agent needs to set user-requested values directly. Wire `Op::ProposeBrief` stage gating once stages are driven.
- Phase 3 step 7: check Cursor's `~/.cursor/mcp.json` and `.cursor/rules/*.mdc` format, and whether a user-level rules file exists, against cursor.com/docs (DECISIONS.md, step 7).
- Phase 3 step 6: run the real proof in an interactive session (Claude Code and Codex) and flip the BLOCKED rows in VERIFICATION.md.
- Phase 3 step 5: the Codex MCP config (`$CODEX_HOME/config.toml`, `[mcp_servers.<name>]` command/args, MCP server working folder) and AGENTS.md location are from memory: verify against developers.openai.com/codex/mcp and /guides/agents-md. Also confirm Claude Code picks up a plugin enabled only through `extraKnownMarketplaces` + `enabledPlugins` (else ck init must run `claude plugin install`).
- Phase 3 step 4: `plugin/.mcp.json` must be written in an interactive session; headless runs refuse it as a sensitive file. The content is in DECISIONS.md, and the test is ignored until then.
- Phase 5: `ort` has no stable 2.0 release and downloads ONNX Runtime at build time; confirm static linking into `ck` or evaluate a pure-Rust runtime (DECISIONS.md, asset expansion).
- Phase 10: runtime for the optional Demucs sidecar, given the default install must not add Python.
- Pixabay is wave 1 but placed in Phase 8 because ingesting it needs the vetting pipeline.
- `@remotion/whisper-web` and `@remotion/effects` declare UNLICENSED and `@remotion/animation-utils` declares no license; vetting must read the real terms.
- Lock ids for non-layer content: Phase 1 uses `@project` for project-level fields and track ids for track fields and item order (DECISIONS.md, Phase 1). Revisit if the studio needs finer locks.
- Phase 2 step 8: direct crate dates are verified on the crates.io API, and tokio, ureq and rmcp are pinned back. The transitive crates cc 1.10.0, find-msvc-tools 0.1.15, libc 0.2.191, rustix 1.1.6, smallvec 1.16.4, uuid 1.29.0 and ureq-proto 0.6.5 are still under two weeks old. In an interactive session run `cargo update -p <name> --precise <older>`, or wait until they age past two weeks.
- Phase 2/3: any token holder can act as agent or human (author/holder are self-declared). Decide whether the studio and the agent get separate credentials.
- Phase 2 step 4: MCP `lock`/`unlock` require `holder` while the CLI defaults `--holder agent`; revisit if strict input parity is wanted (DECISIONS.md, step 4).
- Phase 2 studio (step 6): `pnpm add` is denied in headless runs; dependencies are written into package.json by hand and installed with `pnpm install`. Step 6a found the npm registry reachable: versions and dates checked with `pnpm view` (DECISIONS.md, step 6a).
- Phase 2 step 6a: the CI action majors (pnpm/action-setup@v4, actions/setup-node@v5) are unverified; confirm on the first CI run. CSP was added in step 8.
- Phase 2 step 6b: spec 5.2's Stop button needs cooperative cancel through the event bus (no route yet). Spec 5.3's global lock for structural changes has no lock id in ck-core. Both are deferred (DECISIONS.md, step 6b); confirm Phase 3 and Phase 6 pick them up.
- Phase 2 step 8: the CI `e2e` job (Playwright Chromium, no cache) is unverified until the first push. Reviewer leftovers: status/summary reads over the cap are refused instead of paged; a no-op heartbeat or unlock still fsyncs and publishes; no cap on concurrent WebSockets; the second-studio check has a check-then-bind gap.

## Roadmap additions: asset supply expansion (2026-10-28)

Spec 14. Implementation stays in the phases below; no finished phase is reopened (only Phase 0 is finished, and it has no affected items).
- Phase 5 (edit slice): local whisper transcription (whisper-rs), Silero VAD via ONNX, ffmpeg scene detection as a second cut-candidate signal, ffmpeg ebur128/loudnorm loudness gates and export presets.
- Phase 8 (assets at scale): wave 2 ingestion (RenderComp Free, cliphou.se, RemotionUI, Remotion Bits, Kenney, Tabler, Simple Icons, IWLTBAP free LUTs) and Pixabay; frame-clock adapter with determinism tests BEFORE any wrappable card (GSAP, Motion, Origin UI, Launch UI); LYGIA and BBC-style restricted fixtures for license mode; site-license snapshots, custom-license and trademark notes.
- Phase 9 (polish and proof): remotion-qa, audio-bed-check and tesseract.js in the evaluator and CI after license checks; ffmpeg loudness in the evaluator; wavesurfer.js, culori, opentype.js in the studio; revisit librosa replacement.
- Phase 10 (stretch): Demucs opt-in sidecar.

## Phase 4: generate slice (2026-10-29, in progress)

Spec: 00, 04 (4.3), 07, 08 (8.1, 8.5), 09. Proof: prompt to brief to a 15 second generated video to export.

Steps:
- [x] 1. Starter taste library (12 files with concrete numbers) and 4 looks (design tokens plus motion personality) in `assets/`, authored originally; `ck-assets` loaders with validation (front matter, size caps, contrast, at most two font families).
- [x] 2. `ck taste search|read` and `ck look list|get` with MCP twins, output caps, parity tests; `assets/INDEX.md` (under 60 lines).
- [x] 3. Brief Studio essentials: `ck brief propose|wait|diff|get` (inferred values only, field sources, at most three dynamic questions) and its MCP twins.
- [x] 4. Studio brief panel (essentials form, pacing slider with numbers, look cards, confirm) with Playwright test.
- [x] 5. Recipes (10, typed slots) and `ck recipe search|apply`: expander builds named layers, applies tokens, records provenance and briefDeps.
  - [x] 5a. Recipe model, validation, expander and the 10 recipes.
  - [x] 5b. `ck recipe search|card|apply` with MCP twins.
- [x] 6. Deterministic layout solver (safe areas, collisions, text fitting, grid) and `ck layout check`.
  - [x] 6a. Pure solver and checker in `ck-assets::layout` with tests.
  - [x] 6b. `ck layout check` with MCP twin; recipe apply runs the solver.
- [x] 7. 8 native core Remotion clips, instrumented, used by the recipes; studio Player renders them.
- [ ] 8. Quality gates of spec 4.3 that apply to GENERATE in `ck verify` (schema, gaps, overlaps, duration, safe area, size, contrast, fonts, motion limits, reuse, provenance, no placeholders).
- [ ] 9. Look re-theme through tokens with zero agent tokens, sound placement (`ck sound plan`), handoff and `ck wait`.
- [ ] 10. Proof run: prompt to brief to a 15 second generated video to export; reviewer, spec-checker, evidence.

Run plan (autonomous run 9): step 7.
- `studio/src/clips/`: pure motion helpers (`motion.ts`: look easings, enter progress, stagger, counter value, box style) with node tests.
- 8 clip components (CORE_CLIPS) reading recipe props (tokens resolved, `box`/`fontSize`/`lines`), each root tagged `data-ck-layer` + `data-ck-clip`; parts tagged `data-ck-part`.
- `ProjectComposition` renders clip items through the registry; unknown clips keep the labelled box.
- Playwright: `ck recipe apply` then the Player shows the clip with its text; registry test that every CORE_CLIPS id has a component.

Run plan (autonomous run 8): step 6b.
- `ck-assets::layout::blocks_from_items`: text layers are clip items with a string `text` prop; role from a `role` prop (else `caption` when `size` is `small`, else `title`).
- Server `layout_check` (read-only): platform from the brief, `check` on the placements written in props (`box`, `fontSize`, `lines`), or solved when absent; reply capped.
- `recipe apply` runs `solve` over the scene's text layers and writes `box`, `fontSize`, `lines` into their props.
- `ck layout check` + MCP `layout_check`; tests: ck-assets, server, tool table, CLI/MCP parity.

Run plan (autonomous run 7): step 6a (solver; CLI and wiring are 6b).
- `ck-assets::layout`: platforms and safe areas, grid, type sizes from the taste files, frame-fraction geometry.
- `solve`: fit (wrap, shrink one step, cut, label floor), full-width grid boxes, stack blocks sharing screen time with 0.03 gap, centered.
- `check`: safe area, overlap, gap, at most 3 blocks, label floor, grid, cut text; unit tests per rule.

Run plan (autonomous run 6): step 5b.
- `library::recipe_search|recipe_card` (offline, caps) and `Item.provenance` in ck-core.
- Server op `recipe_apply`: expand with the request/brief/default look, add layers by track kind, provenance, `assetsUsed`, temporary agent locks; route, `Ops`, cap.
- `ck recipe search|card|apply` and MCP twins; tests: library, server, tool table, CLI/MCP parity.

Run plan (autonomous run 5): step 5a (recipe model, expander, 10 recipes; CLI is 5b).
- Finish the uncommitted `ck-assets/src/recipe.rs` from run 4: wire it in, make it compile under clippy.
- Author 10 original recipes in `assets/recipes/` (typed slots, named layers, token refs, briefDeps); built-in copy in `library`.
- Tests: all 10 load, validation refusals, expansion (layer ids, times, tokens kept, provenance, optional slot drop, bad slots).
- Next run (5b): `ck recipe search|apply` with MCP twins, caps and parity tests.

Run plan (autonomous run 4): step 4.
- `GET /api/looks` (look tokens for the cards); look types exported to TS.
- `studio/src/brief.ts`: essentials, pacing levels from the taste file, confirm patch (changed fields as `user`, answers, status confirmed); unit tests.
- `BriefPanel`: each field with its source, pacing slider with numbers, look cards, questions, confirm; read-only under the `@project` lock.
- Playwright `e2e/phase4.spec.ts`: propose via CLI, adjust, confirm, `ck brief wait` returns it; read-only and server refusal.

Run plan (autonomous run 3): step 3.
- ck-core: `Brief.questions` (at most 3, each names the fields it affects; schema gate), `Document::content_at(rev)` replaying inverses.
- ck-server `brief` module: propose (inferred values only, user-set fields kept, status proposed or auto, through the patch and lock gates as the agent), get (compact view with rev), diff since a rev, wait (blocks on the event bus up to 25 s).
- Routes `/api/brief`, `/api/brief/propose`, `/api/brief/diff`, `/api/brief/wait`; `Ops` with caps; `ck brief propose|wait|diff|get`; MCP twins `brief_propose|wait|diff|get`.
- Tests: propose/get/diff/wait in server tests, caps, schema snapshot, parity.

Run plan (autonomous run 2): step 2.
- Embed the starter taste files and looks in `ck-assets` (`include_str!`), so an installed `ck` needs no assets folder; a test keeps the table in step with `assets/`.
- Library ops `taste_search`, `taste_read` (spec 6.2 name; the plan said get), `look_list`, `look_get` with request types and caps.
- `ck taste search|read`, `ck look list|get` and MCP twins (no studio needed); schema snapshot, caps, parity test.
- `assets/INDEX.md` under 60 lines, with a test.

Run plan (autonomous run 1): step 1.
- Author 12 taste markdown files (front matter: id, title, tags; body with numbers) under `assets/taste/`.
- Author 4 looks as `assets/looks/<id>.json` (colors, type scale, two fonts max, radius, spacing, motion personality).
- `ck-assets`: `taste` and `look` modules with error enum (`E_*` codes), loaders and validators (WCAG contrast at least 4.5 for text on background).
- Tests: all 12 taste files and 4 looks load and pass validation; each taste file under its token cap; a bad look is refused with its code.

## Phase 3: plugin and auto-launch (2026-10-29) - DONE (interactive proof BLOCKED)

Spec: 00, 05 (5.1), 06, 09. Proof: typing a video prompt in Claude Code opens the studio and the agent can patch a layer while the user watches.

Steps:
- [x] 1. `ck studio --ensure [--open] [--quiet]` and its MCP twin `studio`: idempotent, starts a detached server in the background, opens the browser tab once per server.
- [x] 2. First ck or MCP call ensures the studio is running (spec 5.1, agents without hooks).
- [x] 3. `ck hook prompt|guard|stop|session-end` (spec 6.1 hook behavior, fast, never blocks a non-video session).
- [x] 4. (`.mcp.json` BLOCKED: write refused headless; see handoff) `plugin/`: manifest, skills (clipkit-video with spec 9.4 text, clipkit-review), clipkit-critic agent, .mcp.json, hooks/hooks.json; format checked against the current plugins reference; structure tests.
- [x] 5. `ck init` / `ck uninstall` for Claude Code and Codex (detect, ask, idempotent, no residue); adapters/ texts.
- [x] 6. (real Claude Code / Codex session BLOCKED: needs an interactive session) Proof run as headless Playwright stand-in (`e2e/phase3.spec.ts`), reviewer and spec-checker with fixes, evidence.
- [x] 7. (Cursor format UNVERIFIED) Spec 6.3 "Cursor and any MCP client: rules file plus MCP registration" in `ck init` / `ck uninstall` (verify Cursor's current MCP and rules format first), then mark the phase done (interactive items stay BLOCKED).

Run plan (autonomous run 7): step 7.
- Researcher checks Cursor's MCP and rules format (if it cannot reach the docs, the format is UNVERIFIED and recorded).
- `--agent cursor`: merge `mcpServers.clipkit` into `~/.cursor/mcp.json` (the user's own entry is refused); rules in `<--dir>/.cursor/rules/clipkit.mdc`.
- `--agent mcp` (any client, never detected): `~/.clipkit/mcp/mcp.json` and RULES.md as a ClipKit-owned folder.
- Tests: idempotent, user files restored byte for byte, conflict refusal, an edited rules file is kept. Then mark Phase 3 done and set PHASE: 4.

Run plan (autonomous run 6): step 6.
- Headless stand-in for the proof: Playwright `e2e/phase3.spec.ts` runs the plugin's own hooks.json commands (prompt, stop, session-end) through `sh -c` with `CLAUDE_PROJECT_DIR`, opens the URL the fake `$BROWSER` received, and patches a layer through `ck mcp` (the `.mcp.json` command line) while the page watches.
- The real Claude Code / Codex session stays BLOCKED (needs an interactive session); recorded, not faked.
- rust-reviewer on the Phase 3 diff, spec-checker on Phase 3; fix real gaps or record them.
- VERIFICATION.md evidence; mark phase done if only the interactive items remain; PHASE: 4.

Run plan (autonomous run 5): step 5 only.
- Researcher: Claude Code local marketplace (`extraKnownMarketplaces` directory source + `enabledPlugins`) confirmed; Codex docs unreachable, so its format is UNVERIFIED (DECISIONS.md).
- ck-cli `install` module: `ck init` / `ck uninstall` with `--agent claude-code|codex` (default: detected) and `--yes`; without `--yes` it asks on a terminal and otherwise only prints the plan.
- Claude Code: write the embedded plugin (plus .mcp.json) as a local marketplace under `~/.clipkit/claude-plugin`, add two keys to `~/.claude/settings.json`. Codex: a marked `[mcp_servers.clipkit]` block in `$CODEX_HOME/config.toml` and a marked section in `$CODEX_HOME/AGENTS.md` (text in adapters/codex/).
- Tests (fake HOME): plan-only without --yes, init writes, second init unchanged, user content kept, uninstall restores the tree byte for byte, conflict refusal, output cap.

Run plan (autonomous run 4): step 4 only.
- Researcher checks the current plugins/hooks/skills/agents reference; differences recorded in DECISIONS.md (and hook field names re-checked).
- plugin/: .claude-plugin/plugin.json, .mcp.json (`ck mcp`), hooks/hooks.json (prompt, guard on Write|Edit|Bash, stop, session-end via `ck hook <name> --dir "$CLAUDE_PROJECT_DIR"`).
- skills/clipkit-video/SKILL.md with the spec 9.4 text verbatim plus pipeline.md, guardrails.md, voice.md; skills/clipkit-review/SKILL.md; agents/clipkit-critic.md (read-only, at most ten lines).
- Structure tests (Rust, in ck-cli tests): files exist, JSON parses, hook commands name real `ck hook` subcommands, SKILL.md under 100 lines and contains the 9.4 text, frontmatter present, no timestamps.

Run plan (autonomous run 3): step 3 only.
- ck-cli `hook` module: reads the Claude Code hook JSON from stdin (capped), project folder from `--dir` (hooks.json will pass `$CLAUDE_PROJECT_DIR`).
- `prompt`: acts only for a video-looking prompt or an existing .clipkit; ensure + open, one-line note (url, mode, stage, agent). `guard`: only in a .clipkit project; exit 2 with a reason for Write/Edit on .clipkit/, assets/, media files and Bash running ffmpeg/ffprobe/sox, touching .clipkit or mutating assets/ or media.
- `stop`: never starts a studio; respects stop_hook_active; blocks once on agent locks or agent status working. `session-end`: releases agent locks; always exit 0.
- Tests: unit tests for the classifiers; CLI e2e per hook (non-video session silent and exit 0, block exit 2, stop loop guard, session-end unlock).

Run plan (autonomous run 2): step 2 only.
- ck-server `launch::connect(dir, launcher)`: Ops for the running studio, or ensure one first (the start also opens the tab once, best effort, so hookless agents see it).
- CLI ops commands and every MCP tool except `studio` go through it; E_SERVER now only means the start failed.
- Tests: each CLI command and the first MCP call start a studio; a start failure (corrupt project.json) is the same error in both doors; existing E_SERVER tests updated. Test helpers set BROWSER=true so no real browser opens.

Run plan (autonomous run 1): step 1 only.
- ck-server `launch`: running-server probe (moved from `ck studio`), `ensure` under a file lock: spawn `ck studio` detached (new process group, stderr to .clipkit/studio.log), wait until the new pid answers status, report early exits with the log tail.
- `--open`: open `url#token=` with `$BROWSER` or the platform opener once per server (marker .clipkit/opened); `--quiet` prints nothing.
- MCP `studio` tool ({open}) runs the same ensure; tool table, caps and snapshot updated.
- Tests: CLI e2e (start, idempotent second call, open once via a fake `$BROWSER`, quiet, stale server.json, early-exit error), MCP parity for `studio`.

## Phase 2: server and studio shell (2026-10-28) - DONE

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

Run plan (autonomous run 8): step 8 only (phase wrap-up).
- Release dates for the unverified Phase 2 crates from the crates.io API (node fetch); pin back any under two weeks old.
- rust-reviewer on the Phase 2 diff (4f9a064..HEAD) and spec-checker on Phase 2; fix real gaps, record the rest as open.
- Decide CSP and the CI e2e job (simplest option that fits the spec; DECISIONS.md).
- VERIFICATION.md Phase 2 final rows, `just check` + `just e2e` green, tick step 8, `PHASE: 3`, handoff, commit.

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
- [x] 8. Evidence, reviewers, DECISIONS.md, handoff, commit; mark phase done. Review follow-ups: `ck heartbeat` + MCP twin, idle shutdown (spec 5.1), CSP + X-Frame-Options, CI e2e job, open-time check (spec 1.2), crate pins.

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
