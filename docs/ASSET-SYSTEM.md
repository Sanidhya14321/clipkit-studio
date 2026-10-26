# Asset cards and enforcement

This adapts the supplied asset design to ClipKit's existing monorepo. Stable clip IDs, the central validated registry and original public URLs remain compatible with existing projects. It does not relocate user footage, introduce a second renderer registry, or install the prompt's entire one-week GPU/3D/library wish list.

- Native card and schema pointers: `packages/assets/cards/<kind>/<id>/{card.md,schema.ts}`.
- Menu: `packages/assets/cards/_INDEX.md`; auto inventory is regenerated, never hand-edited.
- Shipped cards: `public/packs/native/cards/` and `public/packs/native/asset-cards.json`.
- Loose binaries/previews: sibling `<filename>.md` cards beside existing files.
- Machine evidence: `public/packs/native/determinism.json`.
- Known scene references: `packages/assets/usage.json`.

Cards contain real usage guidance, avoid rules, source/author/license, provenance status, dependency IDs, schema pointer, and explicit test status. They stay within 45 lines and 12KB. YAML aliases and duplicate keys are rejected. The default menu lists clips; request a resource kind such as `font`, `audio` or `image` for loose assets. Read a compact filtered page with `ck assets list`, then `ck assets card ID`. The equivalent MCP tools are `assets_list` and `asset_card`. App-run agent leases must read a current card before adding a new clip, including direct JSON patches; existing-layer property edits do not force unnecessary card context.

Native clips and their generated thumbnails are honestly marked agent-authored. This does not satisfy a strict human-made-only rule for native motion; human-authored replacements would require a separately curated library. Existing external sources are human-attributed from original license/author records and immutable source URLs; this is not a forensic guarantee. Nothing is relabeled "handmade" to pass a checklist. External packs without a reviewed card receive an explicit unverified fallback card; their code still requires the existing human trust flow.

Both source checks and pack installation enforce the asset license allowlist; supplied card/manifest license conflicts are rejected. This is not proof of ownership or blanket approval of every upstream file. The checker validates IDs, schemas, license allowlist, checksums, source paths, size caps, dependency references and test evidence. Missing cards and stale generated inventories fail. Unused assets are warned about and flagged as candidates when their card file is at least 48 hours old; this age heuristic is not continuous usage telemetry. No file is deleted. The full asset smoke places the Kenney click once at a UI beat. In a product scene: import the local WAV via Media, place it at an actual UI click, trim and lower its volume; do not add it to all scenes automatically.

`public/vendor/inter.ttf` is the one explicit, checksum-bound legacy size exception. Retaining the full variable font preserves existing glyph coverage and projects. New font files must meet the normal caps. The checker reports this exception rather than pretending all assets satisfy the prompt's smaller font budget.

```sh
pnpm assets:check             # validate and refresh generated inventory
pnpm assets:check --check     # CI: fail if generated files are stale
pnpm assets:verify            # render native start/middle/end twice, regenerate previews/evidence
pnpm assets:verify --full     # also export the complete 42-second smoke (three seconds per clip) twice, compare decoded frames/audio
pnpm ck assets list --kind text
pnpm ck assets card kinetic-title
```

Do not hand-mark a card render-verified. The verifier writes that status only after successful renders; changes to composition/defaults/framework versions invalidate old evidence. Same-machine evidence is not a cross-machine guarantee. The optional full smoke contains synthetic native motion, no human footage or cloned speech.

New repositories from the attachment remain research candidates until their exact files, licenses, authorship evidence and deterministic behavior have been checked. Repository popularity, a named author, and a license identifier alone do not establish permission for every binary in a repository. Runtime-generated user recordings and narration stay in the private `.clipkit/` project registry, not public source folders.
