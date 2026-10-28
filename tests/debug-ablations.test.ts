import {it,expect} from 'vitest';
import {mkdtemp,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {createServer} from 'node:net';
import {startServer} from '../packages/server/src/index';
import {armFlags} from '../packages/debug/src/config';
const freePort=()=>new Promise<number>(resolve=>{const s=createServer();s.listen(0,'127.0.0.1',()=>{const p=(s.address() as any).port;s.close(()=>resolve(p));});});
it('changes actual module behavior only in explicitly selected ablation servers',async()=>{
 for(const arm of ['C1','C2','C4','C5'] as const){
  const root=await mkdtemp(path.join(tmpdir(),'ck-ablation-'));
  const server=await startServer({root,port:await freePort(),publicDir:path.resolve('public'),harness:armFlags(arm),debug:{synthetic:true}});
  try{
   if(arm==='C1')await expect(server.operate('brief_propose',{prompt:'Synthetic fixture'})).rejects.toThrow('brief disabled');
   if(arm==='C2')await expect(server.operate('recipe_list',{})).rejects.toThrow('recipes disabled');
   if(arm==='C4')expect(await server.operate('context_get',{})).toMatchObject({disabled:true,selection:null,playhead:0});
   if(arm==='C5')expect(await server.operate('quality_verify',{})).toMatchObject({disabled:true,pass:true,checks:[]});
  }finally{await server.close();await rm(root,{recursive:true,force:true});}
 }
},30000);
