# Asset use audit

2026-10-30. Audited on `codex/ui-pro`, using the real Rust server and a temporary project. This checks current implementation against spec 07; it does not approve or execute third-party source code.

## Findings

| Path | Result | Evidence and practical meaning |
| --- | --- | --- |
| Original recipe → project → preview | PASS | 10 bundled recipes use 8 registered native clips. Recipe application records `recipe:<id>` in `assetsUsed` and `original` provenance on layers. Real browser tests apply recipes, edit their content, and inspect the resulting preview. |
| Imported image → recipe source slot → preview → render worker | PASS | Fixed source-kind selection, React event capture, authenticated image MIME/URLs, native media rendering and the Rust render job's image map. The `product-reveal` workflow checks a real imported PNG's natural width (1920), layer provenance, and a storyboard rendered by the Node worker. See `ui-workflow-evidence/real-recipe-asset.png` and `real-final-five.txt`. |
| Video import → proxy/source preview → timeline → MP4 | PASS | Real synthetic video is imported, placed by source range, verified, exported and probed for duration and a nonempty file. Imported videos can also fill native recipe media panels; those panels play muted, independently of the footage/audio mix. |
| Search/catalog inventory | PARTIAL | 62 cards/catalog entries, all `rendercomp-free` and `remotion-clip`. Search works through the real CLI. The scene-library UI intentionally exposes the 10 executable original recipes; it is not a browser for all external cards. |
| Source vetting | PARTIAL | 63 sources are recorded; 2 are enabled/vetted (`rendercomp-free`, `simple-icons`), 61 remain unverified/disabled. A vetted source does not imply its assets were ingested: no icon cards are present. |
| External derive → executable component | GAP | A real `assets derive` copies 3 files and records `rendercomp-free/glitch-text`, but leaves `project.derived` empty and places no layer. Neither studio nor worker imports a registry of these derived files. Copying/adapting source is not a successful render. The UI now blocks MP4 export for unknown/derived component layers and explains why. CLI/backend enforcement remains a separate gap. |
| Attribution/license tracking | PARTIAL | The real derived asset report resolves MIT/permissive, permits commercial-safe use, and separately describes the Remotion runtime license. `assetsUsed` is updated at derivation, even before a component is placed. Export report generation and license checks depend on locating the asset hub; when `Hub::locate` fails the export response can omit the report. This is not a universal attribution guarantee. |
| Asset previews and other asset kinds | GAP | No `preview.webp` files. 11 catalog kinds have no cards, including fonts, icons, sound effects, LUTs, shaders and Lottie. No sound kit or LUT library can be claimed available from this catalog. Font names/look tokens do not prove that matching font binaries are installed; browser/system fallback remains possible. |
| Quarantine boundary | PASS for this work | No quarantine files were modified or imported by studio/worker. The external derive audit copied files only; it did not run the derived component or source install scripts. Existing Rust asset safety/license tests also passed in the full suite. |

## Reproducible evidence

All paths below are in `docs/updates/ui-workflow-evidence/`:

- `assets-inventory.json`: metadata-only counts of cards, recipes, looks, taste and previews.
- `assets-sources.json`, `assets-gaps.json`, `assets-search.json`: actual CLI source status, missing-kind report and search response.
- `external-derive-audit.json`: temporary project, derived filenames, registered/placed counts, provenance and license report. No external component execution.
- `real-final-five.txt`: repeated real-server browser workflows, including native image loading through the render worker and MP4 export.
- `just-check.txt`, `just-test.txt`: Rust, hook and frontend checks; no live paid providers or owner media used.

## Follow-up ownership

External components need a reviewed registry/adapter, deterministic frame-clock proof, derived type-check/first-render proof and server-side export rejection when unsupported. They must not be enabled merely by importing quarantine into the bundle. Attribution should fail explicitly or resolve a packaged hub when it is unavailable. Asset previews, font files, icons, LUTs and sound ingestion remain unfinished backend/asset work; coordinate these with the other agent rather than advertising them as usable UI features.

Additional license validation gap: `ck-assets/src/report.rs` classifies any asset ID without `/` as original, in addition to explicit `recipe:`/`clip:` IDs. That convention does not verify arbitrary names against an original-asset registry. It should be tightened alongside backend asset registration; no license is inferred from those names in this audit.
