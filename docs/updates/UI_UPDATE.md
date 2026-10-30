# ClipKit Studio: UI Update Prompt (Pro Editor Interface)

A prompt for Claude Code to rebuild the localhost studio as a professional editor in the style of desktop nonlinear editors and motion-graphics tools: dockable panels, a multi-track timeline with real editing tools, source and program monitors, keyframes, audio mixing, plus ClipKit's own agent, brief, comment, draft and voice features. It also defines fallbacks so the UI keeps working when parts of the system fail.

Save this document as docs/updates/UI\_UPDATE.md, then use the kickoff message in the last section.

## 0. Rules for this update

1. Another agent session may be working on Update 5. Do not run this update at the same time on the same branch. Either wait until the current phase is committed and the loop is stopped, or work on a separate branch (ui-pro) in a separate worktree and merge when both are green. Never edit files another running session is changing.
2. Start with an audit (UI-0), no code changes: read the studio code, the generated TypeScript types, the server's HTTP and WebSocket API, PROGRESS.md, DECISIONS.md and VERIFICATION.md. Write docs/updates/UI\_AUDIT.md listing what exists, what is broken when used, and what is missing against this prompt. Open the studio in a real browser with Playwright, try the main flows, and record every broken interaction with a screenshot.
3. The Rust server stays the source of truth. The UI never writes project files; every edit is a patch through the server, exactly as today. Keep all existing CLI, MCP and agent behavior working.
4. Reuse the existing studio stack (React, Vite, generated types, the preview engine chosen in Update 5). Add libraries only when they clearly save work, check their license and current version, and record each in DECISIONS.md.
5. The look is inspired by professional desktop editors (panel layout, dark theme, timeline conventions), but every visual is original. Do not use Adobe names, logos, icons, splash art, color schemes copied exactly, or any assets from Adobe products. Use an open icon set (for example Lucide or Tabler, after a license check).
6. Function over decoration: every control shown must work. Hide or disable anything not yet implemented, with a tooltip saying so. No dead buttons.
7. Commit after each UI phase with tests green, screenshots stored, and evidence in VERIFICATION.md.

## 1. Workspace layout and panel system

The studio is a full-window, dark, dense workspace made of dockable panels, like a desktop editor. Nothing scrolls the whole page; each panel scrolls on its own.

### 1.1 Default layout (Editing workspace)

```
+--------------------------------------------------------------------------------+
| Menu bar | project name | workspace tabs | agent status pill | tokens | export  |
+------------------+-------------------------------+-----------------------------+
| Project / Media  |  Source monitor | Program mon. |  Inspector (Effect Controls) |
| bin, Assets,     |  (tabs)                       |  Brief, Agent, Comments      |
| Transcript       |                               |  (tabs)                      |
+------------------+-------------------------------+-----------------------------+
| Tools |  Timeline: ruler, beat grid, markers, tracks V3..V1, A1..A4, voice,    |
| strip |  captions lanes, energy curve lane, playhead                | Audio    |
|       |                                                              | meters   |
+--------------------------------------------------------------------------------+
| Status bar: connection, engine, render queue, proxies, selection info, rev     |
+--------------------------------------------------------------------------------+
```

### 1.2 Panel system

- Every panel can be docked, tabbed with others, resized with splitters, maximized (backtick toggles the panel under the cursor), floated, closed and reopened from the Window menu.
- Layouts are saved per workspace in local storage (try and catch around every read and write) and also in the project settings on the server so they survive a new browser. Reset workspace restores the default.
- Minimum sizes prevent panels from collapsing to nothing; at narrow widths panels tab together instead of overlapping.
- Use a proven docking library if one fits the license and stack (verify options such as dockview or flexlayout-react); otherwise build a simple split-and-tab system.

### 1.3 Workspaces (one click in the top bar)

| Workspace | Emphasis |
| --- | --- |
| Edit | Bin, monitors, timeline, inspector (default) |
| Generate | Brief, storyboard, recipe and block library, program monitor, timeline |
| Motion | Composition panel (layer list with property twirl-downs), keyframe graph, program monitor |
| Audio | Mixer, waveform timeline, voice lane, sound kit, loudness meters |
| Captions | Caption list, presets, program monitor with safe areas, transcript |
| Review | Program monitor, comments, draft review bar, history, version compare |
| Color | Program monitor, look and LUT panel, scopes |

