import {it,expect,vi} from 'vitest';
import {mkdtemp,writeFile,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {watchPacks,type refreshPacks} from '../packages/assets/src/packs';
it('drains an active pack refresh before shutdown and suppresses late publication',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'ck-watch-close-'));let finish!:(value:Awaited<ReturnType<typeof refreshPacks>>)=>void;
 const blocked=new Promise<Awaited<ReturnType<typeof refreshPacks>>>(resolve=>{finish=resolve;});const refresh=vi.fn(()=>blocked),changed=vi.fn();
 const watcher=await watchPacks(root,changed,refresh);let closed=false;
 try{
  await writeFile(path.join(root,'.clipkit/assets/packs/trigger.json'),'{}');await vi.waitFor(()=>expect(refresh).toHaveBeenCalled(),{timeout:10000});
  const closing=watcher.close().then(()=>{closed=true;});await Promise.resolve();expect(closed).toBe(false);
  finish({clips:[],version:'test',errors:[],generatedDir:path.join(root,'generated')});await closing;expect(closed).toBe(true);expect(changed).not.toHaveBeenCalled();
 }finally{finish({clips:[],version:'test',errors:[],generatedDir:''});await watcher.close();await rm(root,{recursive:true,force:true});}
});
it('serializes concurrent registry rebuilds for the same project',async()=>{
 const {refreshPacks}=await import('../packages/assets/src/packs');const {readFile}=await import('node:fs/promises');
 const root=await mkdtemp(path.join(tmpdir(),'ck-registry-queue-'));
 try{const results=await Promise.all(Array.from({length:8},()=>refreshPacks(root)));expect(new Set(results.map(result=>result.version)).size).toBe(1);expect(await readFile(path.join(results[0].generatedDir,`registry-${results[0].version}.js`),'utf8')).toContain('ClipKitPackRegistry');}finally{await rm(root,{recursive:true,force:true});}
});
