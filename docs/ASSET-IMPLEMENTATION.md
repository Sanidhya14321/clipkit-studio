# Asset-system implementation

Updated 27 October 2026, local release 0.1.1. The supplied asset prompt was used as a design reference; its embedded instructions and verification claims were not treated as authorization or accepted as facts.

Implemented the part that improves the existing product immediately:

- **25 compact cards:** ten native clips, ten regenerated previews, four existing binaries and one icon dependency. Each describes best uses, avoid rules, source/author/license, provenance and actual test status.
- **Findable in all three interfaces:** an indexed Markdown menu, CLI/MCP `assets_list` and `asset_card`, and a keyboard-accessible **Usage & provenance** dialog in the editor. Reading is available while the edit lease is locked.
- **An enforced agent contract:** new graphic placements require a current card receipt in the agent's lease, including direct JSON patches. A receipt cannot carry over to the next turn or survive a changed card. Ordinary property edits are kept compact.
- **Checks and CI:** required cards, schema pointers, unique IDs, license allowlist/conflicts, binary hashes, per-kind budgets, dependency references, stale render proofs and generated inventory. Unreferenced items are flagged conservatively; no automatic deletion occurs.
- **Deterministic evidence:** every native clip rendered at start/middle/end twice, with matching PNG hashes. The full 30-second asset reel includes a real pinned Kenney click and was exported twice at 360×640; decoded video frames and audio match. Container metadata is excluded from that comparison.

Verification: **96 standard tests passed**, with the two opt-in checks executed in the separate 13-test asset/render suite (**98 unique Vitest tests** across runs). All **five browser tests** passed. Typecheck, build and isolated install/card-tools/custom-pack/portrait-and-landscape export checks passed. The installed dependency audit reports no vulnerabilities and all inspected Zod copies are patched.

A real Codex (`gpt-6.1-sol`, medium) turn read the asset card and added exactly one requested callout at two seconds, preserving the existing title and resolving the comment. It took 29.1 seconds. Estimated ClipKit request/tool payload was 2,237 tokens; reported model input was 178,641 tokens, including client context (162,688 cached), with 275 output tokens. These are different measurements, not a claimed model-bill saving.

The source inventory contains **1,376,531 bytes of carded public payload**. One explicit exception remains: the original full Inter variable TTF exceeds the smaller new font limit. Its exact checksum/size is retained for glyph coverage and compatibility and is reported by the checker. All other current binary budgets pass; there are no unreferenced registered ingredients.

Native motion code and thumbnails are **agent-authored**, so this is **not strict human-made-only compliance**. External author attribution records source evidence, not proof of the entire creation process. External publisher claims remain unverified in the UI; merely supplying a card cannot fabricate successful local tests. No new unreviewed visual repository, 3D engine, Lottie file or GPU simulation was installed. The broader one-week visual roadmap and automatic CC-BY credit scenes remain separate work. No second-machine render check is claimed.

Read [the menu](../packages/assets/cards/_INDEX.md), [usage and maintenance rules](ASSET-SYSTEM.md) and [machine-readable evidence](asset-verification.json). The package is generated at `artifacts/clipkit-studio-local-0.1.1.tgz`; actual render/video evidence remains in the ignored local `.clipkit/asset-verification/` directory.
