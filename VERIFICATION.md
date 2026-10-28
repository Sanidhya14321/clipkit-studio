# Pipeline update verification

Existing release evidence and pending human acceptance remain in [docs/VERIFICATION.md](docs/VERIFICATION.md). This file tracks only the pipeline update; fixture checks do not certify aesthetic quality or live voice providers.

U0: audited current working tree, source receipts, schema, CLI, MCP and workflows in UPDATE_AUDIT.md. No application code changed in U0.
U1: additive AssetKnowledge uses existing UsageCatalogSchema and leaves catalog.json/instructions/pack loading compatible. Generated cards preserve reviewed cards and owner notes. Verification pending U8.

U2: local registered ingestion accepts explicit license evidence/revision, inventories without executing code, groups component helpers, streams content hashes, records metadata and resumable per-asset checkpoints. Drafting is deterministic (zero model calls/cost); `--dry-run` reports estimate. Generic previews are skipped with reasons; derived previews use the existing safe scene path. Tests pending U8.

U3: Babel TypeScript/JSX parser and PostCSS produce symbol offsets, props/defaults, primitives and bounded skeletons. JSON is parsed structurally. GLSL extraction explicitly reports unsupported rather than claiming a parser. Search is ranked and capped; L1/L2/L3/L4 budgets, two-card limits, persisted meter and single justified full-source read are enforced per turn/pass. Verification pending U8.

U4: additive v1 load migration retains pipeline/provenance/layer fields and leaves legacy projects unchanged until edited. Parsed codemods replace literal tokens/defaults and remove selected JSX nodes. Derived output gets local JSON/Zod props schemas and selection paths; publication must pass the existing immutable scene compiler/render verifier before registration. Verification pending U8.

U5: persisted ordered pass state, evidence-based media mode detection, voice-take beat timing, non-destructive base insertion and named deterministic quality gates added. Measurement-dependent checks explicitly return uncertain. Server/CLI wiring and restart/pass atomicity tests follow in U7/U8.

U6: bounded taste search/read loader, 12 concise owner-editable starter guides and validated numeric constraints. Planning reads at most three matching guides before asset retrieval; the same constraints drive search/gates in server wiring. Verification pending U8.

U7: CLI and MCP expose the retrieval ladder, offline build/ingest/validate, taste, detect/plan/base/pass/status/resume/handoff, named verify and meter. Edits use inherited or external leases; patch stages use current baseRev. Agent instructions preserve the human motion-proof approval checkpoint. Lock banner and token popover show pipeline/retrieval state; export notices include adaptations. Rebuilt existing component registry to fix relocated-source paths. Initial typecheck passes; targeted tests exposed and repaired duplicate codemod spans. Full evidence follows in U8.

## U8 — final engineering evidence (28 October 2026)

- TypeScript: `pnpm typecheck` passes.
- Regression: `pnpm exec vitest run --maxWorkers=2`: **23 files, 153 tests passed**. This includes existing patches/leases/media/scenes/creative approval regressions and the new suites below. Invalid-source compiler output in the test log is an intentional failure fixture.
- Browser: `pnpm test:e2e`: **6 tests passed**, including six viewport sizes and the new pipeline stage + ladder meter test. Screenshot `.clipkit/pipeline-banner-meter.png` was visually inspected.
- Sources: `pnpm assets:check`: **211 source/font/license receipts verified**, zero errors. Original bytes remain unchanged.
- Knowledge: `pnpm ck assets build` then `assets validate`: **21 cards, five source groups, 12 parsed code maps**, validation passes. An unchanged repeat reports `changed: 0`, `cardsChanged: 0`, AST `changed: 0`. Root index is 28 lines.
- Real preview verification: `pnpm assets:verify`: **12/12** components passed repeated deterministic frame checks and full moving previews. Evidence: `artifacts/0.5/components.json`; previews under `public/components/286f410ae0e736f36edfcad059dbb8cc8e008eb92ba0c307be7d54b9780533fe/`.
- Production: `pnpm build` passes and produces the Studio, CLI and self-contained package under `artifacts/package`. No npm publish or remote push was performed.
- Local project preserved: revision **50**, SHA-256 **dd5b4e1597d22c3305478fdc4ed0ffb7e0c74dbecb79368343f6bd5d2fad940a**. The updated local Studio starts/attaches at port 4310.

