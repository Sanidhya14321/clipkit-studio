# ClipKit source pack

The ten catalog clips are native, frame-driven React components. They contain editable text, colors, sizes, positions, opacity and reveal timing; there are no flattened template videos. Inter and Space Grotesk are used for typography, a selected illlustrations SVG appears in the illustration card, and Lucide supplies the native callout icons. All ten clip implementations, including their layout, browser framing and frame-driven motion, were written originally for ClipKit. No template source code was adapted or copied from RenderComp or any other reference repository. Licensed upstream material is limited to the bundled fonts, illustration and sound, plus installed Lucide icons.

`public/vendor/manifest.json` records exact upstream commits, original URLs, byte counts and SHA256 hashes for every fetched file and its unmodified license. `pnpm assets:fetch` fetches that same pinned selection without cloning repositories. The default bundled sound is a Kenney UI click, suitable for an imported audio item; it is not automatically added to every clip. Upstream licensing must be checked before changing this selection.

| Source | Use and license |
| --- | --- |
| [RenderComp/free-remotion-templates](https://github.com/RenderComp/free-remotion-templates) | MIT. Template source reference; no files copied. |
| [tabler/tabler-icons](https://github.com/tabler/tabler-icons/blob/main/LICENSE) | MIT. Verified alternative icon source; no icon files copied. |
| [lucide-icons/lucide](https://github.com/lucide-icons/lucide/blob/main/LICENSE) | ISC, with MIT notice for listed Feather-derived icons. Installed `lucide-react` supplies icons; retain both notices. |
| [rsms/inter](https://github.com/rsms/inter/blob/master/LICENSE.txt) | SIL OFL 1.1. Pinned variable TTF and original license bundled. |
| [floriankarsten/space-grotesk](https://github.com/floriankarsten/space-grotesk/blob/master/OFL.txt) | SIL OFL 1.1. Pinned variable TTF and original license bundled. |
| [realvjy/illlustrations](https://github.com/realvjy/illlustrations/blob/develop/LICENSE) | MIT. One pinned SVG and original license bundled. |
| [boringdesigners/boring-avatars](https://github.com/boringdesigners/boring-avatars/blob/master/LICENSE) | MIT. Candidate vector/avatar source; no files copied. |
| [marvelapp/devices.css](https://github.com/marvelapp/devices.css/blob/master/LICENSE) | MIT. Candidate device-frame source; no files copied. |
| [jdecked/twemoji](https://github.com/jdecked/twemoji/blob/main/LICENSE-GRAPHICS) | Graphics CC BY 4.0; code MIT. Candidate only; no graphics copied. Future graphics use needs visible attribution. |
| [useAnimations/react-useanimations](https://github.com/useAnimations/react-useanimations/blob/main/LICENSE) | MIT. Candidate icon motion source; no files copied. |
| [Calinou/kenney-ui-audio](https://github.com/Calinou/kenney-ui-audio/blob/master/LICENSE.txt) | CC0. One pinned WAV and original license bundled. Source author Kenney Vleugels / Kenney.nl. |
| [facebookincubator/Meta-open-sound](https://github.com/facebookincubator/Meta-open-sound) | Per collection: instrument collections CC0; Oculus Audio Pack CC BY 4.0. Candidate only; no files copied. |

`public/vendor/source-audit.json` records twelve verified source repositories with pinned commits, license URLs and SHA256 hashes. Unmodified license evidence is retained under `public/vendor/references/`, including the separate Meta Oculus collection license. This is an audit of licensed candidates; only the five marked bundled sources supply current assets.

Excluded candidate: [reactvideoeditor/remotion-templates](https://github.com/reactvideoeditor/remotion-templates) is public, but its license endpoint returned 404. Its suggested MIT license was not established, so no files were used and it is excluded from the twelve verified sources.

Do not describe reference-only repositories as bundled assets. In particular, Meta collection licenses and Twemoji code/graphics licenses are distinct. User footage remains subject to the user's provenance and permissions.

Each export writes a separate immutable `.credits.json` containing project revision/hash, user media provenance, and the bundled manifest next to its MP4. A unique suffix prevents exports from overwriting earlier output. The actual source license texts ship in `public/vendor/` and should accompany redistributed source packs.

Verification: `CK_RENDER_TEST=1 pnpm test -- tests/assets.test.ts` renders a selected PNG from every native clip twice, once with preview props and once with export props, and compares SHA256 values. A second pass changes the accent color and requires a different PNG hash, guarding against a blank or ignored-props false pass. Frames and machine-readable evidence are saved under `.clipkit/render-consistency/`. To verify installed runtime sources and assets after building, run `CK_DISTRIBUTION=artifacts/package pnpm exec tsx scripts/render-consistency.ts`.

## Native pack build and actual previews

`packages/assets/native.pack.json` is the starter pack manifest. It references the ten trusted native clip IDs, their licenses, default props, editable field descriptors and full schema locations. `buildAssets(root, outputDir?)` in `packages/assets/src/build.ts` builds metadata from the compiled registry. `root` is the installation directory containing `public/previews`; output defaults to `public/packs/native`. The builder validates unique clip IDs, Zod defaults, the required editable properties and field types, and each PNG's signature, hash and matching default-prop hash before writing any metadata.

The output includes a compact `catalog.jsonl`, `categories.json`, `pack.json` and one full props schema per clip under `schemas/`. Run `pnpm exec tsx scripts/build-assets.ts [output-directory]` from this repository. The CLI exposes the same builder through `ck assets build`.

Catalog `previewPath` values point to actual 240 × 427 PNG thumbnails under `public/previews/`. These are scaled from the native composition's verified default-prop frames, preserving the real font, illustration and layout. `public/previews/manifest.json` records original render hashes, preview hashes, sampled frames and default-prop hashes. `scripts/generate-previews.ts` regenerates them from matching render-consistency fixtures and refuses stale defaults; it does not invent pattern cards or use image generation.

External pack manifests are metadata only. Installing executable clip components requires explicit trusted compilation and registration in the native registry; there is no untrusted hot-loading, arbitrary pack script execution or sandbox claim. A JSON manifest alone cannot provide a new runnable component.
