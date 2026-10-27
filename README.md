# ClipKit Studio 0.5

Local video authoring with real animation components, frame comments, reversible human edits and a creative-agent checkpoint.

```sh
pnpm install
pnpm build
pnpm dev
```

Node 24 and a signed-in Codex CLI are required for live authoring. Remotion is pinned to 4.0.538. Agent turns use gpt-6.1-sol at medium effort. A provider key is optional for Fish voice features; those live integrations remain unverified.

Open **Components** to preview and insert actual Clip Library, Magic UI, Animata and Paper Shader implementations. Open **Storyboard** to prepare a film: the first turn creates a storyboard and a real three-second sample. Human approval resumes the exact agent session. Completed work remains a proposal until **Keep revision**. Direct property edits require no agent call.

```sh
pnpm verify                 # source inventory, typecheck, unit/integration contracts
pnpm verify components      # render all 12 components and regenerate moving previews
pnpm verify scenes          # real scene, scoped-edit and recovery checks
pnpm verify browser         # responsive browser checks
pnpm verify install         # packed installation and export outside the repository
pnpm benchmark attempt-name # preserved 15-second preparation; review in printed studio URL
```

Build/package, verification, benchmark, compatibility and publication scan are the five scripts under `scripts/`. Runtime projects and history live in `.clipkit/`; imported media stays separate from Components. Do not delete `.clipkit` to clear caches. Webpack caches now live under your OS cache directory.

Current status and limitations: [verification](docs/VERIFICATION.md). Contracts: [architecture](docs/ARCHITECTURE.md), [authoring](docs/AUTHORING.md), [workflow](docs/WORKFLOW.md), [sources](docs/SOURCES.md).