| Requested check | Evidence and scope |
| --- | --- |
| U1 small stand-in ingest / incremental index | PASS: `tests/asset-intelligence.test.ts`, actual local MIT test source, original source hashes, unchanged mtimes. |
| U2 notes / reviewed preservation | PASS: generated owner notes survive changed source; reviewed bytes stay exact; stale reviewed source receipts fail validation. |
| U3 Adobe reference-only | PASS for executable-file rejection and reference classification; native author publication/provenance independently exercised in HYBRID. A real owner Adobe technique adaptation remains untested because no Adobe library was supplied. |
| U4 relevant capped search | PASS: three real intents (ribbon/aperture/gooey) rank the expected asset first; default five, hard maximum ten. |
| U5 skeleton/slices | PASS: three real components, exact requested symbols, serialized skeleton/slice caps; TypeScript/JSX and CSS use real parsers. |
| U6 justified L4 | PASS: missing justification and second read in one turn fail; persisted meter retains L4 count/justification. |
| U7 derive | PASS: color/easing/default/JSX-removal codemods; actual tsc check, actual render/tag verifier; original bytes unchanged; local JSON/Zod schema and atomic provenance/derived registry publication. Identical retry reuses published instances. |
| U8 placeholders | PASS: unchanged whole defaults rejected by instantiate; text/default-brand lint, reference-only instantiate rejection and explicit motif exception tested in `tests/layer-pipeline.test.ts`. |
| U9 detection | PASS: no video, video, graphics+video, audio, images, multiple-video sequence, and adding footage while retaining prior layers. Registered evidence plus ffprobe-backed test video. |
| U10 GENERATE | PASS for 30-second generated base, derived title, named scaffold, seven ordered passes and actual four-frame contact sheet. Voice-first beat arithmetic uses active take durations. A real 30-second narrated agent run/provider budget is not claimed. |
| U11 EDIT | PASS for real fixture import/base, seven passes and unchanged source bytes. Existing media tests cover trusted word-time cut boundaries. Natural-speech footage/provider acceptance remains open. |
| U12 quality gates | PASS: all nine gates fail on deliberate defects and pass a valid fixture. Unmeasured audio/contrast/bounds return uncertain; no false measurement success. |
| U13 resume | PASS: stop/restart server after a committed pass, acquire a new lease, continue next pass without duplicate layers. Failed/stale batches do not advance state; identical derive retries reuse instances; current-pass layer IDs are returned for recovery. No claim of a live model process-kill evaluation. |
| U14 comment iteration | PASS: anchored wording edit + one-line resolution leaves other layers/provenance unchanged; schema-only verification exercises a relevant scoped gate. |
| U15 meter | PASS for durable L0–L4 and per-pass estimated payload totals. Test retrieval is measured; actual full-agent context near 20k remains an evaluation target, not a demonstrated result. |
| U16 hostile text | PASS for deterministic data handling and source-path/symlink rejection. Runtime instructions explicitly treat cards/README/transcripts as data. No live adversarial-model robustness claim. |

`tests/pipeline-api.test.ts` exercises GENERATE, EDIT and HYBRID through authenticated HTTP with real scene compilation, media probing, rendering, revision/lease checks and persistent restart. The contact sheet `artifacts/pipeline-update/contact-sheet.png` was inspected as a technical fixture, not a tasteful finished film.

Test history: early checks found stale relocated source paths, overlapping codemod spans, missing selection tags, non-atomic provenance, and incomplete cross-pass timing checks; these were fixed. A parallel stress run returned a still-running render after one 45-second poll; job tests now use bounded repeated waits. One later parallel run terminated with SIGTERM; the final bounded-worker regression above passed. Full shader previews completed separately. These retries are not hidden as successes.

