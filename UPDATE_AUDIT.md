# Pipeline update audit — U0 (28 October 2026)

Audit only: no application code changed in this phase. Existing uncommitted asset relocation, usage cards, server and UI changes belong to the starting workspace and are preserved, excluded from phase commits unless explicitly extended.

Sources of truth: DECISIONS.md; docs/VERIFICATION.md (there was no root VERIFICATION.md); docs/SOURCES.md; packages/assets/components/sources.json; assets/catalog.json; core project/store schemas; CLI/MCP operations; managed SceneService; server creative/video workflows.

| Requirement | Status before update | Existing extension point / missing work |
| --- | --- | --- |
| §1–2 knowledge, progressive retrieval, adaptation, base/layers, handoff | partial | Usage cards, library, human creative checkpoint and patch engine exist; add compact ladder, provenance and resumable pass state. |
| §3.1 cards and preservation | partial | assets/instructions and catalog.json exist; add generated YAML cards, reviewed/owner-note preservation. |
| §3.2 indexes | partial | Existing machine catalog; missing JSONL and root/source compact indexes. Keep old catalog/API. |
| §3.3 kinds/runtime | partial | Existing component/reference/media distinction; add explicit runtime and classification, AST/media extraction. Adobe stays reference-only. |
| §3.4 / §6 taste | partial | Motion direction/rhythm guides exist; add bounded searchable starter taste files and planning retrieval. |
| §4 ingest | missing | Existing source provenance/hash inventory and pack validator are reusable; add incremental offline ingest, metadata, draft cards, resumability and reports. Owner list FOUND in docs/SOURCES.md and component source receipts; do not clone unapproved repositories. |
| §5 ladder/budgets/search/cache | partial | Bounded library/card APIs and estimated telemetry exist; add L0–L4, justification/turn limit, per-pass meter and ranking. |
| §6 AST | missing | Compiler infrastructure exists for scene compilation; add real TypeScript syntax map, exact symbol slices, structured CSS/JSON extraction. |
| §7 adaptation | partial | Managed scene publication, schema controls, instrumentation and immutable versions exist; extend with deterministic codemods, local derived registry/provenance. Preserve explicit pack trust. |
| §8.0 boot | partial | ensure/studio/doctor/project summaries exist; add attach/open behavior and pipeline health summary. |
| §8.1 intake | partial | Media registration/probing exists; add evidence-based edit/hybrid/generate and audio/image handling. |
| §8.2 plan | partial | Independent storyboard/direction/reel/voice slots exist; add compact plan and voice-first beat timing without replacing creative approval. |
| §8.3 base | partial | Ingest/proxy/transcript/cut APIs and native scenes exist; add idempotent base insertion and generated scaffold through existing renderer. |
| §8.4 passes | missing | Existing creative stages differ; add persisted ordered pass orchestration, revision guards, pass checks. |
| §8.5 / §9 gates | partial | Schema, render determinism and source bounds checked; add named short checks; unknown perceptual/audio measurements must be explicitly uncertain. |
| §8.6 handoff / §8.7 iteration | partial | Lease, comments, range protection, exact-session human checkpoint exist. Preserve them; add pipeline stage to banner and scoped gate selection. |
| §10 migration | missing | Version 1 schema strips unknown keys; add additive optional fields with explicit backward migration; avoid rewriting user project on startup. |
| §11 CLI/MCP | partial | Existing asset commands/API remain; extend with ingest/build/validate/search/skeleton/slice/source/derive, taste/pipeline/verify/meter. |
| §12 phases | missing | Implement U1–U8 as incremental commits; keep root project untouched. |
| §13 verification | partial | Existing test suite and archived evidence; add meaningful broken/good fixtures and integration evidence; never report provider or aesthetic success from fixtures. |
| §14 runtime skill | partial | Agent instructions installed by ck init; extend them under ~80 lines while preserving human proof checkpoint. |
| §15 safety/rights | exists/partial | Source receipts, pack trust, scoped mutations and data-only prompts exist; preserve these in all additions. |

Open questions: no additional owner Premiere/AE/LUT/SFX/font source list is present beyond documented sources. Use explicitly local stand-ins in tests, no invented rights. Preview feasibility and actual voice loudness require real provider/media evidence. Existing aesthetic acceptance, full production authorization and live Fish/Claude checks remain open; this engineering update does not close them.
