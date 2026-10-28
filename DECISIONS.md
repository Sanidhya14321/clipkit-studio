# Implementation decisions

- Preserve the JSON-first local architecture. The server serializes all mutations, durably appends a journal entry before replacing the snapshot, and recovers an interrupted snapshot from the journal.
- Persist runtime leases separately from undoable content. A lease has owner, token, turn ID and expiry; every agent mutation is checked. Browser lock styling is only a convenience, never the enforcement boundary.
- Use native timeline items with `at`/`dur` and source-relative `srcIn`; caption words are mapped from immutable source transcripts. Cut boundaries are validated and quantized. Cut audio is cached with short fades.
- Use a real child-process runner. MCP registration alone cannot send work to a client. Codex runs read-only at the filesystem level and edits via authenticated ClipKit MCP tools; its activity and actual committed layer IDs are visible.
- Use original deterministic native clip implementations rather than copying unaudited template repositories. Keep twelve license-verified asset/reference sources and vendor a curated subset with pinned hashes. An unavailable template source is documented as rejected rather than labeled MIT without evidence.
- Package internal server/CLI code into one executable bundle and include a separate source tree for Remotion bundling. Runtime workspace dependencies are not published separately.
- Do not use the already-registered npm name `clipkit`. Deliver `clipkit-studio-local` as a local tarball; public publishing is outside this build.
- Fish requests are real and use explicit model headers. Cache TTS before provider calls. Store mode-specific voice references, not a shared reference that could silently mix AI and cloned voices.
- Never infer individual word times from phrase-level ASR. Keep the transcript text but disable destructive word editing if timings are unreliable.
- Preserve true audio duration when outside the 8% fitting tolerance; flag the mismatch and let the human adjust the slot window. The app does not silently accelerate narration to arbitrary durations.
- Use one 720x1280 30fps H264/AAC preset. The renderer snapshots the revision and uses locally served original media; proxies are preview-only.
- Token estimates use UTF-8 bytes divided by four and are explicitly labeled estimates. Actual client usage, when emitted, is separate. There is no fixed savings number.
- The user supplies a Fish key, real talking-head footage and a consented voice recording. Missing external inputs cannot be replaced by simulated successes. Test fixtures use visible test patterns and tones.

## Document verification on 26 October 2026

The later supplied Word brief contains 79 verification criteria and added hackathon requirements. The feature audit is in docs/PRD-FEATURE-AUDIT.md. This task authorizes verification and public repository creation, not implementation of missing PRD features, so the audit leaves runtime code unchanged.

The audit records unresolved defects in cut-audio restoration, trim bounds, patch-driven lease renewal, pack-license validation, transcript budgets and baseline accounting. These require repair before claiming a full raw-footage demo. Earlier green unit tests and native render proofs cover a reduced MVP rather than all acceptance conditions. General packs, selective revert, expanded voice controls, Cursor integration and pitch/comparison assets remain incomplete or deferred; missing Fish credentials, human speech samples and Claude sign-in block the corresponding live checks.

## Audited gap fixes (26 October 2026)

- Successful patches refresh configurable leases. Async media preparation rechecks ownership before commit. Source bounds are schema invariants; restoring ranges rebuilds cached audio and captions.
- Historical reverts use stable-ID three-way merges; overlapping later edits are rejected atomically. Proposal acceptance follows the same merge discipline, with real ghost preview projects and human-only acceptance.
- Transcript packs use UTF-8 byte budgets and cursor paging; candidate previews are bounded. Full transcript/view operations are explicit.
- Executable asset packs require human trust and whole-folder content receipts, actual editable-tag and deterministic render checks. Untrusted declarative wrappers remain available.
- Voice fitting is explicit. Full-script alignment fails safely on uncertain matches. Browser TTS is marked preview-only. Natural-speech quality is an external acceptance check, not inferred from synthetic-tone tests.
- Public npm publishing remains separate from the installable local tarball because the original package name is occupied.

- Production and development dependency audits identified a transitive Zod advisory. pnpm workspace overrides pin patched Zod and a compatible webpack peer. The tarball includes a shrinkwrap and bundles the patched Remotion dependency tree because npm consumers otherwise re-resolve exact upstream dependencies and ignore dependency-level overrides. Installed-package audit is part of the smoke check.
- Trusted-pack verification drains asynchronous page-close requests before shutting down Chromium, including missing-tag failures; validation errors must not crash the server.

