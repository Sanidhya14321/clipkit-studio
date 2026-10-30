# Update 6 audit (U6-0, 2026-10-30)

No code changed. Sources from `sources.yaml`, cards from `assets/catalog.jsonl`. No license was read from an original source in this audit; `unknown` means not yet vetted, not that a license is bad.

## Sprint precondition

Met: canvas size, stage enforcement, export gates, loudness and skill commands are committed, with 375 tests passing on ffmpeg 6.0 (1 skipped). Not rerun on 8.0.1. The engine decision is still pending, so rule 0.4 applies: build for the adapter, test on Remotion.

## Summary

- 63 sources: 2 vetted, 61 unverified. 61 have license class unknown.
- 62 cards, all from rendercomp-free: 2 grade A, 60 grade B, all permissive.
- simple-icons is vetted with 0 cards. Every starter category except some kinetic and background candidates has no supply today.
- Grades do not yet mean determinism or owner review; no card has an owner review flag or preview.

## Blocked license and supply checks

| Item | Blocks | Needs |
| --- | --- | --- |
| Pixabay media | Music (15 tracks), sound effects, b-roll | Owner decision on Pixabay's content terms; site downloads, which vetting cannot read |
| Kenney audio | Sound kits (40 one-shots) | Confirm license and a download source; archive ingest is not built |
| sfx-elements | Sound kits | License read from the repo |
| IWLTBAP, Q-DDL, G'MIC LUTs | 6 LUTs | Attribution terms; LUT ingest is not built |
| gl-transitions, paper-shaders, color4bg | Transitions, backgrounds | Per-item or repo license read |
| remocn, onda, snapcn, remotion-templates, cliphouse, remotionui, remotion-bits | Direct-use components | Repo license read; site terms for cliphouse |
| gsap, motion and web libraries | U6-7 | GSAP's standard no-charge terms need owner review; Motion's npm license is MIT; frame-clock adapter not built in the product |
| Aceternity | U6-7 | Free parts only; terms must be read |
| Google Fonts, Fontsource, Tabler, Lucide | Fonts, icons | Per-family font licenses; tabler-icons exceeds quarantine size limits |
| Remotion company license | Every render | Owner confirms whether it applies (U5-1) |
| LYGIA, BBC Sound Effects | F2 fixtures | Not in `sources.yaml`; the update expects them as test fixtures |
| remotion-qa, audio-bed-check, tesseract.js | OCR checks in eval | License check |

## Dependencies on Update 5

U6-4 and later need work that does not exist yet: beat grid, beat sheet, energy gates, caption presets, kinetic blocks, playback check, face zones, drafts. Ingest of LUTs, sound effects and archives is also unbuilt. Update 6 sections 3A and 4 cannot pass without them, so U5-3 to U5-7 come first or must fold into U6. U5-2's spec question is also open.

## Conflicts to settle

- Fast-paced default against `assets/taste/pacing.md`, where landscape explainers default to level 2 and documentary to 1. The update wins unless the brief asks for slow; the file changes in U6-4.
- A new default bold-social look, music and sound kit cannot be delivered until supply exists; fallback 6.2 gives a labelled placeholder bed.
- F1 and the golden examples need the owner's review, and example 4 needs owner footage.
- U6-8 needs real agent runs, which have not worked in a live session yet (PROGRESS.md checklist).

## Source list

