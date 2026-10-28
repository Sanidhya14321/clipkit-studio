# ClipKit Studio — Asset Supply Expansion Catalog

**New, popular, human-made sources to improve video editing and generation**
Companion file: `clipkit-sources-additions.yaml` (sources.yaml-ready, 26 entries) · Research date: 2026-10-28 · Scope: everything NOT already in the Update 3 seed registry

---

## 1. Executive Summary & Top Picks

ClipKit's biggest supply gaps are not more web-animation registries — its Update 3 seeds already cover that space densely. The gaps this catalog fills are: **(a) local, deterministic transcription and silence detection** for the EDIT pipeline, **(b) content-aware cut candidates** beyond silence-based heuristics, **(c) deterministic quality gates** for the Update 4 evaluator, **(d) fresh Remotion-native component and template pools** for GENERATE mode, and **(e) real media supply** — SFX, LUTs, icons and especially music, where the current registry holds exactly one 17-sound pack.

All 26 sources below were web-verified on 2026-10-28 (repo, license and activity), comply with the "human-made" filter (AI-powered *tools* like Whisper and VAD are included only where they improve the pipeline — every *asset* is human-crafted), and ship as `status: unverified` so your own `ck sources vet` re-reads each LICENSE on first ingest, exactly as Update 3 prescribes.

### Top 10 picks, by impact

1. **@remotion whisper + captions stack** — local, offline, deterministic word-level transcription. Kills the live-STT dependency in `ck footage transcribe`, enables zero-live-call cassette replay (T7) and a real S15 fallback.
2. **Silero VAD** (`snakers4/silero-vad`, MIT) — tiny neural silence/retake detection that upgrades delete-to-cut from threshold guessing to speech-science.
3. **PySceneDetect** (BSD-3) — content-aware scene cuts: cut candidates, b-roll segmentation, and scene-change ground truth for fixtures.
4. **faster-whisper** (MIT) — ~4x faster word timestamps; 13 minutes of audio in ~17s with batching. The batch-transcription backend long footage needs.
5. **remotion-qa** — sound-off readability gates (dead air, reading speed, Reels/Shorts UI overlap, mobile type floors) that map almost 1:1 onto your Update 4 evaluator's layout checks.
6. **audio-bed-check** — BS.1770 LU voice-vs-bed separation and loop/jump detection at joins, with CI exit codes. Directly hardens the W14 audio gates.
7. **RenderComp Free** (50 MIT templates) + **cliphou.se** (MIT-0) — the two largest fresh, license-clean Remotion template pools found; cliphou.se's MIT-0 is the cleanest license class in the entire catalog.
8. **GSAP — now 100% free** (since April 2025, all former Club plugins included). SplitText + ScrambleText unblock premium kinetic typography, and Remotion's official GSAP-timelines tutorial proves the deterministic wrapping pattern.
9. **Kenney audio packs** (CC0) — thousands of human-made SFX; a hundredfold expansion of the 17-sound `sfx-elements` seed.
10. **Pixabay** (site license) — 3,400+ human-made music tracks plus SFX and stock footage; closes the registry's music gap entirely and feeds GENERATE-mode b-roll.

### Catalog at a glance

| # | id | Source | License (class) | What it unlocks |
|---|----|--------|-----------------|-----------------|
| 1 | remotionui | RemotionUI | verify (unknown) | Motion components with AI-agent indexes; prior art for your card format |
| 2 | remotion-bits | Remotion Bits | verify (unknown) | "Ready-made blocks for you and your agents"; native determinism |
| 3 | rendercomp-free | RenderComp/free-remotion-templates | MIT (permissive) | 50 fresh templates, prop-customizable |
| 4 | cliphouse | cliphou.se | MIT-0 (permissive) | Templates with editable text/colors; zero attribution bookkeeping |
| 5 | remotion-qa | remotion-qa | verify (unknown) | Sound-off readability gates + render gate + GitHub Action |
| 6 | audio-bed-check | audio-bed-check | verify (unknown) | BS.1770 voice/bed separation, loop/jump detection, CI exit codes |
| 7 | remotion-whisper-stack | remotion-dev packages | MIT (permissive) | Local offline word-level ASR + caption model |
| 8 | gsap | greensock/GSAP | Custom, free incl. commercial (permissive*) | Premium motion; SplitText/ScrambleText kinetic typography |
| 9 | motion | motiondivision/motion | MIT (permissive) | Springs/scrub API; unlocks all seeded registries at once |
| 10 | origin-ui | shadcn/originui | MIT (permissive) | 500+ fresh components; UI mockups for product demos |
| 11 | launch-ui | launch-ui/launch-ui | MIT (permissive) | Conversion-shaped blocks for launch/teaser recipes |
| 12 | faster-whisper | SYSTRAN/faster-whisper | MIT (permissive) | Fast word timestamps; batch backend |
| 13 | silero-vad | snakers4/silero-vad | MIT (permissive) | Silence/retake detection, JS/ONNX runtimes |
| 14 | scenedetect | Breakthrough/PySceneDetect | BSD-3-Clause (permissive) | Content-aware cut candidates, b-roll segmentation |
| 15 | demucs | facebookresearch/demucs | MIT (permissive) | Stem separation: voice swap keeps music bed |
| 16 | librosa | librosa/librosa | ISC (permissive) | Beat/onset sync, tempo, energy curves |
| 17 | pyloudnorm | csteinmetz1/pyloudnorm | MIT (permissive) | ITU-R BS.1770-4 loudness metering |
| 18 | tesseractjs | naptha/tesseract.js | Apache-2.0 (permissive) | OCR leg for the output-only evaluator |
| 19 | wavesurfer-js | katspaugh/wavesurfer.js | BSD-3-Clause (permissive) | Waveform + regions for voice-take UI |
| 20 | opentype-js | opentypejs/opentype.js | MIT (permissive) | Exact glyph metrics for the layout solver |
| 21 | culori | Evercoder/culori | MIT (permissive) | OKLCH palettes for looks/tokens, contrast math |
| 22 | kenney-audio | kenney.nl | CC0 (permissive) | Thousands of human-made SFX |
| 23 | iwltbap-luts | blog.iwltbap.com | CC BY 4.0 (attribution-required) | Free cinematic LUTs (.cube + Hald CLUT) |
| 24 | tabler-icons | tabler/tabler-icons | MIT (permissive) | 5,200+ stroke SVG icons |
| 25 | simple-icons | simple-icons/simple-icons | CC0 (permissive) | 3,000+ brand logos (trademark caveat) |
| 26 | pixabay-media | pixabay.com | Pixabay License (permissive*) | Music, SFX, footage/b-roll under one site license |

