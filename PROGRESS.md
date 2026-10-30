# Progress

## UI handoff (2026-10-30)
1. Done: owner-approved workflow integration on `codex/ui-pro`, isolated at `/Users/sanidhyavats/Desktop/clipkit-studio-ui-pro`; functional timeline/inspector, import/source/recipes, recording, review/export and asset-use fixes. Main is based at committed `2d0cc96`; no main-checkout edits or push.
2. Proof: 26 real-server browser flows repeated five times (130/130), 8 frontend contracts repeated five times (40/40), build and 20 current workspace screenshots; Rust/hook/frontend check results are in `docs/updates/ui-workflow-evidence/`. See VERIFICATION.md for exact counts.
3. Open: external derived registry/adapter, attribution without a located hub, missing asset kinds/previews/fonts, live providers/agents, burned-in captions, live narration/music mix and remaining UI-2..UI-8 scope. Visible review-tab launch blocked by Work settings; automated real Chrome checks passed.
4. Next: coordinate branch integration with the other agent, then plan the backend asset adapter/license enforcement and remaining editor capabilities. Do not represent 62 searchable cards as 62 executable components; `docs/updates/ASSET_USAGE_AUDIT.md` documents the distinction.
5. Start: normal Rust/Node/pnpm/just/ffmpeg prerequisites or this session's `/tmp/ck-ui-test-env.sh`; `just check`; `just test`; build studio/ck and worker, then Playwright. Local review project: `http://127.0.0.1:43175/session` while its process is running.


## Update 5 and 6 queue (owner order, 2026-10-30)
- [x] U5-2: requirements stay in docs/updates/ (spec frozen)
- [x] U5-3a separate studio token (patch, consent, stop)
- [ ] U5-3b draft and apply; interrupt and redirect
- [ ] U5-4 energy gates, playback check, mood to look, caption presets, router skill
- [ ] U5-5 beat grid, beat sheet, sound kit on cuts
- [ ] U5-6 kinetic block library
- [ ] U5-7 save as style; face-safe zones and reframing
- [ ] U5-8 early benchmark (PAUSE: needs live agent runs)
- [ ] U6-1 to U6-8 (PAUSE at the gates in UPDATE_6.md 7.1)
- [ ] PAUSE: engine decision (DECISIONS.md, owner answers after U6-0)

## Owner checklist: live-session steps headless runs cannot do
- [ ] Install Rust 1.99.1 (rustup), Node 24, pnpm and ffmpeg 8.0.1 on this Mac; the Stop hook skips its check without cargo.
- [ ] In a live Claude Code session: run `ck init`, restart, confirm `/mcp` lists `clipkit` (`plugin/.mcp.json` is now in the repo).
- [ ] Confirm Claude Code picks up the plugin from `extraKnownMarketplaces` + `enabledPlugins`; if not, run `claude plugin install`.
- [ ] Check the Codex config (`$CODEX_HOME/config.toml`, `[mcp_servers.clipkit]`) and AGENTS.md location against developers.openai.com/codex/mcp.
- [ ] Check `~/.cursor/mcp.json` and `.cursor/rules/*.mdc` format against cursor.com/docs.
- [ ] Run the demo scenario end to end in one real Claude Code and one real Codex session; flip the BLOCKED rows in VERIFICATION.md.


STATUS: IN PROGRESS (Update 5 and 6 queue; engine decision still open)
PHASE: U5-3

## Handoff (latest)
Pipeline audit: server now enforces brief, gates and retry; see DECISIONS.md, end-to-end pipeline audit. Changes uncommitted.
Update 5: U5-0 audit done (docs/updates/UPDATE5_AUDIT.md). U5-1 spike done: recommendation in DECISIONS.md, nothing migrated. Next: owner approves the engine and decides how U5-2 may edit docs/spec/ (audit finding 3); U5-2 and later not started.

