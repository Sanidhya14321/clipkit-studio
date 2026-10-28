# ClipKit: web components → editable films

Give Claude, Codex, or another coding agent a searchable visual vocabulary and reliable local video tools. The host agent directs the film; ClipKit finds and adapts actual React/CSS/GSAP/shader effects, previews them in Studio, and exports native video plus editable source.

```sh
pnpm install
pnpm dev
# In another terminal, with your project selected through --root:
pnpm ck project open --width 1080 --height 1920 --fps 30
pnpm ck component search "gooey typography morph"
pnpm ck component add web-gooey-morphing-text --scene film
# Author scenes/film/Scene.tsx and scene.json, then:
pnpm ck project sync film
pnpm ck render --audio silent
pnpm ck op job_status '{"jobId":"JOB_ID","waitMs":45000}'
```

`ck init` installs compact host-agent instructions and the twelve-tool MCP profile for Codex, Claude Code and Cursor. It does not start another model. Studio provides optional preview, frame comments, property controls and undo. Generation runs through to export unless you ask for a checkpoint.

**Discover → select → adapt → compose → inspect → export.** A pinned metadata index covers 774 discovered source files; search ranks visual intent with local BM25; sources include React Bits, Magic UI, Motion Primitives, Animata, Paper Shaders, selected Codrops demos and the existing Clip Library components. Entries clearly distinguish ready adapters from source needing frame-clock adaptation. Selected source and dependency closures are fetched on demand; successful adaptations are cached by their source, dependency and runtime versions.

Fonts and media stay as files: `ck resource add /path/to/font.ttf` uploads directly and returns a resource ID. `ck project sync film` sends changed source only. `ck property set ITEM headline '"New headline"'` updates a declared value without regenerating code. Export uses the native project size unless you explicitly select another size/FPS/audio mode.

See [toolkit authoring](docs/TOOLKIT.md) for the manifest, source adaptation and editing contracts. [Earlier workflows](docs/WORKFLOW.md) remain available through the legacy CLI/API and `CLIPKIT_TOOL_PROFILE=legacy`; storyboard approval, seven-pass pipelines and nested agents are not prerequisites for the toolkit.

```sh
pnpm typecheck
pnpm test
pnpm exec tsx scripts/build-toolkit-film.ts   # isolated 24-second real-component film
```

Remotion is pinned to 4.0.538. Node 24, Chromium and FFmpeg are used for local rendering. Model clients are needed only for agent-directed authoring. Existing user projects and immutable scenes retain their formats; the new disk-scene manifest is backward compatible. No visual-quality or token-saving claim follows solely from technical tests. Historical comparison evidence remains in [VIDEO_COMPARISON.md](docs/VIDEO_COMPARISON.md).
