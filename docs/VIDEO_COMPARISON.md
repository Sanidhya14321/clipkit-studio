Local interactive viewer: http://127.0.0.1:4335/

Evidence: `.clipkit/video-comparison-20261028/` (ignored local outputs). `protocol.json` is the frozen protocol for the actual run; the current driver additionally requires regular-file exports after discovering lost namespace-temporary symlinks. This report covers real Codex generation with synthetic local media fixtures. Provider replay applies to optional external media adapters, not to Codex model inference.

# With and without ClipKit — measured video comparison

Same Codex client/model, 15 seconds, 1920×1080, 30 fps, silent editorial brief; three scheduled repetitions each. Exact facts, palette, font, copy, constraints and scripted feedback are in protocol.json.

| Arm | Repetition | Initial tokens | Initial seconds | Headline edit tokens | Edit seconds | Pacing edit tokens | Pacing seconds | Session outcome |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | --- |
| Without plugin | 1 | 145,812 | 111.41 | 165,533 | 48.33 | 185,008 | 70.01 | All rounds completed |
| With plugin | 1 | unmeasured | unmeasured | unmeasured | unmeasured | unmeasured | unmeasured | E_ENV: Selected model is at capacity. Please try a different model. |
| Without plugin | 2 | 449,338 | 209.99 | unmeasured | unmeasured | unmeasured | unmeasured | E_ENV: 2026-10-28T12:56:54.838153Z ERROR codex_core::session::session: failed to initialize thread persistence: thread-store internal error: failed to resume local thread recorder: Quota exceeded (os error 122)
2026-10-28T12:56:54.839382Z ERROR codex_core::session: Failed to create session: thread-store internal error: failed to resume local thread recorder: Quota exceeded (os error 122)
Error: thread/resume: thread/resume failed: error resuming thread: Fatal error: Failed to initialize session: thread-store internal error: failed to resume local thread recorder: Quota exceeded (os error 122) (code -32603)
 |
| With plugin | 2 | 1,414,427 | 331.04 | 597,647 | 104.97 | unmeasured | unmeasured | E_AGENT_TIMEOUT: Agent exceeded wall limit |
| Without plugin | 3 | 690,364 | 220.32 | unmeasured (counter reset) | 80.89 | unmeasured | unmeasured | E_ENV: 2026-10-28T13:01:57.786610Z ERROR codex_core::session::session: failed to initialize thread persistence: thread-store internal error: failed to resume local thread recorder: Quota exceeded (os error 122)
2026-10-28T13:01:57.786768Z ERROR codex_core::session: Failed to create session: thread-store internal error: failed to resume local thread recorder: Quota exceeded (os error 122)
Error: thread/resume: thread/resume failed: error resuming thread: Fatal error: Failed to initialize session: thread-store internal error: failed to resume local thread recorder: Quota exceeded (os error 122) (code -32603)
 |
| With plugin | 3 | 785,741 | 182.21 | unmeasured (counter reset) | 46.37 | unmeasured | unmeasured | E_ENV: Harness interrupted by local write quota failure (errno -122); final child exit and remaining token usage were not recorded. |

Token counts include cached input. See report.json / interactive report for cached, uncached and output breakdowns.

## Aggregate by reported turn (includes turns without retained video)

| Arm / stage | Measured n | Tokens median (range) | Seconds median (range) |
| --- | ---: | --- | --- |
| A0 / initial | 3 | 449,338 (145,812–690,364) | 209.99 (111.41–220.32) |
| A0 / headline fix | 2 | 165,533 (165,533–165,533) | 64.61 (48.33–80.89) |
| A0 / pacing | 1 | 185,008 (185,008–185,008) | 70.01 (70.01–70.01) |
| B / initial | 2 | 1,100,084 (785,741–1,414,427) | 256.62 (182.21–331.04) |
| B / headline fix | 2 | 597,647 (597,647–597,647) | 75.67 (46.37–104.97) |
| B / pacing | 0 | unmeasured (unmeasured–unmeasured) | unmeasured |

## Retained-video subset

| Arm / stage | Retained clips | Exact incremental token median | Step seconds median |
| --- | ---: | ---: | ---: |
| A0 / initial | 1 | 145,812 | 111.41 |
| A0 / headline fix | 1 | 165,533 | 48.33 |
| A0 / pacing | 1 | 185,008 | 70.01 |
| B / initial | 1 | 1,414,427 | 331.04 |
| B / headline fix | 1 | 597,647 | 104.97 |
| B / pacing | 0 | unmeasured | unmeasured |

