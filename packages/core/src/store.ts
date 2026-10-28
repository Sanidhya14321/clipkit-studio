import {isDeepStrictEqual} from 'node:util';
import {migrateProject} from './migration';
import {mkdir,readFile,writeFile,rename,open} from 'node:fs/promises';
import {join} from 'node:path';
import jsonPatch, {type Operation} from 'fast-json-patch';
import {ProjectSchema,emptyProject,allItems,id,type Project,type PatchRequest,type HistoryEntry,type Lease,type JsonPatchOperation,type Item} from './index.ts';
import {mergeRevert,RevertConflict} from './revert.ts';

export class ClipKitError extends Error {
  constructor(public code:string,message:string,public details:Record<string,unknown>={}) {super(message);}
}
export class ProjectStore {
  project:Project = emptyProject();
  history:HistoryEntry[]=[];
  lease:Lease|null=null;
  private layerLeases=new Map<string,Lease&{layerIds:string[]}>();
  layerLocks(){for(const [token,lease]of this.layerLeases)if(lease.expiresAt<=this.now())this.layerLeases.delete(token);return [...this.layerLeases.values()];}
  acquireLayers(layerIds:string[],owner:string,label:string){if(this.currentLease())throw new ClipKitError('LOCKED','Structural editing owns the global lease');if(!layerIds.length||layerIds.length>50||new Set(layerIds).size!==layerIds.length)throw new ClipKitError('INVALID_LOCK','Choose 1–50 distinct layers');const items=allItems(this.project);if(layerIds.some(id=>!items.some(i=>i.id===id)))throw new ClipKitError('UNKNOWN_ITEM','Reread the requested layers');if(this.layerLocks().some(l=>l.layerIds.some(id=>layerIds.includes(id))))throw new ClipKitError('LOCKED','A requested layer is already locked');const lease={owner,label,token:id('layers')+id('secret'),turnId:id('turn'),expiresAt:this.now()+this.timeoutMs,layerIds};this.layerLeases.set(lease.token,lease);this.onChange();return {...lease,revs:Object.fromEntries(layerIds.map(id=>[id,items.find(i=>i.id===id)!.rev??0]))};}
  layerLockState(){return this.layerLocks().map(({token,...lease})=>lease)}
  private chain:Promise<unknown>=Promise.resolve();
  private requests=new Map<string,HistoryEntry>();
  private undoStack:number[]=[];
  private redoStack:number[]=[];
  onChange:()=>void=()=>{};
  private timeoutMs:number;
  constructor(public root:string,private validateExtra:(project:Project)=>void=()=>{},private now:()=>number=Date.now,private options:{leaseTimeoutMs?:number;groupTurnId?:(turnId:string)=>string;prepare?:(next:Project,previous:Project)=>Promise<void>}={}) {this.timeoutMs=Math.max(100,options.leaseTimeoutMs??120000);}
  get dir(){return join(this.root,'.clipkit');}
  async load() {
    await mkdir(join(this.dir,'history'),{recursive:true});
    try {this.project=migrateProject(JSON.parse(await readFile(join(this.dir,'project.json'),'utf8')));} catch(e) {if((e as NodeJS.ErrnoException).code!=='ENOENT')throw e;}
    let journal='';try{journal=await readFile(join(this.dir,'history','journal.jsonl'),'utf8');}catch(e){if((e as NodeJS.ErrnoException).code!=='ENOENT')throw e;}
    const lines=journal.split('\n');
    for(let n=0;n<lines.length;n++){
      if(!lines[n].trim())continue;
      let h:HistoryEntry;try{h=JSON.parse(lines[n]);}catch(e){if(n===lines.length-1&&!journal.endsWith('\n')){await writeFile(join(this.dir,'history','journal.jsonl'),lines.slice(0,n).join('\n')+'\n');break;}throw new ClipKitError('CORRUPT_HISTORY',`History entry ${n+1} cannot be read`);}
      if(h.rev!==this.history.length+1)throw new ClipKitError('CORRUPT_HISTORY','History revision sequence is invalid');
      this.history.push(h);this.requests.set(h.requestId,h);this.rebuildStacks(h);
      if(h.rev>this.project.rev){const next=jsonPatch.applyPatch(structuredClone(this.project),h.ops as Operation[],true,true).newDocument;next.rev=h.rev;this.project=ProjectSchema.parse(next);}
    }
    if(this.project.rev>this.history.length)throw new ClipKitError('CORRUPT_HISTORY','Snapshot is ahead of its journal');
    this.validateExtra(this.project);
    await this.snapshot();
    return this;
  }
  private rebuildStacks(h:HistoryEntry){
    if(h.kind==='edit'){this.undoStack.push(h.rev);this.redoStack=[];}
    else if(h.kind==='undo'){this.undoStack.pop();if(h.target)this.redoStack.push(h.target);}
    else if(h.kind==='redo'){this.redoStack.pop();if(h.target)this.undoStack.push(h.target);}
  }
  private serial<T>(fn:()=>Promise<T>):Promise<T>{const result=this.chain.then(fn,fn);this.chain=result.catch(()=>{});return result;}
  private async snapshot(){const tmp=join(this.dir,`project.${process.pid}.tmp`);await writeFile(tmp,JSON.stringify(this.project,null,2));await rename(tmp,join(this.dir,'project.json'));}
  currentLease(){if(this.lease&&this.lease.expiresAt<=this.now()){this.lease=null;this.onChange();}return this.lease;}
  acquire(owner:string,label:string){if(this.currentLease()||this.layerLocks().length)throw new ClipKitError('LOCKED','An agent is already editing');this.lease={owner,label,token:id('lease')+id('secret'),turnId:id('turn'),expiresAt:this.now()+this.timeoutMs};this.onChange();return {...this.lease};}
  heartbeat(token:string){const lease=this.currentLease()??this.layerLocks().find(l=>l.token===token);if(!lease||lease.token!==token)throw new ClipKitError('LEASE_EXPIRED','This editing lease is no longer valid');lease.expiresAt=this.now()+this.timeoutMs;return {...lease};}
  release(token:string){this.layerLocks();if(this.layerLeases.has(token)){this.layerLeases.delete(token);this.onChange();return;}if(this.currentLease()?.token!==token)throw new ClipKitError('LEASE_EXPIRED','This editing lease is no longer valid');this.lease=null;this.onChange();}
  guard(actor?:{leaseToken?:string;turnId?:string}){
    const lease=this.currentLease();
    const scoped=actor?.leaseToken?this.layerLocks().find(l=>l.token===actor.leaseToken&&l.turnId===actor.turnId):undefined;
    if(actor?.leaseToken&&!scoped&&(!lease||lease.token!==actor.leaseToken||lease.turnId!==actor.turnId))throw new ClipKitError('LEASE_EXPIRED','The agent turn has ended; reread before editing');
    if(lease&&(!actor?.leaseToken||lease.token!==actor.leaseToken))throw new ClipKitError('LOCKED',`${lease.owner} is editing. Wait or cancel its turn.`,{owner:lease.owner});
  }
  async patch(req:PatchRequest,author:HistoryEntry['author']='human') {return this.serial(()=>this.apply(req,author));}
  private async apply(req:PatchRequest,author:HistoryEntry['author'],kind:HistoryEntry['kind']='edit',target?:number){
    if(!req.requestId||req.requestId.length>200)throw new ClipKitError('INVALID_REQUEST','requestId is required');
    const seen=this.requests.get(req.requestId);if(seen)return seen;
    this.guard(req);
    if(req.baseRev!==this.project.rev)throw new ClipKitError('REV_CONFLICT','The project changed; reread affected layers.',{currentRev:this.project.rev});
    if(!Array.isArray(req.ops)||req.ops.length>1000)throw new ClipKitError('INVALID_PATCH','Use at most 1000 patch operations');
    for(const op of req.ops){
      if(!op.path.startsWith('/')||op.path==='/'||/^\/(rev|schemaVersion|id)(\/|$)/.test(op.path)||/__(?:proto)__|constructor|prototype/.test(op.path+(op.from??'')))throw new ClipKitError('INVALID_PATCH',`Protected patch path: ${op.path}`);
    }
    let next:Project;
    try {next=ProjectSchema.parse(jsonPatch.applyPatch(structuredClone(this.project),req.ops as Operation[],true,true).newDocument);await this.options.prepare?.(next,this.project);next=ProjectSchema.parse(next);this.validateExtra(next);}
    catch(e){throw new ClipKitError('INVALID_PATCH',(e as Error).message.slice(0,700),{currentRev:this.project.rev});}
    this.guard(req);
    const activeLocks=this.layerLocks(),scoped=activeLocks.find(l=>l.token===req.leaseToken);if(activeLocks.length){const global=(p:Project)=>({...p,rev:0,tracks:p.tracks.map(t=>({...t,items:t.items.map(i=>i.id)}))});if(JSON.stringify(global(this.project))!==JSON.stringify(global(next)))throw new ClipKitError('LAYER_SCOPE','Structural changes need the global lease');const end=(p:Project)=>Math.max(0,...allItems(p).map(i=>i.at+i.dur));if(Math.abs(end(this.project)-end(next))>1/this.project.settings.fps)throw new ClipKitError('LAYER_SCOPE','Changing total duration needs the global lease');for(const item of allItems(next)){const old=allItems(this.project).find(i=>i.id===item.id)!;if(!isDeepStrictEqual(item,old)){if(scoped&&!scoped.layerIds.includes(item.id))throw new ClipKitError('LAYER_SCOPE','Patch only your locked layers');if(activeLocks.some(l=>l.token!==req.leaseToken&&l.layerIds.includes(item.id)))throw new ClipKitError('LOCKED','This layer belongs to another editor');}}}
    const oldItems=new Map(allItems(this.project).map(i=>[i.id,i]));for(const item of allItems(next)){const old=oldItems.get(item.id);const semantic=(i:Item)=>{const {rev,...data}=i;return data};if(!old)item.rev=0;else if(!isDeepStrictEqual(semantic(old),semantic(item)))item.rev=(old.rev??0)+1;else if(old.rev!==undefined)item.rev=old.rev;else delete item.rev;}
    const before=this.project;
    const forward=jsonPatch.compare(before,next) as JsonPatchOperation[];
    const inverse=jsonPatch.compare(next,before) as JsonPatchOperation[];
    const previous=new Map(allItems(before).map(i=>[i.id,i]));
    const current=new Map(allItems(next).map(i=>[i.id,i]));
    const changedItemIds=[...new Set([...previous.keys(),...current.keys()])].filter(k=>!isDeepStrictEqual(previous.get(k),current.get(k)));
    next.rev=before.rev+1;
    const entry:HistoryEntry={id:id('tx'),rev:next.rev,baseRev:before.rev,requestId:req.requestId,label:req.label.slice(0,200),author,turnId:req.turnId?(this.options.groupTurnId?.(req.turnId)??req.turnId):undefined,timestamp:new Date(this.now()).toISOString(),ops:forward,inverse,changedItemIds,kind,target};
    const file=await open(join(this.dir,'history','journal.jsonl'),'a');try{await file.write(JSON.stringify(entry)+'\n');await file.sync();}finally{await file.close();}
    this.project=next;this.history.push(entry);this.requests.set(req.requestId,entry);this.rebuildStacks(entry);
    await this.snapshot();if(req.leaseToken)this.heartbeat(req.leaseToken);this.onChange();return entry;
  }
  patchLayers(changes:Array<{id:string;rev:number;changes:Partial<Item>}>,actor:{leaseToken:string;turnId:string},validate:(draft:Project)=>void=()=>{}){return this.serial(async()=>{this.guard(actor);const scoped=this.layerLocks().find(l=>l.token===actor.leaseToken);if(!scoped)throw new ClipKitError('LAYER_SCOPE','Acquire layer locks first');const draft=structuredClone(this.project);for(const change of changes){const item=allItems(draft).find(i=>i.id===change.id);if(!item||!scoped.layerIds.includes(change.id))throw new ClipKitError('LAYER_SCOPE','Layer is not locked by this caller');if((item.rev??0)!==change.rev)throw new ClipKitError('LAYER_REV_CONFLICT','Reread only the changed layer',{layerId:change.id,currentLayerRev:item.rev??0});if(Object.keys(change.changes).some(k=>!['props','name','purpose','at','dur','srcIn','volume','z'].includes(k)))throw new ClipKitError('INVALID_PATCH','Unsupported layer change');Object.assign(item,{...change.changes,...change.changes.props?{props:{...item.props,...change.changes.props}}:{}})}validate(draft);return this.apply({requestId:id('layer-edit'),baseRev:this.project.rev,label:'Patch locked layers',...actor,ops:jsonPatch.compare(this.project,draft) as JsonPatchOperation[]},'agent');});}
  mutate(label:string,fn:(draft:Project)=>void,author:HistoryEntry['author']='human',actor?:{leaseToken?:string;turnId?:string},requestId=id('req')){
    return this.serial(async()=>{this.guard(actor);const draft=structuredClone(this.project);fn(draft);return this.apply({requestId,baseRev:this.project.rev,label,...actor,ops:jsonPatch.compare(this.project,draft) as JsonPatchOperation[]},author);});
  }
  undo(actor?:{leaseToken?:string;turnId?:string},author:HistoryEntry['author']='human'){return this.serial(async()=>{this.guard(actor);const target=this.undoStack.at(-1);if(!target)throw new ClipKitError('NOTHING_TO_UNDO','Nothing to undo');const h=this.history.find(h=>h.rev===target)!;return this.apply({requestId:id('undo'),baseRev:this.project.rev,label:`Undo: ${h.label}`,...actor,ops:h.inverse},author,'undo',target);});}
  redo(actor?:{leaseToken?:string;turnId?:string},author:HistoryEntry['author']='human'){return this.serial(async()=>{this.guard(actor);const target=this.redoStack.at(-1);if(!target)throw new ClipKitError('NOTHING_TO_REDO','Nothing to redo');const h=this.history.find(h=>h.rev===target)!;return this.apply({requestId:id('redo'),baseRev:this.project.rev,label:`Redo: ${h.label}`,...actor,ops:h.ops},author,'redo',target);});}
  snapshotAt(rev:number):Project{
    if(!Number.isInteger(rev)||rev<0||rev>this.project.rev)throw new ClipKitError('UNKNOWN_REVISION','Choose an existing revision');
    let snapshot=structuredClone(this.project);
    for(const h of [...this.history].reverse())if(h.rev>rev)snapshot=jsonPatch.applyPatch(snapshot,h.inverse as Operation[],true,true).newDocument;
    snapshot.rev=rev;return snapshot;
  }
  revert(rev:number,actor?:{leaseToken?:string;turnId?:string}){return this.serial(async()=>{
    this.guard(actor);const target=this.history.find(h=>h.rev===rev);if(!target)throw new ClipKitError('UNKNOWN_REVISION','Choose an existing revision');
    const before=this.snapshotAt(rev-1),after=this.snapshotAt(rev);before.rev=after.rev=this.project.rev;
    let merged:Project;try{merged=mergeRevert(before,after,this.project) as Project;}catch(e){if(e instanceof RevertConflict)throw new ClipKitError('REVERT_CONFLICT',e.message,{paths:e.paths});throw e;}
    return this.apply({requestId:id('revert'),baseRev:this.project.rev,label:`Revert revision ${rev}: ${target.label}`,...actor,ops:jsonPatch.compare(this.project,merged) as JsonPatchOperation[]},'human');
  });}
  revertLatestTurn(){return this.serial(async()=>{
    this.guard();const top=this.history.find(h=>h.rev===this.undoStack.at(-1));
    if(!top||top.author!=='agent'||!top.turnId)throw new ClipKitError('REVERT_CONFLICT','Only the latest unmodified agent turn can be reverted. Undo newer edits first.');
    const turnId=top.turnId;const entries:HistoryEntry[]=[];
    while(true){const h=this.history.find(h=>h.rev===this.undoStack.at(-1));if(!h||h.turnId!==turnId)break;entries.push(await this.apply({requestId:id('undo'),baseRev:this.project.rev,label:`Undo: ${h.label}`,ops:h.inverse},'human','undo',h.rev));}
    return {rev:this.project.rev,reverted:entries.length};
  });}
}
