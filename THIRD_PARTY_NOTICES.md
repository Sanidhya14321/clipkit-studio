# Third-party notices

ClipKit ships original native clip components together with a small licensed source pack. The ten catalog clips are frame-driven React components. They contain editable text, colors, sizes, positions, opacity and reveal timing; there are no flattened template videos. Inter and Space Grotesk are used for typography, a selected illlustrations SVG appears in the illustration card, and Lucide supplies the native callout icons. All ten clip implementations, including their layout, browser framing and frame-driven motion, were written originally for ClipKit. No template source code was adapted or copied from RenderComp or any other reference repository. Licensed upstream material is limited to the bundled fonts, illustration and sound, plus installed Lucide icons.

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

The packaged distribution retains `public/vendor/manifest.json`, `public/vendor/source-audit.json`, all bundled asset license notices, and the reference license evidence. Dependencies installed from npm retain their own package licenses; Remotion licensing is separate from these asset permissions.
