# ClipKit Studio 0.3.0 verification

Verified locally on 27 October 2026. This release verifies agent-authored video editing and generation, not the removed stock library. Earlier 0.2 reports are historical.

## Real end-to-end evidence

- Codex gpt-6.1-sol at medium effort authored a new TSX scene through `scene_draft`, recovered from a validation error, and published a real immutable artifact to localhost. This was not a property-only substitution.
- A separate real turn authored the 25-second editorial product film. A second turn handled the exact `[150,300)` range comment and changed only that passage. Actual generation and feedback waits are retained in the workflow recording.
- Human controls changed the headline; undo/redo restored it. A human source save compiled and published, and undo restored the prior code version. No fabricated editor screenshots or agent session are used as proof.
- A final real property-edit turn used **18 focused scene-edit tools**, changed the headline, resolved feedback and preserved the source version. Incremental request/tool payload was **2,214 estimated tokens**. Actual reported provider usage was 120,056 input, 583 output, 94,080 cached. These numbers measure different things; there is no claim of 2,214 total model tokens or universal savings. The initial source-generation check used 6,048 incremental tokens and did not meet the old 3,000 target.
- The finished film is **1080×1920, 30 fps, 750 frames**, H.264/AAC, approximately 25 seconds, with original rhythmic sound and cropped genuine editor recordings. The editable project retains scene versions and transaction history.
- The browser Export action completed and its Download export link saved a real MP4. The workflow recording includes generation, range feedback, human editing, export progress and download across separate genuine sessions.

## Local regression and rendering

**107 default tests pass across 15 files.** Typecheck, built application and five real-backend browser tests cover ownership, stale revisions, atomic patches, undo, source errors, late canceled publication, independent instances, frame ranges and protected timing. Browser checks exercise 1920×1080, 1440×900, 1366×768, 1280×720, 1024×600 and 900×700, equivalent 125% zoom, reachable final controls, keyboard boundaries, saved comments, resizing/reset and source-save focus.

Rendered outside frames 0,149,300,749 match the original exactly after the scoped edit; frame 225 changes visibly. Preview/export frame 225 matches. The real Paper shader returns identical pixels for seek order 20→65→20, changes over time, and matches preview/export. These are representative local frame checks, not exhaustive cross-machine or every-frame equivalence.

The separate frozen compatibility archive renders all 46 legacy components twice. It is absent from new-project installation/discovery. Adopted Video Forge/Paper/font records retain licenses and hashes. Source inventory check reports zero errors/warnings. A cache variation regression protects fonts across concurrent loopback renderer origins.

## Longer videos and optimized invocation

Plans support stable nonoverlapping chapters, compact paginated summaries, chapter-only score replacement, protected reading windows, and a Generate selected chapter action. Server tests prove chapter scenes stay inside their budget and unrelated content survives. Source can be read in bounded line windows, footage in timestamp windows, and simple edits can reuse supplied props without re-reading skills/source. Tool profiles omit irrelevant voice/media/authoring operations.

The schema allows 108,000 frames (one hour at 30 fps). **An hour-long film has not been generated or rendered as an acceptance benchmark.** Chapter generation is explicitly initiated one chapter at a time; no whole-film autonomous batch scheduler is claimed. Pacing/duration verification uses explicit choreography and score changes, not a global playback-speed multiplier.

## Remaining live acceptance

Claude Code is installed but unauthenticated; its new-scene live acceptance is unverified. Fish transcription/TTS/private cloning require supplied credentials and remain unverified live provider gates. Cached/local media and original sound work without those providers. Full 3D authoring, a visual object/keyframe editor and unrestricted third-party executable packs are outside this release.

See `verification-0.3.json`, the release artifacts, and the committed verification scripts for exact evidence. The packed-install report records outside-repository compilation/export, idempotent initialization, separate legacy restoration and dependency audit results.
