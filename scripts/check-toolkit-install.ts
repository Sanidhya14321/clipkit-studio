import {mkdir,readFile,writeFile,copyFile} from 'node:fs/promises';
import path from 'node:path';
import {spawn,execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {probe} from '../packages/server/src/media';
const kit=process.cwd(),root=path.resolve('.clipkit/packed-toolkit-check'),cli=path.resolve('artifacts/package/dist/cli.js');
await mkdir(root,{recursive:true});const env={...process.env,CLIPKIT_COMPONENT_CACHE_DIR:path.join(root,'component-cache')};
const child=spawn(process.execPath,[cli,'--root',root,'studio','--no-vite','--port','4478'],{env,stdio:['ignore','pipe','pipe']});let errors='';child.stderr.on('data',d=>errors=(errors+d).slice(-1500));child.stdout.resume();
const command=async(args:string[])=>{const result=await promisify(execFile)(process.execPath,[cli,'--root',root,...args],{env,timeout:120000,maxBuffer:1024*1024});return JSON.parse(result.stdout)};
const wait=async(j:any)=>{while(['queued','running'].includes(j.status))j=await command(['job',j.id]);if(j.status!=='done')throw Error(j.error);return j.result};
try{
  let ready=false;for(let n=0;n<100;n++){try{const connection=JSON.parse(await readFile(path.join(root,'.clipkit/server.json'),'utf8'));ready=(await fetch(connection.url+'/api/bootstrap')).ok;if(ready)break}catch{}await new Promise(r=>setTimeout(r,100))}if(!ready)throw Error('Packed server did not start: '+errors);
  await command(['project','open','--width','1080','--height','1920','--fps','30']);
  // Upload from a client-side file; only file bytes cross the API, not a client pathname.
  const clientFolder=path.join(root,'client-only');await mkdir(clientFolder,{recursive:true});await copyFile(path.join(kit,'public/vendor/inter.ttf'),path.join(clientFolder,'brand.ttf'));
  const font=await command(['resource','add',path.join(clientFolder,'brand.ttf')]);
  const added=await command(['component','add','codrops-repeat-text','--scene','film']);
  const folder=path.join(root,'scenes/film'),manifest=JSON.parse(await readFile(path.join(folder,'scene.json'),'utf8'));
  Object.assign(manifest,{durationFrames:60,defaults:{headline:'FRAME-SOUGHT',headlineSize:140,color:'#f5f1e8',fontResourceId:font.id},fields:[],resources:[font.id],propsSchema:{type:'object',additionalProperties:false,properties:{headline:{type:'string'},headlineSize:{type:'number'},color:{type:'string'},fontResourceId:{type:'string'}},required:['headline','headlineSize','color','fontResourceId']}});
  const source=`import React from 'react';import {AbsoluteFill} from 'remotion';import {resourceUrl,useResourceFont} from '@clipkit/motion';import Repeat from './components/codrops-repeat-text';export default function Film(p){const family=useResourceFont('PackedFont',resourceUrl(p.fontResourceId,p.assetBaseUrl,p.accessToken));return <AbsoluteFill style={{background:'#171717',color:p.color,justifyContent:'center',textAlign:'center',fontFamily:family,fontSize:p.headlineSize}}><Repeat text={p.headline} increment={80}/></AbsoluteFill>}`;
  await writeFile(path.join(folder,'scene.json'),JSON.stringify(manifest));await writeFile(path.join(folder,'Scene.tsx'),source);
  const sync=await command(['project','sync','film']);
  const result=await wait(await command(['op','render',JSON.stringify({width:640,height:360,fps:60,audio:'silent',range:{startFrame:0,endFrame:120}})]));
  const encoded=await probe(result.path);if(encoded.width!==640||encoded.height!==360||encoded.audio||Math.abs(encoded.durationSec-2)>.02)throw Error('Packed native render contract failed');
  const bundle=await wait(await command(['source','bundle']));
  const receipt={passed:true,sourceVersion:sync.version,dependencies:added.dependencies,resourceBytes:font.bytes,encoded,video:result.path,sourceBundle:bundle.path,nestedAgentCalls:0};
  await mkdir(path.join(kit,'artifacts/rebuild'),{recursive:true});await writeFile(path.join(kit,'artifacts/rebuild/install-check.json'),JSON.stringify(receipt,null,2));console.log(JSON.stringify(receipt));
}finally{child.kill('SIGTERM');await new Promise<void>(resolve=>{const timeout=setTimeout(()=>{child.kill('SIGKILL');resolve()},10000);child.once('close',()=>{clearTimeout(timeout);resolve()})})}
