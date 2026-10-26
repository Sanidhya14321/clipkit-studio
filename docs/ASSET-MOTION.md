# Native motion additions — 0.1.2

The native library now contains fourteen editable clips. Four additions use the existing shared Remotion preview/export engine, with no new visual dependencies:

| Clip | Editable controls | Limits |
|---|---|---|
| Starfield | Seed, count, drift, radius, colors, optional title | 12–240 stars; deterministic ambient DOM effect |
| Confetti burst | Seed, count, two colors, spread, scale, optional title | 8–240 pieces; fixed ballistic arcs, transparent overlay |
| Terminal typewriter | Script, heading, color, type size, reveal duration | Display-only; Unicode code-point reveal; no command execution |
| Animated bar chart | Four values and labels, title, colors, reveal duration | Nonnegative values, shared zero baseline; default numbers are illustrative |

All four are original **agent-authored** components. Fonts retain their separate licenses. Usage cards and previews are included in the compact catalog, and agent placement requires reading the card. Existing projects retain their original native IDs and default props; the regression fixture is `tests/fixtures/native-1.0.0.json`.

## Verification

The current source digest, repeated frame hashes and full smoke result are recorded in `public/packs/native/determinism.json`. Each of fourteen clips was rendered at its start, middle and end twice. Two complete 42-second, 360 × 640 exports contain all fourteen clips and the Kenney click; decoded video and audio hashes match. This establishes repeatability on local Chromium, not a cross-machine guarantee.

The asset checker covers 33 cards and 1,520,039 bytes of carded public media. The existing pinned Inter font budget exception remains; there are no added budget exceptions. Preview/export pixel equality and visibly effective edits are checked separately. Local test results and installation evidence are summarized in the accompanying JSON report.

The broader asset brief's GPU, 3D and advanced simulation roadmap remains deferred. This release adds useful native components without claiming those systems exist. Live provider and second-agent acceptance still require the previously documented Fish credentials and Claude authentication.

Results: 105 default Vitest tests, 13 opt-in tests (107 unique across both runs), and six real-backend browser tests passed. Typecheck, build and fresh-package installation passed. A real Codex medium-effort turn read the chart card and inserted only the requested chart in 27.06 seconds; its incremental payload estimate was 2,562 tokens, separate from runner-reported usage. See [machine-readable results](asset-motion-verification.json).