## Asset-card foundation (27 October 2026, 0.1.1)

- The supplied master asset document is a design reference, not an instruction to override user intent, falsely certify human authorship or install every listed library. We adopted cards, discovery, budgets and verification in the existing monorepo while preserving stable IDs and URLs.
- Original native implementations and previews remain explicitly agent-authored. Upstream author/license evidence is recorded separately. Publisher-provided assertions are not displayed as independent ClipKit verification.
- YAML cards are bounded, strict, alias-free data. The only added product dependency is the ISC-licensed yaml parser. App-run agent leases need an actual current card read before inserting new graphics, including direct patches.
- Preview images render at thumbnail resolution. A single checksum-bound legacy full Inter TTF exception is reported to preserve existing glyph coverage and projects; it does not relax the limits on new fonts.
- The verifier performs start/middle/end twice per native clip and can render the complete 30-second smoke twice, comparing decoded video/audio instead of nondeterministic container metadata. These are single-machine proofs.
- Orphan warnings use declared scene references and card age, never auto-delete files. Heavy visual-engine adoption, strict human-only replacement curation and automatic CC-BY credit scenes require separate work.

## Additive native motion library (0.1.2)

Added starfield, confetti burst, terminal typewriter and four-value bar chart through the existing Remotion runtime. Particle positions use seeded inputs and elapsed frames; there is no live physics state, external animation dependency or executable terminal. Charts accept nonnegative values with a shared zero baseline and explicitly label defaults as illustrative. Existing ten IDs and default props are preserved against the 1.0.0 manifest fixture. Native pack metadata advances to 1.1.0; compiled registry identity remains compatible.

These are agent-authored native components, not imported human-made animations. Usage cards retain that distinction. Optional starfield/confetti titles add font credits only when text is actually used. Three-frame repetition evidence is supplemented by two complete 42-second exports with decoded video/audio comparison. This adds four useful effects; it does not complete the broader 3D, shader and simulation roadmap.

Final regression caught a pack-watch shutdown race: closing the watcher stopped new events but did not drain an active registry write. Server shutdown now awaits that work and suppresses late state publication. A deferred-refresh test forces the race and verifies close waits before temporary-project cleanup.

The same check also exposed simultaneous registry rebuilds colliding on generated files. Rebuilds now serialize per project, with installation waits outside the queue to avoid deadlocks. Eight concurrent rebuild requests are covered by a real-filesystem regression.

## 0.2.0 workspace and library

- Use a resizable panel workstation, with one sidebar on compact desktops and a focus-managed drawer on narrow screens; do not introduce floating docking complexity.
- Count only ready timeline insertions: 46 native templates/animations plus 26 media resources. Fonts and supporting source files are separate.
- Keep registration separate from placement. Use copied, immutable source media and transaction-controlled instances; legacy footage_add remains compatible.
- Review/compile TSX and JSX packs behind content-bound trust. Raw HTML and source JSON stay descriptive until adapted and validated.
- Retain upstream source and notices, with separate original and adapted checksums. Adapt selected RenderComp components directly and replace browser clocks with project frames.
- Compare repeat stills exactly. Complete encoded video checks use exact decoded hashes first, then explicitly reported strict sparse raster-equivalence limits; audio stays exact. This is local Chromium evidence, not cross-machine certification.
- Allow loopback renderer GET requests for resource routes, including null-origin font loads. Keep mutation origin checks and private-media authentication intact.
- Preserve two-frame SFX audibility with bounded fades; property/volume edits must never quantize or retime unchanged source duration.
- Bundle the exact lottie-web peer explicitly so exports work outside the development checkout. Verify packed export duration against the project frame count, including legacy footage append behavior, rather than a fixed demo duration.
- Exclude cached copies of bundled/local resources from the ready catalog count while retaining recovery visibility when the original resource disappears.

## 0.3.0 agent-authored video editing and generation

