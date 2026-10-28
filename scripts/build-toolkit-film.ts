import {mkdir,readFile,writeFile,copyFile} from 'node:fs/promises';
import path from 'node:path';
import {startServer} from '../packages/server/src/index';
import {fieldsFor} from '../packages/assets/src/property-path';
const workspace=process.cwd(),root=path.resolve(process.argv[2]??'.clipkit/rebuild-film-24'),port=Number(process.env.CLIPKIT_FILM_PORT??4460);
await mkdir(root,{recursive:true});
const server=await startServer({root,port,publicDir:path.join(workspace,'public'),studioDir:path.join(workspace,'dist/studio')});
const wait=async(j:any)=>{while(['queued','running'].includes(j.status))j=await server.operate('job_status',{jobId:j.id,waitMs:45000});if(j.status!=='done')throw Error(j.error);return j.result};
try{
  await server.operate('project_open',{settings:{width:1080,height:1920,fps:30},baseRev:server.store.project.rev});
  const font:any=await server.operate('resource_import',{path:path.join(workspace,'public/vendor/inter.ttf')});
  const art=`<svg xmlns="http://www.w3.org/2000/svg" width="1080" height="1920"><defs><linearGradient id="a" x2="1" y2="1"><stop stop-color="#fe593e"/><stop offset="1" stop-color="#8f2596"/></linearGradient><filter id="s"><feGaussianBlur stdDeviation="80"/></filter></defs><rect width="1080" height="1920" fill="url(#a)"/><circle cx="950" cy="600" r="490" fill="#fbdd94" filter="url(#s)"/><path d="M-120 1350L520 150 1180 1350Z" fill="none" stroke="#211d29" stroke-width="170"/><path d="M-220 1450L520 250 1260 1450Z" fill="none" stroke="#f5f1e8" stroke-width="28"/><circle cx="540" cy="1040" r="105" fill="#fe593e"/><path d="M270 1670H810" stroke="#f5f1e8" stroke-width="8"/></svg>`;
  const artPath=path.join(root,'art.svg');await writeFile(artPath,art);const image:any=await server.operate('resource_import',{path:artPath});
  const defaults={headline:'MAKE\nIT\nMOVE.',headlineSize:274,accent:'#ff6347',cta:'MAKE YOUR NEXT FRAME.',morphWords:['WEB','FILM'],apertureLead:'OPEN',apertureTrail:'UP.',apertureCaption:'A component is a starting point. The frame is yours.',kineticWords:['ONE','PRECISE','EDIT.'],repeatText:'BEND',staggerText:'KEEP\nYOUR\nTASTE.',fontResourceId:font.id,artResourceId:image.id};
  const properties=Object.fromEntries(Object.entries(defaults).map(([k,v])=>[k,Array.isArray(v)?{type:'array',minItems:1,maxItems:10,items:{type:'string',maxLength:120}}:typeof v==='number'?{type:'number',minimum:50,maximum:400}:{type:'string',maxLength:500}]));
  const propsSchema={type:'object',additionalProperties:false,required:Object.keys(defaults),properties};
  const folder=path.join(root,'scenes/film');await mkdir(folder,{recursive:true});
  const source=await readFile(path.join(workspace,'examples/web-motion-24/Film.tsx'),'utf8');await writeFile(path.join(folder,'Scene.tsx'),source);
  const current=server.store.project.tracks.flatMap(t=>t.items).find(i=>i.sceneId==='film');
  const manifest={...current?{baseVersion:current.sceneVersion,targetItemIds:[current.id]}:{},manifestVersion:2,sceneId:'film',name:'Web energy. Film control.',durationFrames:720,entry:'Scene.tsx',files:{'Scene.tsx':''},defaults,fields:fieldsFor(propsSchema,defaults),propsSchema,resources:[font.id,image.id]};await writeFile(path.join(folder,'scene.json'),JSON.stringify(manifest,null,2));
  const receipts=[];for(const id of ['web-gooey-morphing-text','web-aperture-reveal','web-kinetic-center-build','codrops-repeat-text','primitives-text-effect'])receipts.push(await server.operate('component_add',{id,sceneId:'film'}));
  console.log(JSON.stringify({stage:'components',count:receipts.length}));
  const sync=await wait(await server.operate('project_sync',{sceneId:'film'}));console.log(JSON.stringify({stage:'published',version:sync.version,items:sync.itemIds}));
  const result=await wait(await server.operate('render',{audio:'silent',crf:18}));
  const bundle=await wait(await server.operate('source_bundle'));
  const destination=path.resolve('artifacts/rebuild');await mkdir(destination,{recursive:true});await copyFile(result.path,path.join(destination,'web-motion-24.mp4'));await copyFile(bundle.path,path.join(destination,'web-motion-24-source.zip'));
  await writeFile(path.join(destination,'receipt.json'),JSON.stringify({root,revision:server.store.project.rev,video:path.join(destination,'web-motion-24.mp4'),source:path.join(destination,'web-motion-24-source.zip'),components:receipts.map((r:any)=>r.receipt),usage:{source:'current implementation agent; provider usage unavailable',input:null,cached:null,output:null}},null,2));
  console.log(JSON.stringify({stage:'delivered',destination}));
}finally{await server.close()}
