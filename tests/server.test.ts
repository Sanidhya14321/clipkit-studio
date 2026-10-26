import {describe,it,expect,beforeAll,afterAll} from 'vitest';
import {mkdtemp,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join,resolve} from 'node:path';
import {WebSocket} from 'ws';
import {startServer} from '../packages/server/src/index.ts';

describe('real HTTP/WebSocket shared editor',()=>{
 let root:string;let backend:Awaited<ReturnType<typeof startServer>>;
 const port=24317;
 beforeAll(async()=>{root=await mkdtemp(join(tmpdir(),'clipkit-api-'));backend=await startServer({root,port,publicDir:resolve('public')});},30000);
 afterAll(async()=>{await backend?.close();await rm(root,{recursive:true,force:true});});
 const op=async(name:string,args:Record<string,unknown>={},extra:Record<string,string>={})=>{const response=await fetch(`${backend.url}/api/op/${name}`,{method:'POST',headers:{'Content-Type':'application/json','X-ClipKit-Token':backend.token,...extra},body:JSON.stringify(args)});return {status:response.status,body:await response.json() as any};};
 it('requires auth and rejects cross-origin writes',async()=>{expect((await fetch(`${backend.url}/api/state`)).status).toBe(401);expect((await fetch(`${backend.url}/api/bootstrap`,{headers:{Origin:'https://example.com'}})).status).toBe(403);expect((await fetch(`${backend.url}/api/bootstrap`,{headers:{Origin:'http://127.0.0.1:5173'}})).status).toBe(200);});
 it('refuses a second writer for the same project',async()=>{await expect(startServer({root,port:24318})).rejects.toMatchObject({code:'ALREADY_RUNNING'});});
 it('broadcasts real edits, enforces ownership and rejects stale patches',async()=>{
   await op('assets_add',{clipId:'kinetic-title',at:0});const item=backend.store.project.tracks[1].items[0];
   const socket=new WebSocket(`${backend.url.replace('http','ws')}/events?token=${backend.token}`);await new Promise<void>(r=>socket.once('open',()=>r()));
   const lease=(await op('status_working',{owner:'integration-agent',label:'Editing title'})).body;
   expect((await op('item_update',{itemId:item.id,changes:{props:{...item.props,text:'Blocked'}}})).status).toBe(409);
   const start=Date.now();const event=new Promise<any>(resolve=>{socket.on('message',data=>{const event=JSON.parse(String(data));if(event.state?.project.tracks[1].items[0]?.props.text==='Committed')resolve(event);});});
   const actor={'X-ClipKit-Lease':lease.token,'X-ClipKit-Turn':lease.turnId};
   const rev=backend.store.project.rev;
   expect((await op('project_patch',{requestId:'agent-change',baseRev:rev,label:'Committed title',ops:[{op:'replace',path:'/tracks/1/items/0/props/text',value:'Committed'}]},actor)).status).toBe(200);
   await event;expect(Date.now()-start).toBeLessThan(1000);
   expect((await op('project_patch',{requestId:'stale',baseRev:rev,label:'stale',ops:[]},actor)).body.code).toBe('REV_CONFLICT');
   await op('status_idle',{leaseToken:lease.token});
   expect((await op('project_patch',{requestId:'late',baseRev:backend.store.project.rev,label:'late',ops:[]},actor)).body.code).toBe('LEASE_EXPIRED');
   await op('history_undo');expect(backend.store.project.tracks[1].items[0].props.text).not.toBe('Committed');socket.close();
 });
 it('validates clip props and never permits patching source paths',async()=>{const item=backend.store.project.tracks[1].items[0];expect((await op('item_update',{itemId:item.id,changes:{props:{...item.props,color:'not a color'}}})).status).toBe(400);expect((await op('project_patch',{requestId:'file',baseRev:backend.store.project.rev,label:'unsafe',ops:[{op:'add',path:'/sources/private',value:{path:'/etc/passwd'}}]})).body.code).toBe('INVALID_PATCH');});
 it('returns affected layers only and compact bounded default outputs',async()=>{const item=backend.store.project.tracks[1].items[0];await op('comments_add',{text:'Bigger please',anchor:{tStart:1,layerId:item.id}});const bundle=(await op('comments_bundle')).body;expect(bundle.layers).toHaveLength(1);expect(bundle.layers[0].id).toBe(item.id);expect(JSON.stringify(bundle).length).toBeLessThan(12000);for(const operation of ['project_get','assets_search','comments_list','history_list'])expect(JSON.stringify((await op(operation)).body).length).toBeLessThan(12000);});
 it('serves only registered media with real range requests',async()=>{const fixture=await backend.media.fixture();const imported=await backend.media.importFile(fixture.path);const url=`${backend.url}/media/${imported.sourceId}?token=${backend.token}`;const response=await fetch(url,{headers:{Range:'bytes=0-63'}});expect(response.status).toBe(206);expect((await response.arrayBuffer()).byteLength).toBe(64);expect((await fetch(`${backend.url}/media/not-registered?token=${backend.token}`)).status).toBe(400);},30000);
});
