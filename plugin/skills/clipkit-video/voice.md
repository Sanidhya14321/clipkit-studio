# Voice

Voice slots: the script is split into slots with target windows. Each slot is AI, recorded or
cloned, can be switched any time, and shows a badge. Use ck voice script, speak, clone and mode.

- Fit: small differences are time-stretched; otherwise extend the visuals, shorten the line or
  ask the user to re-record.
- Cloning only from audio recorded in the studio, with the consent notice; never from uploaded
  third-party audio.
- Loudness is normalized across slots, music is ducked under voice, and captions follow the
  active take's real timing.

Voice lanes:
- Lane A, the user's own dictation into the prompt: when they say this, here or that, call
  ck context first (selection, hover, playhead, loop range, open comments).
- Lane B, the studio microphone: simple commands run in the studio; free-form phrases arrive as
  comments anchored to the selection or playhead, so handle them like comments.
- Lane C, you ask: ck ask shows a question with options and an open microphone in the studio;
  the answer is the tool result. Use it only for real ambiguity.

After handoff, loop on ck wait: it returns comments, brief edits, voice requests and
checkpoint confirmations as a compact event list.