## Visual and workflow inspection

The retained baseline is more typographically prominent and changes scenes more often; the retained plugin clip is more restrained and uses a simpler three-part structure. Both decode to 450 silent 1080p30 frames with no black frames. Neither is a universal visual winner. The plugin provides structured project/brief/scene provenance, but this short task incurred more tool traffic and reported tokens. Its initial CTA fades in after 12 s, leaving less than the requested final 3 s at full readability. A narrow saved-source headline edit does not establish identical downstream output: the plugin exports differ at the 12 s sample. No plugin pacing revision completed. These are assistant frame/source observations, not blind independent ratings.

### rep-1-A0, round 0

- **Copy and content:** All three requested phrases appear across fixed frame samples. The headline and support are split over two lines; extra editorial labels and a bridge sentence were added. Exact punctuation/glyph compliance was checked in retained source, not with formal OCR.
- **Opening hook:** The opening headline is visible at 0 s and within the requested first 2 s. Its larger two-line hierarchy gives it more visual weight in this sample.
- **Typography:** Retained Python source loads fixtures/brand-inter.ttf via Pillow. Initial headline is 176 px, support/CTA 160 px; sampled text is legible. Pixel metrics do not establish font preference.
- **Safe areas and layout:** Sampled headline/support/CTA are separated from the right-side shapes and inside the 8% text safe area. Persistent small editorial labels are less readable at thumbnail scale. No exhaustive all-frame text-bound certification.
- **Motion and transitions:** Drawn square, moving block, stacked frame positions and square-to-play morph. Horizontal wipes deliberately partially obscure text during transitions; readable time is shorter than nominal section length.
- **Story progression:** Four sections: opening 0–4.8 s, support 4.8–9.2 s, bridge 9.2–12 s, CTA 12–15 s (source timings).
- **Color and contrast:** Navy background, white primary text and mint accent match the requested palette; extra muted line colors also occur. Sampled high contrast primary type; no subjective numeric quality score.
- **CTA hold:** CTA text is present from 12.0 s through the end, with stable positioning. Decorative underline continues briefly; the text itself remains readable for the final 3 s.
- **Headline edit scope:** Initial checkpoint; see round 1 for edit evidence.
- **Pacing revision:** Initial balanced sequence. Only this arm completed the requested pacing feedback with an intact export.
- **Editability and provenance:** Standalone frame-driven Python/Pillow source and local font; editable as code. No ClipKit project/layer controls. Initial source was recovered from successful recorded literal file writes; original clip bytes were retained.

### rep-1-A0, round 1

- **Copy and content:** All three requested phrases appear across fixed frame samples. The headline and support are split over two lines; extra editorial labels and a bridge sentence were added. Exact punctuation/glyph compliance was checked in retained source, not with formal OCR.
- **Opening hook:** The opening headline is visible at 0 s and within the requested first 2 s. Its larger two-line hierarchy gives it more visual weight in this sample.
- **Typography:** Opening headline is now 64 px in both retained Python draw calls; support/CTA sizes remain 160 px. Keeping the original two-line positions leaves a visibly large inter-line gap.
- **Safe areas and layout:** Sampled headline/support/CTA are separated from the right-side shapes and inside the 8% text safe area. Persistent small editorial labels are less readable at thumbnail scale. No exhaustive all-frame text-bound certification.
- **Motion and transitions:** Drawn square, moving block, stacked frame positions and square-to-play morph. Horizontal wipes deliberately partially obscure text during transitions; readable time is shorter than nominal section length.
- **Story progression:** Four sections: opening 0–4.8 s, support 4.8–9.2 s, bridge 9.2–12 s, CTA 12–15 s (source timings).
- **Color and contrast:** Navy background, white primary text and mint accent match the requested palette; extra muted line colors also occur. Sampled high contrast primary type; no subjective numeric quality score.
- **CTA hold:** CTA text is present from 12.0 s through the end, with stable positioning. Decorative underline continues briefly; the text itself remains readable for the final 3 s.
- **Headline edit scope:** Recovered source diff changes the opening headline sizes and adds the 64 px font to the font cache. Other scene timings/layout values are preserved for this step; sampled downstream sections look consistent.
- **Pacing revision:** Initial balanced sequence. Only this arm completed the requested pacing feedback with an intact export.
- **Editability and provenance:** Standalone frame-driven Python/Pillow source and local font; editable as code. No ClipKit project/layer controls. Initial source was recovered from successful recorded literal file writes; original clip bytes were retained.