1. Done: Phase 10 (stretch): critic contact sheet (`verify --critic`), `ck reference analyze`, local taste profile (`ck profile`), storyboard checkpoint with studio confirm, OpenTimelineIO and SRT export. All phases 0 to 10 have run; open items below.
2. Proof: `just check` 356 of 356 nextest (1 skipped), hooks 41 of 41, studio clean; `just e2e` 17 of 17. Evidence in VERIFICATION.md, Phase 10.
3. Open: Fish Audio provider calls (key and API check), bench results (real agent runs and owner media), sound design placement (vetted CC0 sound set), studio side-by-side variants view, frame-clock adapter and card previews, packaged installers, more agents, Demucs, reviewer agents never run; see Open questions.
4. Next: owner priorities among the open items; first candidates: add FISH_AUDIO_API_KEY and verify the API, run the reviewer and spec-checker agents over phases 5 to 10, run the benchmark.
5. Start: `just check`; `just worker`; README.md.

## Open questions
- Audit 2026-10-30: full list of what was missed and why in docs/AUDIT.md; CI is green again.
- Phase 9 close: not built: sound design placement (needs a vetted CC0 sound set), the critic pass, studio views for history, branches and variants, `ck repro`, per-layer render cache (whole-video cache built), original-resolution media for the render path (proxies used), audio items (music beds) in the mix, C-arm ablation switches for the bench. Bench results are PLACEHOLDER until real agent runs.
- Phase 8 close: the Phase 9 render worker must also cover the wave 3 frame-clock adapter with determinism tests (same frame twice, out of order) before any wrappable card goes live, card previews (preview.webp) and the derived copy's type check and first render. Non-code ingest (SVG icons, LUTs, sfx) and a path filter for very large repos (tabler-icons) are not built.
- Phase 7 close: Fish Audio TTS (`FishTts`), STT (`FishStt`) and cloning are stubs; with the key, verify the API (spec 8.3), implement them behind the existing `Speaker` and `Transcriber` traits, and add cassette tests. Spec 8.3's crossfades and music ducking belong to the mix in the Phase 9 render path; the sound plan already emits duck cues.
- Phase 7 step 6: `ck-server` `tests::idle::only_authorized_calls_count` uses 40 ms wall-clock margins and failed once under a loaded full run; move it to paused tokio time like the other idle tests.
- Phase 6 close: self review against spec found these gaps outside spec 12's Phase 6 list, carried: Stop button with cooperative cancel (spec 5.2), timeline drag/trim/split/snapping/zoom (spec 5.4), loop range in the studio. Run the reviewer agents on e33924a^..HEAD when available.
- Phase 5 close: run the rust-reviewer and spec-checker subagents on the Phase 5 diff (05509be..HEAD) in a session that has them. Self review found no blocker; known gaps: export and import block one request thread (no queue or progress), `/media/` non-range requests read the whole proxy, packed transcript cap trims lines for sources much longer than 10 min.
- Phase 5 step 4b: owner will add FISH_AUDIO_API_KEY; then verify the Fish Audio STT API, build `FishStt`, record a cassette test. Decide whether whisper-rs (C++ build) goes in by default or behind a cargo feature.
- Phase 4 step 10c: spec-checker gaps carried forward (verify --critic, audio and captions gates, motion direction and holds, license-mode check, retrieval budgets, token meter, variants, brief more-options, verify before idle); assign each to a later phase (DECISIONS.md, step 10c).
- Phase 4 step 10b: the Phase 4 proof ends at a verified project; exporting the generated video waits for the Phase 9 render worker (DECISIONS.md, step 10). Confirm this is acceptable or pull a minimal render path forward.
- Phase 4 step 9c: verify how Claude Code and Codex handle long-running MCP tool calls and tune the `wait` timeout (default 20 s, at most 25 s) (DECISIONS.md, step 9c).
- Phase 4 step 9b: sound cues carry no sfx asset until CC0 sound sets are ingested (Phase 8); timing constants and the `hook` name match are choices (DECISIONS.md, step 9b).
- Phase 4 step 8: reuse limit (3 per asset) and the overlay-track overlap exemption are choices; `verify` does not yet block setting `idle` (DECISIONS.md, step 8).
- Phase 4 step 9a: the project look is now `tokens.look`; a recipe applied with a request `look` different from it still previews in the project look (request look is not stored). A brief `look` change does not re-theme once a look is stored (DECISIONS.md, step 9a).
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