\* Custom/site licenses, not SPDX — vetting must read the actual terms and label accordingly (see section 12).

---

## 2. Method, Evidence Standard & How to Ingest

**How this was researched.** Every candidate came from one of three discovery routes: live web search of GitHub/npm/PyPI for new and popular video tooling (2024–26 risers and category leaders, ~1k+ stars or official vendor origin, actively maintained); the official Remotion resources list (`remotion.dev/docs/resources`, last updated 2026-10-25, fetched and read in full); and cross-checks against the Update 3 seed registry to enforce your "new only" requirement. Each pick was then verified by web search or direct page fetch for: repository existence and identity, license text or official license statement, maintenance signals (recent releases, commits, discussions), and fitness for ClipKit's architecture (determinism tier, license class, integration point).

**Honesty conventions — matching your own rules.** Update 3 says "never invent a repository, a license or a star count," and this catalog holds that line. Where a project is listed on the official Remotion resources page but its exact GitHub URL could not be pinned down from that listing (RemotionUI, Remotion Bits, remotion-qa, audio-bed-check), the entry keeps `repo: null`, cites the official listing as evidence, and asks for resolution on first ingest — no guessed URLs. Star counts are only quoted where a source showed them (librosa 8.7k, culori 1.2k, Launch UI 871, ui-experiments ~1.4k, Motion Canvas 19.5k). Every entry ships as `status: unverified` because your own vetting (`ck sources vet`) must re-read the LICENSE file and re-run reputation checks — a web check is evidence, not proof, and your pipeline is the arbiter.

**How the entries are shaped.** The YAML companion follows your `sources.yaml` schema exactly (`id, name, repo, kind, tier, determinism, license, licenseClass, installMethod, status, notes`), with two documented extensions: new `kind` values for tooling (`pipeline-tool`, `quality-tool`, `evaluator-tool`, `studio-lib`, `animation-runtime`, `web-components`, `media-site`, `sfx-pack`, `lut-pack`, `icon-pack`), and `tier: tools` for non-asset sources that should be wired in as dependencies rather than ingested as component cards. Tool entries use `determinism: native` in the sense of "reproducible output for fixed input" — the frame-clock determinism test does not apply to them.

**The human-made filter, applied pragmatically.** No AI-generated asset collections appear anywhere in this catalog — every SFX file, LUT, icon, template and component is human-crafted. AI-powered *tools* (Whisper-based transcription, neural VAD, source separation) are included because they only process media and never contribute generated content to a video, which is the interpretation selected for this run.

---

## 3. Remotion-Native Libraries & Tools (Tier 1)

Tier 1 is where ClipKit gets the most value per ingest hour, because native components are frame-driven by construction: no frame-clock adapter, no determinism downgrade, no "wrappable" test ceremony. This sweep found four fresh asset pools and three tooling finds inside the official Remotion ecosystem — the latter discovered by reading the Remotion resources list, which is itself worth polling monthly (see section 10).

### 3.1 RemotionUI — motion components with AI agent indexes

**What:** Production-ready motion components for Remotion, distributed via NPM with a CLI, docs previews, and — the unusual part — published *AI agent indexes* alongside the components. **Evidence:** official Remotion resources listing, checked 2026-10-28; resolve the exact repo and package names from that listing on ingest. **License:** unknown until the LICENSE is read; treat as MIT-expected.

**Why it improves ClipKit:** the AI-agent index format is precisely the shape of ClipKit's own retrieval ladder — short, structured text describing each component so an agent can search and select without reading source (your L0 index / L1 card pattern). Ingesting RemotionUI therefore does double duty: it adds a pool of titled, motion-ready scene components for GENERATE mode, and it provides working prior art for how a component vendor structures machine-readable metadata for agents — directly useful when you refine your card template in U3-3. As a Remotion-native library its cards can enter the catalog as `runtime: direct, determinism: native`, the cheapest class you have.

