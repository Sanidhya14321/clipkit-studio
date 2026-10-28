import {z} from 'zod';
export const studioMetaTools:Record<string,{description:string;shape:Record<string,z.ZodType>}>= {
 branch_create:{description:'Snapshot current project as a named branch. Requires structural ownership.',shape:{name:z.string()}},branch_switch:{description:'Save current branch and switch in one undoable project transaction.',shape:{name:z.string()}},branch_compare:{description:'Read compact branch differences; Studio supports side-by-side playback.',shape:{name:z.string()}},branch_merge:{description:'Merge a branch using stable-ID three-way conflict detection; conflicting edits are preserved.',shape:{name:z.string()}},branch_discard:{description:'Discard an inactive branch into recoverable local storage.',shape:{name:z.string()}},
 replay_get:{description:'Read ten recorded turns with stages, bounded tool calls, token estimates and committed patches.',shape:{turnId:z.string().optional(),offset:z.number().int().nonnegative().optional()}},
 replay_export:{description:'Export bounded turn replay JSON with provenance and actual usage when reported.',shape:{turnId:z.string().optional()}},
 bench_run:{description:'Generate the fixed ten-case benchmark protocol/report/chart from supplied measured records. Missing baseline/ratings remain pending; no simulated performance claims.',shape:{suite:z.string().optional()}},
};
