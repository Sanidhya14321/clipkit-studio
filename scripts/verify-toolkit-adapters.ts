import {mkdir,readFile,writeFile,copyFile} from 'node:fs/promises';
import path from 'node:path';
import {startServer} from '../packages/server/src/index';
import {probe} from '../packages/server/src/media';
const kit=process.cwd(),out=path.resolve(process.argv[2]??'.clipkit/toolkit-adapter-checks');await mkdir(out,{recursive:true});
const cases=[
  {id:'primitives-text-effect',body:'<Effect text={text} per="char" preset="fade-in-blur" style={{fontSize:140,fontWeight:800}}/>'},
  {id:'reactbits-decrypted-text',local:'DecryptedText.tsx',body:'<div style={{fontSize:110}}><Effect text={text} revealDirection="center"/></div>'},
  {id:'reactbits-scroll-velocity',local:'ScrollVelocity.tsx',body:'<Effect texts={[text]} velocity={110} fontSize={160}/>'},
  {id:'web-gooey-morphing-text',body:'<Effect {...p} texts={[text,"FILM"]} size={220} color="#f5f1e8" background="#171717" morphFrames={20} holdFrames={18} bindPath="text"/>'},
  {id:'web-aperture-reveal',body:'<Effect {...p} lead={text} trail="UP" caption="The frame is yours." size={110} captionSize={65} color="#171717" background="#f5f1e8" delayFrames={0} expandFrames={75} imageSourceId="" bindPath="text"/>'},
  {id:'web-kinetic-center-build',body:'<Effect {...p} words={["ONE",text]} size={100} gap={20} pushFrames={15} background="#171717" color="#f5f1e8" bindPath="text"/>'},
  {id:'web-liquid-metal',body:'<><Effect {...p} background="#171717" color="#dfefff" imageSourceId="" repetition={2} distortion={.4} angle={0} size={.5} blur={.1} speed={1} bindPath="text"/><div style={{position:"absolute",top:130,left:70,fontSize:90,fontWeight:700}}>{text}</div></>'},
  {id:'codrops-repeat-text',body:'<Effect text={text} increment={80} delay={.06} duration={1} style={{fontSize:190,fontWeight:800,lineHeight:1}}/>'},
];
const rows:any[]=[];
for(const fixture of cases){
  const root=path.join(out,fixture.id),folder=path.join(root,'scenes/probe');await mkdir(folder,{recursive:true});
  const server=await startServer({root,port:4465,publicDir:path.join(kit,'public')});
  const wait=async(j:any)=>{while(['queued','running'].includes(j.status))j=await server.operate('job_status',{jobId:j.id,waitMs:45000});if(j.status!=='done')throw Error(j.error);return j.result};
  try{
    const schema={type:'object',additionalProperties:false,properties:{text:{type:'string'}},required:['text']};
    const manifest:any={manifestVersion:2,sceneId:'probe',name:fixture.id,durationFrames:90,files:{'Scene.tsx':''},defaults:{text:'MOVE'},fields:[{key:'text',label:'Text',type:'text'}],propsSchema:schema};await writeFile(path.join(folder,'scene.json'),JSON.stringify(manifest));
    const added:any=await server.operate('component_add',{id:fixture.id,sceneId:'probe'});let imported=path.relative(folder,added.file??'');
    if(fixture.local){imported='components/Effect.tsx';await mkdir(path.join(folder,'components'),{recursive:true});await copyFile(path.join(kit,'.clipkit/local-adapters',fixture.local),path.join(folder,imported));const current=JSON.parse(await readFile(path.join(folder,'scene.json'),'utf8'));current.files[imported]='';current.componentReceipts=[added.receipt];await writeFile(path.join(folder,'scene.json'),JSON.stringify(current));}
    const source=`import React from 'react';import {AbsoluteFill} from 'remotion';import Effect from './${imported.replace(/\.tsx$/,'')}';export default function Probe(p){const {text}=p;return <AbsoluteFill data-ck-layer={p.item.id} data-ck-path="text" data-ck-element="effect" style={{background:'#171717',color:'#f5f1e8',justifyContent:'center',fontFamily:'Inter,monospace'}}>${fixture.body}</AbsoluteFill>}`;
    await writeFile(path.join(folder,'Scene.tsx'),source);
    const validation=await wait(await server.operate('component_validate',{id:fixture.id,sceneId:'probe',adapterFile:imported}));
    const published=await wait(await server.operate('project_sync',{sceneId:'probe'}));
    const result=await wait(await server.operate('render',{width:540,height:960,audio:'silent',crf:23,range:{startFrame:0,endFrame:90}}));
    const destination=path.join(out,fixture.id+'.mp4');await copyFile(result.path,destination);
    rows.push({id:fixture.id,status:'passed',validationVersion:validation.validationVersion,adapterKey:validation.key,projectVersion:published.version,video:destination,encoded:await probe(destination),fidelity:'Mechanism retained by source adaptation; moving comparison is separate from these technical checks'});
  }catch(e){rows.push({id:fixture.id,status:'failed',error:(e as Error).message})}finally{await server.close()}
  await writeFile(path.join(out,'results.json'),JSON.stringify(rows,null,2));console.log(JSON.stringify(rows.at(-1)));
}
await mkdir(path.join(kit,'artifacts/rebuild'),{recursive:true});await copyFile(path.join(out,'results.json'),path.join(kit,'artifacts/rebuild/adapter-checks.json'));
if(rows.some(r=>r.status!=='passed'))process.exitCode=1;
