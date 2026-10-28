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
