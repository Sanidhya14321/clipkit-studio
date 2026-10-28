# Source Hub

Open **Sources** in Studio or `/sources`. The Hub registers sources, records static vetting and original license evidence, pins commits, previews updates, and maintains a small searchable supply. Third-party files and README/AGENTS text are data. Nothing installs dependencies, runs upstream scripts, launches demos or automatically imports upstream code into the renderer.

## Storage and trust

`sources.yaml` is the shipped unverified seed list. Project changes live in `.clipkit/sources.yaml`; `.clipkit/sources.lock` records original commit, license hash/text/path, inventory hashes, vet reasons, nullable reputation evidence and pending diffs. Quarantine clones are shallow with Git hooks/global config disabled; symlinks and submodules are rejected. Ingestion reads separate snapshots with read-only files, verifying all pinned bytes again before ingestion or scoring. Historical snapshots remain after removal for existing project receipts.

Hub cards and AST maps use `.clipkit/source-hub/knowledge`, overlaid on the existing owner catalog. The same ingestion, YAML card, AST parser, source-hash checks and retrieval ladder are reused. Original snapshots and local generated cards are not shipped as application dependencies. A per-source namespace prevents collisions with prior owner assets. Missing legacy metadata reads unknown; original project files are not migrated on load.

## CLI and approvals

Use `pnpm ck` in this checkout, or installed `ck`:

```sh
ck sources list --offset 0
ck sources add https://github.com/owner/repo
ck sources vet SOURCE_ID
ck sources license SOURCE_ID --offset 0
ck sources score SOURCE_ID
ck sources lock
ck sources update SOURCE_ID                 # fetch and preview only
ck sources update SOURCE_ID --apply --reason 'I reviewed the changed license and static flags'
ck sources override SOURCE_ID --reason 'I reviewed the listed flags and approve static indexing only'
ck sources disable SOURCE_ID
ck sources enable SOURCE_ID
ck sources remove SOURCE_ID                 # warns if used
ck sources remove SOURCE_ID --apply
ck sources discover 'kinetic typography Remotion'
ck sources gaps
ck collections list
ck collections enable Transitions
ck collections disable Starter
ck assets search 'calm transition' --license-mode commercial-safe --min-grade B --collection Transitions
ck license report
```

Mutating operations return a job; poll `ck job JOB_ID` (or MCP job_status) until it finishes. Source/collection changes require human ownership. Agent actors may inspect/discover/gap-check, but cannot add, update, override or remove sources. A vet **review** or **reject** cannot ingest without an explicit owner reason of at least twelve characters. An override never changes the recorded vet result, invents a license, loosens project license mode, or permits code execution. Missing original license files still prevent code ingestion.

Discovery uses GitHub metadata, installs nothing, and returns provisional review status. It does not label an unseen original LICENSE safe. Stars/last push/fork metadata are fetched values; contributor counts of 100 or more remain unknown rather than pretending one API page is the total. Failed metadata lookups remain nullable. API credentials, when used, come from the local environment and are never stored in receipts.

## License and collection behavior

New briefs default to **commercial-safe**. Brief → More options exposes licenseMode; changes use the normal impact/apply contract. Legacy projects without a brief/mode retain `any` for compatibility, visibly labeled in Sources. `personal` allows commercial-restricted material for owner review but still excludes unknown, copyleft, account-gated and proprietary reference assets. `any` explicitly permits viewing all classes; it does not grant legal rights. Search flags may tighten an any-mode query; they cannot loosen an existing safer project mode.

Commercial-safe search and placement/derivation/export block unknown, non-commercial, Commons Clause, copyleft, account-gated and proprietary-reference material. Per-asset nearest original license evidence is recorded where present. Missing/conflicting root evidence remains unknown. SPDX identifiers are recorded only when recognized; classifiers are conservative technical labels, not legal certification. Read the original text using Sources → Read original license.

Export reports retain every active receipt's source URL/ID/commit, SPDX, class and attribution. ZIP exports copy original pinned license files as well. Unknown-rights imported media blocks safe export; Library → About → **I created and control this media** is an explicit owner declaration for original work, not a way to relicense another person's content. Recorded/generated voice rights likewise are not assumed. An explicit `any` mode remains available for projects whose rights are handled separately. Exporting an SRT does not bypass the project's rights gate.

Collections are Starter, Text, Backgrounds, Transitions, Product demo, Social, Cinematic, Audio and Look packs. Starter has at most twelve external assets across the Hub (at most eight per source); enable additional collections to widen supply. Unverified sources start disabled and quarantined; enabling requires a passing vet or explicit owner review. Disabled sources/collections disappear from eligible searches. Grade C is hidden unless minimum grade C is requested and the relevant collection is enabled. Source and collection filters and used-in-project badges extend the existing Library.

## Quality and determinism

Cards store per-asset source pins, SPDX/license evidence, determinism, A/B/C grade, sub-scores, collections, attribution and vettedAt. Scores are static estimates: frame-driven code ranks above wall-clock code; typed controls and instrumentability help; WebGL/dependency complexity increases cost. Visual score **0 means unrendered**, not a measured aesthetic failure. Owner notes are preserved. Source pin metadata updates do not reparse unchanged ASTs; changed code does. Updates show new/modified/removed asset files separately from license changes and do not apply automatically.

A static **native** classification is not execution approval. Newly indexed upstream cards remain **runtime reference-only**. Read the ladder and implement the technique natively; copied original code cannot run through derive/instantiate. Arbitrary upstream previews and automatic wrapper certification wait for a real network-less sandbox with no project secrets. This is the explicit U3-7 cut line, not an assertion that a normal browser is a secure sandbox.

`FrameClockAdapter` and `seekFrame` provide a host-reviewed seekable WAAPI/GSAP contract: pause, then set time from frame/fps; never patch global Date/rAF. `verifyFrameDeterminism` compares exact bytes from repeated and reverse-order frames supplied by a trusted renderer. Real local Canvas pixel fixtures pass for frame-driven output and fail for clock/random-driven output. No third-party source has been certified by those fixtures. Failed/unproven sources retain reference-only classification/runtime; the Hub never silently enables them.

## Original Tier 1 audit

See SOURCE_HUB_EVIDENCE.json for the exact checked commits, original license paths/hashes and fetched GitHub facts. `pnpm exec tsx scripts/source-hub-evidence.ts` checks the four asset-library seeds into an isolated local audit workspace without executing them; `scripts/source-hub-promote-audit.ts` copies new quarantine receipts into this Studio without overwriting existing pins. It does not ingest flagged sources or alter the saved film.

The checked Remocn, onda and snapcn snapshots contain MIT license files, with install-hook/dependency/AI-text review flags. They remain quarantined: the owner directed that unverified sources stay quarantined. No override was recorded and no public-source code was ingested. The checked reactvideoeditor/remotion-templates snapshot has no original license file, so README claims do not establish redistribution rights and its code is not ingested. Remaining seeds are registrations only, not verified licenses, star counts or promises that a repository exists. Remotion runtime and agent-skill/reference sources are not executable asset pools.

P2 arbitrary sandbox previews, broad Tier 2/3 ingestion, portable-media catalog expansion and registry account integrations are deferred. Existing LUT/GLSL adapter limitations remain. The Source Hub is usable with vetted local/static sources now; live public-source ingestion follows the same owner-review gate.
