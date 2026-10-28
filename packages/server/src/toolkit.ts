import {mkdir,readFile,writeFile,cp} from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {allItems,findItem,duration} from '../../core/src/index';
import type {ProjectStore} from '../../core/src/store';
import {ClipKitError} from '../../core/src/store';
import {getProperty,setProperty} from '../../assets/src/property-path';
import {ComponentIndex} from '../../assets/src/component-index';
import {Resources} from '../../assets/src/resources';
import {sourceHashes,readDiskScene,sourceName} from '../../scenes/src/disk';
import type {SceneService,SceneDraft} from '../../scenes/src/service';

export const toolkitOperations=['project_open','project_context','project_sync','component_search','component_inspect','component_add','component_validate','resource_import','property_patch','render','source_bundle'];
const sceneId=(id:unknown)=>{if(typeof id!=='string'||!/^[a-z][a-z0-9-]{0,60}$/.test(id))throw Error('Use a lowercase scene ID');return id};
export class Toolkit {
  readonly components:ComponentIndex;readonly resources:Resources;
  constructor(readonly o:{root:string;kitRoot:string;url:string;store:ProjectStore;scenes:SceneService;operate:(name:string,args?:Record<string,any>)=>Promise<any>;job:(label:string,work:()=>Promise<Record<string,unknown>>)=>unknown}){this.components=new ComponentIndex(o.root,o.kitRoot);this.resources=new Resources(o.root)}
  async call(name:string,a:Record<string,any>){
    const {root,store,scenes,operate}=this.o;
    if(name==='component_search')return {...a.refresh?{refresh:await this.components.refresh()}:{},results:await this.components.search(String(a.query??''),{clock:a.clock,status:a.status,limit:a.limit}),note:'Discoverable effects may need adaptation; ready effects have executable adapters.'};
    if(name==='component_inspect')return {...await this.components.inspect(String(a.id)),...a.source?{source:await this.components.source(String(a.id))}:{}};
    if(name==='project_open'){
      store.guard();if(a.settings){if(allItems(store.project).length&&a.baseRev!==store.project.rev)throw new ClipKitError('REV_CONFLICT','Supply baseRev to change settings of an existing film');await store.mutate('Set film settings',p=>{p.settings={...p.settings,...a.settings}})}
      let starter:Record<string,string>|undefined;
      if(a.sceneId||!allItems(store.project).length){const id=sceneId(a.sceneId??'film'),directory=path.join(root,'scenes',id);await mkdir(directory,{recursive:true});const manifestPath=path.join(directory,'scene.json');try{await readFile(manifestPath)}catch(error){if((error as NodeJS.ErrnoException).code!=='ENOENT')throw error;
        const manifest={manifestVersion:2,sceneId:id,name:'Untitled film',entry:'Scene.tsx',durationFrames:a.durationFrames??24*store.project.settings.fps,files:{'Scene.tsx':''},defaults:{headline:'Make it move',headlineSize:200,color:'#f5f1e8'},fields:[],propsSchema:{type:'object',additionalProperties:false,properties:{headline:{type:'string'},headlineSize:{type:'number',minimum:1,maximum:1000},color:{type:'string'}},required:['headline','headlineSize','color']}};
        await writeFile(path.join(directory,'Scene.tsx'),`import React from 'react';import {AbsoluteFill} from 'remotion';export default function Film({headline,headlineSize,color}){return <AbsoluteFill style={{background:'#171717',color,fontSize:headlineSize,fontFamily:'Inter',justifyContent:'center',padding:80}}>{headline}</AbsoluteFill>}\n`);
        await writeFile(manifestPath,JSON.stringify(manifest,null,2));
      }starter={sceneId:id,manifest:manifestPath,source:path.join(directory,'Scene.tsx')};}
      return {...starter?{starter}:{},projectId:store.project.id,revision:store.project.rev,settings:store.project.settings,studioUrl:this.o.url,sourceDirectory:path.join(root,'scenes'),workflow:'Host agent authors TSX, sync publishes it, render exports automatically. No nested model or approval checkpoint.'};
    }
    if(name==='project_context'){
      const context=await operate('context_get'),selected=a.itemId??context.selection?.layerId;
      const items=allItems(store.project).filter(i=>!selected||i.id===selected).slice(0,12);
      return {revision:store.project.rev,settings:store.project.settings,durationFrames:Math.round(duration(store.project)*store.project.settings.fps),selection:context.selection,playhead:context.playhead,range:a.range??context.loop,items:items.map(i=>({id:i.id,name:i.name,sceneId:i.sceneId,version:i.sceneVersion,at:i.at,duration:i.dur,props:i.props,sourceDirectory:i.sceneId?path.join(root,'scenes',i.sceneId):undefined})),comments:store.project.comments.filter(c=>c.status!=='resolved'&&(!selected||c.anchor.layerId===selected)).slice(0,8)};
    }
    if(name==='resource_import'){store.guard();return this.resources.import(String(a.path))}
    if(name==='property_patch'){
      store.guard();const item=findItem(store.project,String(a.itemId));if(!item)throw Error('Unknown item');
      const expected=JSON.stringify(a.expected);if('expected'in a&&JSON.stringify(getProperty(item.props,String(a.path)))!==expected)throw new ClipKitError('REV_CONFLICT','Property changed; reread the selected value');
      const entry=await store.mutate('Edit '+String(a.path),p=>{const target=findItem(p,item.id)!;if('expected'in a&&JSON.stringify(getProperty(target.props,String(a.path)))!==expected)throw new ClipKitError('REV_CONFLICT','Property changed');target.props=setProperty(target.props,String(a.path),a.value)},'agent');
      return {revision:entry.rev,itemId:item.id,path:a.path,value:a.value};
    }
    if(name==='project_sync'){
      store.guard();const id=sceneId(a.sceneId);let draft:SceneDraft;
      if(a.manifest){
        if(!a.files||typeof a.files!=='object')throw Error('Provide changed source files with the manifest');
        const existing=a.baseVersion?await scenes.read(String(a.baseVersion)):undefined;
        const files:Record<string,string>={};for(const file of Object.keys(a.manifest.files??{})){
          if(!sourceName(file))throw Error('Invalid source path');const code=a.files[file]??existing?.files[file];if(typeof code!=='string')throw Error('Missing changed source: '+file);files[file]=code;
        }
        draft={...a.manifest,sceneId:id,files};
      }else draft=await readDiskScene(root,id);
      draft={...draft,manifestVersion:2,fps:store.project.settings.fps};
      const targets=allItems(store.project).filter(i=>i.sceneId===id);
      if(targets.length){draft.targetItemIds=a.targetItemIds??draft.targetItemIds??targets.map(i=>i.id);draft.baseVersion=a.baseVersion??draft.baseVersion;if(!draft.baseVersion)throw new ClipKitError('REV_CONFLICT','Provide the current source version');}
      if(a.range)draft.range=a.range;
      for(const resource of draft.resources??[])await this.resources.get(resource);
      // Compilation/verification happens without a long-lived editor lease. Publication checks versions.
      return this.o.job('Compile '+draft.name,async()=>{
        const result=await scenes.draft(draft,undefined);
        return {...result,files:sourceHashes(draft.files),changedFiles:Object.keys(a.files??draft.files),componentReceipts:draft.componentReceipts??[]};
      });
    }
    if(name==='component_add'){
      store.guard();const selected=await this.components.inspect(String(a.id)),id=sceneId(a.sceneId),folder=path.join(root,'scenes',id);await mkdir(path.join(folder,'components'),{recursive:true});
      if(selected.status!=='ready'){
        const source=await this.components.source(selected.id),destination=path.join(folder,'component-references',selected.id);await cp(source.sourceDirectory,destination,{recursive:true});await cp(source.licensePath,path.join(destination,'LICENSE.txt'));
        return {id:selected.id,status:'needs-adaptation',sourceDirectory:destination,receipt:source,task:'Adapt the actual selected source into components/. Declare its dependencies and a test scene, then component_validate. Raw browser code has not been executed.'};
      }
      const file=selected.adapterDirectory?'components/'+selected.id+'/'+selected.adapterFile:'components/'+selected.id+'.tsx';
      const code=selected.adapterPath?await readFile(path.join(this.o.kitRoot,selected.adapterPath),'utf8'):selected.adapterDirectory?await readFile(path.join(selected.adapterDirectory,selected.adapterFile!),'utf8'):`export {${selected.exportName} as default} from '${selected.module}';\n`;
      if(selected.adapterDirectory)await cp(selected.adapterDirectory,path.join(folder,'components',selected.id),{recursive:true});
      await writeFile(path.join(folder,file),code);
      const receipt={id:selected.id,revision:selected.revision,module:selected.module,exportName:selected.exportName,sourcePath:selected.sourcePath,repository:selected.repository,mechanism:selected.mechanism,status:'ready'};
      const manifestPath=path.join(folder,'scene.json');try{const manifest=JSON.parse(await readFile(manifestPath,'utf8'));manifest.files={...manifest.files,[file]:''};if(selected.adapterDirectory){const cached=JSON.parse(await readFile(path.join(selected.adapterDirectory,'adapter.json'),'utf8'));for(const name of cached.files)manifest.files['components/'+selected.id+'/'+name]='';}manifest.dependencies={...manifest.dependencies,...selected.dependencies};manifest.componentReceipts=[...(manifest.componentReceipts??[]).filter((r:any)=>r.id!==selected.id),receipt];await writeFile(manifestPath,JSON.stringify(manifest,null,2))}catch(e){if((e as NodeJS.ErrnoException).code!=='ENOENT')throw e}
      return {id:selected.id,status:'ready',dependencies:selected.dependencies??{},file:path.join(folder,file),import:`import ${selected.exportName??'Effect'} from './${file}';`,defaults:selected.defaults,receipt};
    }
    if(name==='component_validate'){
      const id=sceneId(a.sceneId),draft=await readDiskScene(root,id);const source=await this.components.source(String(a.id));
      if(!a.adapterFile||!sourceName(String(a.adapterFile))||!draft.files[a.adapterFile])throw Error('Declare the adapter file in the validation scene');
      return this.o.job('Validate '+a.id,async()=>{
        const result=await scenes.draft({...draft,manifestVersion:2,validateOnly:true},undefined);
        const key=createHash('sha256').update(JSON.stringify({source:source.sourceHash??source.revision,files:sourceHashes(draft.files),dependencies:draft.dependencies,abi:2})).digest('hex'),directory=path.join(this.components.cacheDirectory,'adapters',key);
        await mkdir(directory,{recursive:true});for(const [name,code]of Object.entries(draft.files)){await mkdir(path.dirname(path.join(directory,name)),{recursive:true});await writeFile(path.join(directory,name),code)}
        const receipt={id:a.id,key,source,adapterFile:a.adapterFile,directory,files:Object.keys(draft.files),defaults:draft.defaults,dependencies:draft.dependencies??{},validationVersion:result.version,technicalStatus:'passed',visualFidelity:'Requires comparison of moving preview against source; technical checks do not establish fidelity'};
        if(source.licensePath)await cp(source.licensePath,path.join(directory,'LICENSE.txt'));
        await writeFile(path.join(directory,'adapter.json'),JSON.stringify(receipt,null,2));return receipt;
      });
    }
    if(name==='render')return operate('export_start',{...a,format:'mp4',frameRange:a.range?[a.range.startFrame,a.range.endFrame-1]:undefined});
    if(name==='source_bundle')return operate('export_start',{format:'bundle'});
    throw Error('Unknown toolkit operation');
  }
}
