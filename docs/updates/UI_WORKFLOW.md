# Studio workflow integration

2026-10-30. This extends the owner-approved UI-1 shell with the separately approved workflow implementation. Work is isolated in `/Users/sanidhyavats/Desktop/clipkit-studio-ui-pro`, branch `codex/ui-pro`. It does not complete every UI-2 through UI-8 requirement from `UI_UPDATE.md`.

## Working paths

- Edit: add titles, select preview/project/timeline layers, edit supported text/style/timing properties, move and edge-trim clips, split, delete/ripple delete, snap, zoom, seek, undo and redo. Gestures submit one revisioned patch. Locks are enforced both by disabled controls and the Rust server.
- Media: import actual local paths, inspect source video, mark a range and place it at the playhead. The scene library uses real recipe data, typed slot inputs, imported source choices and the existing brief gate. Source-range placement adds a layer; it does not insert time into the sequence.
- Generate: brief, looks, questions, recipe actions, agent context and element-anchored requests use existing APIs. Edit, Generate, Audio, Captions and Review presets provide focused panel arrangements.
- Audio/captions: create a recording script, request microphone access from the record action, record/upload takes, manage existing voice/transcript operations and use caption/timeline interchange export. Existing consent/provider limits are retained.
- Review: comments, history/branches, verification with layer navigation, storyboard confirmation and export. MP4 uses `/api/export`; OTIO/SRT use `/api/export/timeline`. Completion shows the real output path; failures show server codes/hints.
- Resilience: refused edits roll back, Escape cancels inspector drafts, an acknowledged patch is not retried because its refresh failed, closed panels cannot blank a maximized workspace, form state survives panel moves, and denied browser storage has a fallback.

## Performance and browser evidence

The program monitor is memoized and an external playback clock updates time displays/playhead without rerendering the entire workspace for every frame. Timeline hit targets are limited to visible clips while the canvas draws the viewport. The 300-clip real-server test records scroll/paint time and visible target count; this is a measured case, not a claim that all target FPS/memory budgets are met.

Screenshot inspection also caught standalone titles using dark look text on a hardcoded dark background. A failing browser regression captured that mismatch; the composition now uses the active look's background, with the same behavior in preview and the render worker. `title-contrast-regression.txt` records the failure before the fix, and the final real-server run includes the passing assertion.

Production build output is recorded in `ui-workflow-evidence/build.txt`. Visual coverage includes five workspaces at 1440/1920 in dark/high contrast, narrow panels and real editing/export/recipe screenshots. `shell-final-five.txt` labels frontend-only API contracts; `real-final-five.txt` uses fresh real Rust servers, real WebSockets, synthetic media, ffmpeg, microphone simulation and the Node render worker. It does not simulate a live coding agent or Fish Audio service.

Runtime prerequisites were provisioned in an isolated user cache: Rust 1.99.1, Node 24.22.1, just 1.59.0, nextest 0.9.150 and ffmpeg/ffprobe 6.1.1. These are the versions actually used; this does not claim the owner's requested global ffmpeg 8.0.1 setup is complete. Browser recordings required explicit mono channel-layout normalization with this ffmpeg; a failing regression was reproduced and fixed. Remotion jobs now share one browser per job, close it afterward and allow `CK_RENDER_BROWSER` for the installed Chrome. The engine and dependency versions were retained.

## Remaining limits

External derived components are not executable render assets yet; see `ASSET_USAGE_AUDIT.md`. Narration/music are mixed during export, not in the live program monitor; recipe media panels are muted. Captions can be exported as SRT, but burned-in captions are not implemented. AI voice/transcription/cloning require unfinished provider integration. Speed changes, ducking, keyframes, a motion graph, transform handles, waveforms, floating panels, server-stored layouts, draft/apply and variant comparison remain unavailable or outside this approved integration. UI copy and disabled controls identify these limits.

The main branch's committed studio-token separation was integrated without weakening authentication or pipeline gates. Test fixtures use a studio credential for human patches and advance the real pipeline before logged overrides. The narrow server image-route/render-job fixes and recording normalization were made after rebasing committed backend work; they add no new endpoint or model field. Main's files were not edited. Coordinate integration of this branch before changing overlapping studio/server files elsewhere.

An attempt to open a fresh visible review tab at the end was blocked by Codex's “Local browser use is disabled in Work settings” setting. Automated local Chrome verification succeeded; visible-browser interaction was not claimed as completed in that attempt.
