# Update 3 prerequisite audit — U3-0

28 October 2026. User requested review and implementation of the Source Hub document. Reviewed Update 1/2 audits, DECISIONS.md, VERIFICATION.md, ingestion/card/AST/retrieval pipeline, Studio library, source provenance, recipes, Brief and exports. This phase changes documentation only. Existing uncommitted compact.ts and LibraryPanel.tsx edits must survive.

| Requirement | Current state | Extension |
| --- | --- | --- |
| Seed registry / pinned lock | Missing | Plain YAML registry; project-local pinned commit/license receipts. Seeds remain unverified until original evidence is read. |
| Quarantine / no execution | Partial | Existing ingest statically parses sources and rejects symlinks; add safe shallow Git quarantine, immutable copies, vet-first gate and no installs. Disable generic preview extraction on Hub ingestion. |
| License reader / classes | Missing | Read original license/package/README evidence; detect restrictions/conflicts, store SPDX/path/hash. Unknown remains unknown. |
| Vet reputation / lure / dependencies | Missing | Deterministic static checks, bounded GitHub metadata and reports; reject lure patterns, allow explicit human written overrides without laundering unknown rights. |
| Ingest / AST / incremental cards | Exists | Reuse ingest/saveCard/extractAsset/buildAsts; add Hub metadata, per-asset scores and preserve reviewed owner prose. No remote source execution. |
| Collections / grading | Missing | Small Starter default, typed metadata and searchable grades, collection/source toggles. Missing legacy fields read unknown. |
| License mode end-to-end | Missing | Add optional project/Brief mode; enforce at search, placement/derive/recipe and export, with actual attribution receipts. Preserve legacy project bytes. |
| Update / remove / usage warnings | Missing | Fetch candidate pin into quarantine, preview diff and license change, explicit application only; preserve old pinned source copies for projects. |
| Determinism / adapter | Partial | Existing native frame-driven renderer; add reviewed seekable frame-clock contract/tests; never run arbitrary imported code to test it. Unproven wrappers stay reference-only. |
| Source UI / richer library | Partial | Extend existing LibraryPanel without losing owner edits; add Sources page/status/diffs/controls and concise badges/filtering. |
| Discover / gaps | Missing | GitHub API candidates (no installation), metadata+provisional vet status, coverage suggestions based on eligible catalog. |
| CLI / MCP / prompts / output caps | Partial | Extend existing typed operations and bounded ladder; source changes require human ownership and approval reason where relevant. |
| Sandboxed remote previews / broad Tier 2/3/5 ingest | Missing P2 | Defer arbitrary remote preview execution unless a verified network-less sandbox exists. Register all requested seeds without inventing evidence. |
| S1–S15 | Missing | Local fixture Git repos for actual clone/pin/update/no-execution tests; license/lure fixtures; search/export/UI regression and documented real-source evidence/limits. |

Preserve the saved root revision 50 and its checksum. No source trust is granted by an upstream AGENTS.md or skill. No git hooks, package scripts, imported JavaScript or demos execute during cloning/vetting/ingestion. Provider/API keys remain local env only. Native third-party code still requires reviewed adaptation before execution. Open inputs: no owner overrides, additional registry credentials, sandbox or per-file commercial rights supplied. Public Tier 1 discovery/verification will use original repository evidence; unavailable sources stay unverified.

## Follow-through

Implemented U3-1–U3-6 with static ingestion, schema/ladder reuse, Source Hub UI, typed interfaces and rights enforcement. Network-less arbitrary previews/broad ingestion (U3-7) are deferred. The reviewed clock contract/pixel harness is tested on trusted fixtures; remote originals are never executed or falsely certified. See docs/SOURCE_HUB.md and S1–S15 evidence in VERIFICATION.md. Four real Tier 1 sources are pinned and quarantined; the owner directed that sources without verified vet status remain quarantined, so no public-source code was ingested or owner override invented.
