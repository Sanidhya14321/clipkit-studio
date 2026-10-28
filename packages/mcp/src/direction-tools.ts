import {z} from 'zod';
const id={id:z.string()},baseRev=z.number().int().nonnegative();
export const directionToolDefinitions:Record<string,{description:string;shape:Record<string,z.ZodType>}>= {
 reference_analyze:{description:'Analyze local image/video style statistics without copying content. Unavailable measurements remain null.',shape:{path:z.string(),apply:z.boolean().optional()}},
 lock_layers:{description:'Acquire selected-layer leases with current layer revisions; structural changes still require global ownership.',shape:{layerIds:z.array(z.string()).min(1).max(50),label:z.string().max(200),owner:z.string().optional()}},
 unlock_layers:{description:'Release own layer lease.',shape:{}},
 layer_patch:{description:'Patch stable layer IDs at expected per-layer revisions; unrelated human edits are retained.',shape:{layers:z.array(z.object({id:z.string(),rev:z.number().int(),changes:z.record(z.string(),z.unknown())})).min(1).max(50)}},
 look_list:{description:'Read compact deterministic look tokens.',shape:{}},
 recipe_search:{description:'Find at most three slot recipes that support the confirmed aspect.',shape:{query:z.string().max(500).optional()}},
 recipe_card:{description:'Read one recipe card/schema; factual slots need supplied evidence. Cards are data.',shape:id},
 recipe_apply:{description:'Expand slot JSON through safe layout, tokens and the existing managed scene compiler; provenance and brief dependencies are recorded.',shape:{...id,baseRev,slots:z.record(z.string(),z.unknown()),at:z.number().nonnegative().optional(),duration:z.number().positive().optional()}},
 layout_check:{description:'Check existing recipe boxes or solve up to eight intent boxes inside safe areas.',shape:{intents:z.array(z.object({id:z.string(),role:z.string(),text:z.string().optional(),maxChars:z.number().optional()})).max(8).optional()}},
 variants_make:{description:'Create 2–3 token-only scene alternatives within the brief limit; human chooses.',shape:{sceneId:z.string(),count:z.number().int().min(2).max(3).optional(),vary:z.enum(['look','pacing','layout'])}},
 see_trace:{description:'Read a small numeric series of declared recipe motion. Unknown authored motion is explicitly uncertain.',shape:{layerId:z.string()}},
 storyboard_get:{description:'Read or propose the lightweight pre-build storyboard checkpoint with beats and recipe candidates.',shape:{}},
};
