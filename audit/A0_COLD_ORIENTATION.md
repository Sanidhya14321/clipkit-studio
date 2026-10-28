# A0 — Cold Orientation (fresh-eyes baseline)

Auditor: Independent Audit Agent (GLM-5.3-Flash via OpenCode).
Date: 2026-10-28. Machine: Linux, Node v24.21.0, pnpm not on PATH (node_modules already present).
No spec or builder files were read before this document, EXCEPT: package.json, README.md (A1 input),
and source code. Builder files (VERIFICATION.md, DECISIONS.md, UPDATE*_AUDIT.md, docs/VERIFICATION.md)
are quarantined until after A0+A1.

## What I believe the product is (before reading any spec)

ClipKit Studio is a **local, agent-operable video editor**:

- **What it is**: A localhost web studio (React + Remotion) that renders video timelines; a CLI
  (`ck`) and MCP server that expose the same editing operations to coding agents (Codex, Claude Code);
  a single HTTP server (`ck studio` on 127.0.0.1:4310) that owns a JSON project file under `.clipkit/`.
- **Who it is for**: Creators/hackathon teams who want a coding agent to produce/edit short vertical
  social videos (kinetic text, footage edits, narration) with human review, locally, without SaaS.
- **How it works (as observed in code)**:
  - `package.json` → workspace of 10 packages: core (project schema/patch store), server (Hono + WS,
    one giant op registry `server/src/index.ts`), studio (React UI, single 3,348-line `main.tsx`),
    cli (159-line dispatcher hitting `/api/op/:name`), mcp, assets (knowledge cards, source hub,
    vetting), render (Remotion export + layer cache), scenes, motion, debug (flight recorder,
    traces, doctor, bisect fixtures).
  - Project = single JSON doc revised via JSON patches with baseRev, lease/lock tokens for agent
    turns, history/undo, branches, versions.
  - Media: ffmpeg-static bundled; Fish Audio used for voice (TTS/transcribe/clone) behind
    FISH_API_KEY; recordings/upload endpoints.
  - "Source Hub": sources.yaml registry with license classification/grades, collections,
    commercial-safe license mode, vetting of third-party packs (postinstall fixtures must not run).
  - Additional subsystems glimpsed: Brief Studio (creative brief, waits for user answers), pipeline
    modes EDIT/HYBRID/GENERATE, direction engine (looks/tokens/recipes/layout/variants/sound),
    taste profile, debug/eval/bench harnesses.

### Observed scale facts (measured)
| Package | Files | LOC (src) |
|---|---|---|
| assets | 24 | 535 |
| cli | 1 | 159 (wrapped, real width far larger: 9 lines > 2,000 chars in server file) |
| core | 12 | 350 |
| debug | 15 | 214 |
| mcp | 9 | 246 |
| motion | 3 | 41 |
| render | 14 | 205 |
| scenes | 2 | 31 |
| server | 22 | 1,065 (single file `index.ts` is 118KB/344 lines; 8 lines > 2,000 chars) |
| studio | 16 | 3,512 (single `main.tsx` of 3,348 lines) |
| **Total** | 120 | **~6,374** |

Tests at `tests/*.test.ts` (~39 files) + e2e/integration + Playwright config.

### Structural red flags noted cold (to verify later)
1. Two giant single files carry most behavior (`server/src/index.ts`, `studio/main.tsx`) —
   maintainability risk, lines > 2–5k chars.
2. Only ~6.4k lines implement a very long feature list (briefs, direction engine, voice lanes,
   locks, source hub, debug harness…) — suspicion of thin implementations or heavy reliance on
   the agent; needs Wizard-of-Oz detection.
3. Version pins look unusual: `typescript: 7.0.2`, `react: 19.3.1`, `zod 4.6.7`, remotion 4.0.538 —
   verify toolchain actually typechecks.
4. Two pre-existing dirty files in git (`packages/mcp/src/compact.ts`, `packages/studio/src/LibraryPanel.tsx`)
   — user modifications; audit must not touch or judge them as findings without checking.
5. `ffmpeg` is NOT on PATH; product bundles `ffmpeg-static` — install experience depends on this
   being handled (A1 will verify).

## Differences found after reading specs — populated later (A2+)
(Placeholder; fills after reading docs/UPDATE2.md, UPDATE4.md and spec sources.)
