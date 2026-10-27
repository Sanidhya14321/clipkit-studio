# Pipeline update verification

Existing release evidence and pending human acceptance remain in [docs/VERIFICATION.md](docs/VERIFICATION.md). This file tracks only the pipeline update; fixture checks do not certify aesthetic quality or live voice providers.

U0: audited current working tree, source receipts, schema, CLI, MCP and workflows in UPDATE_AUDIT.md. No application code changed in U0.
U1: additive AssetKnowledge uses existing UsageCatalogSchema and leaves catalog.json/instructions/pack loading compatible. Generated cards preserve reviewed cards and owner notes. Verification pending U8.

U2: local registered ingestion accepts explicit license evidence/revision, inventories without executing code, groups component helpers, streams content hashes, records metadata and resumable per-asset checkpoints. Drafting is deterministic (zero model calls/cost); `--dry-run` reports estimate. Generic previews are skipped with reasons; derived previews use the existing safe scene path. Tests pending U8.

U3: Babel TypeScript/JSX parser and PostCSS produce symbol offsets, props/defaults, primitives and bounded skeletons. JSON is parsed structurally. GLSL extraction explicitly reports unsupported rather than claiming a parser. Search is ranked and capped; L1/L2/L3/L4 budgets, two-card limits, persisted meter and single justified full-source read are enforced per turn/pass. Verification pending U8.

U4: additive v1 load migration retains pipeline/provenance/layer fields and leaves legacy projects unchanged until edited. Parsed codemods replace literal tokens/defaults and remove selected JSX nodes. Derived output gets local JSON/Zod props schemas and selection paths; publication must pass the existing immutable scene compiler/render verifier before registration. Verification pending U8.

U5: persisted ordered pass state, evidence-based media mode detection, voice-take beat timing, non-destructive base insertion and named deterministic quality gates added. Measurement-dependent checks explicitly return uncertain. Server/CLI wiring and restart/pass atomicity tests follow in U7/U8.
