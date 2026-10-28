import {AsyncLocalStorage} from 'node:async_hooks';
import {createReadStream} from 'node:fs';
import {createHash,randomUUID} from 'node:crypto';
import {mkdir,appendFile,writeFile,readFile,readdir,rename,stat} from 'node:fs/promises';
import path from 'node:path';
import jsonPatch from 'fast-json-patch';
import {zipSync,strToU8} from 'fflate';
import {diagnostic} from './errors';
export const sha=(value:unknown)=>createHash('sha256').update(typeof value==='string'||value instanceof Uint8Array?value:JSON.stringify(value)).digest('hex');
const canonical=(value:any):any=>Array.isArray(value)?value.map(canonical):value&&typeof value==='object'?Object.fromEntries(Object.keys(value).sort().map(k=>[k,canonical(value[k])])):value;
export const projectHash=(value:any)=>sha(canonical(value));
export const traceContext=new AsyncLocalStorage<{traceId:string;runId?:string;recorder?:FlightRecorder}>();
const personal=/prompt|text|headline|title|subtitle|copy|caption|narration|content|description|body|transcript|note|goal|message|label|name|question|answer|comment|script|voiceRef|referenceId|file|path|url|directory|sourceBase|attribution/i;
const secret=/(?:^|_)(?:token|access_token|refresh_token|secret|password|authorization|api_key|cookie|credentials?)$|apiKey|leaseToken|accessToken|refreshToken|auth\.json/i;
const secrets=new Set<string>();export function registerSecrets(values:string[]){for(const value of values)if(value.length>=8)secrets.add(value)}
registerSecrets(Object.entries(process.env).filter(([key])=>/TOKEN|SECRET|PASSWORD|API_KEY/.test(key)).map(([,v])=>v??''));
export function redact(value:any,options:{synthetic?:boolean}={},key='',depth=0):any{
 if(depth>30)return '[depth limit]';if(secret.test(key))return '[redacted]';
 if(typeof value==='string'){for(const known of secrets)value=value.split(known).join('[redacted]');if(/(?:^|\.)(?:sha|sha256|hash)$/i.test(key)&&/^[a-f0-9]{64}$/i.test(value))return value;if(value.startsWith('[redacted')||key==='path'&&/^\/(name|tracks|sources|settings|comments|brief|tokens|rev|voiceSlots|pipeline|plan|assetsUsed|enabledCollections|licenseMode|reel|brand|mode|derived|recipesUsed|variantSets|pacingProfile|exports|branches|activeBranch|benchRuns|replan|storyboardCheckpoint|criticReviews|videoPlans|directionPlans|soundDesign)(?:\/|$)/.test(value))return value;if(!options.synthetic&&(personal.test(key)||value.includes('/home/')||value.includes('/Users/')))return '[redacted:'+sha(value).slice(0,16)+']';return value.replace(/\b(?:lease|layers)_[a-f0-9]+secret_[a-f0-9]+\b/gi,'[redacted]').replace(/(?:Bearer\s+)[\w.~-]+/gi,'Bearer [redacted]').replace(/\b(?:sk-|sk_)[A-Za-z0-9_-]{8,}/g,'[redacted]').replace(/([?&](?:token|key|secret)=)[^&\s]+/gi,'$1[redacted]').slice(0,100000)}
 if(Array.isArray(value))return value.map(v=>redact(v,options,key,depth+1));if(value&&typeof value==='object')return Object.fromEntries(Object.entries(value).map(([k,v])=>[k,redact(v,options,/^files(?:\.|$)/.test(key)||personal.test(key)&&['value','code','raw','data'].includes(k)?key+'.'+k:k,depth+1)]));return value;
}
export function validRun(id:string){if(!/^[a-zA-Z0-9][a-zA-Z0-9_-]{0,100}$/.test(id))throw Error('E_SCHEMA: invalid run id');return id}
export class FlightRecorder{
 readonly runId:string;readonly dir:string;private writes:Promise<unknown>=Promise.resolve();private previous:any;private initialized=false;private mediaHashes=new Map<string,string>();
 constructor(readonly root:string,readonly options:{runId?:string;synthetic?:boolean;enabled?:boolean}={}){this.runId=validRun(options.runId??'run-'+Date.now()+'-'+randomUUID().slice(0,8));this.dir=path.join(root,'.clipkit/trace',this.runId)}
 private queue(work:()=>Promise<unknown>){if(this.options.enabled===false)return Promise.resolve();this.writes=this.writes.then(work);return this.writes}
 event(event:string,data:Record<string,unknown>={}){const ctx=traceContext.getStore(),entry={timestamp:new Date().toISOString(),level:'info',component:'studio',traceId:ctx?.traceId??this.runId,runId:this.runId,sessionId:null,turnId:null,passId:null,layerId:null,rev:this.previous?.rev??null,duration:0,event,...redact(data,this.options)};return this.queue(async()=>{await mkdir(this.dir,{recursive:true});await appendFile(path.join(this.dir,'events.jsonl'),JSON.stringify(entry)+'\n')})}
 snapshot(project:any,metadata:Record<string,unknown>={}){const sources=structuredClone(project.sources??{});const next=redact(project,this.options),previous=this.previous,ctx=traceContext.getStore();this.previous=structuredClone(next);return this.queue(async()=>{await mkdir(path.join(this.dir,'snapshots'),{recursive:true});await this.recordMediaHashes(sources);if(!this.initialized){await writeFile(path.join(this.dir,'manifest.json'),JSON.stringify({format:'clipkit.flight/v1',runId:this.runId,synthetic:!!this.options.synthetic,media:'hash references only; no media included',clock:'recorded timestamps; no claim of original live clock replay',node:process.version,...redact(metadata,this.options)},null,2));await writeFile(path.join(this.dir,'initial.json'),JSON.stringify(next));this.initialized=true}else if(previous?.rev!==next.rev){const patch={traceId:ctx?.traceId??this.runId,runId:this.runId,revBefore:previous.rev,rev:next.rev,ops:jsonPatch.compare(previous,next),hash:projectHash(next),hashAlgorithm:'canonical-json-v1'};await appendFile(path.join(this.dir,'patches.jsonl'),JSON.stringify(patch)+'\n')}const tmp=path.join(this.dir,'snapshots',next.rev+'.tmp');await writeFile(tmp,JSON.stringify(next));await rename(tmp,path.join(this.dir,'snapshots',next.rev+'.json'))})}
 private async recordMediaHashes(sources:Record<string,any>){const index:Record<string,unknown>={};for(const [id,source]of Object.entries(sources)){const file=path.resolve(this.root,source.path??'');if(!file.startsWith(path.resolve(this.root)+path.sep)){index[id]={status:'outside-workspace; excluded'};continue}try{const info=await stat(file);if(!info.isFile())continue;const key=file+':'+info.size+':'+info.mtimeMs;let hash=this.mediaHashes.get(key);if(!hash){const digest=createHash('sha256');for await(const chunk of createReadStream(file))digest.update(chunk);hash=digest.digest('hex');this.mediaHashes.set(key,hash)}index[id]={sha256:hash,bytes:info.size,kind:source.kind}}catch{index[id]={status:'unavailable'}}}await writeFile(path.join(this.dir,'media-index.json'),JSON.stringify(index))}
 async artifact(name:string,data:unknown){if(!/^[a-zA-Z0-9_-]+\.json$/.test(name))throw Error('Invalid trace artifact name');return this.queue(async()=>{await mkdir(this.dir,{recursive:true});await writeFile(path.join(this.dir,name),JSON.stringify(redact(data,this.options),null,2))})}
 async close(){await this.writes}
}
async function replayDirectory(dir:string,runId:string){let project=JSON.parse(await readFile(path.join(dir,'initial.json'),'utf8'));const revisions=[{rev:project.rev,hash:projectHash(project),traceId:undefined as string|undefined,project:structuredClone(project)}];let lines='';try{lines=await readFile(path.join(dir,'patches.jsonl'),'utf8')}catch(e){if((e as NodeJS.ErrnoException).code!=='ENOENT')throw e}for(const line of lines.split('\n').filter(Boolean)){const patch=JSON.parse(line);if(project.rev!==patch.revBefore)throw Error('E_PATCH_STALE: trace revision gap');project=jsonPatch.applyPatch(project,patch.ops,true,true).newDocument;if((patch.hashAlgorithm==='canonical-json-v1'?projectHash(project):sha(JSON.stringify(project)))!==patch.hash)throw Error('E_PATCH_INVALID: replay hash mismatch');revisions.push({rev:project.rev,hash:patch.hash,traceId:patch.traceId,project:structuredClone(project)})}return {runId,project,revisions}}
export async function replayTrace(root:string,runId:string){return replayDirectory(path.join(root,'.clipkit/trace',validRun(runId)),runId)}
export async function bisectTrace(root:string,runId:string,check:(project:any)=>boolean){const replay=await replayTrace(root,runId);/* Full ordered scan is correct even for non-monotonic predicates. */const failed=replay.revisions.find(r=>!check(r.project));return {runId,firstFailingRevision:failed?.rev??null,tested:replay.revisions.length,method:'ordered revision scan; handles non-monotonic failures'}}
export async function bundleTrace(root:string,runId:string){
 const dir=path.join(root,'.clipkit/trace',validRun(runId)),files:Record<string,Uint8Array>={};let size=0;
 async function walk(folder:string){
  for(const entry of await readdir(folder,{withFileTypes:true})){
   if(entry.isSymbolicLink())continue;const file=path.join(folder,entry.name);
   if(entry.isDirectory()){await walk(file);continue;}
   if(!/\.(json|jsonl|md)$/.test(entry.name))continue;
   const bytes=await readFile(file);size+=bytes.length;if(size>20*1024**2)throw Error('E_EXPORT: trace exceeds 20MB bundle limit');
   let value:string;
   if(entry.name.endsWith('.jsonl'))value=bytes.toString().split('\n').filter(Boolean).map(line=>JSON.stringify(redact(JSON.parse(line)))).join('\n')+'\n';
   else if(entry.name.endsWith('.json'))value=JSON.stringify(redact(JSON.parse(bytes.toString())));
   else value='See docs/DEBUG_PLAYBOOK.md';
   files[path.relative(dir,file)]=strToU8(value);
  }
 }
 await walk(dir);
 // Anonymizing synthetic/older traces changes project values. Rebuild their patches
 // and hashes against the anonymized revisions, after verifying the original log.
 for(const name of Object.keys(files).filter(name=>/(^|\/)initial\.json$/.test(name))){
  const prefix=name.slice(0,-'initial.json'.length);
  try{
   const original=await replayDirectory(path.join(dir,prefix),runId),revisions=original.revisions.map(revision=>({...revision,project:redact(revision.project)}));
   files[name]=strToU8(JSON.stringify(revisions[0].project));
   const patches=revisions.slice(1).map((revision,index)=>({runId,traceId:revision.traceId??runId,revBefore:revisions[index].rev,rev:revision.rev,ops:jsonPatch.compare(revisions[index].project,revision.project),hash:projectHash(revision.project),hashAlgorithm:'canonical-json-v1',originalHash:revision.hash}));
   files[prefix+'patches.jsonl']=strToU8(patches.map(patch=>JSON.stringify(patch)).join('\n')+(patches.length?'\n':''));
   for(const revision of revisions)files[prefix+'snapshots/'+revision.rev+'.json']=strToU8(JSON.stringify(revision.project));
   files[prefix+'bundle-replay.json']=strToU8(JSON.stringify({anonymized:true,originalLogVerified:true,revisions:revisions.length,hashAlgorithm:'canonical-json-v1'}));
  }catch(error){files[prefix+'bundle-replay-error.json']=strToU8(JSON.stringify({originalLogVerified:false,...redact(diagnostic(error))}));}
 }
 files['README.md']=strToU8('ClipKit redacted diagnostic trace. No media, credentials or original user copy. Anonymized revision hashes are rebuilt after original-log verification; see bundle-replay.json. A corrupt original log is preserved as redacted evidence with bundle-replay-error.json. Regenerate synthetic fixtures with ck fixtures generate. See docs/DEBUG_PLAYBOOK.md.');
 const out=path.join(root,'.clipkit/exports',runId+'-trace.zip');await mkdir(path.dirname(out),{recursive:true});await writeFile(out,zipSync(files));
 return {path:out,url:'/exports/'+path.basename(out),files:Object.keys(files).length,mediaIncluded:false};
}
export async function reproTrace(root:string,runId:string,layerIds:string[]=[]){const replay=await replayTrace(root,runId),p=structuredClone(replay.project);if(layerIds.length){for(const t of p.tracks??[])t.items=t.items.filter((i:any)=>layerIds.includes(i.id));const needed=new Set((p.tracks??[]).flatMap((t:any)=>t.items.flatMap((i:any)=>[i.src,...Object.values(i.props??{})].filter((v:any)=>p.sources?.[v]))));for(const id of Object.keys(p.sources??{}))if(!needed.has(id))delete p.sources[id];p.comments=[];p.voiceSlots=p.voiceSlots.filter((slot:any)=>p.tracks.some((t:any)=>t.items.some((i:any)=>i.slotId===slot.id)));for(const slot of p.voiceSlots)for(const take of slot.takes??[])if(replay.project.sources[take.sourceId])p.sources[take.sourceId]=replay.project.sources[take.sourceId]}const recorder=new FlightRecorder(root,{runId:validRun(runId+'-repro')});await recorder.snapshot(p,{originalRun:runId,minimized:layerIds.length>0,notIncluded:'media and voice samples'});await recorder.event('repro.created',{layerIds});await recorder.close();return bundleTrace(root,recorder.runId)}
export async function traceList(root:string,offset=0){let dirs:string[]=[];try{dirs=await readdir(path.join(root,'.clipkit/trace'))}catch{}return {items:dirs.sort().reverse().slice(offset,offset+8),next:offset+8<dirs.length?offset+8:null}}
export async function traceEvents(root:string,runId:string,offset=0){const lines=(await readFile(path.join(root,'.clipkit/trace',validRun(runId),'events.jsonl'),'utf8')).split('\n').filter(Boolean);return {runId,events:lines.slice(offset,offset+10).map(l=>{const e=JSON.parse(l);const {payload,args,...metadata}=e;return {...metadata,...payload?{payloadSummary:{type:payload.type,sha256:sha(payload),bytes:Buffer.byteLength(JSON.stringify(payload))}}:{}}}),next:offset+10<lines.length?offset+10:null}}

export function debugEvent(event:string,data:Record<string,unknown>={}){const c=traceContext.getStore();if(c?.recorder)void c.recorder.event(event,data).catch(()=>{})}