### rep-1-A0, round 2

- **Copy and content:** All three requested phrases appear across fixed frame samples. The headline and support are split over two lines; extra editorial labels and a bridge sentence were added. Exact punctuation/glyph compliance was checked in retained source, not with formal OCR.
- **Opening hook:** The opening headline is visible at 0 s and within the requested first 2 s. Its larger two-line hierarchy gives it more visual weight in this sample.
- **Typography:** Opening headline is now 64 px in both retained Python draw calls; support/CTA sizes remain 160 px. Keeping the original two-line positions leaves a visibly large inter-line gap.
- **Safe areas and layout:** Sampled headline/support/CTA are separated from the right-side shapes and inside the 8% text safe area. Persistent small editorial labels are less readable at thumbnail scale. No exhaustive all-frame text-bound certification.
- **Motion and transitions:** Drawn square, moving block, stacked frame positions and square-to-play morph. Horizontal wipes deliberately partially obscure text during transitions; readable time is shorter than nominal section length.
- **Story progression:** Source remaps the first 12 s to six beats (2.8, 2.8, 2.2, 1.2, 1.4, 1.6 s), then holds the CTA from 12–15 s. Opening/support/bridge repeat; this is faster but less linear.
- **Color and contrast:** Navy background, white primary text and mint accent match the requested palette; extra muted line colors also occur. Sampled high contrast primary type; no subjective numeric quality score.
- **CTA hold:** CTA text is present from 12.0 s through the end, with stable positioning. Decorative underline continues briefly; the text itself remains readable for the final 3 s.
- **Headline edit scope:** Recovered source diff changes the opening headline sizes and adds the 64 px font to the font cache. Other scene timings/layout values are preserved for this step; sampled downstream sections look consistent.
- **Pacing revision:** Completed. Mean RGB delta rises from 0.05016% to 0.09505% after the prior headline edit; sections change more frequently. Requested copy, 15 s duration and final 3 s CTA are retained. More motion does not prove better storytelling.
- **Editability and provenance:** Standalone frame-driven Python/Pillow source and local font; editable as code. No ClipKit project/layer controls. Initial source was recovered from successful recorded literal file writes; original clip bytes were retained.

### rep-2-B, round 0

- **Copy and content:** All three exact requested phrases appear in inspected source and readable frame samples. Single-line headline/support/CTA with small section labels; no extra bridge sentence.
- **Opening hook:** Opening headline is visible at 0 s. It occupies less screen area than the retained baseline; restrained hierarchy is an aesthetic tradeoff, not an automatic quality advantage.
- **Typography:** Scene.tsx uses BrandInter with embedded local font data; initial opening/CTA 118 px, support 112 px, weight 650 and -5 px letter spacing. Tight spacing is visible. No inference from the separate failed-run font 404s.
- **Safe areas and layout:** Sampled primary text fits inside the 8% safe region; decorative shapes sit below it. No obvious sampled type overlap or clipping. All-frame OCR/bounds verification remains unmeasured.
- **Motion and transitions:** Growing underline, rotating outline square, expanding circle, animated diagram and CTA arrow. More low-motion holding than the retained baseline by measured RGB delta; this is not a smoothness or quality score.
- **Story progression:** Three source sections at 0–5, 5–12 and 12–15 s. Supporting text and CTA enter with easing rather than hard cuts.
- **Color and contrast:** Requested navy/white/mint are used. Thin low-opacity rules and small section labels are subtler than primary type. Formal accessibility contrast across every frame was not measured.
- **CTA hold:** At exactly 12.0 s the initial export sample has no visible CTA text; at 13.5 and 14.8 s it is visible. Source opacity eases from zero over 24 frames, so a full 3 s readable CTA is not demonstrated.
- **Headline edit scope:** Initial checkpoint; see round 1. Source snapshots describe the retained code state, which is not alone proof of which revision the renderer encoded.
- **Pacing revision:** No completed pacing-feedback export. Timeout means a head-to-head final pacing comparison is unavailable.
- **Editability and provenance:** ClipKit project, confirmed brief, storyboard, versioned TSX scene, bound headline/support/CTA/color props and recorded agent critic evidence. One generated scene item contains the whole 15 s composition; primary text is not represented as three independent timeline text clips. Font data dominates source snapshot bytes; it is not creative code complexity.

