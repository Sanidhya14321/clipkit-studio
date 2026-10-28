import {z} from 'zod';
import {SourceHub} from '../../assets/src/source-hub';
import {collections} from '../../core/src/source-policy';
import type {Project} from '../../core/src/index';
export const sourceReads=['sources_license','sources_list','sources_lock','sources_vet','sources_gaps','sources_discover','collections_list','license_report'];
export const sourceWrites=['sources_override','sources_add','sources_update','sources_remove','sources_toggle','sources_score','collections_enable','collections_disable'];
export async function sourceOperation(hub:SourceHub,name:string,a:any,c:{project:Project;actor?:any;guard:()=>void;mutate:(label:string,fn:(p:Project)=>void)=>Promise<any>;job:(label:string,work:()=>Promise<any>)=>any;changed:()=>Promise<void>;licenses:()=>Promise<any>}){
 if(![...sourceReads,...sourceWrites].includes(name))return undefined;
 if(name==='sources_license')return hub.license(z.string().parse(a.id),z.number().int().nonnegative().parse(a.offset??0));
 if(name==='sources_list')return hub.list(z.number().int().nonnegative().parse(a.offset??0));
 if(name==='sources_lock'){const lock=await hub.lock();const entries=Object.entries(lock);const offset=z.number().int().nonnegative().parse(a.offset??0);return {items:entries.slice(offset,offset+8).map(([id,p])=>({id,commit:p.commit,licenseHash:p.license.hash,licenseClass:p.license.licenseClass})),next:offset+8<entries.length?offset+8:null}}
 if(name==='sources_gaps')return hub.gaps(c.project);
 if(name==='collections_list')return {items:collections.map(name=>({name,enabled:(c.project.enabledCollections??['Starter']).includes(name)}))};
 if(name==='license_report')return c.licenses();
 if(name==='sources_discover')return hub.discover(z.string().min(1).max(150).parse(a.query));
 if(name==='sources_vet'){const r=await hub.vet(z.string().parse(a.id));return {id:r.id,result:r.result,reasons:r.reasons.map(r=>r.slice(0,180)),license:{class:r.license.licenseClass,spdx:r.license.spdx,path:r.license.path,hash:r.license.hash,contradictory:r.license.contradictory},dependencies:Object.entries(r.dependencies).slice(0,20).map(([name,value])=>[name.slice(0,100),value.slice(0,120)]),dependencyCount:Object.keys(r.dependencies).length,facts:r.facts,noExecution:r.noExecution}}
 c.guard();if(c.actor)throw Error('HUMAN_SOURCE_APPROVAL_REQUIRED: ask the owner to approve source/collection changes in Studio or CLI');
 if(name.startsWith('collections_')){const collection=z.enum(collections).parse(a.name);return c.mutate('Collection: '+collection,p=>{const enabled=new Set(p.enabledCollections??['Starter']);if(name==='collections_enable')enabled.add(collection);else enabled.delete(collection);p.enabledCollections=[...enabled]})}
 const id=a.id===undefined?undefined:z.string().regex(/^[a-z0-9][a-z0-9-]{0,59}$/).parse(a.id);
 return c.job(name,()=>hub.serialize(async()=>{let result:any;if(name==='sources_add')result=await hub.add({repo:a.repo,id,overrideReason:a.overrideReason});if(name==='sources_update')result=await hub.update(id!,!!a.apply,a.overrideReason);if(name==='sources_remove')result=await hub.remove(id!,c.project,!!a.apply);if(name==='sources_toggle')result=await hub.toggle(id!,z.boolean().parse(a.enabled));if(name==='sources_override')result=await hub.override(id!,z.string().parse(a.reason));if(name==='sources_score')result=await hub.score(id!);await c.changed();return result}));
}
