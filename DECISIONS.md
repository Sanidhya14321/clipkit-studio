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
