import {describe,it,expect,beforeEach,afterEach} from 'vitest';
import {mkdtemp,rm,readFile,writeFile} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import {join} from 'node:path';
import {ProjectStore} from '../packages/core/src/store.ts';
import {routeVoice,timelineWords,emptyProject,tokenEstimate} from '../packages/core/src/index.ts';
let root:string;
beforeEach(async()=>{root=await mkdtemp(join(tmpdir(),'clipkit-test-'));});
afterEach(async()=>{await rm(root,{recursive:true,force:true});});
describe('durable shared transactions',()=>{
  it('commits, deduplicates and rejects stale revisions',async()=>{const s=await new ProjectStore(root).load();const req={requestId:'one',baseRev:0,label:'Rename',ops:[{op:'replace' as const,path:'/name',value:'Story'}]};await s.patch(req);await s.patch(req);expect(s.project.rev).toBe(1);await expect(s.patch({...req,requestId:'two'})).rejects.toMatchObject({code:'REV_CONFLICT'});expect(s.project.name).toBe('Story');});
  it('rejects invalid atomic patches without touching disk',async()=>{const s=await new ProjectStore(root).load();const before=await readFile(join(root,'.clipkit/project.json'),'utf8');await expect(s.patch({requestId:'bad',baseRev:0,label:'bad',ops:[{op:'replace',path:'/name',value:'Changed'},{op:'replace',path:'/settings/fps',value:0}]})).rejects.toMatchObject({code:'INVALID_PATCH'});expect(await readFile(join(root,'.clipkit/project.json'),'utf8')).toBe(before);});
  it('undoes and redoes exact content across restart',async()=>{let s=await new ProjectStore(root).load();const initial=structuredClone(s.project);await s.mutate('rename',p=>{p.name='Changed';});await s.undo();expect({...s.project,rev:0}).toEqual(initial);s=await new ProjectStore(root).load();await s.redo();expect(s.project.name).toBe('Changed');expect(s.project.rev).toBe(3);});
  it('recovers a snapshot from durable journal',async()=>{const s=await new ProjectStore(root).load();const before=structuredClone(s.project);await s.mutate('recovered',p=>{p.name='Recovered';});await writeFile(join(root,'.clipkit/project.json'),JSON.stringify(before));const recovered=await new ProjectStore(root).load();expect(recovered.project.name).toBe('Recovered');expect(recovered.project.rev).toBe(1);});
  it('serializes competing mutations',async()=>{const s=await new ProjectStore(root).load();await Promise.all([s.mutate('a',p=>{p.name+='a';}),s.mutate('b',p=>{p.name+='b';})]);expect(s.project.name).toBe('Untitled storyab');expect(s.project.rev).toBe(2);});
  it('enforces lease ownership, expiry and late patch rejection',async()=>{let now=100;const s=await new ProjectStore(root,undefined,()=>now).load();const l=s.acquire('codex','Editing title');await expect(s.mutate('human',p=>{p.name='No';})).rejects.toMatchObject({code:'LOCKED'});await s.mutate('agent',p=>{p.name='Yes';},'agent',{leaseToken:l.token,turnId:l.turnId});now+=120001;expect(s.currentLease()).toBeNull();await expect(s.mutate('late',p=>{p.name='Late';},'agent',{leaseToken:l.token,turnId:l.turnId})).rejects.toMatchObject({code:'LEASE_EXPIRED'});expect(s.project.name).toBe('Yes');});
  it('rejects protected fields and prototype paths',async()=>{const s=await new ProjectStore(root).load();for(const path of ['/rev','/id','/__proto__/polluted'])await expect(s.patch({requestId:path,baseRev:0,label:'bad',ops:[{op:'add',path,value:1}]})).rejects.toMatchObject({code:'INVALID_PATCH'});});
  it('reverts only a latest agent turn',async()=>{const s=await new ProjectStore(root).load();const l=s.acquire('codex','change');await s.mutate('a',p=>{p.name='A';},'agent',{turnId:l.turnId,leaseToken:l.token});await s.mutate('b',p=>{p.name='B';},'agent',{turnId:l.turnId,leaseToken:l.token});s.release(l.token);expect((await s.revertLatestTurn()).reverted).toBe(2);expect(s.project.name).toBe('Untitled story');});
});
describe('compact context and local command grammar',()=>{
  it.each(['play','pause','undo','redo','split here','delete selected','zoom in','zoom out','loop this section'])('routes %s locally',text=>{expect(routeVoice(text).kind).toBe('local');});
  it('parses seeks and keeps creative edits as comments',()=>{expect(routeVoice('Go to 12 seconds.')).toEqual({kind:'local',action:'seek',value:12});expect(routeVoice('make this bigger').kind).toBe('comment');expect(routeVoice('clear all').kind).toBe('confirm');});
  it('maps retained source words without rewriting transcripts',()=>{const p=emptyProject();p.tracks[0].items.push({id:'a',type:'footage',src:'s',srcIn:2,at:1,dur:3,volume:1,props:{}});const words=[{text:'hello',start:2.5,end:3}];expect(timelineWords(p,{s:words})[0]).toMatchObject({start:1.5,end:2,sourceStart:2.5});expect(words[0].start).toBe(2.5);});
  it('keeps an ordinary affected-layer bundle below the incremental budget',()=>{expect(tokenEstimate({rev:12,comments:[{text:'Make this bigger',anchor:{layerId:'a',tStart:3}}],layers:[{id:'a',props:{text:'ClipKit',size:70}}]})).toBeLessThan(3000);});
});
