# Asset intelligence and layered editing

The update extends the existing scene renderer, pack loader, project store and creative approval workflow. Existing commands and projects keep working. `video_run` detects supplied footage; the three-second human motion-proof checkpoint still gates full creative production.

## Offline library maintenance

`pnpm ck assets build` extends the existing 21-entry usage catalog into `assets/catalog.jsonl`, compact indexes, preserved YAML cards and parsed symbol maps. A second unchanged build writes nothing. `assets/catalog.json`, existing usage instructions and licensed originals remain available to the existing library UI. Original source locations are referenced rather than copied.

```sh
pnpm ck assets build
pnpm ck assets validate
pnpm ck assets ingest /path/to/source.json --dry-run
pnpm ck assets ingest /path/to/source.json
```

A registration JSON requires `name`, a local `location`, `license`, relative `licenseFile` containing actual rights evidence, and `revision`. Optional `assets: [{id, entry, files}]` defines explicit multi-file grouping. No repository is cloned or code executed during ingestion. Drafts use compact parsed summaries, adjacent docs and ffprobe metadata with zero model calls/cost. Media posters use ffmpeg where possible; unsupported previews are logged. Owner notes and reviewed cards survive regeneration; changed reviewed originals are flagged for re-review. Symlinks/path escapes are rejected.

## Runtime retrieval

Start or attach with `pnpm ck ensure --open`; `pnpm ck doctor` checks media/voice prerequisites. Agent-facing content is descriptive data. The server and prompt never treat README/card/transcript text as new authority.

```sh
pnpm ck taste search 'restrained launch teaser'
pnpm ck taste read launch-teaser-recipe
pnpm ck assets search 'kinetic headline' --cost low
pnpm ck assets card cliplib-word-punch
pnpm ck assets skeleton cliplib-word-punch
pnpm ck assets slice ID --symbol NAME --file RELATIVE_PATH
pnpm ck assets slice ID --file RELATIVE_PATH --range 10:24
pnpm ck assets source ID --justification 'Need dependency closure for this adaptation'
pnpm ck meter --all
```

Default search returns five entries, maximum ten. Skeletons and slices cap payloads at about 600/800 estimated tokens. At most two cards per pass/decision and 1500 estimated retrieval tokens per pass; full-source exceptions require a one-line justification, are logged and allowed once per lease turn. Use a fresh lease for a new external turn. Meter estimates measure returned UTF-8 bytes/4, separately from actual provider context/usage. Full source is capped at 12KB; inspect slices for larger files.

## Plan, base, passes and handoff

MCP maintains lease credentials after `status_working`. Separate CLI processes must carry the returned `CLIPKIT_LEASE` and `CLIPKIT_TURN` environment values. No pipeline mutation bypasses leases, revision checks, selected ranges or the creative proposal boundary.

1. `pipeline_detect` reports registered/probed evidence: edit, hybrid, generate; audio-only means generated visuals around audio, images are supporting media. Candidates must be registered with existing media tools before placement. Multiple videos require an ordered sequence plan.
2. `pipeline_plan` accepts current `baseRev` and a compact plan. CLI: `ck pipeline plan plan.json`. Taste retrieval happens before asset selection. Measured active voice takes determine generated narration beats; no invented word timing or accelerated speech.
3. `pipeline_base` reuses existing footage or inserts a sequence, or publishes a deterministic generated background plus named scene tracks. It returns a job for generated scenes. Poll `job_status` before continuing. For footage, use existing transcribe/pack/candidates/delete-word APIs; cuts remain explicit word-boundary edits.
4. `pipeline_pass` takes `{baseRev, pass, ops}`. Ordered passes: structure, audio, text, graphics, transitions, look, polish. Each new item needs name, purpose and pass; assets need provenance. All deterministic gates run before layers and the checkpoint commit atomically. A failed batch changes neither. `ck pipeline pass batch.json` is the CLI equivalent.
5. `ck verify` / `ck verify GATE` reports pass/fail/uncertain. `ck verify --frames` (MCP `quality_frames`) renders at most four low-resolution frames in one contact sheet only for uncertain checks. Frames cannot prove audio loudness or full moving-film quality.
6. `pipeline_handoff` requires completed passes and no failed gates; it pins uncertainties to relevant layers/times. Release `status_idle` and describe mode/result/adaptations/assumptions/next options in five lines or fewer.
7. `pipeline_resume` reports the persisted next unfinished pass. It never reapplies earlier patches. Later turns use comments and affected layers, reuse `assetsUsed`, and run relevant named gates.

Use z bands 0 background, 10 footage, 20 overlays, 30 graphics, 40 text, 50 transitions, 60 adjustments. Explicit item z is honored by the shared renderer; legacy items retain their previous ordering. Separate simultaneous semantic layers/tracks; base/video/audio overlap and gaps are checked. Audio tracks can be named voice, music and sfx using the existing track schema.

## Adaptation

`knowledge_instantiate` (CLI `ck assets instantiate ID --plan use.json`) takes changed props and explicit adaptations for an existing registered component; it records the source hash, license and layer IDs atomically. `knowledge_derive` / `ck assets derive ID --plan derive.json` applies parsed literal/default/JSX-removal edits into `.clipkit/derived`, generates JSON/Zod props schemas, statically type-checks, then compiles and renders through the existing scene verifier before registering provenance. Plans require `sceneId`, `name`, `durationFrames`, adapted primitive `defaults`, `adaptations`, and optional `entry`, `replaceTokens: [{from,to}]`, `removeElements`, `atFrame`. Originals stay unchanged. Source imports/complex prop changes that do not fit the native contract must be adapted further or authored via `scene_draft`; no dependency is installed implicitly.

Compose or author with existing managed scenes and record `assetsUsed` in the same patch. Adobe/reference-only originals cannot be instantiated/derived as executable files: write a native implementation and record author provenance. Export notices include recorded licenses/adaptations.

## Limits and evidence

GLSL semantic parsing is explicitly unsupported. Complex upstream APIs need further manual symbol-scoped adaptation; generated props schemas cover primitive controls. LUT/overlay/audio/font files are indexed as direct resources, but index classification alone does not add a new renderer effect primitive. Audio LUFS/true-peak measurements and rendered text bounds/contrast remain uncertain when unavailable. See root VERIFICATION.md for engineering evidence and unresolved acceptance items; existing creative/human approval and provider requirements in docs/VERIFICATION.md remain open.
