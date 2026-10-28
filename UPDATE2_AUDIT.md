# Update 2 review and prerequisite audit — U2-0

28 October 2026. Audit phase only; no application code changed. User authorized review and implementation of the attached Update 2 brief. Existing uncommitted changes in mcp/compact.ts and studio/LibraryPanel.tsx are preserved.

Reviewed: DECISIONS.md, VERIFICATION.md, UPDATE_AUDIT.md, project/pipeline schemas and migration, store and leases, CLI/MCP, creative workflow, Studio UI/voice handling, asset knowledge/taste/derivation, shared renderer and exporters. Owner sources are in docs/SOURCES.md and assets/catalog.jsonl: 21 knowledge records, 12 code components; no additional Adobe/voice keys are assumed.

| Requirement | Before Update 2 | Implementation approach |
| --- | --- | --- |
| §2/3 Brief flow, waiting, defaults, fields, autopilot | missing | Fixed validated field catalog; project brief; explicit confirmation outside the edit lease; bounded waits. |
| §3 persistent brief / diff / impact / chips | missing | briefDeps metadata; preview impacts before apply; deterministic tokens, selective pass invalidation; no broad rewriting of footage. |
| §3 defaults precedence, questions, presets | partial | Reuse media evidence/brand and taste; max three dynamic questions; user values dominate; local presets. |
| §3 reference analysis | missing P1 | Statistics only from ffprobe/ffmpeg and trusted transcripts. No copied reference content or fabricated measurements. |
| §3 storyboard checkpoint | partial | Existing storyboard and human motion-proof approval exist. Add lightweight pre-build beat/recipe checkpoint without replacing that workflow. |
| §4 voice/context/wait/ask | partial | Existing browser/Fish voice, local routing and anchored comments exist; add context/event cursor bridge and questions, brief answer routing and privacy notice. Native terminal dictation remains the client’s own capability. |
| §5 looks/tokens/pacing | missing | Three/four local token presets, transparent pacing data and shared deterministic token binding in renderer. |
| §5 recipes/layout/motion | missing | Declarative slot recipes expanded through managed scene compilation and existing atomic provenance; grid/safe-area text layout; measured/uncertain checks stay distinct. |
| §5 sound / see / critic / variants | partial P1 | Reuse SFX cards, media gains/ducking and contact-sheet renderer; explicit audio measurements; numeric trace only for declared animation controls; optional human/model critic is never invented. |
| §6 local profile | missing P2 | Local opt-in signal log and editable defaults, never overrides explicit fields. |
| §7 layer locks/revs | missing P1 | Extend existing serialized patch store with scoped leases and per-layer revision preconditions; structural edits retain global exclusivity. |
| §7 cache | partial P1 | Existing immutable scene/module/media caching; add content-bound per-layer render receipts and invalidate by tokens/source identity. Do not claim full compositing-cache speedups without proof. |
| §7 branches/snapshots | missing P1 | Durable named snapshots and stable-ID three-way merge using existing journal/revert machinery. |
| §8 exports | partial P1 | Existing queued MP4/presets; add SRT, portable bundle/Remotion and OTIO where supported. Keep unsupported formats explicit. |
| §9 asset optimization | partial P2 | Existing source hashes/diffs, ranker and zero-model ingestion; add cannot-do/preset metadata and usage ranking where useful. No paid smaller-model ingestion is needed for deterministic drafts. |
| §10 benchmark/replay | missing P0 | Fixed suite/protocol, actual tool/patch/usage trace, blinded ratings UI and reports; absent baseline/provider/human data stays missing rather than fabricated. |
| §11 schema/CLI/MCP/UI | partial | Add optional fields through v1 additive migration; bounded typed operations and persistent UI panels. |
| §13 W1–W29 | partial | Add meaningful schema/store/server/browser tests and documented demos; live microphones/providers, subjective acceptance and vanilla-agent runs require real evidence. |
| §14 skill | partial | Extend installed prompt under ~100 lines; context first for deictic requests, confirmed-brief planning, bounded event waits after handoff. |

Review decisions: brief confirmation is separate from permission to incur paid voice/API usage. Autopilot may skip the brief form but does not grant voice-cloning consent or approve an existing protected motion proof. Existing projects must load without inserting inferred data or altering their saved content. Layer cache correctness precedes cache-hit claims. Benchmarks must never label fixture output as a real model comparison. P2 delegation is optional and no subagents are required for this implementation.

Prerequisites: preserve Update 1’s known measurement/GLSL/LUT limitations; use native managed scenes for recipes rather than restoring removed legacy stock packs. Reference analysis and extra interchange formats follow the documented cut line. Open inputs: no additional owner media libraries, consented clone sample, real benchmark acceptance ratings or provider credentials supplied.
