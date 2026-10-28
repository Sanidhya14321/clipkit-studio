# Toolkit authoring

The host coding agent owns creative decisions and ordinary file edits. ClipKit's default MCP profile contains twelve operations: project_open, project_context, project_sync, component_search, component_inspect, component_add, component_validate, resource_import, property_patch, render, source_bundle and job_status. None invokes a model.

## A disk scene

project_open creates a minimal scene starter for a new project (or a requested sceneId/durationFrames). Edit `scenes/film/Scene.tsx`, exporting a React component. Props include the manifest defaults, `item`, `assetBaseUrl`, `mediaBaseUrl` and `accessToken`. Use explicit frame time and tag exposed controls with `data-ck-layer={item.id}`, `data-ck-path="headline"` and a stable `data-ck-element`. V2 scenes receive a selectable root automatically; hand-authored selection tags are optional and never block export. Inspector fields derive from the property schema when omitted. Not every decorative element needs a property.

```json
{
  "manifestVersion": 2,
  "sceneId": "film",
  "name": "My film",
  "entry": "Scene.tsx",
  "durationFrames": 720,
  "files": {"Scene.tsx": "", "components/Title.tsx": ""},
  "dependencies": {},
  "resources": [],
  "defaults": {"headline": "Make it move"},
  "fields": [{"key":"headline","label":"Headline","type":"text"}],
  "propsSchema": {"type":"object","additionalProperties":false,"required":["headline"],"properties":{"headline":{"type":"string"}}}
}
```

The files map declares the closure; files contain normal source on disk. Nested TSX/JSX/TS/JS/CSS/SVG/JSON are supported. Import CSS for scoped styling or use inline styles. Import React, Remotion and @clipkit/motion directly; component_add materializes a module under components/. Exact dependencies currently supported are gsap, @remotion/gsap, motion, @paper-design/shaders and lucide-react. React/Remotion share the existing runtime; declared packages bundle into the immutable scene. All @remotion packages must match 4.0.538. New dependency installs disable lifecycle scripts. Source validation is not a security sandbox.

Call `ck project sync film` after a completed edit batch. The CLI transfers only changed files using its previous hash receipt; project_sync also accepts a server-local scene ID or explicit manifest/changed-file payload. The service compiles and seeks the candidate before publication, reusing identical verified artifacts and coalescing concurrent checks of the same version. Existing instance values survive source updates. `baseVersion` and stable target IDs prevent stale publications. Failed compilation leaves the last valid scene visible. V2 scenes do not auto-publish intermediate file writes.

One authored film can contain named shot subcomponents with connected choreography. Internal exposed elements are selectable, but they do not automatically become independent timeline clips. Existing footage/audio/clip tracks remain supported. Timeline placement, current props and output settings live in the project; choreography/defaults live in TSX.

## Resources

`ck resource add PATH` reads the file in the client filesystem and uploads its bytes, avoiding host/namespace path mismatches. It returns a SHA-256 resource ID and metadata. MCP resource_import accepts a server-readable path; use the CLI upload if the server cannot see the client path. Never print/base64-encode font, image, audio or video bytes into source or tool messages.

Add IDs to the manifest resources list. In scene code use `resourceUrl(id, assetBaseUrl, accessToken)` from @clipkit/motion. `useResourceFont(family,url)` blocks captures until the font loads. Images can use Remotion Img with that URL. Resources travel in source bundles.

## Components

Search descriptions, mechanisms and tags using visual actions. The bundled metadata snapshot indexes 774 upstream source files; this count is discoverability, not 774 tested adapters. `ck component refresh` explicitly refreshes repository-tree metadata without cloning source repositories or making model calls. Source is downloaded only after selection. Inspect a few candidates, not whole repositories. `component_add` copies a ready adapter and records its dependency receipt. A source without an adapter is fetched at one pinned revision with its relative imports and license, then materialized under component-references/. It is not executed as browser code.

Adapt the selected source under components/: keep its JSX, geometry, CSS/shader formulas or timeline, and replace its clock and interactions. Paused CSS/WAAPI and GSAP timelines seek from frame time; scroll/pointer inputs become scripted tracks. Stateful effects need deterministic replay/baking or a concrete unsupported result. Simply importing motion/react does not make a layout animation deterministic.

For a newly adapted component, prepare an isolated test scene, declare its adapter file, props and dependencies, then call `component_validate`. It compiles and checks rendered frames without inserting a timeline item, stores an immutable source/dependency/runtime receipt and caches the adapter for subsequent component_add calls. Technical repeatability is separate from visually comparing the moving result with the upstream effect. Do not claim fidelity solely from hashes.

React Bits is indexed and may be fetched/adapted for permitted project-local application use; ClipKit does not distribute its components as a renamed/ported component pack. Preserve each selected source's notices. Reference media and code can have different rights.

## Render and edit

`render` uses native settings by default. Optional width/height must be specified together, even integers 64–3840. FPS is 1–120. Audio modes are auto (include authored sound), silent, or required (include an audio track). Optional frame ranges are integral, end exclusive. Read job_status with up to 45 seconds of bounded waiting. Render jobs snapshot project state; Studio and export share Composition.

`property_patch` changes a nested exposed property without editing source. Supply expected when preserving concurrent edits. For choreography, read project_context for the selected source/range and change the relevant files. Scope a publication with targetItemIds and range; preserve the source clock and all outside timing.

Render a moving draft and inspect it before final export. Default creative work is one composition and up to two corrections, with no mandatory human checkpoint. source_bundle exports actual authoring sources, project metadata, resources, dependency locks, source receipts and compiled artifacts. Open the bundle with the matching ClipKit runtime. The legacy standalone remotion interchange remains compiled playback and is not the editable-source delivery path.