### 1.4 Panels to build

Editor panels: Project and media bin, Source monitor, Program monitor, Timeline, Tools, Inspector (effect controls with keyframes), Composition (layer stack for one clip or recipe), Keyframe graph, Audio mixer and meters, Assets library (clips, recipes, kinetic blocks, captions, SFX, music, LUTs, fonts), Markers, History, Scopes (waveform and vectorscope, P2).

ClipKit panels: Agent, Brief, Storyboard and beat sheet, Comments, Draft review, Transcript, Voice lane and recorder, Styles and moods, Verification (gates, energy, playback), Exports and render queue, Sources (Source Hub, P2).

## 2. Timeline

The timeline must feel like a real editor: precise, fast and predictable. It is drawn on canvas (or WebGL) for clips, waveforms and thumbnails, with DOM only for headers, menus and text inputs, so it stays at 60 frames per second with hundreds of clips.

### 2.1 Structure

- Time ruler with timecode (HH:MM:SS:FF) or frames, switchable; work area or in and out range bar; markers row (user markers, comment pins, agent markers, beat downbeats, drop markers).
- Video tracks (V1 at the bottom, higher numbers above) and audio tracks (A1 and down), plus special lanes: voice slots, captions, and an optional energy curve lane.
- Track headers: name (rename by double-click), eye or mute, solo for audio, lock, target toggle (which track receives inserts), track height (small, medium, large), color label, collapse.
- Clips show name, color by type (footage, generated clip, kinetic block, caption, voice, music, SFX), thumbnails at the start and end (or filmstrip at large height), waveforms on audio, a keyframe row when expanded, and badges: draft (agent), locked by agent, offline media, missing license, effect applied.
- Playhead with a timecode field you can type into; the current-time line spans all tracks.

### 2.2 Tools (Tools strip, with single-key shortcuts)

| Tool | Key | Behavior |
| --- | --- | --- |
| Selection | V | Select, move, trim edges; Shift adds to selection; drag a box to select many |
| Track select forward | A | Select everything to the right on a track (Shift: all tracks) |
| Ripple edit | B | Trim an edge and move everything after it |
| Rolling edit | N | Move a cut point between two clips |
| Rate stretch | R | Change clip speed by dragging its edge |
| Razor | C | Split at click; Shift splits all tracks |
| Slip | Y | Change in and out inside a clip without moving it |
| Slide | U | Move a clip and adjust its neighbors |
| Hand | H | Pan the timeline |
| Zoom | Z | Click to zoom in, Alt-click out |
| Text | T | Add a text layer at the playhead in the program monitor |

Shortcut keys are suggestions; the user can remap them, and all actions are also in menus and the command palette.

### 2.3 Editing operations

- Insert and overwrite edits from the source monitor or bin (comma and period), lift and extract with in and out points, ripple delete, close gap, nudge by one frame (arrow keys) or ten frames (Shift).
- Snapping (S toggles) to playhead, clip edges, markers, in and out points, and beats or downbeats when a beat grid exists.
- Linked audio and video move together; Alt (Option) temporarily unlinks.
- Copy, paste, paste attributes (effects and keyframes), duplicate, group, nest a selection into a compound clip (P2).
- Transitions dragged from the library onto cuts; default transition shortcut.
- J, K, L playback shuttle; I and O set in and out; space play and pause; Home and End; up and down arrows jump between edit points.
- Zoom with Ctrl or Cmd and scroll, minus and plus keys, and backslash to fit the sequence.

### 2.4 How edits flow

- Every gesture builds one patch sent on release (a drag is one undo step). The UI shows the change immediately (optimistic), then reconciles with the server's revision. If the server refuses (lock, stale revision, schema), the change rolls back with a short toast that says why.
- Undo and redo go through the server and include agent revisions, labeled by author in the History panel.

### 2.5 ClipKit overlays on the timeline

- Agent ghost cursor and touched-clip highlight while the agent works; agent-locked clips show a lock badge and refuse edits.
- Draft overlay: when an agent draft exists, changed clips show before and after outlines; the review bar sits above the timeline.
- Beat grid lines (faint for beats, stronger for downbeats, colored drop markers) with a toggle.
- Energy curve lane with failing spans tinted, clickable to jump there.
- Comment pins on the markers row; clicking opens the thread in the Comments panel.