### rep-2-B, round 1

- **Copy and content:** All three exact requested phrases appear in inspected source and readable frame samples. Single-line headline/support/CTA with small section labels; no extra bridge sentence.
- **Opening hook:** Opening headline is visible at 0 s. It occupies less screen area than the retained baseline; restrained hierarchy is an aesthetic tradeoff, not an automatic quality advantage.
- **Typography:** Opening headline is now 64 px, with the same -5 px letter spacing. The source diff changes the opening font-size expression; support and CTA retain their previous sizes.
- **Safe areas and layout:** Sampled primary text fits inside the 8% safe region; decorative shapes sit below it. No obvious sampled type overlap or clipping. All-frame OCR/bounds verification remains unmeasured.
- **Motion and transitions:** Growing underline, rotating outline square, expanding circle, animated diagram and CTA arrow. More low-motion holding than the retained baseline by measured RGB delta; this is not a smoothness or quality score.
- **Story progression:** Three source sections at 0–5, 5–12 and 12–15 s. Supporting text and CTA enter with easing rather than hard cuts.
- **Color and contrast:** Requested navy/white/mint are used. Thin low-opacity rules and small section labels are subtler than primary type. Formal accessibility contrast across every frame was not measured.
- **CTA hold:** At exactly 12.0 s the initial export sample has no visible CTA text; at 13.5 and 14.8 s it is visible. Source opacity eases from zero over 24 frames, so a full 3 s readable CTA is not demonstrated.
- **Headline edit scope:** Retained Scene.tsx diff changes only the opening font-size expression. However, the edited 12 s export sample shows the CTA already visible whereas the initial 12 s sample did not. Therefore strict output-level isolation of the change is not established, despite the narrow saved-source diff.
- **Pacing revision:** No completed pacing-feedback export. Timeout means a head-to-head final pacing comparison is unavailable.
- **Editability and provenance:** ClipKit project, confirmed brief, storyboard, versioned TSX scene, bound headline/support/CTA/color props and recorded agent critic evidence. One generated scene item contains the whole 15 s composition; primary text is not represented as three independent timeline text clips. Font data dominates source snapshot bytes; it is not creative code complexity.

Known study usage is **at least 4,433,870 reported tokens**, including cached input. This sums the largest unambiguous completed checkpoint counter in each distinct run; provider-failed turns, unfinished steps and reset segments are not assigned invented usage. Exact whole-study total is unavailable.

## Exact incremental token breakdown

Cached tokens are a subset of input, not additional tokens. All numbers below are client-reported. Missing/resumed-reset steps are excluded from exact incremental totals.

| Run / step | Input | Cached input | Uncached input | Output | Total |
| --- | ---: | ---: | ---: | ---: | ---: |
| rep-1-A0 / 0 | 142,903 | 124,288 | 18,615 | 2,909 | 145,812 |
| rep-1-A0 / 1 | 165,001 | 161,664 | 3,337 | 532 | 165,533 |
| rep-1-A0 / 2 | 183,853 | 177,152 | 6,701 | 1,155 | 185,008 |
| rep-2-A0 / 0 | 443,395 | 412,544 | 30,851 | 5,943 | 449,338 |
| rep-2-B / 0 | 1,406,542 | 1,367,808 | 38,734 | 7,885 | 1,414,427 |
| rep-2-B / 1 | 595,752 | 586,496 | 9,256 | 1,895 | 597,647 |
| rep-3-A0 / 0 | 683,272 | 637,696 | 45,576 | 7,092 | 690,364 |
| rep-3-B / 0 | 782,254 | 729,472 | 52,782 | 3,487 | 785,741 |

## Retained export properties

| Clip / step | Duration / frames | Codec / pixel format | File bytes | Bitrate kb/s | RGB motion mean % | Black frames / audio streams | Fast start |
| --- | --- | --- | ---: | ---: | ---: | --- | --- |
| rep-1-A0 / 0 | 15s / 450 | h264 profile 100 / yuv420p | 191,380 | 102.069 | 0.06653 | 0 / 0 | true |
| rep-1-A0 / 1 | 15s / 450 | h264 profile 100 / yuv420p | 176,164 | 93.954 | 0.05016 | 0 / 0 | true |
| rep-1-A0 / 2 | 15s / 450 | h264 profile 100 / yuv420p | 227,961 | 121.579 | 0.09505 | 0 / 0 | true |
| rep-2-B / 0 | 15s / 450 | h264 profile 100 / yuv420p | 320,765 | 171.075 | 0.02916 | 0 / 0 | true |
| rep-2-B / 1 | 15s / 450 | h264 profile 100 / yuv420p | 308,110 | 164.325 | 0.0267 | 0 / 0 | true |

