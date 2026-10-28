# A1 — First-Run Experience (clean sandbox)

Sandbox: /tmp/opencode/firstrun/proj — a copy of the repo WITHOUT node_modules/.git/.clipkit/dist/artifacts.
Commands timestamped; raw logs in /tmp/opencode/firstrun/.

## Metric-by-metric log (new user following only README.md)

| Step | Command | Result | Time |
|---|---|---|---|
| Install | `pnpm install` (via `npx -y pnpm@11.25.0`) | OK, 0 errors | 22 s |
| Build | `pnpm build` | OK (studio 851 KB bundle + CLI + runtime + package) | 41 s |
| Start | `pnpm dev` | First failure: port 4310 already used by a *different* ClipKit project on the same machine → `EADDRINUSE`, hint says "Run ck doctor" (wrong hint for a port conflict; README never mentions `--port`) | — |
| Start (retry, `CLIPKIT_PORT=4311`) | `pnpm dev` | OK: API server 4311 + Vite dev server 5173 | ~15 s to both 200 |
| `pnpm doctor` | **runs pnpm's builtin doctor, not ClipKit's** (versions/cache/registry checks) — confusing duplicate name | <1 s |
| `ck doctor` (real) | ok:false expected in sandbox: codex/claude adapters missing, provider key not configured; ffmpeg ✓ (static), catalog 21 records | — |
| `ck init` | wrote AGENTS.md block, .mcp.json, .codex/config.toml, .claude/skills/clipkit/SKILL.md, .cursor/rules/clipkit.mdc, assets/README.md; second run idempotent | fast |
| `pnpm demo:reset` | **FAILS on fresh project**: "This historical demo needs the separate 0.2 compatibility archive. Create an original reel from Motion instead." — no shipped way to get the archive on a clean machine | — |
| `ck uninstall` | removed AGENTS.md, .codex/config.toml, .cursor/rules/clipkit.mdc, .mcp.json, .claude/skills/clipkit; left EMPTY dirs .codex/, .cursor/, .claude/; project preserved | good, minor residue |

## Time to first working preview (non-editor creator persona)
- UI open (dev): ~5 s, zero console errors at open.
- **BUT the component registry 404s in dev mode** → placing a component puts it on the
  timeline yet the player stays blank and the console shows `404 /components/<hash>/registry.js`
  + pageerror `Trusted component registry could not load` (screenshots: ui-start.png shows placed
  Word Punch clips + empty player; ui-card.png).
- Root cause observed by direct probes: `public/components/<hash>/registry.js` EXISTS and the API
  server serves it (HTTP 200 on origin 4311), but `packages/studio/vite.config.ts` proxy list omits
  `/components` (also `/component-fonts`, `/scene-artifacts`, `/layer-cache`, `/video-reviews`,
  `/bench-media`, `/bench-artifacts`) → dev server 404s.
- Working preview is reachable following the README **only implicitly**: after `pnpm build`, open
  the API server origin (`http://127.0.0.1:4311` serves `dist/studio`) — in packaged mode the same
  flow renders "Into." and property editing works with ZERO console errors (packaged-start.png,
  packaged-playing.png) → time-to-working-preview ≈ 65–75 s including `pnpm build`.
- The README never says "open the API origin for the built UI"; it presents `pnpm dev`, which is
  the broken preview path → developer-experience defect.

## Persona narratives
1. **Non-editor creator** (packaged mode): insert a bundled component ✓, edit text/size in Properties ✓
   (screenshot packaged-playing.png), undo/pencil affordances visible. Voice/Brief flows need
   agent+keys; not testable here (disclosed as unverified elsewhere).
2. **Professional editor**: timeline scrub, In/Out fields, frame comments, split/duplicate/delete, z
   bands. No NLE import (FCPXML/EDL explicitly not implemented in spec; honest).
3. **Developer using Codex/Claude**: `ck init` smooth; but in dev server mode the MCP server config
   points at `tsx` dev CLI (works only inside this checkout); component previews broken in dev; web
   pack caches under OS cache dir (disclosed in README).
4. **Hackathon judge, 2 minutes**: README is honest about limits but the "Fast track" is 3 commands
   and one build away from a real editor with 12 original components — impressive IF they follow
   `pnpm build` then open 4311; if they follow `pnpm dev` literally they get a blank player and a
   console error — first impression damaged.

## Errors, waits, confusing messages recorded
- EADDRINUSE → "Run ck doctor" hint (misleading; should suggest `--port`).
- `pnpm doctor` name collision with pnpm builtin (repeat trap).
- demo:reset failure message is clear but offers no in-product path (needs unshipped artifact).
- No install step validates Node 24 beforehand (npm engines field absent from package.json).

## Unstated prerequisites found
- pnpm itself (documented only in README code block; packageManager field pins 11.25.0 — corepack
  not mentioned.
- Node ≥24 required (README says so) but not enforced on install.
- Remotion license terms (see README) — noted, not an install requirement.

## Findings raised (ids assigned in FINDINGS.md)
- F-UX-001 dev-mode component registry 404 (S1, verified by running twice).
- F-UX-002 `pnpm doctor` masks ClipKit doctor (S3).
- F-UX-003 demo:reset dead on fresh machine (S2, verified).
- F-UX-004 EADDRINUSE wrong hint (S3, verified).
- F-UX-005 uninstall leaves empty dirs (S4, verified).
