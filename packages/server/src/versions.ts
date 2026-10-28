import {mkdir,readFile,writeFile,rename,readdir,rm} from 'node:fs/promises';
import path from 'node:path';
import {z} from 'zod';
import {mergeRevert} from '../../core/src/revert';
import {ProjectSchema,allItems,type Project} from '../../core/src/index';
const slug=(name:unknown)=>z.string().regex(/^[a-z0-9][a-z0-9-]{0,59}$/).parse(name);
const clean=(input:Project)=>{const p=structuredClone(input) as any;delete p.rev;delete p.activeBranch;delete p.branches;delete p.exports;delete p.benchRuns;for(const item of allItems(p))delete item.rev;return p};
export class ProjectVersions {
 private writes:Promise<unknown>=Promise.resolve();
 constructor(readonly root:string){}
 private file(name:string){return path.join(this.root,'.clipkit/branches',slug(name)+'.json')}
 private async save(name:string,data:unknown){await mkdir(path.dirname(this.file(name)),{recursive:true});const file=this.file(name),temp=file+'.tmp';await writeFile(temp,JSON.stringify(data));await rename(temp,file)}
 async read(name:string){const raw=JSON.parse(await readFile(this.file(name),'utf8'));return {name:slug(raw.name),base:ProjectSchema.parse(raw.base),project:ProjectSchema.parse(raw.project)}}
 snapshot(p:Project,label:string){this.writes=this.writes.catch(()=>{}).then(async()=>{const dir=path.join(this.root,'.clipkit/snapshots');await mkdir(dir,{recursive:true});await writeFile(path.join(dir,`${p.rev}-${label.toLowerCase().replace(/[^a-z0-9]+/g,'-').slice(0,50)}.json`),JSON.stringify(p))});return this.writes}
 async handle(name:string,a:any,c:{project:Project;actor?:any;guard:()=>void;mutate:(label:string,fn:(p:Project)=>void)=>Promise<any>}):Promise<any>{const p=c.project;if(!['branch_create','branch_switch','branch_compare','branch_merge','branch_discard','branch_view'].includes(name))return undefined;const branch=slug(a.name??'main');
 if(name==='branch_view'){if(c.actor)throw Error('Full branch snapshots are a Studio-only view');return {name:branch,project:branch===(p.activeBranch??'main')?p:(await this.read(branch)).project}}
 if(name==='branch_compare'){const other=(await this.read(branch)).project;return {left:p.activeBranch??'main',right:branch,leftRev:p.rev,rightRev:other.rev,changedLayerIds:[...new Set([...allItems(p).map(i=>i.id),...allItems(other).map(i=>i.id)])].filter(id=>JSON.stringify(allItems(p).find(i=>i.id===id))!==JSON.stringify(allItems(other).find(i=>i.id===id))).slice(0,30),note:'Compare in Studio for side-by-side playback; merge uses stable-ID three-way conflict detection.'}}
 c.guard();
 if(name==='branch_create'){try{await this.read(branch);throw Error('Branch already exists')}catch(e){if((e as NodeJS.ErrnoException).code!=='ENOENT')throw e}try{await this.read('main')}catch{await this.save('main',{name:'main',base:p,project:p})}await this.save(branch,{name:branch,base:p,project:p});return c.mutate('Create branch '+branch,next=>{next.activeBranch??='main';next.branches??=[];next.branches.push({name:branch,baseRev:p.rev,revision:p.rev})})}
 if(name==='branch_switch'){const target=await this.read(branch);const current=p.activeBranch??'main';let original;try{original=await this.read(current)}catch{original={name:current,base:p,project:p}}await this.save(current,{...original,project:p});return c.mutate('Switch branch '+branch,next=>{const rev=next.rev,branches=next.branches;for(const key of Object.keys(next))delete (next as any)[key];Object.assign(next,target.project,{rev,activeBranch:branch,branches});})}
 if(name==='branch_merge'){const branchState=await this.read(branch);const merged=mergeRevert(clean(branchState.project),clean(branchState.base),clean(p));return c.mutate('Merge branch '+branch,next=>{const rev=next.rev;for(const key of Object.keys(next))delete (next as any)[key];Object.assign(next,merged,{rev,activeBranch:p.activeBranch??'main',branches:p.branches})})}
 if(branch==='main'||branch===(p.activeBranch??'main'))throw Error('Cannot discard main or the active branch');await mkdir(path.join(this.root,'.clipkit/branches/discarded'),{recursive:true});await rename(this.file(branch),path.join(this.root,'.clipkit/branches/discarded',branch+'-'+Date.now()+'.json'));return c.mutate('Discard branch '+branch,next=>{next.branches=next.branches?.filter(b=>b.name!==branch)})
 }
 async close(){await this.writes}
}
