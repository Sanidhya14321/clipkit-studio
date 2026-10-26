# Gap fixes and full checkup

Updated 27 October 2026. This is an implementation checkup against the supplied Word PRD, not a certification of a flawless live hackathon demo.

The original audit found 12 failing criteria. Those reproducible implementation failures have been addressed. The updated [79-criterion matrix](PRD-FEATURE-AUDIT.md) records **52 PASS, 0 FAIL, 14 PARTIAL and 13 BLOCKED**. Partial and blocked items are still unfinished acceptance work; zero FAIL does not mean 100% completion. The [baseline](PRD-FEATURE-AUDIT-BASELINE.md) is preserved for comparison.

## What changed

- **Editing correctness:** source-bound trims, regenerated cut audio, exact cached range restoration, configurable lease renewal, post-processing ownership checks and conflict-aware historical reverts.
- **Editor:** live local inspector drafts, grouped sliders, image/easing controls, persistent canvas selection, mini inspector, Ask agent, range comments/pins/loops, actual before/after values, history highlighting and voice attribution.
- **Context:** byte-bounded paged transcript packs/candidates and comment bundles; orphan transcripts excluded from the baseline. Actual model usage stays separate from payload estimates.
- **Assets:** folder validation, catalogs/watch, declarative wrappers, explicitly trusted TSX registration and rendered tag/determinism checks. Changed executable content loses trust. Versioned registry artifacts preserve already-requested renders.
- **Voice:** whole-script alignment with uncertainty rejection, explicit fitting choices, bulk clone conversion preserving recordings, persisted human consent, safe silence trim, measured normalization and overlapping-take envelopes. Browser speech preview is clearly non-exportable; held recognition has repeat/confidence handling.
- **Agent review:** real proposal/ghost previews, human accept/reject, safe merging around unrelated edits, scoped mutation guards and late-turn rejection. This is additional enforcement, not a claim of immunity to prompt injection.
- **Export/install:** quality and aspect-fit presets, used-license dialog/sidecars, raw/export comparison, Cursor rules, doctor-on-init and collision-safe repeated initialization. Alternate aspects fit the scene with gutters; automatic subject reframing is not implemented.
- **Demo:** prepared local snapshot save/reset, architecture diagram and thirty-second pitch script. A real recorded before/after story still depends on the user's media/voice.

## Checks and evidence

**Final automated results:** 82 default tests passed; both skipped real-render checks passed in the separate 12-test opt-in suite (84 unique Vitest tests across runs). All four real-backend browser tests passed. Typecheck, build, mixed render and isolated install/custom-pack/portrait-and-landscape export checks passed.

The [machine-readable verification report](verification-report.json) records final test results and artifact paths. Reproducible checks:

```sh
pnpm typecheck
pnpm test
CK_RENDER_TEST=1 CK_PACK_RENDER_TEST=1 pnpm exec vitest run tests/assets.test.ts tests/assets-packs.test.ts tests/render-audio.test.ts
pnpm build
pnpm test:e2e
pnpm exec tsx scripts/render-mixed-proof.ts
pnpm exec tsx scripts/install-smoke.ts
pnpm exec tsx scripts/audit-prd.ts --real-agent-cycles
python3 scripts/audit-publication.py
```

The real-agent audit completed three Codex (`gpt-6.1-sol`, medium) edits in **22.6, 25.4 and 28.0 seconds**. Each resolved its comment and changed only the requested title text. Estimated ClipKit request/tool payloads were **1,130 / 1,136 / 1,131 tokens**; actual reported input was approximately **97,000 per turn**, including client context. Do not describe the smaller numbers as total model usage or a proven regeneration bill.

The mixed-media proof combines an original/proxy video, four source ranges, native graphics, timed captions, two overlapping active synthetic-tone takes and ducked background tone. Three cut frames had mean RGB errors **5.58, 6.49 and 3.67**, below the 12-point tolerance. Mixed audio peak was **0.157** with RMS **0.058**. This proves mechanical composition/mixing; it is not speech-quality evidence.

Real FFmpeg measurements verify near -16 LUFS normalization, less than 1.5 LU variation between tested inputs, safe preservation of continuous sound, known boundary-pause trimming and faded segment edges. Alignment/provider contract tests use clearly identified fixtures or mocked network responses. They do not stand in for successful live Fish calls.

Local detailed evidence remains in gitignored `.clipkit/verification/`, `.clipkit/mixed-proof/` and renderer test directories. Test videos and recordings are excluded from public Git history. The installable tarball and verified test export are in `artifacts/`.

Production and full dependency audits now report **zero advisories** after patching transitive Zod and correcting the webpack peer mismatch. The distribution bundles the patched Remotion tree and explicitly includes an npm shrinkwrap. The final isolated install reports zero vulnerabilities and verifies every installed Zod copy is 4.6.7. The resulting local tarball is approximately 53MB. The publication scan covers both Git history and staged additions, and reports no credential-pattern matches or tracked private/build media. These are targeted checks, not an exhaustive security guarantee.

The build retains a non-fatal size advisory for its approximately 767kB uncompressed editor chunk. Fonts deliberately resolve from the local server at runtime. Trusted-pack verification now drains pending page-close requests before closing Chromium, including invalid-pack rejection.

## Still required before claiming the complete demo

1. Supply and rehearse the real talking-head clip, including the five-minute import/transcription accuracy case.
2. Configure Fish and verify real transcription, TTS and a private clone from a consented sample. Alternatively install a real whisper.cpp binary/model for local transcription; recognition accuracy is still unverified here.
3. Test the physical microphone, retakes, confidence/errors and spoken-command latency; listen to mixed recorded/cloned narration, captions and transitions.
4. Authenticate Claude Code and run the same real editing scenario. Registration and installation alone do not prove two working clients; Cursor's generated rules have not been exercised inside Cursor.
5. Complete three full timed rehearsals and record the clearly labeled backup. Three successful Codex edits are not three full presentation rehearsals.
6. Complete broader accessibility and adversarial-agent evaluation. Focus/width/boundary tests cover specific cases, not every assistive technology or prompt-injection attack.

Public npm publishing requires an available package name and publishing credentials. The verified local tarball is the supported installation artifact.