## Phase 10: stretch (2026-10-30, done; more agents and Demucs not built)

Spec: 12 (Phase 10), 04 (critic pass, storyboard checkpoint), 06 (clipkit-critic subagent, storyboard and reference commands), 08 (8.1 reference, 8.8 taste profile, 8.6 OTIO and SRT). Owner asked to continue (2026-10-30); the cut line drops this phase first, so each step is small and verified.

Steps:
- [x] 1. Critic pass: `ck critic sheet` (6 low-resolution frames at cuts and motion peaks, stitched into one contact sheet) and the plugin's read-only `clipkit-critic` subagent (sub-agent roles).
- [x] 2. Reference match: `ck reference analyze <video>` (statistics only: duration, aspect, shot lengths, cuts per 15 s, loudness) with suggested brief fields sourced `reference`.
- [x] 3. Taste profile: local editable `profile.json` learned from what the user picks; fills brief defaults (source `profile`), never overrides explicit choices; `ck profile show|clear|disable`.
- [x] 4. Storyboard checkpoint: `ck storyboard get` (a still per scene) and the human confirm.
- [x] 5. OpenTimelineIO and SRT export.
- [ ] 6. (not built: needs format verification and a Python runtime) More agents and the Demucs sidecar: verify formats and runtime; build only what can be verified.
- [x] 7. Proof, evidence.

## Phase 9: polish and proof (2026-10-30, done; sound design, critic pass, bench results carried)

Spec: 02 (2.3), 08 (8.6), 10, 14 (14.5, 14.7). Owner approved Remotion rendering (2026-10-30). Proof: a generated project exports to MP4 through the render worker; the evaluator scores outputs; trace, replay and bisect work; the demo resets in under 10 s. Order follows the cut line (branches and variants last).

Steps:
- [x] 1. Render worker (`render-worker/`, Node, @remotion/renderer and @remotion/bundler 4.0.535): bundles the studio composition once, renders a job from stdin with progress lines; `E_RENDER`.
- [x] 2. Full export path: `ck export` uses the worker when component layers exist (muted video), ck mixes the audio (footage, voice takes) and muxes; render cache keyed by content hash; Phase 4 step 10b and Phase 5 carried export items close here.
- [x] 3. (determinism check done; adapter and previews carried) Frame-clock adapter and determinism test (same frame twice, frames out of order) for wrappable cards; card previews.
- [x] 4. `ck eval`: output-only evaluator (duration, aspect, loudness, clipping, black or frozen frames, cut accuracy, brief conformance) with fixture tests.
- [x] 5. Debuggability: `ck trace bundle` (redacted), `ck replay`, `ck bisect`.
- [x] 6. Demo reset (seeded project in under 10 s) and README (install, requirements, demo, voice privacy, Remotion licensing, asset licenses).
- [x] 7. Bench harness (results PLACEHOLDER until real runs) for A0/A1/B (scenarios, arms, report format); real runs need agents and tokens: results PLACEHOLDER until run.
- [x] 8. History and branches, variants (CLI and MCP; studio views not built) (cut-line items).
- [x] 9. Proof run, self review, evidence.

## Phase 8: assets at scale (2026-10-30, done; wave 3 adapters, previews and non-code ingest carried)

Spec: 07, 09 (retrieval budget), 14 (wave 2 and 3, 14.7 checks). Owner approved network fetches and pipeline-first (2026-10-30). Proof: a vetted source is ingested into cards and ast.json without running its code, search returns it within the budget gate in the right license mode, a restricted fixture is blocked in commercial-safe mode, a derived copy carries tokens and instrumentation, and the export license report lists it.