## Interpretation

Initial-generation counters across all three baseline turns have a median of 449,338 tokens (145,812–690,364). The plugin has one retained initial export at 1,414,427 tokens; another reported initial turn spent 785,741 tokens without producing a video. Do not confuse a reported turn with a successful export. The same-repetition initial receipts (rep 2) are 449,338 versus 1,414,427 tokens, and 209.994 versus 331.036 seconds: 3.15× tokens and 1.58× time with the plugin. That baseline video was lost through a harness retention bug, so this pair supports a cost/time comparison, not a visual one.

The retained visual pair is rep-1-A0 versus rep-2-B: 145,812 versus 1,414,427 initial tokens and 111.411 versus 331.036 seconds. These are different repetitions; the baseline is also its fastest initial attempt. They must not be presented as a typical 9.70× token advantage. The plugin does not show token savings in this sample; the study cannot certify a general quality or reliability advantage for either arm.

Only baseline rep 1 completed the full three-step script, with 491,757 input + 4,596 output = 496,353 tokens, including 463,104 cached input; 229.758 s through its final checkpoint (232.495 s active wall recorded by the harness). Plugin rep 2 reached 2,012,074 reported cumulative tokens through the headline edit, then timed out; its complete session cost is unknown. Failed or incomplete attempt costs must remain visible and cannot be summed into an exact whole-study cost.

Generation token counters exclude this assistant’s orchestration, analysis and report-writing tokens, which are unavailable. No USD/credits, peak RAM, energy, CPU/GPU-only render time, audio/voice quality, speech timing, real footage quality, all-frame OCR, or independent human preference measurement was collected. File size is not a quality score: compression settings were not fixed. No fresh model calls were made to repair losing outputs or replace failures.

## Workflow and diagnostics

| Run | Calls by step 0 / 1 / 2 | Recorded failed tool calls | Full script + continuous usage |
| --- | --- | ---: | --- |
| rep-1-A0 | 8 / 3 / 3 | 0 | true |
| rep-1-B | 39 / 0 / 0 | 0 | false |
| rep-2-A0 | 19 / 0 / 0 | 3 | false |
| rep-2-B | 87 / 40 / 46 | 11 | false |
| rep-3-A0 | 21 / 10 / 0 | 3 | false |
| rep-3-B | 46 / 15 / 0 | 7 | false |

Tool-name breakdowns, raw usage snapshots, outcomes, version/model, setup time and counter-continuity flags are in report.json and the interactive viewer. More calls are not inherently worse, but they add context/latency here. The retained baseline used shell/Python; the plugin used ClipKit MCP, project operations, scene drafts, render/status and quality tools plus shell calls. No unverified Source Hub repositories were ingested. Verified local font fixtures and original geometric scene code were used; this is not a license audit of every transitive dependency. Provider-capacity failure, 600 s timeout and local EDQUOT persistence failures confound completion rates. Three temporary-video symlinks were not retained; this is a harness evidence failure, not an authored-video defect.

## Scope and limitations

- Fresh-session initial counters are exact reported values. Later differences are exact only while counters remain continuous. A completed agent turn does not prove an exported video exists. Full-session incomplete totals are lower bounds, never complete cost comparisons.
- Provider capacity and local persistence failures are environmental confounds; they are not visual quality failures.
- The case is a silent synthetic teaser. Speech, music, transcription, cloning, footage cleanup and real-brand acceptance are outside this comparison.
- Aesthetic observations are assistant inspection, not independent human ratings.
- No client-reported dollar cost is available. Cached input is shown separately; no price estimate is invented.
- Codec choices and compression settings were measured outputs, not fixed requirements.
- Baseline default-CLI skill hashes and clean namespaces are recorded. Plugin uses its full MCP/skill with no additional model delegation.
- A retention bug left three baseline checkpoint exports as dangling links to temporary files. Their videos cannot be inspected; token receipts are retained. Future stage copies request dereferencing and the brief requires a regular file; namespace-private temporary symlinks cannot be recovered after the client exits. Source observers retain authored files; library/dependency bytes are not treated as model-authored code complexity.
