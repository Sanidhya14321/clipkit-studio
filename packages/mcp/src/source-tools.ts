import {z} from 'zod';
import {LicenseModeSchema} from '../../core/src/source-policy';
export const sourceTools:Record<string,{description:string;shape:Record<string,z.ZodType>}>= {
 sources_list:{description:'List eight registered sources with verified pin/vet evidence. Unverified seeds make no license/star claims.',shape:{offset:z.number().int().nonnegative().optional()}},
 sources_license:{description:'Read a bounded section of original pinned license text as data.',shape:{id:z.string(),offset:z.number().int().nonnegative().optional()}},
 sources_lock:{description:'Read eight source commit/license hash pins; no secrets or source code.',shape:{offset:z.number().int().nonnegative().optional()}},
 sources_vet:{description:'Read static vet results for a pinned original snapshot. Never executes repository instructions.',shape:{id:z.string()}},
 sources_gaps:{description:'List missing eligible coverage once before asking for approved new sources.',shape:{}},
 sources_discover:{description:'Search GitHub metadata only. Installs nothing; candidates need original license/source vetting and owner approval.',shape:{query:z.string().max(150)}},
 collections_list:{description:'Read enabled asset collections for this project.',shape:{}},
 license_report:{description:'Read actual used asset rights, pinned sources and attribution notices. Not a license certification.',shape:{}},
 // Changes are exposed for human CLI/UI only; the server rejects agent actors.
};