Steps:
- [x] 1. Card format (spec 7.2 front matter and body), loader and validator; catalog.jsonl; original fixture source packs in `fixtures/sources/` (permissive pack, restricted BBC-style pack, prompt-injection pack).
- [x] 2. Sources registry over `sources.yaml` and `sources.lock`: `ck sources list|add|gaps`; unverified or disabled sources never searchable.
- [x] 3. Vetting without running code: shallow read-only fetch into `assets/quarantine/<id>` (hooks and submodules off) or local copy; license reading and classification; malware-lure and static dependency checks; `ck sources vet` writes the lock entry (commit, license hash, class, findings).
- [x] 4. Static AST extraction (oxc) for TS/TSX: exports, props, imports, animation primitives, outline; `ck sources ingest` writes cards, ast.json, per-source INDEX.md and catalog.jsonl.
- [x] 5. Tantivy search with filters (kind, tags, aspect, license mode, grade, collection) and the retrieval ladder with budget gate: `ck assets search|card|skeleton|slice|source` + MCP twins.
- [x] 6. License mode in the brief, respected by search, recipes and export; `ck license report` (asset, Remotion runtime, attribution and trademark notes).
- [x] 7. Derive with codemods (type check and first render carried to Phase 9): project-local copy with tokens, easing and data-ck instrumentation, props schema; provenance recorded; `ck assets derive`.
- [x] 8. Wave 2 for real (rendercomp-free ingested; simple-icons vetted; tabler-icons too large; others lack a confirmed repo): vet and ingest the sources whose license reads permissive from the original repo; everything else stays unverified.
- [x] 9. Proof run (`crates/ck-cli/tests/assets_proof.rs`), self review, evidence.

Versions (2026-10-30): tantivy 0.26.2 (2026-09-08; 0.27.0 is under two weeks old), oxc_parser/oxc_ast/oxc_allocator/oxc_span 0.154.0 (2026-10-13).

## Phase 7: voice slice (2026-10-29, done; Fish Audio provider calls carried)

Spec: 06 (6.2 voice commands, 6.4 lanes), 08 (8.3). Proof: a script becomes voice slots, each slot gets a take (AI through a fake speaker in tests, recorded in the studio), takes are normalized and fit-checked, captions come from the active take, and push-to-talk runs a local command; `ck ask` returns the user's answer. Fish Audio calls (TTS, cloning) are PLACEHOLDER stubs until the owner adds the key (same as Phase 5 step 4b).

Steps:
- [x] 1. `ck voice script` (script to slots with target windows) and `ck voice mode <slot> ai|recorded|cloned`, MCP twins.
- [x] 2. Speaker interface (Fish TTS stub, E_VOICE), audio cache by content hash, take processing (silence trim, loudness to the voice target, fit check: time-stretch within 8%, else options); `ck voice speak <slot>` + MCP twin.
- [x] 3. Record-along: the studio uploads a recorded take per slot (teleprompter, countdown, takes list, active take); server converts, trims, normalizes, fit-checks; Playwright with a fake microphone.
- [x] 4. Cloning with consent: only from takes recorded in the studio, after a consent record; Fish clone stub; refusal tests. `ck voice clone` + MCP twin.
- [x] 5. Captions from the active take's timing (transcriber over the take), as caption items.
- [x] 6. Lane B push-to-talk: local command parser (play, pause, undo, redo, go to N seconds, split here), free-form phrases become comments at the selection or playhead.
- [x] 7. Lane C `ck ask` + MCP twin: a question card with options in the studio, the answer returns as the tool result.
- [x] 8. Proof run (`crates/ck-cli/tests/voice_proof.rs`), self review, evidence.

## Phase 6: interaction slice (2026-10-29, done)

Spec: 05 (5.3, 5.4), 08 (8.4), 04 (scope gate), 06 (comments CLI, ck context). Proof: the user comments on a layer, the agent picks it up through `ck wait`, fixes it inside the scope gate, and resolves it with a before/after diff; hover-select and tuning chips work in the studio.