Remaining scope: GLSL semantic parser, complex upstream prop/structure codemods, new native LUT/effect primitives, measured integrated LUFS/true peak, exact rendered text bounds/contrast, additional owner Adobe/media sources, real provider/agent token-budget evaluation and the existing human aesthetic acceptance. Direct-media classification is knowledge metadata, not a claim that every external format executes natively. Existing release acceptance items in docs/VERIFICATION.md remain open.

## Update 2 — Brief Studio and direction engine (28 October 2026)

Implemented against the owner’s Update 2 document after the prerequisite review in UPDATE2_AUDIT.md. No live model comparison, microphone session, paid voice synthesis or human aesthetic rating is represented by test fixtures. User edits already present in mcp/compact.ts and studio/LibraryPanel.tsx were preserved.

Validation: `pnpm typecheck`; bounded-worker regression: **27 files / 168 tests passed**; Playwright: **7 tests passed**, including six laptop viewports (900–1920px), existing edit/undo/range/lock flows and the new Brief workflow. `pnpm assets:check` passes with **211 source receipts** and unchanged source digest `e7f1cd4fcd15a010b8af3537c98a271e3f4d9ade1e076f3f0759550cf1288427`. `pnpm build` creates Studio, CLI, render runtime and portable package. Expected diagnostics from deliberately broken scene fixtures are negative-test evidence. The production build still emits its existing large-chunk and runtime-font-resolution notices.