- Remove stock packs from defaults/main source; preserve the fixed 0.2 renderer and notices in a separately checksummed compatibility archive. Existing project content/history is not migrated.
- Author managed TSX scenes, validate and publish immutable artifacts, and pin references per instance/export. Reuse one renderer rather than embedding Video Forge's standalone engine.
- Treat end-exclusive frame scopes as server-enforced edit boundaries. Preserve animation phase across splits; do not treat comments as advisory prompts alone.
- Keep Motion mechanisms and focused Video Forge guidance distinct from Media. New content should form connected editorial choreography, not a stock-card sequence.
- Route app turns to focused MCP tool families and load source/transcripts only as needed. Report incremental payload estimates separately from actual provider usage; no hardcoded savings figures.
- Extend planning to nonoverlapping chapters and chapter-scoped generation/score changes. Maximum duration is a schema limit; long-form performance remains unbenchmarked. No second agent/provider/project system.
- Source-file watches require an external authoring lease; app turns submit drafts directly. Compilation/static import validation is not an untrusted-code sandbox.
- File responses explicitly vary by Origin to prevent cross-renderer font-cache CORS mismatches during parallel export/verification.

## 0.4.0 film-directed pipelines

- Generation and editing use different finite state machines. Narrative/copy and shot/key-pose direction precede source; editing diagnoses a selected passage and preserves its outside content.
- Combine story and choreography in one director invocation with ordered, validated commits. Source production remains a separate tool profile; visual critique sees actual rendered images and cannot write code.
- Use an original visual study to communicate staging. The first A/B showed that extra planning fields alone made the result more poster-like. Preserve that failed experiment and its metrics.
- Keep one automatic repair budget and explicitly report remaining defects. A draft can skip critique but cannot claim to be reviewed. Unchanged validated direction may be reused.
- Long-form generation begins with a continuous chapter outline, followed by bounded chapter production. Planning is not completion of a film.
- Use MIT Motion Primitives adaptations for actual website-style TSX motion. Keep React Bits as a visual reference under its redistribution restriction. Do not bundle the user's Video Forge production captures.
- Treat A/B evidence as one observed pair, not proof of universal quality or efficiency. Preserve exact creative prompt, raw usage, initial weak result and original-speed outputs.

## 0.5 candidate: instruction-first creative preparation

Human feedback rejected the initial component-first proof. Component inclusion does not establish visual quality. Generation now starts with story/copy, a visual thesis, shot key poses, reading windows and concrete geometric handoffs; adopted components are optional implementation choices. Original React/CSS/SVG is equally valid. The first invocation renders a real three-second proof and pauses. Human approval resumes the exact session with a fresh lease; Keep revision merges the proposal in one undoable transaction. No critic grants aesthetic acceptance.

Independent beats/shots/copy/components/transitions/audio replace the one-shot-per-phrase contract for new preparation; older data stays readable. Nested control paths retain declared labels. Source-schema updates carry compatible human values and remove obsolete bindings. Job status waits up to 45 seconds for agent calls to reduce repeated model polling. Actual runner usage and incremental estimates remain separate; no token-savings claim is made.

Historical docs/scripts/outputs were hash-verified in the requested external archive. Five durable script entrypoints and current setup/architecture/authoring/workflow/sources/verification documents replace historical clutter. Root project bytes were preserved. This is a candidate implementation, not a visually accepted 0.5 release: the clean same-prompt full-film comparison and human preference gate remain open.

## Pipeline update U1

Extend the existing usage catalog with generated JSONL/card indexes; retain old library and pack interfaces. Originals stay at their existing licensed paths instead of duplicating source bytes. Root VERIFICATION.md tracks update evidence; existing release evidence stays in docs/VERIFICATION.md. Open question: additional owner Adobe/media libraries were not supplied.

## Pipeline update U2

Ingestion reads local sources without modifying originals or cloning new libraries. Registration requires license text evidence and revision. Deterministic card drafts cost zero provider tokens; no hidden batch model call. ffprobe metadata is optional; previews without an established renderer are explicitly skipped and logged. Open question: owner-specific grouping can be supplied in the source manifest when directory conventions are ambiguous.

## Pipeline update U3

Installed TypeScript 7 exposes a native compiler rather than the old createSourceFile API. Use the already resolved MIT Babel parser directly, and MIT PostCSS, as declared runtime dependencies for ingestion/codemods. No model-based parsing. A missing GLSL parser is reported as unsupported; no fake symbol map. Budgets measure returned UTF-8 bytes, not unobservable provider context. Open question: GLSL-specific semantic extraction remains unsupported until a parser is selected.

## Pipeline update U4