Steps:
- [x] 1. Comments: lock id `@comments` (comments never wait behind the project lock); `ck comments add|list|reply|resolve` + MCP twins; `list` is the send bundle (open and working comments plus affected layer summaries); resolve records `resolvedRev` and the touched layers between created and resolved revs.
- [x] 2. Scope gate (spec 4): an agent reply can start work on a comment with a declared scope; while a comment is working, agent patches may touch only the anchored layer, the declared dependents and comments; anything else is E_SCOPE.
- [x] 3. Global structural lock (spec 5.3): track set/order, mode and total duration changes touch `@structure`; while the agent holds it every human patch is refused.
- [x] 4. `ck context` + MCP: the studio reports selection, hovered element, playhead and loop range; the answer adds open comments, about 300 tokens.
- [x] 5. Studio comments panel and pins (moment, range, layer, element), Send to agent; hover-select on the preview (highlight with label, click selects, Ask agent anchors a comment to the path); Playwright.
- [x] 6. Brief panel tuning chips (punchier, shorter, calmer, more premium) and diff re-planning: `brief diff` lists the layers whose briefDeps the change hits; impact preview in the studio; Playwright.
- [x] 7. Proof run (`crates/ck-cli/tests/interaction_proof.rs`), self review, evidence.

Crates: ck-core (comments lock id, scope gate, structure lock, model field `scope`), ck-server (sessions, routes, context), ck-mcp, ck-cli, studio.

## Phase 5: edit slice (2026-10-29, done; 4b transcription provider and Silero VAD carried)

Spec: 04 (BASE), 08 (8.2, 8.6 export), 14 (wave 1 editing tools). Proof: raw footage to a cut, exported MP4.

