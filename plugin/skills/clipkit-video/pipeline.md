# Pipeline stages

The server runs the pipeline as a state machine. A call outside the current stage fails with
E_STAGE and a hint naming the next allowed step; follow the hint.

1. BOOT: the studio is running and open (usually done by the hook). ck status, then the project summary.
2. DETECT: mode from evidence, never guesses. Video plus a request to improve it: EDIT. Video plus
   graphics, voice or new scenes: HYBRID. No video: GENERATE. Audio only: audio is the base.
   Images only: images are media. Several videos: a sequence plan first.
3. BRIEF: propose inferred values, then wait until the user confirms, uses defaults or skips. With
   no questions or autopilot, the brief is auto and shown afterward as an editable card.
4. PLAN: from the confirmed brief only. Taste files first, then looks and recipes. In GENERATE mode
   write the script first and time beats from voice slot durations. Optional storyboard checkpoint.
5. BASE: EDIT or HYBRID: import, proxy, transcribe, packed transcript, cut candidates, cuts as
   patches. GENERATE: background and empty named scene groups on the beats.
6. PASSES, in order: structure, audio, text, graphics and motion, transitions, look, polish. Each
   pass is one or a few patch batches with its own retrieval budget and quick checks.
7. VERIFY: all quality gates, then one critic pass.
8. HANDOFF: release locks, status idle, five-line summary, agent comments pinned on any
   uncertainty, then wait for events.
9. ITERATE: comments, brief edits and voice requests become small jobs that touch only affected layers.

Patches: every patch carries baseRev and the layer revs it read. Patch, never rewrite. Times are
in seconds.
