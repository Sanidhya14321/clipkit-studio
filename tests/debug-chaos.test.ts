import {it,expect} from 'vitest';
import {fork,type ChildProcess} from 'node:child_process';
import {mkdtemp,readFile,rm,symlink,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {createServer} from 'node:net';
import WebSocket from 'ws';
import {ProjectStore} from '../packages/core/src/store';
import {ProjectSchema} from '../packages/core/src/index';
import {startServer} from '../packages/server/src/index';
import {FishClient} from '../packages/server/src/fish';
import {diagnostic} from '../packages/debug/src/errors';
const temporary=()=>mkdtemp(path.join(tmpdir(),'ck-debug-chaos-'));
const freePort=()=>new Promise<number>(resolve=>{const s=createServer();s.listen(0,'127.0.0.1',()=>{const port=(s.address() as any).port;s.close(()=>resolve(port));});});
function message(child:ChildProcess,key:string){return new Promise<any>((resolve,reject)=>{const timer=setTimeout(()=>{cleanup();reject(Error('Worker did not reach '+key));},15000);const receive=(m:any)=>{if(m[key]){cleanup();resolve(m);}};const closed=()=>{cleanup();reject(Error('Worker exited before '+key));};function cleanup(){clearTimeout(timer);child.off('message',receive);child.off('exit',closed);}child.on('message',receive);child.once('exit',closed);});}
async function kill(child:ChildProcess){if(child.exitCode!==null||child.signalCode)return;const done=new Promise<void>(resolve=>child.once('exit',()=>resolve()));child.kill('SIGKILL');await done;}
function worker(mode:string,root:string,port=0){return fork(path.resolve('tests/fixtures/debug-chaos-worker.ts'),[mode,root,String(port)],{execArgv:['--import','tsx'],stdio:['ignore','ignore','ignore','ipc']});}
it('recovers a valid project after the server dies before patch commit',async()=>{
 const root=await temporary(),child=worker('server',root,await freePort());
 try{const ready=await message(child,'ready'),prepared=message(child,'prepared');
  const request=fetch(ready.url+'/api/op/project_patch',{method:'POST',headers:{'Content-Type':'application/json','X-ClipKit-Token':ready.token},body:JSON.stringify({baseRev:0,requestId:'interrupted',label:'Synthetic interrupted edit',ops:[{op:'replace',path:'/name',value:'Must not commit'}]}),signal:AbortSignal.timeout(15000)}).catch(()=>null);
  await prepared;await kill(child);expect(await request).toBeNull();
  const recovered=await new ProjectStore(root).load();expect(ProjectSchema.safeParse(recovered.project).success).toBe(true);expect(recovered.project.rev).toBe(0);expect(recovered.project.name).not.toBe('Must not commit');
 }finally{await kill(child);await rm(root,{recursive:true,force:true});}
},20000);
it('does not persist a killed agent process lease across restart',async()=>{
 const root=await temporary(),child=worker('lock',root);
 try{await message(child,'ready');await kill(child);const recovered=await new ProjectStore(root).load();expect(recovered.currentLease()).toBeNull();expect(ProjectSchema.safeParse(recovered.project).success).toBe(true);}
 finally{await kill(child);await rm(root,{recursive:true,force:true});}
},20000);
it.skipIf(process.platform!=='linux')('rejects a real ENOSPC journal write without corrupting saved state',async()=>{
 const root=await temporary();try{
  const store=await new ProjectStore(root).load(),journal=path.join(root,'.clipkit/history/journal.jsonl');
  await rm(journal,{force:true});await symlink('/dev/full',journal);
  let failure:unknown;try{await store.patch({baseRev:0,requestId:'disk-full',label:'Synthetic disk full',ops:[{op:'replace',path:'/name',value:'No space'}]});}catch(error){failure=error;}
  expect(diagnostic(failure).code).toBe('E_ENV');expect(store.project.rev).toBe(0);
  expect(ProjectSchema.safeParse(JSON.parse(await readFile(path.join(root,'.clipkit/project.json'),'utf8'))).success).toBe(true);
  await rm(journal);await writeFile(journal,'');expect((await new ProjectStore(root).load()).project.rev).toBe(0);
 }finally{await rm(root,{recursive:true,force:true});}
});
it('detects complete corrupt history and keeps its valid project snapshot intact',async()=>{
 const root=await temporary();try{await new ProjectStore(root).load();await writeFile(path.join(root,'.clipkit/history/journal.jsonl'),'{invalid}\n');let failure:unknown;try{await new ProjectStore(root).load();}catch(error){failure=error;}expect(diagnostic(failure).code).toBe('E_PATCH_INVALID');expect(ProjectSchema.safeParse(JSON.parse(await readFile(path.join(root,'.clipkit/project.json'),'utf8'))).success).toBe(true);}
 finally{await rm(root,{recursive:true,force:true});}
});
it('reports provider failure without leaking provider response details',async()=>{
 const fish=new FishClient('synthetic-secret-key',async()=>new Response('PRIVATE_RESPONSE',{status:503}));
 let failure:unknown;try{await fish.tts('Synthetic speech','fixture-voice');}catch(error){failure=error;}
 expect(diagnostic(failure).code).toBe('E_VOICE_PROVIDER');expect(String(failure)).not.toMatch(/PRIVATE_RESPONSE|synthetic-secret-key/);
});
it('reconnects a dropped WebSocket to the latest durable revision and records both connections',async()=>{
 const root=await temporary(),server=await startServer({root,port:await freePort(),publicDir:path.resolve('public'),debug:{runId:'socket-chaos'}});
 const sockets:WebSocket[]=[];
 const connect=()=>new Promise<any>((resolve,reject)=>{const ws=new WebSocket(server.url.replace('http:','ws:')+'/events?token='+server.token);sockets.push(ws);ws.once('error',reject);ws.once('message',data=>resolve(JSON.parse(data.toString())));});
 try{expect((await connect()).state.project.rev).toBe(0);sockets[0].terminate();
  await server.store.patch({baseRev:0,requestId:'socket-gap',label:'Synthetic edit',ops:[{op:'replace',path:'/name',value:'After reconnect'}]});
  expect((await connect()).state.project.rev).toBe(1);await server.flight.close();const events=(await readFile(path.join(server.flight.dir,'events.jsonl'),'utf8')).split('\n').filter(Boolean).map(l=>JSON.parse(l));expect(events.filter(e=>e.event==='websocket.connect')).toHaveLength(2);
 }finally{for(const socket of sockets)socket.terminate();await server.close();await rm(root,{recursive:true,force:true});}
},20000);
