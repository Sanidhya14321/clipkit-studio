# Brief Studio and direction tools

The agent begins a new film with `ck brief propose --prompt "…"` (or `ck video generate …`). Studio opens the Creative Brief with inferred values, their sources and confidence. Confirmation releases waiting clients; builds cannot start from a proposed brief. `just do it`, `no questions`, or the local autopilot setting bypass the form. This does not approve a protected motion proof or grant paid speech/clone consent.

The Brief button reopens the form. Pacing numbers come from `assets/taste/pacing.yaml`. Four original looks live in `assets/taste/looks`; token bindings are shared by preview and export. Editing a confirmed field first returns affected layer IDs and passes. Apply commits a brief revision, deterministic token/reflow changes and a selective re-plan receipt. Changes needing semantic timing/script work remain **pending** for the agent; speech is never silently cut. Effort budgets freeze per turn; draft uses fewer layer passes. Presets are local JSON in `.clipkit/brief-presets`.

Useful CLI operations (use `pnpm ck` in this checkout):

```sh
ck brief get
ck brief diff --since 1
ck brief set pacing calm                 # impact preview
ck brief set pacing calm --apply --rev 2
ck context
ck wait --cursor 12 --timeout 45
ck ask 'Which opening?' --options 'Product,Speaker'
ck recipe search 'hook title'
ck recipe card hook-title
ck layout check
ck sound plan
ck see --contact-sheet
ck branch create alternate
ck branch compare alternate
ck replay export TURN_ID
ck export --format bundle
```

Use `ck help` and the typed MCP definitions for exact JSON arguments. The seven native recipes accept factual slots and generate editable managed scene versions, provenance and safe-area layout. Text fitting is conservative; arbitrary imported scene geometry requires actual render inspection. Recipe thumbnails are original **layout diagrams**. They are not screenshots of a finished film. Unsupported aspects are excluded from recipe search. A reference upload produces measured local statistics and never inserts its content. Caption density remains unavailable without OCR.

A requested storyboard/plan checkpoint holds building until its beat durations and recipes are confirmed. Studio supports reorder, delete, add and recipe swap. This pre-build checkpoint is separate from the existing human approval of an actual motion sample.

Voice has three paths: native terminal dictation supplies text to the agent; Studio mic input uses configured Fish transcription or browser speech; `ck ask` presents an answer card supporting typed, clicked or mic input. Studio publishes playhead, range, hover and selection through `ck context` / MCP `clipkit://context`. Deictic requests need context before editing. Local playback commands use no agent turn; other speech creates an anchored queued comment. Text routing is tested, but live microphone permissions/provider behavior need a real device session. Wait calls cap at 45 seconds; repeat with the returned cursor. End Session wakes listeners.

Scoped leases let an agent edit its locked layer while a human edits other layers. `layer_patch` requires stable IDs and expected per-layer revisions. Track/global edits still require exclusive ownership. Layer revisions increase even on undo. Cache preparation supports isolated native recipe scenes only; cache identity includes tokens, compiled scene, settings and media. A changed headline invalidates its layer; arbitrary authored scenes remain live.

Systems opens branch comparison, replay, local taste settings and `/bench`. Branches preserve revision history and use stable-ID three-way conflict detection. Discards retain recovery files. Confirmation/pass changes create snapshots. Replay is paginated, hashes source payloads and redacts secret-shaped arguments; exported replay contains the complete selected turn trace. Actual provider usage and estimated tool payload counts stay distinct.

Export supports queued MP4, SRT, OTIO, a portable ClipKit ZIP and a standalone Remotion ZIP. SRT uses existing captions/trustworthy words. OTIO preserves editorial timing but generated scenes need pre-rendered media for an NLE. A ClipKit bundle includes current project/media/compiled scene resources, derivations, brief and licenses, starting a new revision-0 history. The standalone Remotion project plays compiled native scenes and basic media without the ClipKit server; advanced caption/mix/crop behaviors are not a full-fidelity substitute for the ClipKit renderer. FCPXML and EDL are not implemented.

Taste learning stays local in `.clipkit/usage.jsonl`; use Systems → taste to generate/edit/disable/clear it. Repeated answers affect defaults; explicit prompt/project/reference facts take precedence. Profile defaults are validated data. Sound planning uses actual licensed local assets; missing risers/room tone are skipped. Source LUFS/true-peak measurement does not certify the final mix. Contact sheets contain at most six frames; optional critic submissions must cite those frames. Aesthetic acceptance remains human.

## Reproducible benchmark protocol

`ck bench run default` writes the ten-case manifest, method/report and chart using available records. Missing measurements remain missing. `ck bench record record.json` imports operator measurements; each record identifies model, limits and method. Pairing requires matching model/limits and both local video paths. `/bench` randomizes opaque A/B video routes; ratings are human input, and the report aggregates the latest rating for each presented pair. Repeated ratings of a pair are not independent raters.

`ck bench execute manifest.json --out receipts/` runs explicit local driver commands sequentially and imports their receipts. It does **not** turn reference, recipe or transcript text into commands. A minimal manifest:

```json
{
  "model": "exact-model-id",
  "limits": "same context, tool and time limits for both arms",
  "timeoutMs": 300000,
  "trials": [{
    "caseId": "teaser",
    "evidence": ["facts.md"],
    "rubric": "15 seconds; factual copy; readable title; no placeholders; human rates motion",
    "pipeline": {"cwd": "pipeline", "argv": ["node", "driver.mjs"], "preview": "preview.mp4", "result": "metrics.json"},
    "vanilla": {"cwd": "vanilla", "argv": ["node", "driver.mjs"], "preview": "preview.mp4", "result": "metrics.json"}
  }]
}
```

Drivers receive `CLIPKIT_BENCH_MODEL`, `LIMITS`, `CASE`, `ARM`, `RUBRIC`, `PREVIEW` and `RESULT` environment variables with the common prefix `CLIPKIT_BENCH_`. A driver runs its real agent workflow with identical evidence. Pipeline uses the confirmed brief and tools; vanilla writes Remotion directly in an isolated directory without ClipKit tools. Persist the first playable preview to the specified path. Write result JSON with `tokens` (actual provider usage or null), `turnsToAcceptance` (human rubric, or null), `lintScore` (measured 0–1 or null), `accepted` and `method`. Outputs must be fresh. The executor measures wall time/preview appearance at 100ms resolution, saves logs, input SHA-256s and argv receipts, and fails on timeout/nonzero exit. A driver still has to enforce its declared model/budgets; the harness does not certify operator claims.

Run all ten cases with the same source facts/media/transcripts and written rubrics, repeat trials in fresh directories, retain each receipts directory, and report variance. Preserve raw receipts before importing another repeat (the compact dashboard holds the latest case/arm record). Do not claim a pipeline quality/token win without real paired runs and blinded human ratings. No such win is claimed by the implementation fixtures.
