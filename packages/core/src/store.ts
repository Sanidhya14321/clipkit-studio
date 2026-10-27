import {migrateProject} from './migration';
import {mkdir,readFile,writeFile,rename,open} from 'node:fs/promises';
import {join} from 'node:path';
import jsonPatch, {type Operation} from 'fast-json-patch';
import {ProjectSchema,emptyProject,allItems,id,type Project,type PatchRequest,type HistoryEntry,type Lease,type JsonPatchOperation} from './index.ts';
import {mergeRevert,RevertConflict} from './revert.ts';

export class ClipKitError extends Error {
  constructor(public code:string,message:string,public details:Record<string,unknown>={}) {super(message);}
}
export class ProjectStore {
  project:Project = emptyProject();
  history:HistoryEntry[]=[];
  lease:Lease|null=null;
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
  acquire(owner:string,label:string){if(this.currentLease())throw new ClipKitError('LOCKED','An agent is already editing');this.lease={owner,label,token:id('lease')+id('secret'),turnId:id('turn'),expiresAt:this.now()+this.timeoutMs};this.onChange();return {...this.lease};}
  heartbeat(token:string){const lease=this.currentLease();if(!lease||lease.token!==token)throw new ClipKitError('LEASE_EXPIRED','This editing lease is no longer valid');lease.expiresAt=this.now()+this.timeoutMs;return {...lease};}
  release(token:string){if(this.currentLease()?.token!==token)throw new ClipKitError('LEASE_EXPIRED','This editing lease is no longer valid');this.lease=null;this.onChange();}
  guard(actor?:{leaseToken?:string;turnId?:string}){
    const lease=this.currentLease();
    if(actor?.leaseToken&&(!lease||lease.token!==actor.leaseToken||lease.turnId!==actor.turnId))throw new ClipKitError('LEASE_EXPIRED','The agent turn has ended; reread before editing');
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
    const before=this.project;
    const forward=jsonPatch.compare(before,next) as JsonPatchOperation[];
    const inverse=jsonPatch.compare(next,before) as JsonPatchOperation[];
    const previous=new Map(allItems(before).map(i=>[i.id,JSON.stringify(i)]));
    const current=new Map(allItems(next).map(i=>[i.id,JSON.stringify(i)]));
    const changedItemIds=[...new Set([...previous.keys(),...current.keys()])].filter(k=>previous.get(k)!==current.get(k));
    next.rev=before.rev+1;
    const entry:HistoryEntry={id:id('tx'),rev:next.rev,baseRev:before.rev,requestId:req.requestId,label:req.label.slice(0,200),author,turnId:req.turnId?(this.options.groupTurnId?.(req.turnId)??req.turnId):undefined,timestamp:new Date(this.now()).toISOString(),ops:forward,inverse,changedItemIds,kind,target};
    const file=await open(join(this.dir,'history','journal.jsonl'),'a');try{await file.write(JSON.stringify(entry)+'\n');await file.sync();}finally{await file.close();}
    this.project=next;this.history.push(entry);this.requests.set(req.requestId,entry);this.rebuildStacks(entry);
    await this.snapshot();if(req.leaseToken)this.heartbeat(req.leaseToken);this.onChange();return entry;
  }
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
