# Pipeline update verification

Existing release evidence and pending human acceptance remain in [docs/VERIFICATION.md](docs/VERIFICATION.md). This file tracks only the pipeline update; fixture checks do not certify aesthetic quality or live voice providers.

U0: audited current working tree, source receipts, schema, CLI, MCP and workflows in UPDATE_AUDIT.md. No application code changed in U0.
U1: additive AssetKnowledge uses existing UsageCatalogSchema and leaves catalog.json/instructions/pack loading compatible. Generated cards preserve reviewed cards and owner notes. Verification pending U8.

U2: local registered ingestion accepts explicit license evidence/revision, inventories without executing code, groups component helpers, streams content hashes, records metadata and resumable per-asset checkpoints. Drafting is deterministic (zero model calls/cost); `--dry-run` reports estimate. Generic previews are skipped with reasons; derived previews use the existing safe scene path. Tests pending U8.

U3: Babel TypeScript/JSX parser and PostCSS produce symbol offsets, props/defaults, primitives and bounded skeletons. JSON is parsed structurally. GLSL extraction explicitly reports unsupported rather than claiming a parser. Search is ranked and capped; L1/L2/L3/L4 budgets, two-card limits, persisted meter and single justified full-source read are enforced per turn/pass. Verification pending U8.
