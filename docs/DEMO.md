# Three-minute demo and recovery

## Prepare before presenting

1. Run `pnpm ck doctor`, `pnpm typecheck`, `pnpm test`; verify both agent clients are signed in.
2. Use a 20–40 second talking-head clip with pauses and a filler. Import once, finish proxy/transcription, check word timestamps and listen to proposed cuts.
3. Prepare an in-app clean voice sample of at least ten seconds. Train the private Fish clone and generate the demo lines in advance. Keep at least one real recorded line and one real cloned line in the narration.
4. Verify every active take, the mixed audio, captions and original-media export. Measure export time on the presentation machine.
5. Keep the prepared project, an exported MP4 and a screen recording. A screen recording is a clearly identified backup, not live activity.

## Present

Measured cold Codex latency was about 28 seconds to the patch and 36 seconds to completion. Explain the patch/context/ownership story while waiting, and adjust the schedule after rehearsing your actual machine and providers. Do not promise instant free-form edits.

| Time | Action |
|---|---|
| 0:00–0:20 | Play raw footage; explain the shared non-destructive timeline. |
| 0:20–0:50 | Use prepared transcript, review/delete a word, add captions and lower third. |
| 0:50–1:20 | Pin a title request, send to Codex, show lock and actual touched layer. |
| 1:20–1:45 | Hover/select the title, adjust a property manually, demonstrate undo. |
| 1:45–2:10 | Local voice command, then anchored free-form voice feedback. Show context estimate and distinguish actual usage. |
| 2:10–2:35 | Play real recorded/cloned narration. Use the second signed-in client for a tiny title edit. |
| 2:35–3:00 | Revert latest agent turn, show export/progress and verified result. Start export earlier if measured render time requires it. |

## Recover honestly

- Provider outage: keep cached takes/transcript, or type the command. New uncached speech remains unavailable.
- Agent failure: cancel, retain committed edits, inspect the error, and retry after authentication or connectivity is restored.
- Unexpected edit: undo after the turn ends; do not reset a real project casually.
- Demo graphics reset: `pnpm demo:reset` is useful for rehearsing editor controls. It contains no real talking-head footage or speech.
- Do three rehearsals with the actual clip, microphone, credentials and machine before presenting. Fixture tests cannot certify those external inputs.