Keep schemaVersion 1 and journal pointer compatibility with an additive migration; infer no mode or defaults during startup. Derive uses a project-local copy, parser-position edits and existing SceneService publication. Derived wrappers bind the runtime item ID with data-ck-layer and a declared prop path; derived nodes add path/element tags beneath it. Reference-only assets cannot be copied through derive. Unsupported imports/complex props fail with guidance to author a native scene, rather than installing unknown dependencies. Open question: arbitrary upstream layout rewrites remain symbol-level authoring work.

## Pipeline update U5

Add an ordered layer pipeline beside the established creative workflow, retaining its mandatory human proof checkpoint. Resume consumes persisted completed passes, never replays layer operations. Pass patches may edit tracks/provenance and must validate before committing state and layers together. Gates distinguish failure from unmeasured contrast/LUFS; no synthetic claims of perceptual/audio success. Open questions: automatic cut selection is deliberately delegated to existing transcript/candidate operations; without trustworthy word timing, destructive speech edits remain disabled.

## Pipeline update U6

Taste prose is descriptive data with a separate validated constraints.yaml for deterministic checks. Starter values are explicit defaults rather than inferred from arbitrary owner prose. The owner can edit both. Open questions: exact platform-specific exclusion zones and font availability need actual target platform/media review.

## Pipeline update U7

Integrate the pre-existing source/card reorganization as the required baseline for knowledge ingestion; preserve original file hashes/notices and rebuild source paths in the component catalog. Keep unrelated creative-workflow edits in the working tree. TypeScript becomes a runtime dependency so derived components undergo actual static checks, then the existing deterministic render verifier, before publication. The new pass orchestrator uses the same store, lease, proposal and range guards. Existing stock/native/pack commands remain available; the new ladder is the runtime agent path. Open question: a real creative turn still depends on the owner's signed-in client and human approval; fixture runs cannot certify taste.

## Pipeline update U8

Real render verification caught missing scene selection bindings; generated/derived roots now bind the runtime item ID and a declared prop path. Scene publication commits layer/provenance/derived receipts together. Every layer pass runs all deterministic gates before advancing, since an audio pass can also change timing. Resume reports existing current-pass layers and identical derive retries reuse instances. Legacy source previews remain available to humans; agents use justified ladder retrieval.

Bound concurrent regression workers after renderer contention exposed a one-poll timeout assumption; a running job is not a failed job. Retain explicit uncertain results for unmeasured properties and pin them to affected layers. Preserve owner source bytes and existing project revision/checksum. Detailed evidence and outstanding format/provider/aesthetic limitations are in VERIFICATION.md; no full-agent 20k-token or taste-quality claim is made.

Open questions: owner-specific Adobe/LUT/SFX/font libraries and their rights were not supplied beyond existing source receipts. Complex source adapters and actual voice/film evaluation require those inputs; GLSL semantic parsing and new native LUT primitives remain explicitly unsupported.

## Update 2 U2-1

Brief confirmation is durable project metadata, separate from creative acceptance and paid provider consent. Optional additive v1 fields preserve old snapshots without inference on load. Agent results are compact summaries, not the full form. Waiting uses bounded HTTP/MCP long polls and releases the caller's global lease; default Codex MCP tool timeout is 60 seconds, so each wait caps at 45 seconds ([official MCP documentation](https://learn.chatgpt.com/docs/extend/mcp?surface=cli)). The fetched CLI docs do not establish one universal dictation shortcut; no browser-to-terminal STT API is assumed. Open question: microphone/provider behavior requires live permission and credentials.

## Update 2 U2-2

Brief impact previews precede explicit field application. A brief diff records changed fields and dependent layer IDs; unrelated layer JSON stays exact. Pacing changes mark the relevant passes for re-plan rather than silently cutting speech. Looks update project tokens and explicitly bound controls with zero model calls. Waiting, question and context state use a bounded persistent event bridge, with no edit lease held during user input. Open questions: layout measurement for arbitrary authored components still needs actual DOM/render inspection; no generic bounding-box claim is made.

## Update 2 U2-3

Ship four original token looks and transparent pacing.yaml values. Token bindings are resolved by the shared preview/export composition, so changing a look needs no model call or source rewrite. Existing components without explicit bindings preserve their original props. Brand facts override a preset where validated. The safe-area layout solver uses conservative text fitting and reserves collision-free boxes; arbitrary custom scene geometry remains outside its guarantees. Open question: exact font-shaping bounds still require rendered measurement for third-party components.
