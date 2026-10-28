import {z} from 'zod';
const range=z.object({startFrame:z.number().int().nonnegative(),endFrame:z.number().int().positive()});
export const toolkitTools:Record<string,{description:string;shape:Record<string,any>}>= {
  project_open:{description:'Attach to the shared film and Studio. Returns source paths and current settings. No model invocation or approval checkpoint.',shape:{sceneId:z.string().optional(),durationFrames:z.number().int().positive().optional(),settings:z.object({width:z.number().int(),height:z.number().int(),fps:z.number().int()}).optional(),baseRev:z.number().int().optional()}},
  project_context:{description:'Read selection, affected scene source versions, properties, comments and timing. Author code using normal file tools.',shape:{itemId:z.string().optional(),range:range.optional()}},
  project_sync:{description:'Compile and publish a completed source edit; returns a job. Read local scenes/<sceneId>/scene.json, or send a manifest plus only changed source files. Checks source version and preserves current property values. No lasting lease.',shape:{sceneId:z.string(),baseVersion:z.string().optional(),targetItemIds:z.array(z.string()).optional(),range:range.optional(),manifest:z.record(z.string(),z.unknown()).optional(),files:z.record(z.string(),z.string()).optional()}},
  component_search:{description:'Find five web effects by visual action or creative intent. Returns previews and separates ready adapters from effects needing adaptation.',shape:{query:z.string(),refresh:z.boolean().optional(),clock:z.string().optional(),status:z.enum(['ready','needs-adaptation','unsupported']).optional(),limit:z.number().int().min(1).max(10).optional()}},
  component_inspect:{description:'Inspect one component. source=true fetches only its pinned source closure and license to disk; returns paths, not source dumps.',shape:{id:z.string(),source:z.boolean().optional()}},
  component_add:{description:'Materialize a selected adapter into a scene, or fetch actual upstream source for project-local frame adaptation. Returns import path and receipt.',shape:{id:z.string(),sceneId:z.string()}},
  component_validate:{description:'Compile and seek-check an isolated adapter test scene without changing the timeline. Caches source/dependency versions; moving-source fidelity still needs visual inspection.',shape:{id:z.string(),sceneId:z.string(),adapterFile:z.string()}},
  resource_import:{description:'Register a server-readable font/image/audio/video as a hashed file. If client and server paths differ, run ck resource add PATH to upload file bytes directly. Never print or submit binary/base64 source.',shape:{path:z.string()}},
  property_patch:{description:'Change one nested exposed property without source regeneration. Optional expected value protects concurrent human edits.',shape:{itemId:z.string(),path:z.string(),value:z.unknown(),expected:z.unknown().optional()}},
  render:{description:'Render the same immutable composition used in Studio. Native project size by default, optional exact dimensions/FPS/audio and frame range. Returns a job; inspect the moving result.',shape:{width:z.number().int().optional(),height:z.number().int().optional(),fps:z.number().int().optional(),audio:z.enum(['auto','silent','required']).optional(),crf:z.number().int().min(0).max(51).optional(),range:range.optional()}},
  source_bundle:{description:'Export the project, actual editable TSX, resources, component receipts, dependencies and immutable artifacts. Returns a job.',shape:{}},
};
export const toolkitInstructions=`# ClipKit: web components → editable films
The current coding agent directs and authors the film. Toolkit operations never call another model.
1. project_open / project_context: inspect the brief, material, settings and selected range.
2. Choose a visual argument, exact copy, reading intervals, key poses and connected shot actions.
3. component_search: query actions such as typography reflow, media aperture, velocity or refraction.
4. Inspect a few moving previews. component_add materializes only selected sources/adapters.
5. Write normal TSX/CSS in scenes/<id>/. project_open creates a minimal Scene.tsx/scene.json starter. Keep manifestVersion:2, entry, files, dependencies, resources, defaults, propsSchema and durationFrames; inspector fields derive from the schema when omitted. The starter is scaffolding, not a required layout.
6. Keep actual source mechanisms. Map time/scroll/pointer to frames; pause animations; seed randomness. Package imports require exact declared versions. Cache new adaptations with component_validate when needed; do not revalidate already-ready adapters or repeatedly validate the same full film.
7. Fonts/images/audio remain files: ck resource add PATH uploads bytes without model encoding. Import resourceUrl/useResourceFont from @clipkit/motion. Use resourceUrl(id,assetBaseUrl,accessToken); the URL query parameter is token. Never guess a different authentication format.
8. project_sync publishes a completed edit, keeps last-good code on failure, and preserves existing instance property values. Await job_status; no editing lease or storyboard approval is required.
9. Render a moving draft with render(range). Inspect actual hierarchy, pacing, handoffs, crops, copy and sound. One initial composition and at most two correction cycles by default.
10. render exports at native dimensions; source_bundle includes editable code and assets. Deliver the best valid result and concrete remaining defects.
For a property-only edit use property_patch with expected value: no code rewrite, model delegation, or full resource resend. For choreography edit only affected source and boundary context; preserve outside timing.
Studio is optional playback, comments, selection, properties, history and timeline editing. The project owns current instance values; TSX owns choreography/defaults. Do not overwrite human values from old defaults.
No mandatory recipes, seven-pass procedure, human checkpoints, effect quota, or automatic aesthetic pass. Use original code where it serves the film, and claim component reuse only when the actual source/adapter is used.
Legacy editor/voice/source-hub commands remain available under CLIPKIT_TOOL_PROFILE=legacy; they are not prerequisites for this workflow.
`;
