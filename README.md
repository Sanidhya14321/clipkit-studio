# ClipKit Studio 0.3.0

A local studio for agent-authored reels: **brief → frame score → live animation code → human controls/source edits → frame-range feedback → MP4**.

The old stock/template collection has been removed. Motion is a kit of composable mechanisms and Video Forge direction skills, not a JPG/video catalog or a sequence of generic presentation slides. User media remains available separately.

## Run

Requires Node 22+ and pnpm. Development uses Node 24 and the matching Remotion 4.0.538 family.

```sh
pnpm install
pnpm ck init
pnpm dev
```

Development editor: http://127.0.0.1:5173, API: http://127.0.0.1:4310. Use `pnpm ck studio --no-vite` for the built editor. A packed installation runs `ck init` followed by `ck studio`; `ck ensure` starts/reconnects localhost for external coding agents.

## Direct and edit

In Motion, enter a creative brief, duration and Fast/Balanced/Slow pacing. Create with agent uses the installed Codex or Claude CLI. Codex uses gpt-6.1-sol at medium effort; clients require their own authentication and available quota. ClipKit does not fabricate agent progress or replace failed turns with predetermined patches.

A valid `scene_draft` compiles in a worker, checks representative rendered frames, publishes an immutable version and updates the open Player. Invalid code keeps the last working scene. Properties exposes declared bindings; Scene source supports syntax highlighting, diagnostics and Save/Ctrl+S. Undo restores both code versions and props. Instances and running exports pin their versions.

Use the visible Range tool or I/O. Enter HH:MM:SS:FF or integer frames, drag the boundaries, and add a comment. At 30 fps, 5–10 seconds means **[150,300)**. Comments freeze their anchor while you compose. Clicking a saved band restores it. Agent edits preserve content outside that range, including animation phase. Narration timing is protected.

## Skills and authoring

Video Forge 0.7.1 contributes selected MIT direction/rhythm guidance, twelve shot recipes and frame-based helpers. ClipKit retains one project, renderer, history and provider system. Eight mechanism examples teach type, masks, match movement, depth, proof, collage, captions and material reveals. Paper Shaders 0.0.82 is explicitly frame-seeked. Reference sources and actual adopted files are distinguished in [the inventory](packages/motion/sources.json).

```sh
ck ensure
ck motion search typography
ck motion read agent-guide
ck scene draft scene.json
ck scene read ITEM_ID
ck scene status
ck reel plan score-plan.json
```

A draft contains sceneId, name, files with Scene.tsx, defaults, fields, an explicit scalar propsSchema and durationFrames. Updates supply targetItemIds and baseVersion; range updates can specify startFrame/endFrame. Source uses React, Remotion, local CSS/SVG and @clipkit/motion. Tags map rendered elements to declared controls. Local file edits under scenes/ require an active authoring lease; imported third-party code still needs explicit trust. Static checks are quality gates, not a security sandbox.

## Existing projects

0.2 projects use a **separate compatibility archive**, absent from new installations. Before this checkout was cleaned, its old renderer/resources were frozen and verified. To open an older project elsewhere, obtain clipkit-compat-0.2.0.tgz from the release and run:

```sh
ck compat install /absolute/path/clipkit-compat-0.2.0.tgz --root /absolute/project
ck studio --root /absolute/project
```

The archive is checksum-pinned. It preserves clip IDs and history; it does not restore the rejected catalog to new-project discovery.

## Verification and limits

See [verification](docs/VERIFICATION.md), [architecture](docs/SCENE-AUTHORING.md) and [release notes](docs/RELEASE-0.3.0.md). The release includes a 25-second vertical editorial reel, actual workflow recording, editable project, compatibility archive and installable tarball.

Video source, transcripts and selected context are sent to the chosen coding-agent provider when requested. Fish is optional for speech/transcription/cloning and uses a server-side user key; registration does not transcribe automatically. Keys and server credentials are excluded from demo deliveries. Fish and Claude live acceptance are reported separately. Remotion framework licensing is separate from the motion-source licenses.

## Editing and longer productions

Agent calls use focused tool profiles for source authoring, targeted scene edits, footage, or voice. Long-form plans support chapter summaries and per-chapter frame scores. Generate selected chapter constrains a turn to that chapter; unrelated scenes and score events remain protected. Large source files can be read in line windows. See [the authoring contract](docs/SCENE-AUTHORING.md). The one-hour planning limit is not a measured one-hour export or generation guarantee.

The delivered reel and workflow recording demonstrate real source generation, a frame-range revision, human property/source edits and export. Live Claude and Fish acceptance remain separate from verified Codex/local functionality. Current evidence is in [verification](docs/VERIFICATION.md); earlier 0.2 reports describe the removed library and are historical.