| Check | Evidence and limits |
| --- | --- |
| W1 Prompt → Brief | PASS: authenticated server test + browser prompt/proposal opens Creative Brief with prefilled essentials and source/confidence. Managed GUI prefill uses video_run and persists its pending continuation. |
| W2 Waiting / unlock / compact result | PASS: actual lease released, confirmation wakes blocked request, bounded 45s waits/timeouts and compact result under 2.4KB in server tests. No lock is held while awaiting input. |
| W3 Skip / autopilot | PASS: prompt skip sets auto; preset and normal confirmation tested. Autopilot does not approve the existing human motion proof. |
| W4 Dynamic questions | PASS: max-three and high-confidence-repeat schema failures; affects metadata and bounded extra answer controls. |
| W5 Pacing transparency | PASS: YAML-backed 10/35 cuts-per-minute display changes in browser; transparent timing/speech-rate/motion values. Semantic beat changes still need the flagged agent re-plan. |
| W6 Footage-only fields | PASS: absent in generated browser fixture; registered-video detection selects cleanup/transcript defaults. |
| W7 Brief diff / selective work | PASS for field diff, dependent IDs/pass invalidation, exact unchanged unrelated layer JSON and deterministic token restyling. Semantic structure/script changes are recorded as pending re-plan, not falsely marked rebuilt. No claim of automatic speech recutting. |
| W8 Impact preview | PASS: browser verifies old pace remains until Apply; server lists affected IDs before mutation and enforces briefRev. |
| W9 Tuning chips | PASS by deterministic mapping: punchier→fast, calmer→calm, more-premium→noir, shorter→80% length; all use preview/apply contracts. |
| W10 References | PASS: ffmpeg-created real PNG, ffprobe aspect and first-frame palette, source reference precedence, no placed content. Video scene-cut/loudness sampling is implemented with method/nullable stats. OCR/caption density remains unavailable. |
| W11 Storyboard checkpoint | PASS: build returns waiting with no runner; reordered/edited beats confirm at exact total duration. UI add/delete/reorder/swap controls implemented. Actual rendered motion-proof acceptance remains a separate checkpoint. |
| W12 Recipes | PASS: real recipe scene compilation/render, stable tags, briefDeps, atomic provenance, identical retry reuse and unsupported-aspect filtering. Seven original MIT native recipes; no imaginary third-party sources. |
| W13 Layout | PASS for declared recipe boxes across 9:16/1:1/16:9, minimum size, safe bounds, collision checks and aspect reflow. Conservative font estimation is not a browser-measured guarantee for arbitrary custom code. |
| W14 Sound | PARTIAL: actual licensed local SFX placement, quiet gains, event timing, music duck flag and ffmpeg source LUFS/true-peak measurement implemented. No certified final-mix target or listening acceptance claimed. Unavailable risers/room tone are skipped explicitly. |
| W15 Cheap seeing / critic | IMPLEMENTED: at-most-six-frame sheet, eight-value native recipe trace, one sheet per actor/pass and sampled-frame-only critic submission. Trace identifies declared recipe controls; arbitrary authored animation returns uncertain. No invented model verdict. Existing renderer/contact-sheet regressions pass. |
| W16 Variants | IMPLEMENTED: brief-limited two/three token variants, actual side-by-side Players, human pick preserves only chosen treatment and applies only to target layer. No subjective human A/B verdict is claimed. |
| W17 Native terminal voice | PENDING LIVE: instructions require context_get for deictic speech; context API publishes current layer/playhead/hover. Native agent dictation cannot be invoked from the Studio browser. A real terminal speaking/agent editing session was not run. |
| W18 Studio voice | PASS for deterministic local command/text routing and anchored free-form comment/event paths using existing voice infrastructure; live microphone/provider permission remains pending. No automatic model turn is launched by free-form speech. |
| W19 Ask | PASS for waiting question plus voice-text answer and option/typed answer routes. UI provides mic control; a physical spoken answer was not tested. |
| W20 Event bridge | PASS: brief confirmation/comments/voice events, cursor wait, timeouts, session end and shutdown wakeup implemented. |
| W21 Scoped locks | PASS: two leases, human edit on unlocked layer, owner-only patch, stale per-layer rev rejection, global structural exclusion, monotonically revised undo. Fixed semantic comparison so JSON key order cannot create false conflicts. |
| W22 Cache | PASS: two actual alpha recipe renders, second prepare hits both, changed headline renders one/hits one with stable other hash. Preview/export share matching cache receipts; cache-enabled MP4 decoding covered by the focused render test. Only isolated recipes are eligible. |
| W23 Branches | PASS: create/switch/independent stable-ID merge/discard, snapshots and retained discard recovery. Switching also removes optional fields absent on the target branch. |
| W24 Exports | PASS for actual MP4 and extracted standalone Remotion frame render without ClipKit imports; valid portable ZIP, SRT timecodes and OTIO schema/timing tests. Bundle resets revision history while retaining current content/resources/licenses. Independent OTIO/NLE reader round-trip unavailable; native scenes are MissingReference metadata until pre-rendered. Standalone basic media playback has documented advanced mix/caption/crop limits. FCPXML/EDL are unsupported. |
| W25 Taste | PASS: local repeated-answer defaults, edit validation, generate/disable/clear and explicit precedence; profile source visible. Kept/reverted ranking only uses affected asset layers. No hidden external analytics. |
| W26 Benchmark | PASS for ten-case method/report/chart, missing-data handling, explicit paired-command execution with fresh-output checks, input hashes, measured preview timings and fixture receipts; blind randomized UI/aggregate ratings implemented. Real ten-case agent trials, repeat variance and human ratings are PENDING. Fixture metrics are not a pipeline-versus-vanilla result. |
| W27 Replay | PASS: bounded paginated calls/stages/token estimates/patches, complete selected-turn export, source hashing and secret redaction. Actual provider usage records are distinguished from estimates. |
| W28 Untrusted content | PASS for deterministic validated fields/slots/profile data, escaped React rendering, no card/reference/transcript-to-command path, existing injection/path tests and explicit agent instructions. Live adversarial model robustness is not certified. Benchmark driver argv is explicitly supplied operator configuration. |
| W29 Three complete demos | PENDING LIVE: server/renderer/browser building blocks pass, including real recipe output, incremental tuning/comments and exports. Three consecutive provider-backed creative demos with spoken change and human motion acceptance were not run and are not replaced by fixture claims. |