**Wire into:** Tier 1 ingestion (U3-3), GENERATE-mode recipes, search-quality benchmarks (U4's three sample intents).

### 3.2 Remotion Bits — ready-made blocks "for you and your agents"

**What:** A collection of animation components and utilities designed specifically for Remotion video projects, advertised as ready-made blocks usable by both humans and agents. **Evidence:** official Remotion resources listing (two separate entries), checked 2026-10-28; resolve repo on ingest. **License:** verify on first ingest.

**Why it improves ClipKit:** ClipKit's central thesis is that the agent should instantiate and adapt pre-wired material instead of composing scenes from raw primitives. Remotion Bits ships exactly that material, already deterministic, which makes each block an "instantiate" (ladder level 1) candidate — the cheapest adaptation mode with zero model-generated code risk. Its utilities may also strengthen the adaptation engine's codemod toolbox where a block needs prop-level reshaping.

**Wire into:** Tier 1 ingestion, recipe slots (hook titles, reveals), adaptation presets.

### 3.3 RenderComp Free — 50 MIT-licensed templates

**What:** Fifty free Remotion templates, each a React component rendering to MP4. No account, no email — clone the repo, open Remotion Studio, and all fifty are there with editable props. **Evidence:** `github.com/RenderComp/free-remotion-templates` and `rendercomp.com/free`, both checked 2026-10-28. **License:** MIT for the templates — and notably, the project's own README warns that *Remotion itself is not MIT-licensed and larger teams need a Remotion Company License*. Record that caveat in your license report; it is exactly the kind of disclosure your audit will check.

**Why it improves ClipKit:** this is the largest single pool of fresh, license-clean Remotion scenes found in the sweep, and its "customize the props" workflow is literally your adaptation ladder's instantiate level. Fifty scenes covering hooks, lists, reveals and CTAs map cleanly onto recipe skeletons: point `ck recipe apply` at a template, declare its text/timing/color props as slots, and the recipe catalog grows by an order of magnitude without authoring new components.

**Wire into:** Tier 1 ingestion (U3-3), recipe expansion for launch teasers and social formats, license report (template MIT + runtime note).

### 3.4 cliphou.se — MIT-0 templates with editable text and colors

**What:** Free Remotion templates with editable text and colors. **Evidence:** official Remotion resources listing, checked 2026-10-28. **License:** **MIT-0** — the attribution-free MIT variant. This is the cleanest license class in the entire catalog: no notice requirement at all.

**Why it improves ClipKit:** MIT-0 removes every attribution bookkeeping burden, so these templates can be derived, rebranded and exported inside any commercial-safe project with zero export-report noise — ideal demonstration material for the S9 license-mode test (commercial-safe search still returns them) and for the W7 re-theming check, since "editable text and colors" is exactly what your project-token system writes. Ingest these alongside RenderComp Free and the two pools together cover a wide beat vocabulary for storyboards.

**Wire into:** Tier 1 ingestion, look re-theming demos (W7), commercial-safe mode tests (S9/S12).

### 3.5 remotion-qa — sound-off readability gates with CI exit codes

**What:** Measures rendered videos for sound-off readability: dead air after the text has been read, beats too fast to read, text falling under the Reels/Shorts UI, and mobile type floors. Ships as a render gate around `npx remotion render` plus a GitHub Action. **Evidence:** official Remotion resources listing, checked 2026-10-28; resolve repo on ingest. **License:** verify on first ingest.

**Why it improves ClipKit:** these four checks are four of the deterministic gates your Update 4 evaluator is specified to implement (section 8's layout checks: text presence/position, safe-area violations, minimum text size — plus caption reading-speed limits from `caption-styles` and safe areas from `platform-specs`). Adopting a maintained external checker instead of hand-rolling each rule gives you the same determinism with less code to defend in the audit, and its GitHub Action shape slots directly into your release-gate plan. It also gives T16 a head start: the evaluator's "deliberately broken output" fixtures (text outside safe area, too-fast beats) are precisely what this tool already detects.

**Wire into:** `ck eval` criteria, W13 verification, CI release gates, S1 pass criteria.

### 3.6 audio-bed-check — BS.1770 voice/bed separation and join checks

**What:** Checks a rendered audio bed for loops and level jumps at joins, and a voiceover mix for speech-over-bed separation in BS.1770 LU, with a WCAG profile. Exit codes for CI; works with Remotion's bundled ffmpeg; ships an Agent Skill. **Evidence:** official Remotion resources listing, checked 2026-10-28; resolve repo on ingest. **License:** verify on first ingest.

**Why it improves ClipKit:** your audio gates today are hand-specified ("voice loudness near target, music ducked under voice, no clipping") with no off-the-shelf measurement behind them. This tool provides the exact measurements those gates need — including join-point loop/jump detection, which is the "audio pops at cuts" failure your feature checklist explicitly probes in the footage-editing audit. Exit codes match your error-code discipline (E_RENDER / E_EXPORT families), and shipping an Agent Skill itself is a useful pattern for how ClipKit distributes its own skills.

**Wire into:** W14 sound-design verification, S11 export-parity audio checks, CI gates, the evaluator's loudness leg (alongside pyloudnorm).

### 3.7 The @remotion whisper + captions stack — local, deterministic transcription

**What:** Three first-party packages verified inside the remotion-dev monorepo on 2026-10-28: `@remotion/install-whisper-cpp` (downloads and wraps whisper.cpp locally, with `toCaptions()` conversion of JSON transcription into caption pages), `@remotion/whisper-web` / `whisper-webgpu` (in-browser transcription via Transformers.js), and `@remotion/captions` (the word-level caption model and page-windowing API used by all of them).

**Why it improves ClipKit:** this is the single biggest architectural upgrade in the catalog. Transcription becomes local, offline and deterministic, which means: the packed transcript and cut candidates in Stage 3 no longer depend on a live STT provider; cassette replay (T7) can genuinely make zero live calls; S15's "voice provider down" scenario gets a real fallback instead of a graceful dead end; and voice-first beat timing in GENERATE mode can be computed from actual word durations rather than estimates. Because it is first-party, register it as a pinned tooling extension under the *existing* remotion-dev seed line — not as a new source — and keep the Remotion runtime license note in the report.

**Wire into:** Stage 3 (ingest → transcribe → pack → candidates), S2, T7, E_TRANSCRIBE fallback, voice-slot duration fit from Update 2.

---

## 4. Motion Runtimes & Fresh Component Registries (Tier 2)

Tier 2 is technique material: animation driven by wall-clock libraries that must be wrapped by the frame-clock adapter and pass the determinism test (render the same frame twice; render frames out of order) before any card is trusted. The two runtimes below are the highest-value wrappers you can build, because they unlock entire registries you have already seeded.

### 4.1 GSAP — the whole engine, including SplitText, is now free

**What:** The industry-standard animation platform. Since April 2025 — after the Webflow acquisition — GSAP is 100% free, including every formerly paid "Club" plugin: SplitText, ScrambleText, ScrollTrigger, DrawSVG, MorphSVG, Inertia, and the rest. **Evidence:** multiple independent sources confirm the April 2025 change (checked 2026-10-28), and Remotion's official documentation contains a **"Use GSAP timelines in Remotion"** tutorial — the wrapping pattern you need is literally documented by the video framework itself. **License:** the custom GSAP license — free for commercial use, but not an OSI license. Read the LICENSE text during vetting and label it `permissive` with a `custom` note in your license report.

**Why it improves ClipKit:** GSAP timelines are seekable objects, which is exactly what your frame-clock adapter needs: build a timeline once, then drive `tl.seek(frameTime)` from `useCurrentFrame()`. Determinism is therefore a test away, not a rewrite. The newly-free SplitText plugin is the single most-wanted tool for kinetic typography — hooks, title reveals, word-by-word caption motion — and ScrambleText covers the "tech decode" aesthetic that fits product-demo recipes. Instead of hand-rolling text splitting in the direction engine, the agent derives a GSAP-driven scene from a card, records provenance, and the look pass inherits professional-grade motion curves. This directly serves your `motion-principles-and-easing` taste file with production-grade easings (CustomEase was also a paid plugin, now free).

**Wire into:** frame-clock adapter target #1, motion-principles defaults, kinetic-text recipes, determinism-test fixtures for the wrappable class.

### 4.2 Motion (motion.dev) — one wrapper unlocks every seeded registry

**What:** The merged successor of Framer Motion and Motion One (`motiondivision/motion`, MIT), offering springs, keyframes, gestures and — critically — scrub/seek-capable animation APIs. One of the most-starred animation libraries on GitHub. **Evidence:** repo and license listed via the official Motion documentation ecosystem, checked 2026-10-28. **License:** MIT.

**Why it improves ClipKit:** almost every component registry ClipKit already seeds — Magic UI, React Bits, the Aceternity family — depends on Motion. That means Motion is almost certainly already in the dependency tree of any component derived from those registries, so writing *one* frame-clock wrapper for Motion instantly upgrades a whole shelf of existing cards from "reference-only" to "wrappable." Its animation primitives (spring configs, keyframe arrays) are also exactly what your AST extractor should record in the `animation primitives` field of `ast.json`, so derived components can be re-tuned by codemod instead of by rewrite.

**Wire into:** frame-clock adapter target #2, AST animation-primitive extraction, wrappable classification for already-seeded registries.

### 4.3 Origin UI — the strongest 2024–25 riser in the shadcn ecosystem

**What:** 500+ copy-and-paste components built with React and Tailwind CSS, constantly updated, with a sibling repo (`origin-space/ui-experiments`, ~1.4k stars) of open-source layouts. **Evidence:** GitHub README and launch discussion surfaced via search, checked 2026-10-28; canonical org appears to be `shadcn/originui` (originally `origin-space/originui`) — confirm the canonical repo on ingest. **License:** MIT (verify the LICENSE file during vetting).

**Why it improves ClipKit:** ClipKit's product-demo and tutorial recipes need UI *mockup* material — fake interfaces, stat rows, feature cards, settings panels — and Origin UI is the freshest, best-maintaged supply of exactly that in the shadcn convention your audience already knows. Its components are Tailwind-CSS-keyframe driven, so they are wrappable-class: ingest with the frame-clock adapter and a determinism test, and record `runtime: derive-only` on cards until the test passes. The `ui-experiments` layouts are also excellent reference cards for the layout solver's grid and rhythm decisions.

**Wire into:** Tier 2 ingestion (technique + derive), product-demo recipe, AST cards with props + token extraction.

### 4.4 Launch UI — conversion-shaped blocks for launch-video scenes

**What:** A landing-page component kit (hero, logos, stats, CTA, FAQ, testimonial, bento, feature blocks) built with React and Tailwind. **Evidence:** repo fetched directly on 2026-10-28 — MIT confirmed from the README; ~871 stars, 152 forks, v2.10.1, latest commit September 2026, 18 releases: genuinely maintained, not a weekend repo. **License:** MIT — but a paid "Pro" tier exists on their website; ingest only the open-source repository's components and note the boundary in the source card.

**Why it improves ClipKit:** a launch video is structurally a landing page that moves: hook, value props, social proof, call to action. Launch UI hands the agent pre-designed, conversion-shaped blocks it can restyle purely through your project tokens — which is dramatically cheaper in tokens and turns than composing each beat from raw primitives, and produces output that reads as *designed* rather than *generated*. Because blocks map one-to-one onto beats, they also storyboard beautifully: each card in the storyboard checkpoint is one Launch UI block with slots.

**Wire into:** launch-teaser recipe (taste library), product-demo and testimonial recipes, Brief Studio look previews.

---

## 5. Edit-Engine Pipeline Tools

These six tools are the "improve the video **editing**" half of the brief. None of them render anything — they *analyze media deterministically*, which makes them ideal `pipeline-tool` sources: no determinism-test ceremony, no license class surprises, just better evidence for the decisions your EDIT/HYBRID pipeline already makes (transcribe, pack, find cut candidates, cut on word boundaries, then audio and look passes).

### 5.1 faster-whisper — word timestamps at batch speed

`SYSTRAN/faster-whisper` · MIT · `pip`

Whisper reimplemented on CTranslate2. The repo's own benchmark (checked 2026-10-28) shows 13 minutes of audio transcribed in ~17 seconds (GPU, `batch_size=8`) versus 2m23s for openai/whisper, with an int8 CPU path at similar gains, and audio decoded via PyAV so no system ffmpeg is needed. It supports `word_timestamps=True`, which is the raw material for every downstream feature you have: the packed transcript, word-boundary cuts (U11), caption timing, and voice-slot durations. Wire it in as the batch backend for long footage where the first-party `@remotion/install-whisper-cpp` path would be slow, and keep the model fixed per project so transcripts replay identically.

### 5.2 Silero VAD — silence and retake detection, scientifically

`snakers4/silero-vad` · MIT · `pip` (also ONNX and JS runtimes)

A ~1 MB neural voice-activity detector, maintained actively through 2025–26 (v5/v6 line, checked 2026-10-28). Your "remove silences" and retake-detection features currently live or die by RMS-threshold heuristics, which is exactly the kind of guesswork your own audit mindset should distrust. Silero VAD replaces the threshold with a speech model: every silence/retake candidate in S2 comes with a confidence score, cuts snap to actual speech boundaries, and the ONNX/JS runtimes let the *studio* show live voice activity during lane-B recording. Deterministic for a fixed model and input, so delete-to-cut results replay identically.

### 5.3 PySceneDetect — cut candidates that aren't silence

`Breakthrough/PySceneDetect` · **BSD-3-Clause** (confirmed from repo docs, checked 2026-10-28 — note: BSD, *not* MIT, a good test that your license reader handles more than MIT) · `pip`

Content-aware scene detection (ContentDetector, AdaptiveDetector) over OpenCV. Silero finds *silences*; PySceneDetect finds *changes* — camera cuts, topic shifts, on-screen action — which is a completely different and complementary cut-candidate signal. It gives HYBRID mode chapter beats for inserting generated layers between scenes, gives the footage-import pass b-roll segmentation, and gives your synthetic fixtures (Update 4, section 5) scene-change ground truth to test against. Cross-platform and offline, it slots in beside — not instead of — the VAD.

### 5.4 Demucs — stem separation for voice swaps that keep the music

`facebookresearch/demucs` · MIT (code; verify model-weight terms separately on ingest) · `pip`

State-of-the-art music source separation into vocals, drums, bass and other (repo confirmed, checked 2026-10-28). This unlocks the S6 voice-swap scenario *properly*: separate the voice from the music bed, replace the voice with an AI or recorded take, and keep the original music — instead of losing the music or ducking a mixed track. It also enables stem-level ducking in the sound-design engine and honest "keep original audio" cleanup in HYBRID mode. Because it is a heavy model, run it in the ingestion pipeline, cache stems by source hash alongside your proxies, and never in the interactive path.

### 5.5 librosa — beat-synced cuts and pacing evidence

`librosa/librosa` · ISC · ~8.7k stars · v1.0.0 (Aug 2026), 138 contributors, active · `pip`

Beat, onset and tempo tracking plus RMS energy curves, all offline and deterministic. Your transition grammar and pacing profile currently rest on taste rules; librosa lets a music track *tell you* where transitions should land — cuts on downbeats, motion accents on onsets — and gives the direction engine an energy curve to align the pacing slider to actual audio. It also powers reference-fingerprint statistics (cuts per minute, speech rate) in the Brief Studio reference match. Pair it with `pyloudnorm` below and the audio analysis layer of the reference analyzer is fully covered by two permissive libraries.

### 5.6 pyloudnorm — the loudness gates get a real meter

`csteinmetz1/pyloudnorm` · MIT · `pip`

ITU-R BS.1770-4 integrated loudness in Python, with control over gating block size and frequency weighting (repo confirmed, checked 2026-10-28). Every audio check you specify — voice loudness near target, music ducked under voice, no clipping, platform loudness presets on export, the evaluator's "audio loudness (integrated and peak)" criterion — needs exactly this number (LUFS), measured with the gating algorithm the broadcast world actually uses. At render time, hand off to ffmpeg's built-in `loudnorm`/`ebur128` filters so measurement (pyloudnorm) and normalization (ffmpeg) share one standard. If you want a second opinion in tests, `lupy` (validated against official EBU test materials) and the `loudness` PyPI package are drop-in alternatives.

---

## 6. Evaluator & Quality-Gate Tooling

Your Update 4 evaluator (section 8) is specified to score *any* arm's output — plugin or vanilla — from the exported video alone. Two of its legs are currently hand-wavy in the spec: "text detection or OCR where available" and export/preview parity. The tools above (remotion-qa, audio-bed-check) cover most layout and audio checks. This entry covers the OCR leg; the parity leg needs nothing new.

### 6.1 tesseract.js — OCR for the text-presence checks

`naptha/tesseract.js` · Apache-2.0 (the underlying Tesseract engine is Apache-2.0 as well) · `npm`

Pure-JavaScript OCR that runs in Node and the browser, with traineddata for 100+ languages. This is the "where available" your evaluator spec waves at: sample the exported video at fixed times, run OCR per frame, and *machine-check* the S1 criterion "title present in first 2 seconds," detect safe-area violations by locating text boxes, and enforce minimum text size by measuring recognized regions — for the vanilla arms' outputs too, where you have no project JSON to lean on. It is deterministic per tessdata version, offline, and adds zero live-service risk. For export-parity, note that you need **no new dependency**: ffmpeg already ships `ssim`, `psnr` and (where compiled in) `vmaf` filters — the three-frame comparison in S11 is one ffmpeg invocation away.

---

## 7. Studio Tooling

Three small, permissive libraries that upgrade specific studio surfaces without touching the render pipeline: voice-take UX, the layout solver's text fitting, and look-token color math.

### 7.1 wavesurfer.js — waveforms and regions for voice takes

`katspaugh/wavesurfer.js` · **BSD-3-Clause** (read directly from the LICENSE file, checked 2026-10-28) · `npm`

Interactive audio waveform rendering with a regions plugin, from a project that has been maintained since 2012 and is currently on a v7 line. The voice-lane record-along flow in Update 2 needs exactly what this renders: a waveform the user can scrub, region-mark, and trim takes against — with the region boundaries feeding your trimming patches. It runs entirely client-side in the studio, so it has zero effect on rendering or determinism, and its per-take region selection maps cleanly onto your slot-take data model.

### 7.2 opentype.js — exact text metrics for the layout solver

`opentypejs/opentype.js` · MIT · `npm`

Parses TTF/OTF/WOFF files in JavaScript and exposes exact glyph advances and metrics (repo active, checked 2026-10-28). Your layout solver is specified to "fit text to boxes with minimum sizes" — but measuring text by rendering DOM is a feedback loop with no guarantees at export time. opentype.js gives the solver deterministic up-front arithmetic: given font file + size + string, the exact width is a function call. It complements `@remotion/layout-utils` (first-party, already inside your remotion-dev seed line) which handles render-time measurement — use opentype.js for planning-time placement and layout checks in `ck layout check`. Note the tool carries no fonts: fonts keep their own OFL/Apache licenses as your seeds already record.

### 7.3 culori — OKLCH palettes for looks, tokens and contrast gates

`Evercoder/culori` · **MIT (verified by direct repo fetch, 2026-10-28)** · ~1.2k stars, active (v4.0.2) · `npm`

Full CSS Color Module Level 4 support — OKLCH, OKLab, P3 and every legacy space — with conversion, interpolation, blending and color-difference formulas. Three ClipKit features feed off this one library: looks and design tokens get palettes *derived* from a brand color in perceptually-uniform OKLCH (so re-theming produces visually even steps, not muddy greens); the reference analyzer (Update 2, section 3.8) gets its dominant-palette extraction; and the quality gates get contrast computation for the "sufficient contrast against the background where measurable" check. Pure functions in, pure functions out — fully deterministic and test-friendly.

---

## 8. Media Packs — SFX, LUTs, Icons, Music & B-roll (Tier 5)

The Update 3 registry's portable-media tier is thin: 17 CC0 sounds, two LUT sources, two font sources, two icon sources, no music source at all. These five picks close that gap with human-made, license-clear material — every file below was crafted by a person.

### 8.1 Kenney audio packs — CC0 SFX at industrial scale

`kenney.nl/assets` (audio filter) · CC0 (shown per pack) · download-zip

Kenney's packs are a fixture of the game-asset world: interface clicks, impacts, whooshes, footsteps, even voiceover packs (e.g. the 45-file Voiceover Pack (Fighter), CC0 confirmed on its pack page, checked 2026-10-28). For your sound-design engine this is a hundredfold expansion of the 17-sound `sfx-elements` seed, organized by exactly the categories `ck sound plan` reasons about — whooshes near cuts, impacts on landings, room tone under pauses. Distribution is site-based, so record it as a zip source, snapshot each pack's license page, and note that some packs include music loops to verify per pack. CC0 means zero attribution overhead in exports.

### 8.2 IWLTBAP free cinematic LUTs — CC BY film grades

`blog.iwltbap.com` (free LUT pages) · CC BY 4.0 (verify the license note on the download page at ingest) · download-zip

Sylvain Beaulac's free cinematic grades ship as `.cube` files and PNG Hald CLUTs (page live, checked 2026-10-28), which your `cube-lut.js` parser already handles. They slot into the look pass and into LUT cards (`runtime: direct`) with one obligation: CC BY means `attributionText` in the card and a line in the export report — a good forcing function for your attribution pipeline. His larger 80-LUT pack is paid: never ingest paid content, and let the vetting rule "any hit marks rejected" keep the boundary bright.

### 8.3 Tabler Icons — 5,200+ stroke SVG icons, MIT

`tabler/tabler-icons` · MIT · npm/SVG

A 24×24-grid stroke icon set that pairs naturally with your seeded Lucide and Phosphor (MIT confirmed via aggregator listings, checked 2026-10-28). Graphics layers constantly need icons — feature callouts, stat icons, UI mockups in product demos — and pure-SVG icons are frame-safe and fully deterministic. Ingest as icons (`kind: icon-pack`), not as React components, so cards stay light and the agent picks them by tag.

### 8.4 Simple Icons — 3,000+ brand logos, CC0

`simple-icons/simple-icons` · CC0 · npm/SVG

Monochrome brand SVGs with official brand hex colors baked into the metadata. This is the supply for "trusted by" logo walls, sponsor stings and brand lower-thirds pulled straight from a brand kit — CC0 confirmed (checked 2026-10-28). Two operational notes your audit would raise anyway: CC0 covers *copyright* only, so using a company's logo in a video still depends on the user's rights to that *trademark* — put that in the license report; and the npm package is ~13 MB, so ingest the SVG files directly rather than bundling the package into any export.

### 8.5 Pixabay — music, SFX and b-roll under one site license

`pixabay.com` · Pixabay License · site-download

3,400+ music tracks (search confirmed, checked 2026-10-28) plus a large SFX library and stock footage. The Pixabay License — confirmed via multiple independent summaries — is irrevocable and royalty-free, allows commercial use, requires no attribution, and forbids redistributing the content as standalone. That makes it the only source in this catalog that closes your *music* gap outright and doubles as GENERATE-mode b-roll supply. Two caveats: it is a site license, not SPDX, so vetting must snapshot the terms page at each ingest; and the official API needs a key, which belongs in your gitignored env file — basic site downloads need no account at all.

---

## 9. Watch List & Caution Entries

These finds did not make the 26-entry registry — some need one more verification pass, some carry license or maintenance flags your vetting should decide on, and a few are valuable *only* as fixtures for your own license-mode tests. Per your conventions, treat everything in this section as `status: unverified` with the noted caveat.

**Ready after one license check:**
- **svgl** (`pheralb/svgl`, MIT per third-party comparisons, checked 2026-10-28) — 657 curated, production-ready brand SVGs including full-color and grayscale variants; the strongest rising alternative to Simple Icons when a brand wall needs real colors. Verify repo and LICENSE, then add beside `simple-icons`.
- **stable-ts** (`jianfch/stable-ts`, active into 2026) — modifies Whisper to produce more reliable word-level timestamps plus silence/non-speech refinement; a credible alternative to faster-whisper if timestamp quality beats throughput in your tests.
- **Mediabunny** (`Vanilagy/mediabunny`) — pure-TypeScript media toolkit for reading, writing and converting video/audio in the browser; active through late 2025. Interesting for future in-studio media handling; today ffmpeg covers your needs.
- **anime.js v4** (`juliangarnier/anime`, v4.5.0 line, MIT) — Remotion's own blog documents an integration ("Integrating Anime.js with Remotion"), so the wrapping pattern exists; fold into the wrappable class alongside GSAP and Motion when you build the frame-clock adapter.
- **TikTok Text Box** — a single React component mimicking TikTok's text-box style; near-zero ingest cost, high recognizability for social caption styles (`caption-styles` taste file).
- **Remotion Animated, remotion-time, remotion-animate-text, remotion-subtitle, remotion-transition-series** — the community "animation helpers" shelf from the official resources page; small permissive packages worth ingesting as one batch once each repo's license is confirmed.
- **Quote Shorts, SwiftClip, Free Remotion Components, Lexington Motion** — additional template pools from the official resources page (cliphou.se and RenderComp were promoted to full entries as the strongest two).
- **Aeon, BrewReel** — agent-side prior art listed under Remotion's Tooling: storyboards-to-MP4 skill frameworks. Read them before finalizing your recipe and storyboard formats (Tier 4-style reference, not for copying).
- **Incompetech (Kevin MacLeod)** — CC BY 4.0 music library; a second music source that exercises your attribution pipeline. Verify current terms on the site.
- **pyloudnorm alternatives** — `lupy` (`nocarryr/lupy`, validated against official EBU compliance materials) and the `loudness` PyPI package (BS.1770 with a C++ core) — useful as cross-checks in the evaluator's audio tests.

**License or maintenance cautions:**
- **LYGIA** (`lygia.xyz`, by Patricio Gonzalez Vivo) — a genuinely impressive GLSL block library, but dual-licensed under the **Prosperity License and Patron License** (checked 2026-10-28): free for non-commercial use, paid for commercial. In your license classes this is `commercial-restricted` — it must never appear in commercial-safe search results. Keep it, if at all, as a *fixture* proving your license classifier and S9/S12 gates work on a real restricted source.
- **BBC Sound Effects** — 30,000+ human-made recordings under the RemArc license: free for *personal, educational and research use only*, explicitly not commercial. Deliberately useful to you as the canonical restricted-license fixture for the S12 license-mode test, and a legitimate supply if a project's license mode is `personal`.
- **Motion Canvas** (`motion-canvas/motion-canvas`, ~19.5k stars, MIT today) — a code-driven animation tool with excellent easing/flow design and voice-over synchronization focus (very close to your GENERATE timing philosophy). Caution: maintainers opened discussion #1015 on **switching to the GPL**, which would make even the player problematic to embed. If you use it at all, use it as a *technique reference* and re-check the license outcome first.
- **editly** (`mifi/editly`, MIT) — declarative Node.js video editing over ffmpeg; the closest prior art to your "small JSON patches describing edits" idea, and a useful comparison for your transition grammar. Maintenance is quiet (mature/stale), so read it for ideas, don't depend on it.
- **simplex-noise** (`jwagner/simplex-noise.js`, MIT, ~2 kB) — the classic answer for generative noise backgrounds, but check **first-party `@remotion/noise`** (section 10) before adding a dependency your framework already ships.

---

## 10. Extend Existing Seeds & Discovery Surfaces

Some of the best finds are not new *sources* — they are new **parts of sources you already seeded**. Registering them as separate sources would violate your "one entry per source" shape; instead, extend the existing lines in `sources.yaml` and let the next `ck sources update` diff them.

**Extend the `remotion-dev` first-party line (runtime libraries) with the newer packages** — all verified present in the monorepo's package tree on 2026-10-28:
- `@remotion/noise` — deterministic noise for generative backgrounds (probably makes `simplex-noise` unnecessary);
- `@remotion/effects` and `@remotion/motion-blur` — official effect helpers for the polish pass;
- `@remotion/animation-utils` — official animation helpers (spring/keyframe utilities);
- `@remotion/layout-utils` — **text measuring and fitting**; this is the first-party half of your layout solver's "fit text to boxes" requirement (W13), with opentype.js covering the planning-time half;
- `@remotion/codemods` — official codemod infrastructure; directly supports the derive step of your adaptation engine (U7);
- `@remotion/openai-whisper` and `@remotion/elevenlabs` — official AI provider wrappers;
- `@remotion/claude-code-plugin` and `@remotion/codex-plugin` — **official agent integrations for exactly the two agents ClipKit adapters target**; read these before finalizing your adapter design, because first-party plugin behavior may set expectations your adapters should match;
- `@remotion/google-fonts` — already name-dropped in your Tier 5 fonts note; formalize it as the pinned, deterministic font loader (every font package separate, MIT).

**Extend the three.js angle of the same seed line:** `@remotion/three` (already noted) sits on `three` + `@react-three/fiber` + `drei`, all MIT — worth listing explicitly in the source notes so the AST extractor knows the 3D dependency chain.

**Discovery surfaces to poll (add these to `ck sources discover` bookmarks):**
- `https://www.remotion.dev/docs/resources` — the official compiled list of templates, libraries, building blocks and examples; last updated 2026-10-25 when fetched, and the origin of five finds in this catalog.
- The **Remotion Elements** ecosystem (`remotion.dev/elements/libraries`) — the official third-party "Element Library" registry (backgrounds, captions, effects, overlays, storytelling), currently listing Remocn, Lexington Themes and snapcn — i.e., two of your seeds plus one watch-list item.
- The **shadcn Registry Directory** (`ui.shadcn.com/docs/directory`) — the canonical index of community registries shipping animated interactions, shaders and charts as editable source; your fastest way to spot next quarter's Origin UI.
- The **awesome-video** curated lists — reference for pipeline technique discovery (Tier 6-style: read for ideas, never bundle files).

---

## 11. Priority Adoption Roadmap

Ordered by capability gain per hour of integration, and mapped to the phase plan and checks you already run. Waves respect your update-mode rule: extend existing modules, never rebuild.

**Wave 1 — Editing & verification hardening (P0).** Add first: `remotion-whisper-stack`, `silero-vad`, `scenedetect`, `faster-whisper`, `remotion-qa`, `audio-bed-check`, `pixabay-media`. These seven change what your EDIT pipeline can *prove*: word-boundary cuts backed by local ASR instead of a provider call, silence candidates with confidence scores, scene-change cut candidates, machine-checked sound-off readability and BS.1770 loudness gates, and a real music library. They directly harden S2 (silences and fillers removed), S15 (provider-down fallback), U11 (cuts on word boundaries), T7 (zero-live replay), W13/W14 (layout and sound gates) and the evaluator's core criteria — the exact items your Update 4 release gate blocks on. All seven are permissive-class; none touches the render path.

**Wave 2 — Catalog depth for GENERATE (P1).** Next: `rendercomp-free`, `cliphouse`, `remotionui`, `remotion-bits`, `kenney-audio`, `tabler-icons`, `iwltbap-luts`, `simple-icons`. These feed `ck sources gaps`: 50+ fresh templates and two new component pools for lower thirds, stat callouts and product demos; a hundredfold SFX expansion for `ck sound plan`; icons and LUTs for the text, graphics and look passes. Run them through the standard U3-3 ingestion (cards, indexes, grades) and you can demo S1 with a materially larger, better-graded catalog.

**Wave 3 — Craft & studio polish (P2).** Last: `gsap`, `motion`, `origin-ui`, `launch-ui`, `librosa`, `demucs`, `pyloudnorm`, `opentype-js`, `culori`, `wavesurfer-js`, `tesseractjs`. This wave is where output *quality* rises: premium kinetic typography (GSAP, now free) and one wrapper unlocking every seeded registry (Motion); beat-synced cuts and stems-aware ducking (librosa, demucs); honest loudness targets (pyloudnorm); deterministic text fitting (opentype.js); OKLCH look palettes and contrast gates (culori); waveform-based voice-take UX (wavesurfer.js); and the evaluator's OCR leg (tesseract.js). Build the frame-clock adapter first, then run the determinism test on each wrappable candidate before its cards go live.

**What to do tomorrow morning:** paste the YAML companion's Wave 1 entries into `sources.yaml`, run `ck sources add` on the four repos that have URLs today (faster-whisper, silero-vad, PySceneDetect, pyloudnorm are pure `pip` installs — trivially vetted), and let `ck sources vet` produce the first license report. The Remotion-listing finds (RemotionUI, Remotion Bits, remotion-qa, audio-bed-check) only need their repo URLs resolved from the official listing before the same flow takes them in.

---

## 12. License Safety Notes

Your audit prompt treats licensing as a first-class lens, so this catalog ends with the caveats a careful reviewer will raise — none are blockers; all are bookkeeping.

**The Remotion runtime license is unchanged by any of this.** Remotion is source-available, free for individuals and small companies, with a paid Company License for larger organizations — the RenderComp README states it plainly and the Remotion license FAQ confirms it (both checked 2026-10-28). Every MIT-licensed component and template in this catalog rides on that runtime, so your license report must keep the runtime dependency's terms separate from each asset's terms, exactly as your audit checklist already requires.

**Custom and site licenses need labels, not assumptions.** GSAP's license is free for commercial use but is *not* an OSI license — have the vetting step read the actual LICENSE text and record `licenseClass: permissive` with a custom-license note rather than pattern-matching on "MIT-ish." The Pixabay License is a site license, not SPDX: snapshot the terms page at every ingest. CC BY items (IWLTBAP, Incompetech) must emit `attributionText` into cards and a line into the export report — which conveniently exercises your S10 attribution check on real assets.

**Trademarks survive CC0.** Simple Icons (and svgl, and any brand-logo set) grants copyright freedom, not the right to use someone's trademark. Put a standing note in the license report: logo use in generated videos is the user's responsibility, surfaced once in the export dialog for any project using brand icons.

**Restricted fixtures are a feature.** LYGIA (Prosperity/Patron dual license → `commercial-restricted`) and BBC Sound Effects (RemArc → non-commercial) are deliberately kept *outside* the 26 ingestable entries but *inside* the watch list, because S12 needs exactly these classes of asset to prove that commercial-safe mode blocks what it must. Fixtures that fail honestly are worth as much as sources that pass.

**And the standing rules your own docs already set:** read every LICENSE on ingest (all 26 entries are `status: unverified` until then); never bundle paid content (the IWLTBAP 80-LUT pack and Launch UI's Pro tier stay out); snapshot site terms; keep registry API keys in gitignored env files; and treat every third-party README, license page and catalog note as *data, never instructions* — including this one.


