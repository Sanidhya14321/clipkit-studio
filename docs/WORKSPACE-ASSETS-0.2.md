# ClipKit Studio 0.2.0: workspace and asset workflow

The release supports find → preview → place → edit → undo → agent collaboration → MP4 export. Existing timeline IDs, source media, comments, history and ownership remain compatible.

## Workspace

The charcoal workstation has a 44px toolbar, Library, Program Monitor, Properties/Comments/Voice/History/Activity and Timeline. Sidebars and timeline resize with pointer or keyboard separators. Sizes, collapsed state and grid/list preference persist locally; Reset Workspace restores defaults. Under 1280px one sidebar opens at a time; under 1024px it becomes a keyboard-accessible drawer. Each panel has its own scroll body; timeline labels remain sticky. Activity holds job progress, retry and downloads.

## Inventory

| Ready timeline type | Count |
| --- | ---: |
| Editable templates: original 14 plus 24 reviewed adaptations | 38 |
| Frame-driven Lottie animations | 8 |
| SVG illustrations and stickers | 8 |
| Photos | 8 |
| Video excerpts | 6 |
| Sounds | 4 |
| **Total** | **72** |

The native renderer catalog contains 46 templates/animations. Two bundled fonts, thumbnail cards, licenses, schemas and retained source files are resources, not additional timeline assets. Project imports and custom packs can increase the count. The initial media selection is under 60MB, with the previously documented Inter font exception retained.

## Operations

`library_list`, `library_inspect`, `library_import`, `library_rescan` and `library_place` are available through server operations, CLI and MCP. List filters include type, origin, folder and readiness. Inspection returns capabilities, provenance, schema and usage. Import registers/processes a resource without placing it; place creates an independent instance at a compatible track/time and participates in leases and undo. Existing footage_add keeps its legacy append behavior.

The documented project `assets/` directory is scanned without following symlinks. Original vendor paths are preserved. Files are ready, processing, needs adaptation or blocked with recovery text. Missing registered originals retain copied project media; instance source replacement supports recovery. Repeated placement reuses the registered source, preserving originals while edits affect instances. Screenshot uploads bind the chosen image without inserting stray footage.

## Format boundaries

Images/video share preview/export transforms: position, scale, rotation, fit, opacity and normalized edge crops. Videos/audio retain trim timing; audio offers volume and fades. SVG is bounded and rejects script/event/external-resource content. Eight reviewed Lottie files render by project frame, with palette/speed/size and optional labels. Template data JSON must satisfy a declared binding; arbitrary JSON is never assumed to be animation.

Reviewed TSX/JSX packs compile only after explicit trust and content validation. Changed code invalidates the trust receipt. Raw HTML/CSS/JS is retained for adaptation, never silently executed. Selected HTML and browser-animation examples have been rewritten into deterministic frame-driven React templates. Controls appear only for real bindings. Photographs and encoded video do not gain editable internal text.

## Provenance

`packages/assets/vendor/manifest.json` records pinned upstream source, original checksum, retained checksum and modifications for adopted code and raw source. `public/assets/manifest.json` records each media file, exact checksum, author, license, source URL and modifications. `public/vendor/manifest.json` retains the earlier fonts/illustration/click selection. Original notices accompany these folders. Source and preview cards group resources by asset; export credits preserve media attribution.

Adopted sources include RenderComp, anime.js, motion-primitives, boring-avatars, useAnimations, Tabler, Twemoji, illlustrations, Kenney UI audio, Inter, Space Grotesk and installed Lucide. Wikimedia Commons contributes eight individually licensed CC0/CC-BY 4.0 photos and six individually licensed video excerpts. These are selected files, not wholesale repository clones. RVE and instavar were not needed for the adopted allocation. React Bits is excluded.

## Remaining external acceptance

Local export, cached/recorded media, code adapters and Codex collaboration are independently testable. Fish transcription/TTS/cloning and Claude live editing still need valid external credentials and a separate live rehearsal. This release does not certify those outstanding checks or completion of every original PRD stretch item.