Verification found and fixed: lossless audio/container mismatch in alpha caching; false layer-lock conflicts caused by object key order; stale brief-confirm theme values; stale storyboard revision reuse; overlarge replay output; missing UI export choices; invalid edited profile defaults; global branch optional-field leakage. Early regression overlap caused EADDRINUSE and was rerun without the overlapping server. Browser tests exposed two locator ambiguities and persisted proposed-brief fixture state; fixture setup and selectors were corrected. Final browser suite passes.

Existing saved root project remains revision 50, SHA-256 `dd5b4e1597d22c3305478fdc4ed0ffb7e0c74dbecb79368343f6bd5d2fad940a`. Test servers use temporary roots. The executable benchmark protocol and usage/limitations are documented in docs/UPDATE2.md. P2 delegation, remote analytics, source-diff compression and additional paid ingestion are not included. Original owner source bytes were not edited.

## Update 3 — Source Hub (28 October 2026)

Reviewed the Source Hub document before code changes in UPDATE3_AUDIT.md. Reused ingestion/cards/AST/retrieval and additive schema-version-1 fields. Final broad regression: **30 files / 177 tests passed**; **8 browser tests passed**, including the existing six laptop viewport checks and Source Hub collection/filter/Brief controls. Typecheck, build and asset receipt checks passed. The 211 original source receipts retain digest `e7f1cd4fcd15a010b8af3537c98a271e3f4d9ade1e076f3f0759550cf1288427`.

| Check | Evidence and scope |
| --- | --- |
| S1 Add/pin | PASS: actual fixture Git shallow clone with hooks/global configuration disabled, pinned 40-character commit, license hash and independent read-only-file snapshot. Public Tier 1 original clones are pinned in SOURCE_HUB_EVIDENCE.json; source bytes are not executed or committed. |
| S2 License detection | PASS: MIT, Apache-2.0, CC BY, GPL, Commons Clause, missing and conflicting fixtures. Restrictions win over permissive text; conflicting original/package declarations become unknown. SPDX version is not invented. |
| S3 Malware lures | PASS: non-GitHub clone instructions, remote install hooks/obfuscation, executable blobs and download-without-source patterns produce reject reasons. AI-directed text is flagged as data. |
| S4 No execution | PASS: fixture postinstall marker remains absent through vet/update/ingest. No dependency installation, package imports, demos or upstream previews. Hash mismatch prevents scoring/ingestion even after a snapshot is tampered with. |
| S5 Reuse | PASS: native fixture sources produce real YAML cards and AST symbol maps through existing ingest/extractAsset/buildAsts with new source/pin/license/grade/collection metadata. Actual public review sources were **not ingested**: owner explicitly directed unverified sources to quarantine. |
| S6 Grades | PASS: native fixture A vs time-driven fixture C, explicit sub-scores and visual=0/unrendered. Grades are static estimates, not fabricated aesthetic judgments. |
| S7 Determinism | PASS for host-reviewed WAAPI/GSAP seeking and actual local Canvas pixel comparisons, repeated and reverse-order frames; noisy fixture fails. Public imported code remains runtime reference-only; no arbitrary remote component/wrapper has been certified without a network-less sandbox. |
| S8 Collections | PASS: disabled collection/source excluded from search, C opt-in and globally capped external Starter (12, at most 8/source). Browser toggles persist; owner catalog uses a separate overlay. |
| S9 License mode | PASS: commercial-safe hides unknown/restricted cards; derive/placement/recipe asset references and export enforce mode. A slipped-in unknown receipt fails before job creation. Unknown media also blocks export until explicit owner rights declaration or intentional mode change. New briefs default commercial-safe; legacy snapshots retain any-mode behavior without byte migration. |
| S10 Attribution | PASS: active receipt report includes source URL/pin/SPDX/class/attribution; bundle copies original pinned license evidence. Owner media declarations are explicit and preserved. License classifications are technical labels; original texts are available in bounded pages. |
| S11 Updates | PASS: fetched pin is pending until Apply, only changed asset files are listed; license hash change triggers written-review requirement. Changing one fixture file preserves the other AST mtime; removed assets leave the current index while historical snapshots remain. |
| S12 Gaps | PASS: removing the only calm transition reports a calm-transition gap despite an available fast transition; eligible collections/license/source filters apply. |
| S13 Discovery | PASS with deterministic GitHub API fixture: names/stars/push/SPDX are returned as provisional metadata; no clone/index/installation occurs. Real original-source metadata was separately fetched during the public audit. |
| S14 Injection | PASS: hostile README fixture is flagged, never executed or granted authority. Raw newly indexed code remains reference-only; the agent prompt treats cards/readmes as data and cannot approve source changes. |
| S15 Output caps | PASS for source/lock/license/gap/search fixtures and authenticated API results, capped pages, byte-limited reasons/dependency values and existing CLI payload ceiling. Original license text is paginated; no full inventory or secret is sent to agents. |

