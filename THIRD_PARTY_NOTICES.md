# Third-party notices

ClipKit 0.2.0 ships original native components and reviewed upstream adaptations. The first fourteen native templates are agent-authored ClipKit code. The new library includes eighteen adapted RenderComp components, six reviewed frame-driven adaptations of selected browser/vector techniques, and eight MIT-licensed useAnimations JSON animations. Do not describe these as certified human-made artwork or claim all templates are original ClipKit designs.

Exact adopted files, pinned commits, original and retained checksums, authorship and modifications appear in `packages/assets/vendor/manifest.json`. Original notices ship beside those files. Media records with per-file license, attribution, source URL and modifications appear in `public/assets/manifest.json`. Earlier font/illustration/sound originals remain in `public/vendor/manifest.json`. Reference-only candidates in the historical source audit are not additional shipped assets.

| Source | Adopted use and license |
| --- | --- |
| RenderComp/free-remotion-templates | Eighteen selected TSX templates. MIT. Adapted font and logical-canvas integration are recorded per file. |
| juliangarnier/anime | Retained selected HTML/SVG example; frame-driven React adaptation. MIT. |
| ibelick/motion-primitives | Retained text examples; frame-driven adaptations. MIT. |
| boringdesigners/boring-avatars | Beam/Bauhaus vector avatar components. MIT. |
| useAnimations/react-useanimations | Eight selected animation JSON files. MIT. |
| tabler/tabler-icons | Three SVG symbols. MIT. Color replacement recorded. |
| jdecked/twemoji | Three reaction SVGs. Graphics CC BY 4.0; code license is distinct. Attribution retained per item and in export credits. |
| realvjy/illlustrations | Two selected SVG illustrations. MIT. |
| Calinou/kenney-ui-audio | Four distinct sound effects by Kenney Vleugels / Kenney.nl. CC0. |
| rsms/inter | Pinned variable font. SIL OFL 1.1. |
| floriankarsten/space-grotesk | Pinned headline font. SIL OFL 1.1. |
| lucide-icons/lucide | Installed icons. ISC plus inherited MIT notice for applicable Feather icons. |
| Wikimedia Commons | Eight photos and six video excerpts, individually verified CC0 or CC BY 4.0. Per-file author/license/source and trim/transcode modifications are in the media manifest. |

The distribution retains notices under `public/vendor`, `public/assets/references` and `runtime/packages/assets/vendor`. Exports produce immutable `.credits.json` sidecars alongside MP4 files. Include the required attribution when redistributing CC BY artwork or excerpts; users remain responsible for their own imported media rights.

RVE was not vendored because complete license evidence was unavailable. React Bits is excluded under its redistribution restriction. Candidate licenses such as Meta collection-specific licenses do not apply to unrelated files. Dependency packages retain their own notices; Remotion framework licensing remains separate from template permissions.

### YAML parser — ISC

`yaml` 2.9.1, by Eemeli Aro, parses bounded descriptive card metadata. npm metadata and [the versioned license](https://github.com/eemeli/yaml/blob/v2.9.1/LICENSE) were checked. Its original notice remains in the installed package. It is infrastructure, not a visual asset.

### Remotion framework — separate Remotion License

The [current license](https://github.com/remotion-dev/remotion/blob/main/LICENSE.md) provides a free tier for eligible individuals, small organizations with up to three employees, non-profits and evaluation use. The [official pricing page](https://www.remotion.dev/docs/license/pricing), checked 27 October 2026, lists video-automation/editor products under the company automation offering when a company license is required, currently $0.01/render with a $100/month minimum. The owner must check eligibility and current terms before commercial deployment; ClipKit asset MIT notices do not relicense the engine.

No additional third-party animation library, shader, 3D model or Lottie file was adopted solely on the attachment's claims. Candidate repositories require file-specific review before use. No CC-BY artwork is newly bundled by this change; a credits-scene implementation is therefore not represented as completed or tested here.
