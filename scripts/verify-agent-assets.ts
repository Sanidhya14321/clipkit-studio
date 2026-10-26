/** One actual Codex turn in a disposable project. No Fish requests or user media. */
import assert from 'node:assert/strict';
import {mkdtemp,rm,mkdir,writeFile} from 'node:fs/promises';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {startServer} from '../packages/server/src/index.ts';
import {allItems} from '../packages/core/src/index.ts';
const repo=path.resolve(import.meta.dirname,'..'),root=await mkdtemp(path.join(tmpdir(),'ck-live-card-agent-'));let backend:Awaited<ReturnType<typeof startServer>>|undefined;
try{
 backend=await startServer({root,port:4352,publicDir:path.join(repo,'public'),studioDir:path.join(repo,'dist/studio'),cliPath:path.join(repo,'packages/cli/src/index.ts')});const app=backend;
 const op=async(name:string,data:Record<string,unknown>={})=>{const response=await fetch(app.url+'/api/op/'+name,{method:'POST',headers:{'Content-Type':'application/json','X-ClipKit-Token':app.token},body:JSON.stringify(data)});const body=await response.json() as any;if(!response.ok)throw new Error(JSON.stringify(body));return body;};
 await op('assets_add',{clipId:'kinetic-title',at:0});const before=structuredClone(allItems(app.store.project));const comment=await op('comments_add',{text:'Add exactly one icon-callout clip at 2 seconds with the default properties. Read its asset_card before placing it and inspect the schema as needed. Preserve every existing layer and resolve this comment after the insertion succeeds.',anchor:{layerId:before[0].id,tStart:2}});
 const start=Date.now();await op('agent_run',{client:'codex',commentIds:[comment.commentId]});const deadline=Date.now()+190000;while((app.state().lease||app.runner.child)&&Date.now()<deadline)await new Promise(resolve=>setTimeout(resolve,150));if(app.state().lease||app.runner.child)throw new Error('Live Codex turn exceeded the acceptance timeout');
 const state=app.state(),items=allItems(state.project);const added=items.filter(item=>!before.some(old=>old.id===item.id));assert(!state.agentError,state.agentError??'Agent failed');assert.equal(added.length,1);assert.equal(added[0].clipId,'icon-callout');assert.equal(added[0].at,2);for(const old of before)assert.deepEqual(items.find(item=>item.id===old.id),old);assert.equal(state.project.comments.find(c=>c.id===comment.commentId)?.status,'resolved');const cardToolObserved=state.activity.some(event=>/^codex · .*asset_card/.test(event.text));assert(cardToolObserved,'No actual asset_card tool event observed');
 const report={passed:true,client:'codex',model:'gpt-6.1-sol',effort:'medium',seconds:(Date.now()-start)/1000,cardToolObserved,addedOnlyRequestedClip:true,preservedExistingLayers:true,resolvedComment:true,telemetry:state.telemetry};await mkdir(path.join(repo,'.clipkit/asset-verification'),{recursive:true});await writeFile(path.join(repo,'.clipkit/asset-verification/agent.json'),JSON.stringify(report,null,2));console.log(JSON.stringify(report,null,2));
}finally{await backend?.close();await rm(root,{recursive:true,force:true});}
