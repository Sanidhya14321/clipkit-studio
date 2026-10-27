# Current verification: video-first pipeline 0.4.0

Verified locally on 27 October 2026. Earlier workspace/library and 0.3 source-authoring reports are historical; see VERIFICATION-0.3.0.md for the previous release.

## Actual film generation and comparison

- A plain Codex pass and ClipKit pipeline generated real 15-second films from the exact same creative prompt (hash and methodology in VIDEO-AB-COMPARISON.md).
- Both encoded films have 450 frames, 30 fps, 1080×1920 H.264/AAC and duration within one frame. Their decoded audio hashes match. The side-by-side output is also 450 frames with H.264/AAC.
- The first pipeline result was weak and remains delivered. The revised director used an original visual composition reference, committed story then choreography, published actual scene code, inspected rendered images, requested a specific repair and passed the repaired sampled frames. No synthetic agent activity or fabricated UI evidence.
- The plain result remains competitive and faster: 108.4s before export versus 444.0s for the revised reviewed pipeline. This is not a universal superiority or efficiency claim. Production usage is unavailable; reported usage subtotal is explicitly incomplete.
- Follow-up code fixes deadline handling even on a zero CLI exit, narrows unrelated built-ins and model tool replies, and advertises a complete scalar scene schema. A separate actual Codex capability probe inspected a reference and published a new scene with those unrelated features disabled. The complete A/B pair was not rerun after the final context/deadline/schema changes.

## Actual editing loop

A real diagnose → edit → visual review turn changed only the accent during `[210,270)` (7–9s at 30fps), kept the immutable source version, and preserved the complete outside signature. The report is edit-loop-verification.json. Automatic agent-attributed comment finalization after review is separately covered by workflow tests.

## Automated and local checks

**128 default tests pass across 17 files.** The default suite covers ordered planning gates, separate editing/review paths, failed gates, cancellation, one repair budget, unreviewed drafts, validated direction reuse, long-form outlining, compact mutation responses, deadline failure, scene contracts, temporal scope, transaction recovery, ownership, undo, source publication and media operations. Typecheck, source inventory and build are required release gates.

**Five real-backend browser checks pass.** They cover six laptop/desktop viewports, reachable final controls, drawers, exact saved frame ranges, human source/prop edits, keyboard boundaries, resize persistence/reset and equivalent zoom. Existing deterministic-frame, immutable export and separately archived legacy-renderer evidence remains in the 0.3 report. New motion primitives are frame-driven; this is not an exhaustive cross-machine/per-frame guarantee.

The source inventory preserves pinned MIT Motion Primitives originals and adaptations, Video Forge guidance, original visual study, Paper notices and fonts. React Bits is reference-only. No rejected stock catalog or private Video Forge production capture is bundled.

## Limits

The critic inspects representative stills and cannot establish every intervening frame, motion feel or audio quality. Human moving playback with sound remains necessary. Long-form outlining and bounded chapter production are implemented, but no hour-long generation/export benchmark is claimed. Claude authentication and Fish live provider acceptance remain unverified.

Exact test/build/install/CI results and source commit accompany the release. Exported editable projects exclude provider keys, live server credentials and agent-workspace configuration.

The outside-repository tarball check passed initialization preservation, the explicit MCP scene schema, native reference-image return, real source compilation, 1080p export, separate compatibility installation/legacy export and an installed dependency audit with zero advisories.
