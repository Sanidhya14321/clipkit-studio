import {it,expect} from 'vitest';
import {mkdtemp,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {ProjectStore} from '../packages/core/src/store.ts';
import {mergeRevert,RevertConflict} from '../packages/core/src/revert.ts';
import {packTranscript,byteLength} from '../packages/core/src/context.ts';
import {routeVoice} from '../packages/core/src/index.ts';
it('selective revert merges independent later fields and stable IDs, refusing conflicts',()=>{
 const before={items:[{id:'a',text:'old',size:10},{id:'b',text:'B',size:10}]};
 const after={items:[{id:'a',text:'new',size:10},{id:'b',text:'B',size:10}]};
 const current={items:[{id:'b',text:'B',size:10},{id:'a',text:'new',size:20}]};
 expect(mergeRevert(before,after,current)).toEqual({items:[{id:'b',text:'B',size:10},{id:'a',text:'old',size:20}]});
 expect(()=>mergeRevert(before,after,{items:[{id:'a',text:'later',size:10},{id:'b',text:'B',size:10}]})).toThrow(RevertConflict);
});
it('packs 1200 fillers within UTF8 budget without dropping paged words',()=>{
 const words=Array.from({length:1200},(_,i)=>({text:i%2?'um':'uh',start:i*.5,end:i*.5+.3}));let cursor=0,total=0;
 do{const page=packTranscript(words,{start:0,end:600,cursor,maxBytes:1800});expect(byteLength(page)).toBeLessThanOrEqual(1800);total+=page.phrases.reduce((n,p)=>n+p.text.split(' ').length,0);if(page.nextCursor===null)break;expect(page.nextCursor).toBeGreaterThan(cursor);cursor=page.nextCursor;}while(cursor<words.length);
 expect(total).toBe(1200);
});
it('renews accepted patch lease and refuses a lease that expires during asynchronous preparation',async()=>{
 const root=await mkdtemp(join(tmpdir(),'ck-lease-'));let now=0;let expire=false;
 try{const store=await new ProjectStore(root,undefined,()=>now,{leaseTimeoutMs:1000,prepare:async()=>{if(expire)now+=1001;}}).load();const lease=store.acquire('test','test');now=500;await store.mutate('first',p=>{p.name='First';},'agent',{leaseToken:lease.token,turnId:lease.turnId});expect(store.currentLease()?.expiresAt).toBe(1500);expire=true;await expect(store.mutate('late',p=>{p.name='Late';},'agent',{leaseToken:lease.token,turnId:lease.turnId})).rejects.toMatchObject({code:'LEASE_EXPIRED'});expect(store.project.name).toBe('First');}finally{await rm(root,{recursive:true,force:true});}
});
it('selective history reverts preserve later unrelated project changes',async()=>{
 const root=await mkdtemp(join(tmpdir(),'ck-revert-'));try{const s=await new ProjectStore(root).load();await s.mutate('rename',p=>{p.name='New name'});await s.mutate('resolution',p=>{p.settings.width=1080});await s.revert(1);expect(s.project.name).toBe('Untitled story');expect(s.project.settings.width).toBe(1080);await s.mutate('later name',p=>{p.name='Latest'});const rev=s.project.rev;await expect(s.revert(3)).rejects.toMatchObject({code:'REVERT_CONFLICT'});expect(s.project.rev).toBe(rev);}finally{await rm(root,{recursive:true,force:true});}
});
it('uncertain speech repeats while broad deletion requires explicit confirmation',()=>{expect(routeVoice('flobble zzz').kind).toBe('repeat');expect(routeVoice('make this bigger',.3).kind).toBe('repeat');expect(routeVoice('delete the voice track').kind).toBe('confirm');expect(routeVoice('make this bigger').kind).toBe('comment')});