| Source | Kind | Tier | Status | License class | Enabled | Cards |
| --- | --- | --- | --- | --- | --- | --- |
| remocn | remotion-components | 1 | unverified | unknown | no | 0 |
| onda | remotion-components | 1 | unverified | unknown | no | 0 |
| snapcn | remotion-components | 1 | unverified | unknown | no | 0 |
| remotion-templates | remotion-components | 1 | unverified | unknown | no | 0 |
| remotion | runtime | 1 | unverified | unknown | no | 0 |
| react-bits | web-animation | 2 | unverified | unknown | no | 0 |
| magicui | web-animation | 2 | unverified | unknown | no | 0 |
| cult-ui | web-animation | 2 | unverified | unknown | no | 0 |
| motion-primitives | web-animation | 2 | unverified | unknown | no | 0 |
| animata | web-animation | 2 | unverified | unknown | no | 0 |
| aceternity-ui | web-animation | 2 | unverified | unknown | no | 0 |
| eldora-ui | web-animation | 2 | unverified | unknown | no | 0 |
| hover-dev | web-animation | 2 | unverified | unknown | no | 0 |
| syntax-ui | web-animation | 2 | unverified | unknown | no | 0 |
| ui-layout | web-animation | 2 | unverified | unknown | no | 0 |
| gl-transitions | shaders | 3 | unverified | unknown | no | 0 |
| paper-shaders | shaders | 3 | unverified | unknown | no | 0 |
| color4bg | shaders | 3 | unverified | unknown | no | 0 |
| 21st-dev | registry | 3 | unverified | unknown | no | 0 |
| remotion-skills | agent-guidance | 4 | unverified | unknown | no | 0 |
| remotion-video-toolkit | agent-guidance | 4 | unverified | unknown | no | 0 |
| remotion-superpowers | agent-guidance | 4 | unverified | unknown | no | 0 |
| video-shotcraft | agent-guidance | 4 | unverified | unknown | no | 0 |
| claude-video-toolkit | agent-guidance | 4 | unverified | unknown | no | 0 |
| claude-code-templates | agent-guidance | 4 | unverified | unknown | no | 0 |
| agent-skills | agent-guidance | 4 | unverified | unknown | no | 0 |
| sfx-elements | portable-media | 5 | unverified | unknown | no | 0 |
| google-fonts | portable-media | 5 | unverified | unknown | no | 0 |
| fontsource | portable-media | 5 | unverified | unknown | no | 0 |
| cube-lut | portable-media | 5 | unverified | unknown | no | 0 |
| lucide | portable-media | 5 | unverified | unknown | no | 0 |
| gmic-luts | portable-media | 5 | unverified | unknown | no | 0 |
| q-ddl-luts | portable-media | 5 | unverified | unknown | no | 0 |
| lottie | portable-media | 5 | unverified | unknown | no | 0 |
| phosphor-icons | portable-media | 5 | unverified | unknown | no | 0 |
| awesome-ae | reference-only | 6 | unverified | unknown | no | 0 |
| effectcraft | reference-only | 6 | unverified | unknown | no | 0 |
| mixkit | reference-only | 6 | unverified | unknown | no | 0 |
| remotionui | remotion-components | 1 | unverified | unknown | no | 0 |
| remotion-bits | remotion-components | 1 | unverified | unknown | no | 0 |
| rendercomp-free | remotion-components | 1 | vetted | permissive | yes | 62 |
| cliphouse | remotion-components | 1 | unverified | unknown | no | 0 |
| remotion-qa | quality-tool | tools | unverified | unknown | no | 0 |
| audio-bed-check | quality-tool | tools | unverified | unknown | no | 0 |
| gsap | animation-runtime | 2 | unverified | unknown | no | 0 |
| motion | animation-runtime | 2 | unverified | unknown | no | 0 |
| origin-ui | web-components | 2 | unverified | unknown | no | 0 |
| launch-ui | web-components | 2 | unverified | unknown | no | 0 |
| faster-whisper | pipeline-tool | tools | unverified | unknown | no | 0 |
| silero-vad | pipeline-tool | tools | unverified | unknown | no | 0 |
| scenedetect | pipeline-tool | tools | unverified | unknown | no | 0 |
| demucs | pipeline-tool | tools | unverified | unknown | no | 0 |
| librosa | pipeline-tool | tools | unverified | unknown | no | 0 |
| pyloudnorm | quality-tool | tools | unverified | unknown | no | 0 |
| tesseractjs | evaluator-tool | tools | unverified | unknown | no | 0 |
| wavesurfer-js | studio-lib | tools | unverified | unknown | no | 0 |
| opentype-js | studio-lib | tools | unverified | unknown | no | 0 |
| culori | studio-lib | tools | unverified | unknown | no | 0 |
| kenney-audio | sfx-pack | 5 | unverified | unknown | no | 0 |
| iwltbap-luts | lut-pack | 5 | unverified | unknown | no | 0 |
| tabler-icons | icon-pack | 5 | unverified | unknown | no | 0 |
| simple-icons | icon-pack | 5 | vetted | permissive | yes | 0 |
| pixabay-media | media-site | 5 | unverified | unknown | no | 0 |
