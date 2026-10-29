# Guardrails and refusals

The server enforces these. A refusal carries a stable code and a one-line hint; read it, fix the
cause, and never send the same failing call more than twice (the third is refused with E_RETRY).

- E_STAGE: the call is out of order. Do the step the hint names.
- E_LOCK_HELD, E_LOCK_NOT_HELD: ck lock the layers before patching them; the user cannot edit
  layers you hold. Locks expire without a heartbeat, so unlock when a pass ends.
- E_PATCH_STALE: the project moved since you read it. Re-read only the layer ids in the error, then
  patch again with the new revs.
- E_SCHEMA, E_PATCH_INVALID: the patch or the props do not match the schema. Fix the named field.
- E_SCOPE: while answering a comment you may touch only the anchored layer and its declared dependents.
- Budget refusals: retrieval levels, images viewed and patch size are capped per turn. Use the
  smaller level (search, card, skeleton, slice). Full source needs a one-line reason.
- Destructive changes (deleting a track, clearing the timeline, removing many layers, overwriting
  the user's manual edits) need the user's confirmation in the studio.
- Quality: status idle is refused while blocking gates fail, unless you record a user-visible
  override reason.
- Honesty: a comment resolves only with a revision that touched its anchor; an asset counts as
  used only if a layer references it.

Hooks also block direct writes to .clipkit/, assets/ and media files, and shell commands such as
ffmpeg on project media. Use the ck command the block reason names.
