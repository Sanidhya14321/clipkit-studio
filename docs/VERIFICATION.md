# Verification status

Verified on 26 October 2026: **51 automated tests passed**, one default opt-in render test skipped, TypeScript check passed, and the real-backend Playwright test passed. The opt-in renderer was independently run: all ten native clips matched preview/export frame hashes and every tested accent edit changed pixels.

A real Codex (`gpt-6.1-sol`, medium) turn changed only the anchored title, resolved the comment with a diff, and passed exact grouped revert/redo checks. Measured ClipKit request/tool payload was approximately 1,608 tokens; actual client input/output usage is recorded separately, not presented as the same measurement.

The tarball was installed in an isolated directory. Repeated init preserved existing notes and MCP configuration. That installed package served the studio and fonts, imported actual audiovisual test footage, and exported H.264/AAC at 720×1280, 30fps, with exactly 151 expected video frames. Container duration is 5.035 seconds including audio padding. This fixture contains test patterns and tones, not user footage or cloned speech.

See [machine-readable report](verification-report.json) for checks and local artifact locations.

The automated suite covers project transactions, lease ownership, journal recovery, transcript mapping, voice grammar, real FFmpeg media processing, tone mixing, voice-mode provenance, provider request contracts and context limits.

Real Chromium renderer checks compare each of the ten native clips at identical preview/export props and verify that color edits change rendered pixels. Run with `CK_RENDER_TEST=1`.

The browser integration uses the real HTTP/WebSocket backend, not a mocked API. The install rehearsal must run the produced tarball from a separate directory.

The recorded cold Codex turn took about **28 seconds to commit the visual change and 36 seconds to finish**. Local editor commands are immediate after recognition; free-form agent edits are not instant. Budget this latency in the presentation and use the explicitly labeled backup recording if the provider is slow.

External checks remain distinct:

- Fish STT/TTS/cloning requires the owner's `FISH_API_KEY` and voice access.
- Real microphone and voice quality require a human rehearsal.
- The owner's actual talking-head footage has not been supplied.
- Claude Code is installed but requires sign-in before the two-client claim is verified.

No fixture tone is described as AI or cloned speech. No simulated event stream is described as a real agent turn. Detailed local verification artifacts are stored in `.clipkit/verification/` and exports in `.clipkit/exports/`.