Public original-repository evidence is in docs/SOURCE_HUB_EVIDENCE.json: Remocn `7aebcbe6514ed9e54e3a4a6786f83f8425a4a9af`, onda `3c814051269597b31fee1603bef7ccfb93d091b6` and snapcn `ee9ed5f0b1417e968b8e62a55ef514ecf149980b` contain MIT license files but unresolved static vet flags. reactvideoeditor/remotion-templates `6209b724798e48ff395f8df1a6fa2d26082372b5` lacks an original license file. README claims did not substitute for evidence. All four are quarantined in this Studio; **none was ingested**, following the owner's direction to quarantine sources that are not verified. Remaining seed entries stay unverified, with no invented repository/license/star claims.

Verification caught an any-mode ternary precedence bug, a numeric seed-ID validation error, literal badge markup accidentally included in accessible names, and an asynchronous checkbox reset. These were fixed and regression/browser suites rerun. Existing legacy catalog reachability fixtures explicitly use any mode; commercial-safe filtering has separate positive/negative API coverage. Existing broken-scene diagnostics remain deliberate negative tests.

Deferred at the documented cut line: arbitrary upstream previews in a genuinely network-less sandbox, automatic public wrapper certification, broad Tier 2/3 ingestion, extra Tier 5 media packs and account-gated registry integration. No public source code executes merely because static classification says native or grade A. Source registry/knowledge state remains local and separate from the saved film. Root project remains revision 50 with SHA-256 `dd5b4e1597d22c3305478fdc4ed0ffb7e0c74dbecb79368343f6bd5d2fad940a`; pre-existing compact.ts and LibraryPanel usage-guidance edits are preserved.

## Update 4 — debug harness (in-progress evidence)

Cache cleanup removed **2,247,046,448 bytes** from enumerated regenerable build/test caches; original project/history/media/voice/quarantine and historical results were retained. Cleanup receipt: `.clipkit/cache-cleanup.json`. Current saved-film checksum is `095f5d279d27cd92b71c150eae2477e1cc2f33f19ceef99d1ff6f2e0869b36ca`; older revision-50 evidence above is historical.

V-3/V-4 validation: `pnpm typecheck`; `pnpm test --maxWorkers=2` **32 files / 187 tests passed** (`/tmp/clipkit-update4-tests-final.log`); focused debug suite **10 tests passed** (`/tmp/clipkit-update4-focused-final.log`); `pnpm assets:check` retains 211 source receipts and original digest `e7f1cd4fcd15a010b8af3537c98a271e3f4d9ade1e076f3f0759550cf1288427`; `pnpm build` passes with existing font/chunk notices. `pnpm exec playwright test --repeat-each=3`: **27 passed**, `/tmp/clipkit-update4-browser-final.log`, screenshots and trace-correlated driver logs under `test-results/`.

The earlier browser run had 24 passes / 3 failures, all in revision inspection. It exposed order-sensitive JSON hashing; canonical revision hashes and a reordered-field regression fixed it. Failed evidence is retained under `.clipkit/update4/browser-pre-canonical-hash/`; it is not silently retried away. This was a diagnosed deterministic defect, not an unexplained flaky test.