## 3. Monitors, inspector and keyframes

### 3.1 Program monitor

- Plays the sequence through the preview engine at a chosen playback resolution (full, half, quarter) and zoom (fit, 50, 100, 200 percent).
- Transport: play and pause, step frame back and forward, go to in and out, loop in to out, timecode display and entry, dropped-frame indicator.
- Overlays (toggle menu): title and action safe areas, platform UI zones for vertical video, rule-of-thirds grid, face boxes, hover-select outlines, layer bounding boxes.
- Direct manipulation: select a layer or element in the frame (hover-select), drag to move, corner handles to scale, rotate handle, double-click text to edit in place. Each gesture is one patch.
- Ask agent from a selection: right-click or a small button opens a prompt anchored to that element.
- Comparison: split view or toggle between draft and main, and between two branches.

### 3.2 Source monitor

- Shows a bin item before it is in the sequence: set in and out, mark, then insert or overwrite, or drag to the timeline. Video-only and audio-only drag handles.
- For generated clips and kinetic blocks, the source monitor shows a live preview with its slots filled from the current inspector values.

### 3.3 Inspector (Effect Controls)

- Shows the selected clip's properties grouped: Motion (position, scale, rotation, anchor, opacity), Time (speed, reverse), Audio (volume, pan, ducking), the clip's own schema props (generated from its props schema), applied effects in order, and ClipKit metadata (purpose, pass, asset source, license, briefDeps).
- Every animatable property has a stopwatch toggle to enable keyframes, previous and next keyframe buttons, add or remove keyframe at the playhead, and a mini keyframe lane aligned with the clip's time range.
- Scrubby number fields (drag left and right to change, Shift for bigger steps), color pickers using design tokens first, font pickers limited to licensed fonts.
- Effects: drag from the library, reorder by dragging, toggle on and off, reset, copy and paste.
- Multi-selection edits shared properties at once.

### 3.4 Composition panel and keyframe graph (motion-tool style)

