import {it,expect,vi,beforeAll,afterAll} from 'vitest';
import {mkdtemp,mkdir,writeFile,readFile,rm} from 'node:fs/promises';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {startServer} from '../packages/server/src/index';
import {allItems} from '../packages/core/src/index';
import {exportOptions} from '../packages/render/src/options';
import {ComponentIndex,rankComponents,webSources} from '../packages/assets/src/component-index';
import {Resources} from '../packages/assets/src/resources';
import {toolkitProfile} from '../packages/mcp/src/profiles';
vi.mock('../packages/render/src/verify-registry',()=>({verifyAssetRegistry:vi.fn(async()=>[])}));
vi.mock('../packages/render/src/export',()=>({renderProject:vi.fn(async(o:any)=>{const file=path.join(o.root,'.clipkit/exports/result.mp4');await mkdir(path.dirname(file),{recursive:true});await writeFile(file,'test encoded bytes');return {path:file,creditsPath:file+'.json'}})}));
let root:string,server:Awaited<ReturnType<typeof startServer>>,itemId:string,version:string;
const manifest:any={manifestVersion:2,sceneId:'film',name:'Toolkit film',entry:'Scene.tsx',durationFrames:720,files:{'Scene.tsx':'','parts/Title.tsx':'','parts/type.css':''},defaults:{headline:'Original'},fields:[{key:'headline',label:'Headline',type:'text'}],propsSchema:{type:'object',additionalProperties:false,required:['headline'],properties:{headline:{type:'string'}}}};
const code={'Scene.tsx':`import React from 'react';import Title from './parts/Title';import './parts/type.css';export default({item,headline})=><div data-ck-layer={item.id} data-ck-path="headline" data-ck-element="title"><Title text={headline}/></div>`, 'parts/Title.tsx':`import React from 'react';export default({text})=><h1>{text}</h1>`, 'parts/type.css':'h1 {color: coral}'};
async function wait(j:any){while(['queued','running'].includes(j.status))j=await server.operate('job_status',{jobId:j.id,waitMs:1000});return j}
beforeAll(async()=>{root=await mkdtemp(path.join(tmpdir(),'ck-toolkit-'));vi.stubEnv('CLIPKIT_COMPONENT_CACHE_DIR',path.join(root,'.component-cache'));server=await startServer({root,port:24347,publicDir:path.resolve('public')})});
afterAll(async()=>{await server?.close();await rm(root,{recursive:true,force:true});vi.unstubAllEnvs()});
it('opens without a nested agent and publishes nested TSX/CSS without a persistent lease',async()=>{
  const run=vi.spyOn(server.runner,'run');const opened:any=await server.operate('project_open',{settings:{width:1080,height:1920,fps:30}});expect(opened.settings.width).toBe(1080);
  const result=await wait(await server.operate('project_sync',{sceneId:'film',manifest,files:code}));expect(result.status,result.error).toBe('done');itemId=result.result.itemIds[0];version=result.result.version;expect(server.store.currentLease()).toBeNull();expect(run).not.toHaveBeenCalled();expect(allItems(server.store.project)).toHaveLength(1);
  expect(await readFile(path.join(root,'.clipkit/scenes/versions',version,'registry.js'),'utf8')).toContain('color: coral');
});
it('patches one property without resending source and preserves it through a source delta',async()=>{
  await server.operate('property_patch',{itemId,path:'headline',value:'Human value',expected:'Original'});
  const changed='import React from "react";export default({text})=><h2>{text}</h2>';
  const result=await wait(await server.operate('project_sync',{sceneId:'film',baseVersion:version,manifest,files:{'parts/Title.tsx':changed}}));expect(result.status,result.error).toBe('done');expect(result.result.changedFiles).toEqual(['parts/Title.tsx']);expect(allItems(server.store.project)[0].props.headline).toBe('Human value');
  await expect(server.operate('property_patch',{itemId,path:'headline',value:'stale',expected:'Original'})).rejects.toThrow('changed');
  const stale=await wait(await server.operate('project_sync',{sceneId:'film',baseVersion:version,manifest,files:{}}));expect(stale.status).toBe('failed');version=result.result.version;
});
it('retains the last working film after invalid source or path traversal',async()=>{
  const revision=server.store.project.rev;const failed=await wait(await server.operate('project_sync',{sceneId:'film',baseVersion:version,manifest,files:{'parts/Title.tsx':'broken syntax ('}}));expect(failed.status).toBe('failed');expect(server.store.project.rev).toBe(revision);
  await expect(server.operate('project_sync',{sceneId:'film',manifest:{...manifest,files:{'../escape.tsx':''}},files:{'../escape.tsx':'x'}})).rejects.toThrow('Invalid source');
});
it('uploads font bytes when a server cannot see the client filesystem path',async()=>{
  const bytes=await readFile('public/vendor/inter.ttf'),form=new FormData();form.set('file',new File([bytes],'brand.ttf'));
  const response=await fetch(server.url+'/api/resources',{method:'POST',headers:{'X-ClipKit-Token':server.token},body:form}).catch(e=>{throw Error('Font upload failed: '+e.message,{cause:e})});expect(response.status,'font upload response').toBe(200);const resource:any=await response.json();expect(resource.kind).toBe('font');expect(JSON.stringify(resource).length).toBeLessThan(700);
  const copy=await fetch(server.url+resource.url+'?token='+server.token).catch(e=>{throw Error('Resource download failed: '+e.message,{cause:e})});expect(Buffer.from(await copy.arrayBuffer())).toEqual(bytes);
  const denied=await server.app.request(server.url+resource.url,{headers:{Host:new URL(server.url).host}});expect(denied.status).toBe(401);await denied.arrayBuffer();const resources=new Resources(root);await expect(resources.get('../secret')).rejects.toThrow('Invalid resource');
});
it('ranks visual intent and separates discoverability from readiness',async()=>{
  const index=new ComponentIndex(root,process.cwd());expect((await index.search('gooey morph'))[0].id).toMatch(/morph/);
  const search=await index.search('scramble decode');expect(search[0].id).toBe('reactbits-decrypted-text');expect(search[0].status).toBe('needs-adaptation');expect(rankComponents(webSources,'aperture')[0].entry.id).toBe('animata-aperture');
  const added:any=await server.operate('component_add',{id:'codrops-repeat-text',sceneId:'film'});expect(added.dependencies.gsap).toBe('3.15.0');expect(await readFile(added.file,'utf8')).toContain('useGsapTimeline');
  expect(toolkitProfile).toHaveLength(12);expect(toolkitProfile).not.toContain('video_run');
});
it('exports native dimensions and explicit silence without post-upscaling',async()=>{
  const result=await wait(await server.operate('render',{width:1920,height:1080,fps:30,audio:'silent'}));expect(result.status,result.error).toBe('done');
  const {renderProject}=await import('../packages/render/src/export');expect(vi.mocked(renderProject).mock.calls.at(-1)![0]).toMatchObject({preset:{width:1920,height:1080},fps:30,audio:'silent'});
  expect(exportOptions({},server.store.project.settings).preset).toEqual({width:1080,height:1920});expect(()=>exportOptions({width:1920},server.store.project.settings)).toThrow('both');expect(()=>exportOptions({audio:'fake'},server.store.project.settings)).toThrow('Audio');
});
it('ships editable source and resources in a portable bundle',async()=>{
  const result=await wait(await server.operate('source_bundle'));expect(result.status,result.error).toBe('done');
  const {unzipSync}=await import('fflate');const files=unzipSync(await readFile(result.result.path));expect(Object.keys(files)).toContain('scenes/film/parts/Title.tsx');expect(Object.keys(files).some(p=>p.startsWith('.clipkit/resources/')&&p.endsWith('.ttf'))).toBe(true);
});
it('creates a v2 starter with derived controls and no mandatory hand-authored DOM tags',async()=>{
  const opened:any=await server.operate('project_open',{sceneId:'quick',durationFrames:60});
  const submitted=await wait(await server.operate('project_sync',{sceneId:'quick'}));expect(submitted.status,submitted.error).toBe('done');
  const definition=server.state().assetRegistry?.clips.find((c:any)=>c.sceneId==='quick') as any;
  expect(definition.bindingMode).toBe('optional');expect(definition.fields.map((f:any)=>f.key)).toContain('headline');expect(opened.starter.sceneId).toBe('quick');
  const {verifyAssetRegistry}=await import('../packages/render/src/verify-registry');const checked=vi.mocked(verifyAssetRegistry).mock.calls.length;
  const repeated=await wait(await server.operate('project_sync',{sceneId:'quick'}));expect(repeated.status,repeated.error).toBe('done');expect(vi.mocked(verifyAssetRegistry).mock.calls.length).toBe(checked);
});
