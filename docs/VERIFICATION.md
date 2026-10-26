# Current verification (asset release 0.1.2)

Four new native clips bring the library to fourteen, with 33 usage cards. All 105 default tests and 13 opt-in tests passed (107 unique Vitest tests); all six browser tests passed. Repeated frame checks, two matching complete 42-second exports, preview/export equality and editable-control checks passed. A real Codex turn inserted the requested chart after reading its card. See [motion verification](ASSET-MOTION.md) and [machine-readable results](asset-motion-verification.json).

External voice/Claude acceptance limitations documented below still apply.

# Previous asset verification (0.1.1)

The latest asset update passed 96 standard Vitest tests and the separate 13-test opt-in asset/render suite (98 unique tests across runs), all five real-backend browser tests, typecheck, build and a clean installed-package rehearsal. The package exposes the new card tools, accepts a valid custom pack and exports portrait/landscape H.264/AAC. Installed dependency audit reports zero vulnerabilities.

Ten native clips passed start/middle/end PNG comparisons twice. A complete 30-second smoke reel with the pinned Kenney click was exported twice at 360×640; decoded video frames and audio match. A real Codex turn read the required card before inserting exactly one requested callout and preserved existing content.

See [asset implementation](ASSET-IMPLEMENTATION.md), [machine-readable asset evidence](asset-verification.json), and [overall results](verification-report.json). Native code/previews are explicitly agent-authored. One pinned original Inter font-size exception is retained, and no second-machine rendering certification is claimed.

The previous gap-fix verification below is historical context, not a substitute for these newer results.

# Previous gap-fix verification

Verified 27 October 2026: TypeScript passed; **82 default tests passed**, with two opt-in tests skipped by default. The opt-in asset/render suite separately passed all 12 tests, covering those two real-render checks (84 unique Vitest tests across the runs). All **four real-backend browser tests passed**. The build and isolated packed-install check passed, including repeated init, Cursor rules, custom-pack installation and both portrait and landscape H.264/AAC export.

All ten native clip preview/export frames matched and editable accent changes altered pixels. An explicitly trusted TSX pack ran with shared React/Remotion hooks, passed actual selection-tag/determinism checks, and lost trust after its code changed.

Three fresh real Codex (`gpt-6.1-sol`, medium) turns each resolved the anchored comment and changed only the requested text. Their ClipKit payload estimates were 1,130 / 1,136 / 1,131 tokens, distinct from about 97k actual reported input tokens per turn.

The mixed audiovisual proof passed three cut-point comparisons and audible, unclipped audio checks. Its active takes are explicitly synthetic tones. The packed portrait export has exactly 151 expected frames at 30 fps, 720×1280 H.264/AAC, container duration 5.035s. Landscape export is 1280×720 with audio.

Production/full dependency scans report zero advisories and peer checks pass. The public-history/index scan finds no credential patterns or tracked private/build media.

See [full checkup](GAP-FIX-CHECKUP.md), [criterion matrix](PRD-FEATURE-AUDIT.md) and [machine-readable results](verification-report.json). Detailed recordings, test exports and runtime evidence stay in ignored local folders.

**External acceptance remains:** real Fish calls, consented narration/microphone listening and latency, actual local Whisper recognition if used, user talking-head footage, Claude authentication/real edit, Cursor runtime and three complete timed rehearsals. Passing fixture tests does not certify those steps. Public npm publishing is separate from the verified local tarball.