- For a generated clip, recipe or kinetic block, the Composition panel shows its internal layers as a list with twirl-down properties, solo and visibility switches, parenting, and per-layer time bars, like a motion-graphics layer stack.
- The Keyframe graph shows value or speed graphs with bezier handles and easing presets (linear, ease in, ease out, ease in-out, the look's motion personality curves), plus easy-ease shortcuts.
- Edits inside a composition are patches to that clip's props or derived component overrides; the original asset is never changed.

### 3.5 Audio

- Mixer with a strip per audio track and the voice lane: fader, pan, mute, solo, peak and loudness meters (LUFS short-term and integrated) and a master strip with the platform target line.
- Clip gain and volume keyframes on the timeline (rubber band), ducking indicators where music ducks under voice.
- Waveforms at all zoom levels, cached as peak files by the server.

## 4. Bin, library and ClipKit panels

### 4.1 Project and media bin

- List and thumbnail views, folders (bins), sort and search, metadata columns (duration, resolution, fps, codec, proxy status, transcript status, license).
- Import by drag and drop or a file button; progress per file for probe, proxy, waveform and transcript; hover scrubbing on thumbnails.
- Status badges: proxy ready, transcribing, offline (missing file, with Relink), unsupported.

### 4.2 Assets library

- Tabs: Recipes, Kinetic blocks, Clips, Captions, Transitions, Effects and looks, SFX, Music, LUTs, Fonts, Icons.
- Search uses the same catalog as the agent; filters for aspect, license mode, grade and collection. Each item shows a looping preview (poster image while loading), license badge and one-line use-when text.
- Drag onto the timeline or program monitor to add; the recipe expander asks for slots in a small form.

### 4.3 Agent panel

- Status (idle, waiting, working), agent name, current stage and pass, a plain-words label of what it is doing, and token use for the turn.
- Activity feed: one line per step, clickable to select the affected layers.
- Controls: Stop after current step; Interrupt (type or hold to speak a redirect); Ask (send a free-text request anchored to the current selection or playhead).
- Shows which agent is connected (Claude Code, Codex, other) and the event bridge state.

### 4.4 Brief, storyboard and styles

- Brief panel: the confirmed brief as editable fields with source tags; changing a field shows the impact preview (which layers will change) before sending.
- Tuning chips (punchier, shorter, calmer, more premium) and sliders for pacing, energy and density.
- Storyboard and beat sheet: beat strip under the music waveform; cards per block with recipe, text and duration; drag to reorder.
- Styles and moods: picker with previews; Save as style on a finished project.

### 4.5 Comments and draft review

- Comments panel: threads with anchors (time, range, layer, element), filters (open, resolved, mine, agent), Send to agent, before and after previews on resolved threads.
- Draft review bar (above the timeline when a draft exists): grouped changes, Accept all, Reject all, per-change toggles, conflicts flagged; Accept creates one undoable revision.

### 4.6 Transcript, voice and verification

- Transcript panel: words with timestamps, click to seek, select and delete to cut, filler and silence highlighting, speaker labels, search.
- Voice lane and recorder: slots with AI, REC and CLONE badges, teleprompter record-along with countdown, takes list with waveform and trim handles, mode switch per slot, consent notice for cloning.
- Verification panel: quality gates, energy measures, playback check results, each with pass or fail and a jump-to link.
- Exports and render queue: presets, progress, output links, license report.

## 5. Agent working state and turn-taking in the UI

- A status pill in the top bar always shows idle, waiting or working. While working, a thin animated bar runs under the top bar and the agent's label appears in the pill.
- Locked layers show a lock badge and a tinted overlay; any attempt to edit them shows a toast (The agent is editing this layer; it will be yours when it finishes) and does nothing. The server enforces this; the UI only explains it.
- Unlocked layers remain fully editable. Global (structural) locks dim the timeline and tools, keep playback, scrubbing, comments and the Agent panel usable.
- Edits the user makes while the agent works never block on the agent; their patches go straight to main or are queued if they touch a draft.
- When the agent finishes, a short toast says what changed, with buttons Review draft and Undo all.
- The browser tab title shows the state (for example: Working - ClipKit) so the user notices in another tab.

## 6. Menus, command palette and shortcuts

- Menu bar: File (new, open, import, save version, export, project settings), Edit (undo, redo, cut, copy, paste, paste attributes, select all, preferences, keyboard shortcuts), Clip (speed, link and unlink, group, enable), Sequence (render preview, add edit, ripple delete, snapping, beat snapping, markers), Markers, Agent (ask, stop, interrupt, review draft, brief), Window (panels and workspaces), Help (shortcuts, docs, diagnostics).
- Command palette (Ctrl or Cmd plus K): every action searchable by name with its shortcut.
- Context menus on clips, tracks, bin items, keyframes and the monitors.
- Keyboard shortcut editor with a visual keyboard map, presets (ClipKit default, and a layout familiar to users of common desktop editors described generically), conflict detection, import and export.
- Shortcuts never fire while typing in a text field; a shortcut that targets a panel works only when that panel has focus, with a visible focus outline on the active panel.

## 7. Visual design system

A calm, dense, dark professional look that keeps attention on the video. Define everything as tokens in one theme file; no hard-coded colors or sizes in components.

- Surfaces: four dark grey levels for app background, panels, raised elements and hover (for example around #1a1a1a, #232323, #2b2b2b, #333333; tune by eye), one hairline border color, and a subtle focus ring.
- Accent: one original brand accent for selection, playhead and primary buttons. Semantic colors: success, warning, error, info. Agent color (distinct from the user's selection color) for ghost cursor, draft outlines and agent badges.
- Clip colors by type, muted so thumbnails stay readable; user-assignable label colors.
- Typography: one UI font (for example Inter or a system font stack), 11 to 13 px for dense panels, tabular numbers for timecode and values; a monospace font for timecode fields.
- Density: compact by default (24 to 28 px rows, 4 px spacing grid), with a comfortable density option.
- Icons: one open icon set, 16 px, consistent stroke; every icon button has a tooltip with the shortcut.
- Motion in the UI is short (100 to 150 ms) and never delays input; respect reduced-motion settings.
- Light theme optional (P2); high-contrast theme for accessibility.
- Empty states that teach: an empty bin explains drag and drop; an empty timeline offers Drop footage, Start from a brief, or Browse recipes.

## 8. Fallbacks and resilience

The UI must never go blank or lose work. Each failure has a defined fallback, a visible message that says what to do, and a test.

| Failure | What the UI does |
| --- | --- |
| Server unreachable at load | Full-window message with retry countdown, the port it tried, and the command to start it (ck studio); keeps retrying with backoff |
| WebSocket drops mid-session | Status bar turns amber (Reconnecting); edits are queued locally up to a limit; on reconnect, fetch missed revisions and replay queued patches against the new base; conflicts shown in a small dialog |
| Long disconnect | Editing switches to read-only after the queue limit, with a banner; nothing is discarded silently |
| Server refuses a patch | Roll back the optimistic change; toast with the reason and error code (locked, stale, invalid) |
| Preview engine fails to load | Program monitor falls back to server-rendered poster frames at the playhead (scrubbing shows frames on demand); banner with a Retry and a link to diagnostics |
| A single layer fails to render | Show a red placeholder box with the layer name and error in that layer's area; the rest of the frame renders; layer flagged in the timeline |
| Playback cannot keep up | Drop to lower playback resolution automatically, show the dropped-frame indicator, suggest proxies |
| Proxy not ready | Use the original media at reduced resolution, badge Proxy building, swap when ready |
| Media offline (file moved or deleted) | Red offline clip with Relink; export blocked with a clear list of offline items |
| Unsupported codec in the browser | Use the server proxy; if none, show a poster frame and a Create proxy action |
| WebGL or canvas acceleration unavailable | Fall back to 2D canvas for the timeline and to CSS-only effects in preview with a notice; 3D blocks show their 2D fallback |
| Low memory or very large project | Virtualize everything, unload thumbnails and waveforms off-screen, reduce thumbnail density, warn before opening very large projects |
| Microphone denied or missing | Voice controls show a permission hint and fall back to typed commands; record-along disabled with a reason |
| Voice provider down | Use cached audio or the labeled placeholder; show the provider status in the voice lane |
| Agent disconnected or crashed | Agent pill shows Not connected; locks auto-expire after the timeout and the UI unlocks with a notice; draft kept for review |
| Brief or ck wait timeout | Brief page stays editable; a banner says the agent stopped waiting and how to resume |
| Local storage unavailable or full | Use in-memory layout and warn once; layouts saved on the server instead |
| Unsupported browser | Startup check for required features; show a supported-browsers message rather than a broken app |
| A panel throws an error | Error boundary per panel: the panel shows a compact error card with Reload panel and Copy diagnostics; other panels keep working |
| Whole app crashes | Top-level boundary with Reload, Safe mode (default layout, no previews, no extensions) and Copy diagnostics; no project data lives only in the browser, so nothing is lost |
| Clock or version mismatch with the server | On version mismatch, prompt to reload the studio; never apply patches across incompatible versions |

Also: autosave is inherent (every edit is a server revision), so there is no Save button risk; a Save version action creates a named snapshot. Diagnostics (Help menu) shows connection, engine, versions, recent errors with codes, and a button to create a trace bundle without secrets.

## 9. Performance, architecture and accessibility

### 9.1 Performance budgets (measured in tests on a mid-range laptop)

| Measure | Budget |
| --- | --- |
| Studio first load to interactive | Under 2.5 s warm |
| Timeline scroll and zoom | 60 fps with 300 clips |
| Scrub latency (playhead move to frame shown) | Under 100 ms with proxies |
| Patch round trip (gesture end to server-confirmed state) | Under 150 ms locally |
| Agent patch to visible change | Under 300 ms |
| Memory after 30 minutes of editing | No unbounded growth |

Techniques: canvas or WebGL timeline with virtualization, thumbnails and waveform peaks generated by the server and fetched by tile, web workers for heavy UI computation, memoized selectors, no full-project re-render on each patch (apply patches to a normalized store and update only affected components), requestAnimationFrame for playhead updates.

### 9.2 Architecture

- One normalized client store (for example Zustand or Redux Toolkit; pick what the studio already uses) mirroring the server project by id; patches from the server are applied to it; the UI derives views with selectors.
- A command layer: every user action is a named command (used by menus, shortcuts, palette and buttons), which builds a patch, applies it optimistically, sends it, and handles refusal. This makes actions testable and keeps shortcuts consistent.
- A panel registry: each panel declares id, title, icon, default workspace placement and required capabilities, so workspaces and the Window menu are generated.
- Types are generated from Rust as today; no hand-written copies.

### 9.3 Accessibility

- Full keyboard operation of menus, panels, bin, inspector and the timeline (move selection between clips, trim with keys).
- Visible focus, ARIA roles and labels on controls, live region announcements for agent status changes and errors.
- Contrast that meets common guidelines for text and controls; high-contrast theme; reduced motion respected.

## 10. Testing, phases and acceptance

### 10.1 Testing

- Unit tests for every command (patch built, optimistic apply, rollback on refusal).
- Playwright end-to-end flows: import media, make a rough cut with each tool, keyframe a property, add a recipe, record a voice take (with fake media stream), pin a comment, review a draft, export. Each flow runs three times in a row.
- Fallback tests: for every row in the section 8 table, simulate the failure (kill the server, drop the socket, block WebGL, deny the microphone, delete a media file, throw in a panel) and assert the documented behavior.
- Visual regression snapshots of each workspace at 1440 and 1920 px wide, dark and high-contrast themes.
- Performance tests for the budgets in section 9.1, with results recorded per commit.
- An agent-in-the-loop test: the agent edits while Playwright tries to edit locked and unlocked layers, asserting the turn-taking behavior.

### 10.2 Phases

| Phase | Delivers |
| --- | --- |
| UI-0 | Audit with screenshots of broken flows; no code changes |
| UI-1 | Theme tokens, panel and docking system, workspaces, menu bar, command layer, command palette, status bar, error boundaries |
| UI-2 | Timeline core: canvas rendering, tracks and headers, selection, move, trim, razor, ripple, snapping, zoom, JKL, markers, undo through the server |
| UI-3 | Program and source monitors with overlays, direct manipulation, hover-select, playback resolution |
| UI-4 | Inspector with keyframes, composition panel, keyframe graph |
| UI-5 | Bin, assets library, transcript, audio mixer and meters, waveforms |
| UI-6 | ClipKit panels: Agent, Brief, storyboard, comments, draft review, voice lane, verification, exports |
| UI-7 | Fallbacks from section 8, diagnostics, safe mode |
| UI-8 | Shortcut editor, accessibility, performance tuning, visual polish, full test pass |

Cut line if time runs short: scopes, compound clips, the light theme and the shortcut editor go first. Never cut: the timeline tools in UI-2, monitors, the inspector, the agent and draft panels, or the fallbacks.

### 10.3 Acceptance

- A new user can import a clip, cut it, add a title and a caption preset, adjust a keyframe and export without reading docs.
- While the agent works, the user sees what it is doing, cannot edit locked layers, can edit others, and can stop or redirect it.
- Every row in the fallback table passes its test.
- Performance budgets met; no console errors in the main flows; every visible control works or is clearly disabled.
- No Adobe names, logos, icons or copied assets anywhere in the UI.

## 11. Kickoff message

Save this document as docs/updates/UI\_UPDATE.md. If your other session is still running Update 5, either wait for it to commit and stop, or start this in a separate worktree on a ui-pro branch. Then paste into Claude Code:

```
UI UPDATE. Read docs/updates/UI_UPDATE.md section 0 first, then only the sections each phase needs.

Work on branch ui-pro in its own git worktree so you never touch files the other session is editing. Do not change the Rust server's behavior except to add endpoints the UI needs (thumbnail and waveform tiles, layout storage); coordinate those through small, separate commits.

Start with UI-0: open the current studio with Playwright, try every main flow, and write docs/updates/UI_AUDIT.md with what works, what is broken (with screenshots) and what is missing. No code changes. Show me the audit and your plan for UI-1, then continue phase by phase (UI-1 to UI-8), committing after each with tests green, screenshots stored and evidence in VERIFICATION.md.

Rules: every visible control must work or be clearly disabled; every edit is a patch through the server; implement and test every fallback in section 8; the design is original (no Adobe names, logos, icons or assets); check licenses of any new library and record them in DECISIONS.md.
```

When both branches are green, merge ui-pro into the main branch, run the full test suite and the agent-in-the-loop test, and fix conflicts before restarting the autobuild loop.
