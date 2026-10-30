# render-worker

Node sidecar that renders Remotion component layers for export, stills and previews, driven by
`ck` (spec 2.3). `src/Root.tsx` registers one composition drawn by the studio's own
`ProjectComposition`, so the preview and the export match.

- `pnpm install`, then `pnpm bundle` once (writes `bundle/`).
- `ck` runs `node src/render.mjs` with a JSON job on stdin and reads JSON progress lines.
- Rendering uses Remotion (`@remotion/renderer`), which downloads a headless Chrome on first
  use. Remotion is under the Remotion License: companies above a certain size need a company
  license (remotion.dev/license).
