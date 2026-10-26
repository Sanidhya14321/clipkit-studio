# ClipKit Studio

A local video workspace where you and a coding agent edit the same non-destructive timeline. Agents make small, revisioned patches; you can inspect, change and undo them.

**Status:** reduced hackathon MVP. See the [full Word PRD feature audit](docs/PRD-FEATURE-AUDIT.md) for verified behavior, missing features and known defects. Passing automated tests does not mean the full PRD is complete.

## Start

Requires Node 22+ and pnpm. The development machine uses Node 24. FFmpeg falls back to the locally installed `ffmpeg-static` binary; ffprobe uses Remotion's bundled binary when absent from PATH.

```sh
pnpm install
pnpm doctor
pnpm ck init
pnpm dev
```

Open **http://127.0.0.1:5173**. The API listens on **127.0.0.1:4310**. `pnpm dev --open` also opens the browser. `pnpm ck studio --no-vite` starts just the backend. Stop with Ctrl+C.

The first screen can open a **graphics demo fixture**. It is not presented as real footage. Import your own talking-head clip for the before/after demonstration.

## Voice and agent setup

Copy `.env.example` to `.env`, then set `FISH_API_KEY` and `FISH_VOICE_ID` for transcription and AI narration. Secrets stay on the server. Restart after changing the environment. Fish TTS model is explicitly configurable with `FISH_TTS_MODEL` (default `s2-pro`). Provider calls consume your credits.

Sign in to the agent(s) you want to use:

```sh
codex login
claude auth login
```

`ck init` preserves existing configuration while adding a managed Codex configuration block, project MCP registration, short AGENTS instructions and a Claude skill. It does not sign in to providers. App-initiated Codex turns use `gpt-6.1-sol`, medium effort, and only ClipKit's MCP editing workflow. Claude uses its configured model.

Create a comment on a selected layer and choose **Apply open**. You can also use the command field, or hold the voice button/backtick. Commands such as `undo`, `split here` and `go to 12 seconds` use the local deterministic router; free-form instructions become anchored comments. Recognition itself requires Fish access; typed commands always work.

For narration, create one slot per script line, then record or generate each line. A cloned voice requires your own in-app recording of at least ten seconds, confirmation that you have the right to clone it, and a trained private Fish model. Prepare the voice and cached takes before presenting. AI, recorded and cloned takes retain separate provenance and selection. Adjust a slot's time window when its recorded length does not fit.

## CLI examples

```sh
pnpm ck search lower third
pnpm ck schema kinetic-title
pnpm ck assets build
pnpm ck project get --summary
pnpm ck project get --layer LAYER_ID
pnpm ck project patch patch.json
pnpm ck footage add /absolute/path/take.mp4
pnpm ck footage pack SOURCE_ID --range 0 30
pnpm ck comments list --open --compact
pnpm ck history undo
pnpm ck export
```

Patch requests contain `requestId`, `baseRev`, `label` and RFC6902 `ops`. Fetch a layer first to obtain its current pointer. Root/project ID/revision/source paths cannot be patched. The active runner's MCP connection carries its lease automatically. An external manual agent that acquires a lease with `ck status working LABEL` must pass the returned token and turn ID via `CLIPKIT_LEASE` and `CLIPKIT_TURN` for subsequent mutations, then release it with `ck status idle --lease TOKEN`.

## Build and install the tarball

```sh
pnpm build
npm pack ./artifacts/package --pack-destination ./artifacts
```

In a separate project directory, use the **absolute path** to the generated tarball:

```sh
npm exec --yes --package /absolute/path/clipkit-studio-local-0.1.0.tgz -- ck init
npm exec --yes --package /absolute/path/clipkit-studio-local-0.1.0.tgz -- ck studio
```

The packed CLI serves the built studio on port 4310 and includes renderer source, public assets and license notices. It does not rely on unpublished workspace packages. The existing npm package named `clipkit` is unrelated; this project does not publish over it.

## Verify and rehearse

```sh
pnpm typecheck
pnpm test
CK_RENDER_TEST=1 pnpm exec vitest run tests/assets.test.ts
pnpm test:e2e
pnpm exec tsx scripts/render-smoke.ts
```

`pnpm demo:reset` loads the explicitly labeled graphics fixture and is undoable. Do not run it on an unsaved presentation project. See [demo checklist](docs/DEMO.md) and [verification status](docs/VERIFICATION.md).

## Data and limitations

- Originals, proxies, history, recordings and exports live under the project's gitignored `.clipkit/` directory. Source files are copied, never overwritten.
- When a Fish key is configured, importing audio/video automatically schedules transcription. Transcription uploads audio to Fish. TTS sends script text and voice references. Cloning uploads your recorded sample.
- Coding-agent providers receive the request and retrieved text/project context. Local-first does not mean cloud-free AI.
- Token counts labeled **context** are byte-based estimates, not billed model usage. Runner-reported usage is displayed separately when available. The comparison baseline is estimated from full current content, not an invented regeneration cost.
- Word cutting is enabled only for reliable word timestamps. Phrase-only or invalid provider timings are not expanded into fake word timings.
- The first export can download Chromium. Preview uses proxies; export uses originals and an immutable revision snapshot. Rendering takes real time.
- Cached/recorded audio can export without Fish. New uncached speech cannot. Browser speech playback is not an exportable fallback.
- No subject segmentation, full-script recording alignment, arbitrary historical revert, cloud collaboration or mobile editor. Only the latest untouched agent turn supports grouped revert.
- Real Fish requests, your microphone and your footage must be rehearsed with your credentials. Claude must be signed in before claiming a two-client demo.

Original application code and ten native clip components are MIT. Remotion and embedded assets have separate terms; see [third-party notices](THIRD_PARTY_NOTICES.md) and [asset inventory](docs/ASSET-SOURCES.md).

The starter pack includes real rendered PNG previews and generated compact catalog/category indexes plus full per-clip schemas. `ck assets build` validates it and writes a local index. New React clip code must be explicitly registered and compiled; dropping untrusted executable code into a folder does not run it.
