import {Hono} from 'hono';
import {serve} from '@hono/node-server';
import {WebSocketServer,WebSocket} from 'ws';
import {createReadStream} from 'node:fs';
import {mkdir,readFile,writeFile,stat,readdir,rm} from 'node:fs/promises';
import {resolve,join,extname,basename,sep} from 'node:path';
import {randomBytes,timingSafeEqual} from 'node:crypto';
import {Readable} from 'node:stream';
import {AsyncLocalStorage} from 'node:async_hooks';
import {fileURLToPath} from 'node:url';
import {ProjectStore,ClipKitError} from '../../core/src/store.ts';
import {id,allItems,findItem,itemPointer,quantize,emptyProject,tokenEstimate,routeVoice,type Project,type Job,type Word,type StudioState,type Activity,type Telemetry,type JsonPatchOperation,type Item} from '../../core/src/index.ts';
import {catalog,validateClipProps,makeClip,searchAssets,clipSchemas} from '../../assets/src/index.ts';
import {z} from 'zod';
import {renderProject} from '../../render/src/export.ts';
import {MediaService} from './media.ts';
import {AgentRunner} from './runner.ts';

const mime:Record<string,string>={'.mp4':'video/mp4','.webm':'video/webm','.mov':'video/quicktime','.wav':'audio/wav','.mp3':'audio/mpeg','.ogg':'audio/ogg','.json':'application/json','.svg':'image/svg+xml','.png':'image/png','.jpg':'image/jpeg','.woff2':'font/woff2','.ttf':'font/ttf','.css':'text/css','.js':'text/javascript','.html':'text/html'};
const text=(v:unknown,max=4000)=>{if(typeof v!=='string'||v.length>max)throw new ClipKitError('INVALID_REQUEST','Expected a shorter text value');return v;};
const number=(v:unknown,defaultValue=0)=>{if(v===undefined)return defaultValue;if(typeof v!=='number'||!Number.isFinite(v))throw new ClipKitError('INVALID_REQUEST','Expected a finite number');return v;};
const safeJoin=(root:string,relative:string)=>{const full=resolve(root,relative);if(full!==resolve(root)&&!full.startsWith(resolve(root)+sep))throw new ClipKitError('INVALID_PATH','Path must remain within its directory');return full;};
export type ServerOptions={root:string;port?:number;cliPath?:string;publicDir?:string;studioDir?:string;renderEntry?:string};
export async function startServer(options:ServerOptions){
  const root=resolve(options.root);const port=options.port??4310;const url=`http://127.0.0.1:${port}`;
  const publicDir=options.publicDir??join(root,'public');const studioDir=options.studioDir??join(root,'dist','studio');
  const token=randomBytes(32).toString('hex');const app=new Hono();const jobs:Job[]=[];const activity:Activity[]=[];const transcripts:Record<string,Word[]>={};
  let agentError:string|undefined;let agentClient:'codex'|'claude'='codex';let capabilities:Record<string,boolean>={};let notify=()=>{};
  let telemetry:Telemetry={contextTokens:0,baselineTokens:0,estimator:'Estimated request + tool argument/result bytes / 4; excludes agent base prompt/tool definitions. Actual provider usage is separate.'};
  const store=await new ProjectStore(root,p=>{for(const item of allItems(p))if(item.type==='clip')validateClipProps(item.clipId!,item.props);}).load();
  const registeredSources=new Map(Object.values(store.project.sources).map(source=>[source.id,source]));
  const mutationContext=new AsyncLocalStorage<{leaseToken?:string;turnId?:string}>();
  let persistedClone:StudioState['clone'];try{persistedClone=JSON.parse(await readFile(join(root,'.clipkit','voices.json'),'utf8'));}catch{}
  for(const source of Object.values(store.project.sources))if(source.transcriptPath){try{const data=JSON.parse(await readFile(resolve(root,source.transcriptPath),'utf8'));transcripts[source.id]=data.words??[];}catch{/* Missing transcript is visible as untranscribed media. */}}
  const addActivity=(a:Activity)=>{activity.push(a);if(activity.length>50)activity.shift();notify();};
  const jobWork=new Map<string,(update:(progress:number,label?:string)=>void)=>Promise<Record<string,unknown>>>();
  const job=(kind:string,label:string,work:(update:(progress:number,label?:string)=>void)=>Promise<Record<string,unknown>>):Job=>{
    const j:Job={id:id('job'),kind,label,status:'queued',progress:0};jobs.push(j);jobWork.set(j.id,work);if(jobs.length>50)jobs.shift();notify();
    void Promise.resolve().then(async()=>{j.status='running';notify();try{j.result=await work((progress,label)=>{j.progress=Math.min(1,Math.max(0,progress));if(label)j.label=label;notify();});j.progress=1;j.status='done';}catch(e){j.status='failed';j.error=(e as Error).message;addActivity({at:new Date().toISOString(),text:j.error});}notify();});return j;
  };
  const media=new MediaService({root,getProject:()=>store.project,getTranscripts:()=>transcripts,setTranscript:(sourceId,words)=>{transcripts[sourceId]=words;notify();},commit:(label,fn)=>{const actor=mutationContext.getStore();return store.mutate(label,fn,actor?.leaseToken?'agent':'system',actor);},job});
  const bundle=(ids:string[])=>{
    const comments=store.project.comments.filter(c=>ids.includes(c.id));const itemIds=[...new Set(comments.map(c=>c.anchor.layerId).filter(Boolean))] as string[];
    return {rev:store.project.rev,comments:comments.map(c=>({id:c.id,text:c.text,anchor:c.anchor})),layers:itemIds.map(i=>findItem(store.project,i)).filter(Boolean)};
  };
  const runner=new AgentRunner({root,url,token,cliPath:options.cliPath??fileURLToPath(new URL('../../cli/src/index.ts',import.meta.url)),store,bundle,activity:addActivity,telemetry:value=>{telemetry={...telemetry,...value};notify();},error:error=>{agentError=error;notify();},changed:()=>notify()});
  const state=():StudioState=>{const lease=store.currentLease();const {token:unused,...visible}=lease??{token:''};const clone=[...jobs].reverse().find(j=>j.kind==='clone'&&j.status==='done')?.result;return {project:store.project,lease:lease?visible as NonNullable<StudioState['lease']>:null,history:store.history.slice(-60),jobs:jobs.slice(-20),activity:activity.slice(-30),telemetry:{...telemetry,baselineTokens:tokenEstimate({project:store.project,transcripts,catalog})},transcripts,provider:{fish:Boolean(process.env.FISH_API_KEY),voiceId:process.env.FISH_VOICE_ID},agentClient,agentError,capabilities,clone:clone?{id:String(clone.id),status:String(clone.status)}:persistedClone};};
  const ensureItem=(p:Project,itemId:string)=>{const item=findItem(p,itemId);if(!item)throw new ClipKitError('UNKNOWN_ITEM','Select an existing layer');return item;};
  async function operate(name:string,a:Record<string,any>={},actor?:{leaseToken?:string;turnId?:string}):Promise<unknown>{
    const author=actor?.leaseToken?'agent':'human';
    const mutate=(label:string,fn:(p:Project)=>void)=>store.mutate(label,fn,author,actor,a.requestId);
    switch(name){
      case 'project_get': {
        if(a.layerId){const item=ensureItem(store.project,text(a.layerId));return {rev:store.project.rev,pointer:itemPointer(store.project,item.id),item};}
        const offset=Math.max(0,number(a.offset));const limit=Math.min(30,Math.max(1,number(a.limit,15)));
        return {id:store.project.id,name:store.project.name,rev:store.project.rev,settings:store.project.settings,total:allItems(store.project).length,layers:allItems(store.project).slice(offset,offset+limit).map(i=>({id:i.id,type:i.type,clipId:i.clipId,at:i.at,dur:i.dur})),next:offset+limit<allItems(store.project).length?offset+limit:null};
      }
      case 'project_patch':{
        if(!Array.isArray(a.ops))throw new ClipKitError('INVALID_PATCH','ops must be an array');
        if(a.ops.some((op:JsonPatchOperation)=>/^\/sources(\/|$)/.test(op.path)||op.from&&/^\/sources(\/|$)/.test(op.from)))throw new ClipKitError('INVALID_PATCH','Register source media through footage_add; source paths cannot be patched.');
        return store.patch({...a,leaseToken:actor?.leaseToken??a.leaseToken,turnId:actor?.turnId??a.turnId} as any,author);
      }
      case 'project_rename':return mutate('Rename project',p=>{p.name=text(a.name,200);});
      case 'assets_search':return searchAssets(text(a.query??'',200),Math.min(10,Math.max(1,number(a.limit,5)))).map(({id,name,category,durationSec,description,license})=>({id,name,category,durationSec,description,license}));
      case 'asset_schema':{const clipId=text(a.clipId,100);if(!clipSchemas[clipId])throw new ClipKitError('UNKNOWN_ASSET','Unknown clip ID');const asset=catalog.find(c=>c.id===clipId)!;return {id:clipId,defaults:asset.defaults,fields:asset.fields,schema:z.toJSONSchema(clipSchemas[clipId])};}
      case 'assets_add':return mutate('Add graphic',p=>{const item=makeClip(text(a.clipId,100),Math.max(0,quantize(number(a.at),p.settings.fps)));p.tracks.find(t=>t.kind==='overlay')!.items.push(item);});
      case 'item_update':return mutate('Edit layer',p=>{const item=ensureItem(p,text(a.itemId));const changes=a.changes??{};for(const key of Object.keys(changes))if(!['at','dur','srcIn','srcOut','volume','props'].includes(key))throw new ClipKitError('INVALID_REQUEST',`Cannot edit ${key} through item_update`);Object.assign(item,changes);if(item.props&&changes.props)item.props={...changes.props};item.at=quantize(item.at,p.settings.fps);item.dur=Math.max(1/p.settings.fps,quantize(item.dur,p.settings.fps));});
      case 'item_split':return mutate('Split layer',p=>{const item=ensureItem(p,text(a.itemId));const at=quantize(number(a.at),p.settings.fps);if(at<=item.at||at>=item.at+item.dur)throw new ClipKitError('INVALID_RANGE','Playhead must be inside the selected layer');const track=p.tracks.find(t=>t.items.some(i=>i.id===item.id))!;const left=at-item.at;const right={...structuredClone(item),id:id('layer'),at,dur:item.dur-left,srcIn:item.srcIn+left};if(item.words){right.words=item.words.filter(w=>w.end>left).map(w=>({...w,start:Math.max(0,w.start-left),end:w.end-left}));item.words=item.words.filter(w=>w.start<left).map(w=>({...w,end:Math.min(left,w.end)}));}item.dur=left;item.srcOut=item.srcIn+left;track.items.splice(track.items.indexOf(item)+1,0,right);});
      case 'item_duplicate':return mutate('Duplicate layer',p=>{const item=ensureItem(p,text(a.itemId));const track=p.tracks.find(t=>t.items.some(i=>i.id===item.id))!;track.items.push({...structuredClone(item),id:id('layer'),at:quantize(item.at+item.dur,p.settings.fps)});});
      case 'item_delete':return mutate('Delete layer',p=>{const itemId=text(a.itemId);for(const track of p.tracks)track.items=track.items.filter(i=>i.id!==itemId&&i.linkedTo!==itemId);});
      case 'item_move':return mutate('Move layer',p=>{const item=ensureItem(p,text(a.itemId));item.at=Math.max(0,quantize(number(a.at),p.settings.fps));if(a.trackId){const from=p.tracks.find(t=>t.items.some(i=>i.id===item.id))!;const to=p.tracks.find(t=>t.id===a.trackId);if(!to||to.kind!==from.kind)throw new ClipKitError('INVALID_TRACK','Move layers between tracks of the same kind');if(from!==to){from.items=from.items.filter(i=>i.id!==item.id);to.items.push(item);}}});
      case 'history_list':return store.history.slice(-Math.min(30,number(a.limit,15))).map(({rev,label,author,timestamp,changedItemIds,turnId})=>({rev,label,author,timestamp,changedItemIds,turnId}));
      case 'history_undo':return store.undo(actor);
      case 'history_redo':return store.redo(actor);
      case 'history_revert_turn':return store.revertLatestTurn();
      case 'comments_add':{const commentId=id('comment');const result=await mutate('Pin comment',p=>{const anchor=a.anchor??{tStart:0};p.comments.push({id:commentId,author:a.author==='voice'?'voice':author==='agent'?'agent':'human',text:text(a.text),anchor,status:'open',createdRev:p.rev,replies:[]});});return {...result,commentId};}
      case 'comments_list':return {rev:store.project.rev,comments:store.project.comments.filter(c=>!a.open||c.status!=='resolved').slice(0,Math.min(20,number(a.limit,10))).map(({id,text,anchor,status})=>({id,text,anchor,status}))};
      case 'comments_bundle':return bundle((a.commentIds??store.project.comments.filter(c=>c.status==='open').map(c=>c.id)).slice(0,10));
      case 'comments_resolve':return mutate('Resolve comment',p=>{const comment=p.comments.find(c=>c.id===a.commentId);if(!comment)throw new ClipKitError('UNKNOWN_COMMENT','Comment no longer exists');comment.status='resolved';comment.resolvedRev=p.rev+1;if(a.reply)comment.replies.push({author:author==='agent'?'agent':'human',text:text(a.reply,1000)});comment.diff=store.history.filter(h=>h.rev>comment.createdRev&&h.changedItemIds.includes(comment.anchor.layerId??'')).flatMap(h=>h.ops).slice(-30);});
      case 'comments_reply':return mutate('Reply to comment',p=>{const comment=p.comments.find(c=>c.id===a.commentId);if(!comment)throw new ClipKitError('UNKNOWN_COMMENT','Comment no longer exists');comment.replies.push({author:author==='agent'?'agent':'human',text:text(a.text,1000)});});
      case 'agent_client':agentClient=a.client==='claude'?'claude':'codex';notify();return {client:agentClient};
      case 'job_retry':{store.guard(actor);const previous=jobs.find(j=>j.id===a.jobId);const work=jobWork.get(a.jobId);if(!previous||!work||previous.status!=='failed')throw new ClipKitError('INVALID_JOB','Only failed jobs can be retried');if(previous.kind==='import')throw new ClipKitError('REIMPORT_REQUIRED','Choose the media file again to retry its import');return job(previous.kind,previous.label,work);}
      case 'job_status':{const target=jobs.find(j=>j.id===a.jobId);if(!target)throw new ClipKitError('UNKNOWN_JOB','Job no longer exists');const deadline=Date.now()+15000;while(['queued','running'].includes(target.status)&&Date.now()<deadline)await new Promise(r=>setTimeout(r,200));return target;}
      case 'agent_run':{store.guard(actor);agentClient=a.client==='claude'?'claude':'codex';const ids=(a.commentIds??store.project.comments.filter(c=>c.status==='open').map(c=>c.id)).slice(0,10);return runner.enqueue(agentClient,text(a.prompt??'Resolve the attached comments with precise, minimal edits.',6000),ids);}
      case 'agent_cancel':runner.cancel();return {cancelled:true};
      case 'status_working':return store.acquire(text(a.owner??'external-agent',100),text(a.label??'Editing',200));
      case 'status_heartbeat':return store.heartbeat(text(actor?.leaseToken??a.leaseToken,200));
      case 'status_idle':store.release(text(actor?.leaseToken??a.leaseToken,200));return {idle:true};
      case 'footage_add':store.guard(actor);return media.importFile(text(a.path,2000));
      case 'footage_transcribe':store.guard(actor);return media.transcribe(text(a.sourceId));
      case 'footage_candidates':return media.candidates(text(a.sourceId));
      case 'footage_pack':{const sourceId=text(a.sourceId);const start=number(a.start),end=number(a.end,36000);const words=(transcripts[sourceId]??[]).filter(w=>w.end>=start&&w.start<=end);const phrases:Array<{start:number;end:number;text:string}>=[];for(const w of words){const last=phrases.at(-1);if(last&&last.text.length<100&&w.start-last.end<.7){last.text+=' '+w.text;last.end=w.end;}else phrases.push({...w});}return {sourceId,phrases:phrases.slice(0,80),truncated:phrases.length>80,candidates:media.candidates(sourceId)};}
      case 'footage_delete_words':store.guard(actor);return media.deleteWords({sourceId:text(a.sourceId),itemId:text(a.itemId),start:number(a.start),end:number(a.end)});
      case 'voice_script':store.guard(actor);return media.script(text(a.text,50000));
      case 'voice_mode':store.guard(actor);return media.mode(text(a.slotId),a.mode);
      case 'voice_take':store.guard(actor);return media.selectTake(text(a.slotId),text(a.takeId));
      case 'voice_generate':store.guard(actor);return media.generate({slotId:text(a.slotId),voiceRef:a.voiceRef,speed:number(a.speed,1)});
      case 'voice_clone':store.guard(actor);return media.clone(text(a.sampleSourceId));
      case 'voice_transcribe':store.guard(actor);return media.transcribe(text(a.sourceId));
      case 'voice_window':return mutate('Change narration window',p=>{const slot=p.voiceSlots.find(s=>s.id===a.slotId);if(!slot)throw new ClipKitError('UNKNOWN_SLOT','Unknown voice slot');slot.targetWindow={start:number(a.start),end:number(a.end)};for(const item of allItems(p).filter(i=>i.slotId===slot.id)){item.at=slot.targetWindow.start;item.dur=slot.targetWindow.end-slot.targetWindow.start;}});
      case 'voice_command':return routeVoice(text(a.text,4000));
      case 'doctor':return media.doctor();
      case 'export_start':{const snapshot=structuredClone(store.project);const missing=snapshot.voiceSlots.filter(s=>allItems(snapshot).some(i=>i.slotId===s.id)&&!s.activeTakeByMode[s.mode]);if(missing.length)throw new ClipKitError('VOICE_NOT_READY','Record or generate all active narration lines before exporting.');return job('export',`Exporting revision ${snapshot.rev}`,async update=>{const result=await renderProject({project:snapshot,root,mediaBaseUrl:`${url}/media`,assetBaseUrl:url,accessToken:token,onProgress:p=>update(p),...(options.renderEntry?{entryPoint:options.renderEntry}:{}),assetDirectory:publicDir});return {path:result.path,url:`/exports/${basename(result.path)}?token=${token}`,creditsUrl:`/exports/${basename(result.creditsPath)}?token=${token}`,revision:snapshot.rev};});}
      case 'demo_reset':return mutate('Load demo graphics fixture',p=>{const empty=emptyProject();p.name='The next chapter';p.sources={};p.comments=[];p.voiceSlots=[];p.tracks=empty.tracks;const bg=makeClip('gradient');bg.dur=15;const title=makeClip('kinetic-title',.4);title.dur=5.6;title.props={...title.props,text:'Small edits.\nBig stories.',subtitle:'A shared canvas for you and your agent.',size:96};const lower=makeClip('lower-third',6);lower.dur=4;lower.props={...lower.props,text:'Made together',subtitle:'Human instinct. Agent precision.'};const cta=makeClip('cta',10);cta.dur=5;cta.props={...cta.props,text:'Your story.\nYour control.',subtitle:'Make the next frame count.'};p.tracks.find(t=>t.kind==='overlay')!.items=[bg,title,lower,cta];});
      case 'project_clear':return mutate('Clear timeline',p=>{for(const t of p.tracks)t.items=[];p.voiceSlots=[];});
      default:throw new ClipKitError('UNKNOWN_OPERATION',`Unknown operation: ${name}`);
    }
  }
  const allowedOrigin=(origin:string)=>{try{const u=new URL(origin);return ['http:'].includes(u.protocol)&&['127.0.0.1','localhost'].includes(u.hostname)&&[String(port),'5173'].includes(u.port);}catch{return false;}};
  const equal=(value:string|undefined)=>Boolean(value&&value.length===token.length&&timingSafeEqual(Buffer.from(value),Buffer.from(token)));
  app.use('*',async(c,next)=>{const origin=c.req.header('Origin');const host=c.req.header('Host')??'';if(!new RegExp(`^(127\\.0\\.0\\.1|localhost):(${port}|5173)$`).test(host))return c.json({code:'INVALID_HOST',error:'Only loopback hosts are accepted'},403);if(origin&&!allowedOrigin(origin))return c.json({code:'INVALID_ORIGIN',error:'Origin is not permitted'},403);if(origin)c.header('Access-Control-Allow-Origin',origin);c.header('Access-Control-Allow-Headers','Content-Type, X-ClipKit-Token, X-ClipKit-Lease, X-ClipKit-Turn');c.header('Access-Control-Allow-Methods','GET, POST, OPTIONS');c.header('Cache-Control','no-store');if(c.req.method==='OPTIONS')return c.body(null,204);await next();});
  const isPrivate=(path:string)=>path.startsWith('/api/')&&path!=='/api/bootstrap'||path.startsWith('/media/')||path.startsWith('/exports/');
  app.use('*',async(c,next)=>{if(isPrivate(c.req.path)&&!equal(c.req.header('X-ClipKit-Token')??c.req.query('token')))return c.json({code:'UNAUTHORIZED',error:'Open the studio again to reconnect'},401);await next();});
  app.onError((error,c)=>{const e=error as ClipKitError;return c.json({code:e.code??'OPERATION_FAILED',error:e.message.slice(0,1500),...e.details},e.code==='LOCKED'||e.code==='REV_CONFLICT'?409:400);});
  app.get('/api/bootstrap',c=>c.json({token}));
  app.get('/api/state',c=>c.json(state()));
  app.post('/api/op/:name',async c=>{const args=await c.req.json().catch(()=>({}));if(!args||typeof args!=='object'||Array.isArray(args))throw new ClipKitError('INVALID_REQUEST','Expected an argument object');const leaseToken=c.req.header('X-ClipKit-Lease');const actor=leaseToken?{leaseToken,turnId:c.req.header('X-ClipKit-Turn')}:undefined;const result=await mutationContext.run(actor??{},()=>operate(c.req.param('name'),args,actor));if(leaseToken){telemetry.contextTokens+=tokenEstimate({operation:c.req.param('name'),args,result});notify();}return c.json(result as any);});
  app.post('/api/upload',async c=>{store.guard();if(Number(c.req.header('Content-Length')??0)>1024**3)throw new ClipKitError('FILE_TOO_LARGE','Choose a file under 1 GB');const form=await c.req.formData();const file=form.get('file');if(!(file instanceof File))throw new ClipKitError('INVALID_UPLOAD','Upload a media file');const kind=c.req.query('kind')??'footage';if(!['footage','recording','clone-sample','command'].includes(kind))throw new ClipKitError('INVALID_UPLOAD','Unknown recording kind');const dir=join(root,'.clipkit','uploads');await mkdir(dir,{recursive:true});const filePath=join(dir,`${id('upload')}${extname(file.name).replace(/[^.a-z0-9]/gi,'').slice(0,10)||'.webm'}`);await writeFile(filePath,new Uint8Array(await file.arrayBuffer()));let result:unknown;const j=job('import',kind==='footage'?'Preparing imported media':'Preparing microphone audio',async update=>{try{update(.1);result=await media.importFile(filePath,{kind:kind as any,slotId:c.req.query('slotId'),mode:'recorded'});return result as Record<string,unknown>;}finally{await rm(filePath,{force:true});}});while(j.status==='queued'||j.status==='running')await new Promise(r=>setTimeout(r,100));if(j.status==='failed')throw new ClipKitError('IMPORT_FAILED',j.error??'Import failed');return c.json(result as any);});
  const serveFile=async(c:any,path:string)=>{const info=await stat(path);const range=c.req.header('Range');const headers:Record<string,string>={'Content-Type':mime[extname(path)]??'application/octet-stream','Accept-Ranges':'bytes','Cache-Control':'private, max-age=3600',...(c.req.header('Origin')?{'Access-Control-Allow-Origin':c.req.header('Origin')}:{})};let start=0,end=info.size-1;if(range){const match=range.match(/^bytes=(\d*)-(\d*)$/);if(!match)return c.body(null,416);if(!match[1]){start=Math.max(0,info.size-Number(match[2]));}else start=Number(match[1]);if(match[1]&&match[2])end=Math.min(end,Number(match[2]));if(start>end||start>=info.size)return c.body(null,416);headers['Content-Range']=`bytes ${start}-${end}/${info.size}`;}headers['Content-Length']=String(end-start+1);return new Response(Readable.toWeb(createReadStream(path,{start,end})) as ReadableStream,{status:range?206:200,headers});};
  app.get('/media/:id',async c=>{const source=store.project.sources[c.req.param('id')]??registeredSources.get(c.req.param('id'));if(!source)throw new ClipKitError('MISSING_MEDIA','Media is no longer registered');const variant=c.req.query('variant');return serveFile(c,resolve(root,variant==='proxy'?(source.proxyPath??source.path):variant==='audio'?(source.audioPath??source.path):source.path));});
  app.get('/exports/:file',c=>serveFile(c,safeJoin(join(root,'.clipkit','exports'),basename(c.req.param('file')))));
  app.get('/vendor/*',c=>serveFile(c,safeJoin(publicDir,c.req.path.slice(1))));
  app.get('*',async c=>{try{return await serveFile(c,safeJoin(studioDir,c.req.path==='/'?'index.html':c.req.path.slice(1)));}catch{try{return await serveFile(c,join(studioDir,'index.html'));}catch{return c.html('<h1>ClipKit server is ready</h1><p>Development studio: <a href="http://127.0.0.1:5173">open editor</a>. Run pnpm build for the packaged studio.</p>');}}});
  const server=serve({fetch:app.fetch,hostname:'127.0.0.1',port});
  await new Promise<void>((resolve,reject)=>{server.once('listening',resolve);server.once('error',reject);});
  const wss=new WebSocketServer({noServer:true});server.on('upgrade',(req,socket,head)=>{const requestUrl=new URL(req.url??'/',url);if(requestUrl.pathname!=='/events'||!equal(requestUrl.searchParams.get('token')??undefined)||req.headers.origin&&!allowedOrigin(req.headers.origin)){socket.destroy();return;}wss.handleUpgrade(req,socket,head,ws=>{wss.emit('connection',ws,req);ws.send(JSON.stringify({type:'state',state:state()}));});});
  let scheduled=false;notify=()=>{if(scheduled)return;scheduled=true;setTimeout(()=>{scheduled=false;const payload=JSON.stringify({type:'state',state:state()});for(const socket of wss.clients)if(socket.readyState===WebSocket.OPEN)socket.send(payload);},20);};
  store.onChange=()=>{for(const source of Object.values(store.project.sources))registeredSources.set(source.id,source);const h=store.history.at(-1);if(h&&activity.at(-1)?.text!==h.label)addActivity({at:h.timestamp,text:h.label,itemIds:h.changedItemIds});notify();};
  capabilities={...(await media.doctor()),codex:true,claude:true};
  await mkdir(store.dir,{recursive:true});await writeFile(join(store.dir,'server.json'),JSON.stringify({port,url,token,pid:process.pid,root}),{mode:0o600});
  const expiry=setInterval(()=>store.currentLease(),1000);
  return {app,server,store,media,runner,state,operate,url,token,close:async()=>{clearInterval(expiry);runner.cancel();for(const ws of wss.clients)ws.close();wss.close();await new Promise<void>((resolve,reject)=>server.close(e=>e?reject(e):resolve()));await rm(join(store.dir,'server.json'),{force:true});}};
}
