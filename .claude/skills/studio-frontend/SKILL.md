---
name: studio-frontend
description: Use when working in studio/ (React, Vite, Remotion Player) or writing Playwright tests for it. Generated types, data-ck attributes, read-only mode and browser test patterns.
---

# Studio frontend

The studio lives in `studio/` (TypeScript, React, Vite) and is embedded into the `ck` binary
with rust-embed. Read spec 05 (studio) before UI work. Phase 0 has no studio code yet.

## Types
- Never hand-write project types. Import them from the generated bindings (ts-rs or specta,
  chosen in Phase 2 and recorded in DECISIONS.md). Regenerate after changing Rust types.

## State and sync
- The server is the source of truth. The studio applies JSON patches from the WebSocket and sends
  edits as patches with the expected `rev`; it never rewrites the whole project.
- Read-only mode: while the agent holds the turn, every edit control is disabled AND the server
  refuses edits. UI disabling is cosmetic; tests must check the server refusal too.

## data-ck instrumentation
- Every rendered element that maps to a layer carries `data-ck-layer="<layerId>"` (and
  `data-ck-prop` for property-level targets) so hover-select and Playwright can find it.

## Preview
- Remotion Player renders from project JSON using proxies. Keep compositions pure functions
  of props; no fetches inside compositions.

## Playwright
- Select by `data-ck-*` or role, never by CSS classes or text that may change.
- Each test starts the server against a fresh copy of a fixture project, with a session token.
- Assert the read-only state both in the DOM and via a refused API call.
- Formatting runs automatically on edit once prettier is installed in the package.
