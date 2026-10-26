# Thirty-second pitch

“Most AI video workflows regenerate an entire scene for a tiny change. ClipKit gives you and your coding agent one shared timeline. Select the title, leave a precise comment, and watch the agent lock the project and change that layer. You can inspect the actual diff, accept a proposal, or undo it. Footage cuts keep their transcript and audio timing. Narration can mix your recordings with prepared AI or consented cloned takes. The context meter shows the measured request payload separately from real model usage. Small edits, reversible collaboration, one export.”

## Prepare the real before/after story

Use your own consented 20–40 second talking-head take. Keep an unedited original. Review a pause/filler cut, choose readable captions, add a lower third, and prepare one recorded and one cloned narration line. Preview and listen to the complete export. Do not present synthetic tone fixtures as human narration or a backup screen recording as a live run.

Save the prepared project with `ck demo save`. `ck demo reset` restores that local snapshot (including references to cached media, takes and transcripts); `ck demo reset --graphics` explicitly uses the graphics-only fixture. Keep the project directory intact: snapshots contain media references, not portable media copies. Reset is itself undoable.

The raw/final control appears after a completed export. Start rendering early based on the machine's measured duration. Record and clearly label a backup only after the real flow succeeds. Three agent edit tests do not equal three complete microphone/voice rehearsals.