Steps:
- [x] 1. `ck-media` probe: `E_MEDIA` code, safe ffprobe call (argument array, timeout, `-` path refusal), typed `MediaInfo` (duration, video/audio streams, fps, VFR flag, rotation, audio track count) with fixture tests.
- [x] 2. (2a media functions; 2b project source and `ck import`) Import: content hash, 720p proxy and audio extraction into `.clipkit/cache/`, source registered in the project; `ck import` with MCP twin.
- [x] 3. Silence detection (silencedetect) and scene detection (ffmpeg scene score) as cut-candidate signals.
- [ ] 4. (4a core done; 4b BLOCKED: provider waits for the owner's key and API check) Transcription: Fish Audio STT (verify API first) with local whisper fallback (whisper-rs, spec 14); word timestamps; cassette tests.
- [x] 5. Packed transcript (phrases, silences, fillers; about 12 KB per 10 min) and `ck transcript get`.
- [x] 6. Cut candidates (silences, fillers, false starts, repeated takes) on word boundaries with fades; `ck cut suggest|apply`.
- [x] 7. Studio transcript panel delete-to-cut and clip-edge restore, with Playwright test.
- [x] 8. Hybrid mode: generated layers over footage.
- [x] 9. Fast ffmpeg export (typed filter graph), loudness gates (ebur128/loudnorm) and export presets; `ck export`.
- [x] 10. Proof run (`crates/ck-cli/tests/edit_proof.rs`), self review against spec (reviewer agents unavailable), evidence. Silero VAD via ONNX is pending the `ort` question (Open questions).

Run plan (autonomous run 17): step 2a, media side of import.
- Split step 2: 2a media functions in ck-media, 2b project source registration and `ck import` + MCP twin.
- `ck_media::hash_file` (streamed SHA-256; `sha2` is already in the tree via rust-embed, no new code).
- `ck_media::make_proxy` (at most 720p, H.264, keeps aspect) and `extract_audio` (48 kHz WAV), argument arrays, temp file then rename.
- `ck_media::CachePaths` keyed by hash under `.clipkit/cache/`; fixture tests; `just check`, evidence, commit.

Run plan (autonomous run 16): step list and step 1.
- Write the Phase 5 step list above from spec 12 and spec 14.
- Add `ErrorCode::Media` (`E_MEDIA`) to ck-core and update the pinned code table.
- Build `ck_media::probe` over ffprobe JSON with typed output; tests against `fixtures/generated/`.
- `just check`, evidence, commit.

## Phase 4: generate slice (2026-10-29, done; export carried to Phase 9)

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
- [x] 8. Quality gates of spec 4.3 that apply to GENERATE in `ck verify` (schema, gaps, overlaps, duration, safe area, size, contrast, fonts, motion limits, reuse, provenance, no placeholders).
- [x] 9. Look re-theme through tokens with zero agent tokens, sound placement (`ck sound plan`), handoff and `ck wait`.
  - [x] 9a. Stored project look (`tokens.look`) and `ck look apply <id> [--dry-run]` with impact preview and MCP twin.
  - [x] 9b. `ck sound plan`.
  - [x] 9c. Handoff and `ck wait`.
- [x] 10. Proof run: prompt to brief to a 15 second generated video to export; reviewer, spec-checker, evidence.
  - [x] 10a. Generate proof through the real binary (`crates/ck-cli/tests/generate_proof.rs`).
  - [x] 10b. Export: done in Phase 9 step 2 (render worker); the generate proof exports through it.
  - [x] 10c. rust-reviewer and spec-checker on Phase 4; fix gaps; mark the phase done with 10b carried to Phase 9.

Run plan (autonomous run 15): step 10c.
- Run rust-reviewer (Phase 4 diff) and spec-checker (Phase 4 vs spec) in parallel.
- Fix small confirmed gaps with tests; record larger ones as open questions or carry them forward.
- `just check`, evidence in VERIFICATION.md, mark Phase 4 done (PHASE: 5) with 10b carried to Phase 9, commit.

Run plan (autonomous run 14): step 10.
- Split: 10a generate proof through the real `ck` binary (brief propose, human confirm, three recipes filling 15 s, layout check, verify all gates pass, sound plan, handoff, wait); 10b export.
- 10b is blocked: component layers export only through the Node render worker (spec 2, full export path, Phase 9); recorded in DECISIONS.md.
- Test `crates/ck-cli/tests/generate_proof.rs`; evidence in VERIFICATION.md.

Run plan (autonomous run 13): step 9c.
- `ck-core::Document::set_agent`: engine-owned agent state (not a revision, like locks).
- `session::handoff`: `handoff` releases every agent lock, sets agent idle with a label, returns rev, released count, open comments; `wait --since <rev>` blocks (default 20 s, at most 25) until human or voice revisions land after `since` and returns them as compact events (comment, brief, checkpoint, edit).
- `/api/handoff`, `/api/wait`, `Ops::handoff|wait` (4 KB caps), `ck handoff`, `ck wait`, MCP `handoff`, `wait`.
- Tests: session (handoff releases and idles, wait classifies and skips agent revisions, timeout 0), tool table, CLI/MCP parity.

Run plan (autonomous run 12): step 9b.
- `ck-server::session::sound`: pure `sound_plan(project, look)`: whooshes just before cuts on the video track, hits on text landings, risers before hook layers, room tone under cuts, duck suggestions for music under voice.
- Read-only plan (no sfx cards exist yet, so cues name no asset; `sfx` stays empty until Phase 8 ingests CC0 sets).
- `/api/sound/plan`, `Ops::sound_plan` (4 KB cap, trimmed), `ck sound plan`, MCP `sound_plan`.
- Tests: pure cue placement per kind, clamping at 0, tool table, CLI/MCP parity.

Run plan (autonomous run 11): step 9a.
- `session::look`: `project_look_id` (tokens.look, else brief look, else clean-product) shared by recipe apply, verify and the studio preview.
- `look apply`: validates the look, writes `/tokens/look` as one agent revision (project lock), returns previous look and impacted layers (items with `$token` props); `dryRun` only previews.
- `/api/look/apply`, `Ops::look_apply` (capped), `ck look apply`, MCP `look_apply`; studio resolves `tokens.look` first.
- Tests: server (apply, dry run, unknown look, recipe/verify follow stored look), tool table, parity.

Run plan (autonomous run 10): step 8.
- `ck-server::session::verify`: pure `verify(project, look)` running schema, gaps, overlaps, duration, layout, contrast, fonts, motion, reuse, provenance, placeholders.
- `/api/verify`, `Ops::verify` (4 KB cap, trimmed), `ck verify`, MCP `verify`.
- Tests: one clean project plus one fault per gate; client test after a real recipe apply; tool table and CLI/MCP parity.

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

## UI-0 / UI-1 plan (2026-10-30, owner approved UI-1 in chat)

Isolation: branch `codex/ui-pro`, worktree `/Users/sanidhyavats/Desktop/clipkit-studio-ui-pro`, base `af6c2d5`. Do not touch the main checkout's uncommitted Update 5 engine work.

UI-0 (authorized audit): inspect existing studio and API, run current browser flows against a fresh demo, save screenshots and findings in `docs/updates/UI_AUDIT.md`. No application code changes. Generated bindings remain untouched under AGENTS.md.

Proposed UI-1 (approval required by AGENTS.md):
1. Introduce shared theme tokens, compact full-window shell, accessible panel registry, resizable split/tab layout and workspace presets. Retain all existing functional panels and the current preview engine.
2. Add menus, searchable command palette, live project/connection/agent/revision status and per-panel/top-level error boundaries. Wire supported commands to existing HTTP APIs, patches and WebSocket state; disable unsupported actions with reasons.
3. Persist layout safely in browser storage; server persistence requires separately reviewed server capability work if absent. No engine migration, Rust behavior changes or new dependency without verification.
4. Verify existing edits, context selection, locks, comments, brief, history and reconnect against the real server. Run studio checks, full `just check`/`just test` and relevant browser tests; save 1440/1920 screenshots and evidence.
5. Commit only isolated UI phase files. Hand off later UI-2 to UI-8 work explicitly; do not claim unsupported editor features.

Scope: `studio/src` shell/layout/theme, `studio/e2e` coverage and UI-specific audit/progress/evidence docs. Do not edit `docs/spec`, engine code, generated types, render-worker or shared dependencies during UI-1.

### UI-1 delivery checklist
- [x] Original theme tokens, dark/high contrast, compact/comfortable density; no new dependencies.
- [x] Split/tab shell and seven workspace presets; independent scrolling, close/reopen, maximize, desktop splitters, merged narrow tabs.
- [x] Stable mounted panels retain draft forms when docked, across workspaces and responsive changes; recording streams cleaned up on actual unmount.
- [x] Safe per-project/workspace browser storage plus per-key memory fallback; malformed preferences reset.
- [x] Shared shell commands, menus/palette, typing-safe keyboard handling, server undo/redo/stop, live connection/agent/revision status and panel/top-level boundaries.
- [x] Preserve generated types, revisioned existing edit requests, preview/context/lock behavior; adapt real-server tests to explicitly open panels.
- [x] Browser audit, visual matrix, frontend checks and read-only localhost preview opened in the visible browser.
- [ ] Real Rust server, media export, real agent-in-the-loop and performance/fallback acceptance: BLOCKED/unverified; see VERIFICATION.md.
- [ ] Floating, server layout persistence, normalized store/optimistic patch-command migration and richer registry capabilities: explicitly deferred in DECISIONS.md, not represented as working UI.

UI integration: rebased onto committed main `5ec6550` (engine adapter seam and pipeline gates). Only shared handoff/decision/evidence documentation required conflict resolution; both agents' records were retained. Main checkout and its active work were not changed.

## UI workflow integration plan (2026-10-30, owner approved in chat)

User correction: the UI must support the project's actual workflows and be usable; the prior UI-1 shell is insufficient. Continue in the isolated UI worktree. Main currently has active uncommitted Rust work in import/export/edit/voice/session/core/assets; do not edit those files or bring uncommitted work across.

Proposed single integration phase, grounded in spec 5.3–5.4 and 8.1–8.6 and the supplied UI update:
1. Runtime and capability audit: establish a real local ck server if the available official toolchain can be provisioned; use server contracts to distinguish supported functions, provider-dependent functions and capabilities requiring backend work. Retain the current engine decision.
2. Simplify navigation around the workflow: Edit, Generate, Audio/Captions and Review/Export; contextual panel activation, clear empty states, prominent import, add title/recipe, ask agent, verify and export actions. Avoid advertising unavailable specialized tools as complete workspaces.
3. Functional editing: timeline ruler/playhead/seek/zoom, selection, drag/move, trims, split/delete, snapping, keyboard and shared undo/redo. One revisioned patch per gesture; locked layers refuse; optimistic drafts roll back on rejection. Inspector edits real supported text/style/timing/audio properties.
4. Connect existing backend operations into clear panels: local-path media import (no fake browser file paths), source preview/placement, recipe application with slot inputs, verification/export with progress/error/output location, comment review and history, existing voice/transcript controls with their actual capability limits.
5. Improve performance: stable panel state, memoized derived data and rendering, avoid invalidation of the entire workspace per preview frame, bounded/visible timeline rendering and responsive panel access. Measure supported performance cases, do not invent budget results.
6. Verify complete representative workflows against real ck wherever possible, with refusal/disconnect tests and visible browser inspection; frontend contract tests stay separately labeled. Update audit/decisions/evidence, commit isolated changes and rebase only committed compatible main changes.

Files owned here: studio source, studio tests, UI-specific docs, and environment setup outside the repo. New server behavior/endpoints are not included while another agent owns those Rust modules. Existing missing backend capabilities will be called out plainly rather than simulated as working product features.

Approval is required before application code changes by AGENTS.md and .agents/skills/phase-runner/SKILL.md. Independent read-only audit and runtime preparation may proceed meanwhile.

### Workflow integration delivery

- [x] Official runtime provisioned in an isolated cache; real ck, ffmpeg, synthetic fixtures and Node render-worker workflows run locally.
- [x] Functional timeline editing, supported inspector values, optimistic rejection handling, title dialog and contextual workflow navigation.
- [x] Source/media placement, recipe slots/provenance, recording setup/capture, verification and MP4/OTIO/SRT paths connected to existing APIs.
- [x] Memoized monitor, separate playback clock and viewport-limited timeline targets; explicit limits retained instead of simulated features.
- [x] Real server authentication/locks/pipeline preserved after integrating committed main. Narrow mono recording, image serving/render map and shared worker-browser fixes recorded in DECISIONS.md.
- [x] Browser screenshot review caught and fixed the title/background mismatch; regression failed before fix and passed in all five final real runs.
- [x] Asset-use audit completed at the owner's request. Native imported images render in recipes and worker; external derive does not register/execute components. Remaining backend gaps are explicit in docs/updates/ASSET_USAGE_AUDIT.md.
- [x] Final real-server browser suite 130/130 and frontend contracts 40/40; current screenshot matrix is five workspaces × two widths × two themes. No live paid providers or owner media were used.
- [x] Final `just check` and `just test` green: 377 Rust tests passed/1 pre-existing skipped, 41 hooks, frontend format/types and 47 unit tests. Production studio/ck build and worker bundle passed.
- [x] Prior runtime BLOCKED rows in UI-1 are superseded by the workflow evidence. Historical phase limits and the main owner's Update 5/6 queue remain preserved.
- [ ] Merge/integrate with the other agent's branch only after coordinating ownership. No push requested.

Review note: reviewer/test subagents supplied earlier findings, which were addressed; their later turns hit the session quota. Remaining review and final checks were completed by the primary agent. No new independent reviewer pass is claimed. `ck doctor` is absent from this CLI; actual build/media/browser checks replaced that diagnostic step.

## Demo quality follow-up audit

Owner asks which assets are used and why the video is not fast-paced beat-wise. Current URL port 50696 is the original 15-second smoke-test demo: three original recipes, base holds of 4/5/6 seconds, no imported music, external derived asset, beat grid or beat sheet. The existing sound planner has cue timings without sound assets, and energy/beat gates remain absent. Functional test PASS must not be presented as creative-quality acceptance. Compared the supplied Update 5 section 3 and Update 6 core directive/section 4.2; recorded evidence in `docs/updates/ui-workflow-evidence/demo-pacing-audit.json` and `ASSET_USAGE_AUDIT.md`. No application code or main-checkout file changed; Update 5/6 backend implementation remains separately coordinated.
