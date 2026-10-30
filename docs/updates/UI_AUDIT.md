# ClipKit Studio UI audit — UI-0

Date: 2026-10-30. Baseline: `af6c2d5` on main. UI work is isolated in branch `codex/ui-pro` at `/Users/sanidhyavats/Desktop/clipkit-studio-ui-pro`.

The supplied UI prompt is retained in `UI_UPDATE.md` as requirements and reference material. Its kickoff text is not separate authorization to change protected specs, merge into another agent's branch, or select Update 5's preview engine. The owner approved the UI-1 plan in this chat after the audit/plan work began. This task follows the repository's one-phase rule.

## Isolation

Main had existing changes in `crates/ck-media/src/lib.rs`, untracked `engine.rs`, `spike/`, `docs/updates/`, and progress/decision/evidence files. Those are Update 5 work and have not been modified, staged, reset or merged here. The current committed preview is Remotion; its composition, dependencies, worker and Rust APIs remain unchanged by UI-1.

## What was verified

Source inspected: App, API client, state reducer, timeline, composition, existing panels, CLI/browser fixtures, Rust project model, HTTP route registry, WebSocket handler and studio static serving. Generated bindings were consumed by TypeScript, not manually read or modified (AGENTS.md).

Baseline TypeScript and all 37 frontend unit tests passed using the bundled Node runtime. The shell initially had no Node on PATH; bundled Node is available at the path recorded in VERIFICATION.md.

A real headless Chrome browser opened the existing built UI at localhost. Since no Rust toolchain, `just`, ffmpeg or built `ck` binary was present, a Playwright-simulated project snapshot and API replies were used to inspect the baseline layout and selection. These are frontend observations, **not real-server end-to-end proof**. No user project, original media or taste profile was changed.

Screenshots and machine-readable measurements:

- `ui-evidence/before-no-token.png`: session-token guidance works.
- `ui-evidence/before-1440.png`: sidebar exceeds the viewport.
- `ui-evidence/before-selection.png`: a timeline clip can select its basic inspector in the simulated project.
- `ui-evidence/baseline-browser.json`: viewport 1440×900; document height 1236px. Comments start at y=891, History at y=1061 and Activity at y=1211. Lower controls require page scrolling, contrary to the prompt's full-window panel requirement. The JSON's simulated-edit field does not establish server success or refusal.

## Existing capabilities and gaps

| Area | Existing implementation | Gaps against UI prompt |
| --- | --- | --- |
| Workspace | Header, preview, vertically stacked sidebar, DOM timeline | No workspaces, docking, independent panel scrolling, menu bar, palette or status bar. Confirmed overflow above. |
| Sync | Authenticated API calls, WebSocket snapshot/lock/revision/question reducer, reconnect | Whole snapshots; no normalized selector store, version-negotiation UI, pending edits/optimistic command reconciliation. |
| Timeline | Time-positioned clip buttons, selection, locks/agent overlays, comment pins | No canvas, ruler/playhead, move/trim/razor/ripple/snapping/zoom, track tools, waveforms or keyframe lanes. UI-2 onward. |
| Program | Remotion Player, core clips, proxies, hover selection, element-anchored Ask agent | No source monitor, overlay menus, transform handles, comparison or playback-resolution UI. Monitor fit must be verified at large sizes. |
| Inspector | Name, purpose and timing fields; revisioned `/api/patch` requests | No schema property editor, motion/audio controls, keyframes, effect stack, composition or graph. |
| Project/media | Sources exist in project model; `/api/import` endpoint | No bin, import/progress/relink/source-preview UI. File chooser cannot assume a browser path is a server filesystem path. |
| Brief/styles | Editable essentials, source labels, looks, questions, tuning impact | Functional components exist; only one stacked view. Advanced brief/storyboard editing remains absent. |
| Agent | Status banner, stop route, feed, questions, selection/playhead context, typed and spoken commands | No reliable agent identity/token meter data in current model; avoid invented values. Draft review absent. |
| Transcript/voice | Transcript seek/delete requests, voice modes, recording countdown/takes/consent/captions | Requires real source/provider/recording fixtures for integration. No track mixer, waveforms or loudness meters. |
| Comments/history | Moment/layer/element anchors, requests, undo/redo, branch save/switch | Range selection not supplied by timeline; draft/version comparison absent. |
| Export/verification | Server endpoints and CLI implementation | No export queue or quality-gate UI; browser export acceptance cannot be claimed. |
| Resilience/accessibility | Reconnect, expiring layer locks, microphone errors, basic labels | No panel boundaries, storage fallback, diagnostics, safe mode, high-contrast workspace, menu/palette keyboard system. |

## Server integration inventory

Existing routes cover `/api/patch`, `/api/context`, `/api/undo`, `/api/redo`, `/api/split`, `/api/stop`, brief/looks, comments, questions, storyboard, transcript/cut, voice, history/branches, import, export and verify. `/ws` streams project/lock/revision state. Existing edits are retained through those operations.

No layout API, waveform tiles, thumbnail tiles, render queue, source-bin catalog HTTP search, transform/keyframe schema or engine capability-negotiation route was found. Settings currently contain fps, width and height. Browser preferences must not be patched into unsupported fields. Server layout persistence and any new endpoints need separate coordinated capability work.

## Proposed/approved UI-1 plan

1. Original dark theme tokens, high contrast and density options; full-window shell.
2. Simple split/tab docking with resizing, close/reopen, maximize and workspace presets. Preserve mounted forms when moving panels. Floating and server layout persistence remain explicit gaps.
3. Reuse every existing panel and current preview engine; add a project/source inventory from real project state, without inventing import workflows.
4. Named shell commands, menus, searchable keyboard-accessible palette, connection/agent/revision status and panel/top-level boundaries.
5. Keep revisioned patch, WebSocket and lock behavior; expose unsupported commands as disabled with reasons. Add frontend contract coverage and update existing real-server tests for tabs.
6. Record browser screenshots at 1440/1920 in dark/high contrast; run available checks; mark backend/export/real-agent tests BLOCKED until prerequisites exist.

UI-2 through UI-8 are not implemented by UI-1. Full acceptance, performance budgets and every section-8 failure test remain unverified.
