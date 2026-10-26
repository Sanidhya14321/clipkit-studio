import {McpServer} from '@modelcontextprotocol/sdk/server/mcp.js';
import {StdioServerTransport} from '@modelcontextprotocol/sdk/server/stdio.js';
import {z} from 'zod';

export const toolDefinitions:Record<string,{description:string;shape:Record<string,z.ZodType>}>={
  project_get:{description:'Read compact summary (default) or exactly one layer with its JSON Patch pointer and current revision.',shape:{layerId:z.string().optional(),offset:z.number().optional(),limit:z.number().max(30).optional()}},
  project_patch:{description:'Apply small atomic RFC6902 edits at baseRev. Use pointer from project_get. Cannot rewrite whole project. Lease supplied automatically.',shape:{baseRev:z.number().int(),requestId:z.string(),label:z.string(),ops:z.array(z.object({op:z.enum(['add','replace','remove','test','move','copy']),path:z.string(),value:z.unknown().optional(),from:z.string().optional()}))}},
  assets_search:{description:'Find up to five relevant editable clips; full catalog is not included.',shape:{query:z.string(),limit:z.number().max(10).optional()}},
  asset_schema:{description:'Fetch defaults and props schema for one selected clip.',shape:{clipId:z.string()}},
  assets_add:{description:'Insert a clip at a timeline time in seconds.',shape:{clipId:z.string(),at:z.number()}},
  item_update:{description:'Edit one item. changes may contain at,dur,srcIn,volume,props. Supply complete existing props merged with your edits.',shape:{itemId:z.string(),changes:z.record(z.string(),z.unknown())}},
  item_split:{description:'Split the selected item at a timeline time in seconds.',shape:{itemId:z.string(),at:z.number()}},
  item_delete:{description:'Remove one item and its linked captions.',shape:{itemId:z.string()}},
  footage_add:{description:'Import a local user-provided media path through the non-destructive pipeline.',shape:{path:z.string()}},
  footage_pack:{description:'Read compact timestamped source phrases for a range; no full transcript by default.',shape:{sourceId:z.string(),start:z.number().optional(),end:z.number().optional()}},
  footage_candidates:{description:'Read proposed silence/filler cut ranges. Review rather than blindly applying cuts.',shape:{sourceId:z.string()}},
  footage_delete_words:{description:'Ripple-delete a reliable source word range from a footage item.',shape:{sourceId:z.string(),itemId:z.string(),start:z.number(),end:z.number()}},
  comments_list:{description:'Read a short page of open comments.',shape:{open:z.boolean().optional(),limit:z.number().max(20).optional()}},
  comments_bundle:{description:'Read comments with ONLY affected layer JSON and current revision. Use a small group of comment IDs.',shape:{commentIds:z.array(z.string()).max(10).optional()}},
  comments_add:{description:'Leave feedback or an uncertainty note pinned to timeline time and optionally a layer/prop.',shape:{text:z.string(),anchor:z.object({tStart:z.number(),tEnd:z.number().optional(),layerId:z.string().optional(),propPath:z.string().optional()})}},
  comments_resolve:{description:'Mark one completed comment resolved with a concise explanation. Server attaches committed diff.',shape:{commentId:z.string(),reply:z.string()}},
  comments_reply:{description:'Reply to a comment without resolving it.',shape:{commentId:z.string(),text:z.string()}},
  voice_script:{description:'Create narration slots from a newline-separated script.',shape:{text:z.string()}},
  voice_generate:{description:'Generate speech for one slot using Fish; incurs provider usage. Only call when requested.',shape:{slotId:z.string(),voiceRef:z.string().optional(),speed:z.number().optional()}},
  voice_mode:{description:'Choose ai, recorded or cloned mode; takes retain truthful provenance.',shape:{slotId:z.string(),mode:z.enum(['ai','recorded','cloned'])}},
  history_list:{description:'Read compact recent revisions.',shape:{limit:z.number().max(30).optional()}},
  job_status:{description:'Wait up to 15 seconds for an asynchronous job; call again if still running. Check failure before claiming completion.',shape:{jobId:z.string()}},
  export_start:{description:'Start real export from an immutable revision snapshot; returns a job.',shape:{}},
};
export async function serveMcp(invoke:(name:string,args:Record<string,unknown>)=>Promise<unknown>){
  const server=new McpServer({name:'clipkit',version:'0.1.0'});
  for(const [name,def] of Object.entries(toolDefinitions))server.registerTool(name,{description:def.description,inputSchema:def.shape,annotations:{readOnlyHint:['project_get','assets_search','asset_schema','footage_pack','footage_candidates','comments_list','comments_bundle','history_list','job_status'].includes(name),destructiveHint:false,openWorldHint:['voice_generate'].includes(name)}},async(args:Record<string,unknown>)=>{
    try{const value=await invoke(name,args);const json=JSON.stringify(value);if(json.length>16000)return {isError:true,content:[{type:'text',text:'CONTEXT_TOO_LARGE: request one layer, fewer comments, or a shorter transcript range.'}]};return {content:[{type:'text',text:json}]};}
    catch(e){return {isError:true,content:[{type:'text',text:(e as Error).message.slice(0,1200)}]};}
  });
  await server.connect(new StdioServerTransport());return server;
}
