# Verification status

The automated suite covers project transactions, lease ownership, journal recovery, transcript mapping, voice grammar, real FFmpeg media processing, tone mixing, voice-mode provenance, provider request contracts and context limits.

Real Chromium renderer checks compare each of the ten native clips at identical preview/export props and verify that color edits change rendered pixels. Run with `CK_RENDER_TEST=1`.

The browser integration uses the real HTTP/WebSocket backend, not a mocked API. The install rehearsal must run the produced tarball from a separate directory.

External checks remain distinct:

- Fish STT/TTS/cloning requires the owner's `FISH_API_KEY` and voice access.
- Real microphone and voice quality require a human rehearsal.
- The owner's actual talking-head footage has not been supplied.
- Claude Code is installed but requires sign-in before the two-client claim is verified.

No fixture tone is described as AI or cloned speech. No simulated event stream is described as a real agent turn. Detailed local verification artifacts are stored in `.clipkit/verification/` and exports in `.clipkit/exports/`.
