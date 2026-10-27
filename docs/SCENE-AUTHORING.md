# Video editing and generation contract

ClipKit owns the timeline, source artifacts, validation, ownership, preview and export. Coding agents author scene TSX and make bounded edits through the same MCP/API operations as the studio. They do not rewrite the application.

## Invocation

`ck ensure --root /absolute/project` starts or reconnects localhost. In-app Codex turns run gpt-6.1-sol at medium effort with a focused MCP profile: authoring, scene-edit, media or voice. Profiles reduce irrelevant tool definitions; they are not a claim about total provider token usage. The external MCP server exposes all tools unless CLIPKIT_TOOL_PROFILE is set.

A property edit uses the supplied affected-layer props. Source changes read only the selected immutable version. Large sources support `scene_read` with `metadataOnly`, followed by `file`, zero-based `startLine` and `limit` (maximum 120 lines). Footage uses timestamped transcript windows. New generation loads only relevant motion guidance.

External agents acquire `status_working`, heartbeat every 10 seconds and release with `status_idle`. The MCP connection retains its acquired lease for subsequent calls. In-app turns receive their lease automatically. File watching publishes only under an external authoring lease; app turns submit `scene_draft` directly. Static validation is not an executable-code security sandbox.

## Source versions

A scene draft includes `sceneId`, `name`, `files` with a `Scene.tsx` entry, `defaults`, `fields`, `propsSchema`, and `durationFrames`. Existing instances additionally need `targetItemIds` and `baseVersion`. Optional `atFrame` places a new instance. Read the actual tool schema before submission.

Use React, Remotion and `@clipkit/motion`; local CSS/SVG imports are supported. CSS is imported as a string and applied in a scoped style element. Use project-frame time, deterministic randomness and explicit local resources. No timers, autonomous playback, network or wall-clock animation.

Declare meaningful controls and selection tags: `data-ck-layer={item.id}`, `data-ck-path="headline"`, `data-ck-element="stable-name"`. Source and property controls must affect real rendered content. Scene instances pin immutable versions, so changing one does not change every copy. Save/Ctrl+S compiles and validates before publication; errors retain the last working preview. Undo restores prior code references and props. Exports freeze those references.

## Frames and chapters

Ranges are integral and end-exclusive: 5–10 seconds at 30 fps is `[150,300)`. Range comments freeze their anchor while composed. Server-side scope preserves outside content, source phase and narration. Boundary-crossing layers split into scoped segments when needed.

For long form, `reel_plan_set` accepts up to 120 nonoverlapping chapters with stable IDs, titles, summaries and frame boundaries. Maximum planning duration is 108,000 frames (one hour at 30 fps). This is a validation limit, not a benchmark of hour-long generation/render performance.

`reel_read` pages chapter summaries without sending the whole score. Request one `chapterId` to obtain intersecting score and layer references. `reel_chapter_score_set` replaces only that chapter's contained events and preserves protected reading windows and other chapters. The Motion panel's Generate selected chapter creates one bounded turn; scenes must fit the selected chapter. Generate/refine chapters sequentially and reuse unchanged artifacts. There is no claim of an autonomous whole-film batch scheduler in this release.

Duration changes require a new score/choreography. They must not silently truncate dialogue or stretch all playback. Preserve source transcript timing and protected reading holds; report conflicts when material cannot fit.
