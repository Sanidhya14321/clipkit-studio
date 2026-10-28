import {z} from 'zod';
export const qualityToolDefinitions:Record<string,{description:string;shape:Record<string,z.ZodType>}>= {
 sound_plan:{description:'Propose/apply deterministic available licensed scene accents, music ducking and explicit missing-sound notes.',shape:{apply:z.boolean().optional()}},
 audio_analyze:{description:'Measure LUFS/true peak of a registered audio source without modifying it. This does not certify the final mix.',shape:{sourceId:z.string()}},
 layer_cache_prepare:{description:'Render/cache up to ten isolated native recipe layers as transparent video; unchanged content hits the cache.',shape:{layerIds:z.array(z.string()).max(10).optional()}},
 see_sheet:{description:'Return one contact sheet of at most six low-resolution frames in the requested seconds range.',shape:{start:z.number().nonnegative().optional(),end:z.number().positive().optional()}},
 quality_critic:{description:'Run deterministic lints first; prepare at most one image per agent pass for optional critique. Does not invent a model verdict.',shape:{}},
 quality_critic_submit:{description:'Record a human/agent critique citing actual sampled frames; this never grants human aesthetic approval.',shape:{reviewId:z.string(),summary:z.string().max(1000),observedFrames:z.array(z.number().int()).max(6),issues:z.array(z.object({frame:z.number().int(),problem:z.string().max(300),fix:z.string().max(300)})).max(8)}},
};
