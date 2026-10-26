/** Acceptance probes, not feature implementation. Known gaps are recorded as failures.
 * Runs only in a temporary project; never modifies the user's open timeline.
 * --real-agent-cycles performs three real, paid/subscription Codex turns.
 */
import {mkdir,mkdtemp,readFile,writeFile,rm,chmod} from 'node:fs/promises';
import {join,resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {createHash} from 'node:crypto';
import {chromium} from '@playwright/test';
import {ProjectStore} from '../packages/core/src/store.ts';
import {emptyProject,ProjectSchema,routeVoice,allItems,type Item} from '../packages/core/src/index.ts';
import {catalog,searchAssets} from '../packages/assets/src/index.ts';
import {buildAssets} from '../packages/assets/src/build.ts';
import {startServer} from '../packages/server/src/index.ts';
import {probe} from '../packages/server/src/media.ts';
import {toolDefinitions} from '../packages/mcp/src/index.ts';
import {agentInstructions} from '../packages/server/src/runner.ts';

const repo=resolve(import.meta.dirname,'..');
const out=join(repo,'.clipkit/verification/prd-audit');await mkdir(out,{recursive:true});
const temp=await mkdtemp(join(tmpdir(),'clipkit-prd-audit-'));
const evidence:Record<string,unknown>={};
const record=(name:string,details:unknown)=>{evidence[name]=details;console.log(`Recorded ${name}`);};
const hash=(b:Uint8Array)=>createHash('sha256').update(b).digest('hex');
const wait=async(condition:()=>boolean,ms=10000)=>{const end=Date.now()+ms;while(!condition()&&Date.now()<end)await new Promise(r=>setTimeout(r,50));if(!condition())throw new Error('Timed out waiting for audit condition');};
let backend:Awaited<ReturnType<typeof startServer>>|undefined;
let browser:Awaited<ReturnType<typeof chromium.launch>>|undefined;
try{
 const storeRoot=join(temp,'core');let clock=100;
 let core=await new ProjectStore(storeRoot,undefined,()=>clock).load();
 const missing=structuredClone(core.project) as any;delete missing.settings;
 const wrong=structuredClone(core.project) as any;wrong.settings.fps='bad';
 record('schema',{valid:ProjectSchema.safeParse(core.project).success,missingRejected:!ProjectSchema.safeParse(missing).success,wrongTypeRejected:!ProjectSchema.safeParse(wrong).success});
 for(const name of ['One','Two','Three'])await core.mutate(name,p=>{p.name=name;});
 await core.undo();await core.undo();await core.redo();core=await new ProjectStore(storeRoot,undefined,()=>clock).load();
 record('persistence',{name:core.project.name,revision:core.project.rev,history:core.history.length,expectedName:'Two'});
 const lease=core.acquire('audit','Inactivity check');const initialExpiry=lease.expiresAt;clock+=60000;
 await core.mutate('active patch',p=>{p.name='Active';},'agent',{leaseToken:lease.token,turnId:lease.turnId});
 record('patchHeartbeat',{initialExpiry,afterPatchExpiry:core.lease!.expiresAt,expectedExpiry:clock+120000,patchExtendsLease:core.lease!.expiresAt===clock+120000});
 clock=core.lease!.expiresAt+1;record('expiry',{releases:core.currentLease()===null});
 record('grammar',{deleteTrack:routeVoice('delete the track'),unclear:routeVoice('mumbled unrelated speech'),clearAll:routeVoice('clear all'),localCommands:['play','pause','undo','redo','go to 12 seconds','split here','delete selected','zoom in','loop this section'].map(text=>({text,intent:routeVoice(text)}))});
 record('mcp',{tools:Object.keys(toolDefinitions),missingEditingOperations:['status_working','status_idle','footage_transcribe','voice_clone','history_undo','history_redo','history_revert_turn'].filter(name=>!toolDefinitions[name]),instructionLines:agentInstructions.trim().split('\n').length});
 record('search',{query:'a title intro',ids:searchAssets('a title intro').map(a=>a.id)});
 const oldLicense=catalog[0].license;let rejected=false;
 try{catalog[0].license='';await buildAssets(repo,join(temp,'invalid-pack'));}catch{rejected=true;}finally{catalog[0].license=oldLicense;}
 record('packMissingLicense',{rejected,nativeRegistryOnly:false});

 backend=await startServer({root:join(temp,'studio'),port:4340,publicDir:join(repo,'public'),studioDir:join(repo,'dist/studio'),cliPath:join(repo,'packages/cli/src/index.ts')});
 const app=backend;
 const op=async(name:string,args:Record<string,unknown>={})=>{const r=await fetch(app.url+'/api/op/'+name,{method:'POST',headers:{'Content-Type':'application/json','X-ClipKit-Token':app.token},body:JSON.stringify(args)});const body=await r.json() as any;if(!r.ok)throw new Error(`${name}: ${body.error}`);return body;};
 const fixture=await app.media.fixture();const beforeHash=hash(await readFile(fixture.path));
 const imported=await app.media.importFile(fixture.path);const source=app.store.project.sources[imported.sourceId];
 record('import',{originalUnchanged:beforeHash===hash(await readFile(fixture.path)),original:await probe(fixture.path),proxy:await probe(resolve(app.store.root,source.proxyPath!)),hasExtractedAudio:Boolean(source.audioPath)});
 app.state().transcripts[source.id]=[{text:'remove',start:1,end:2}];
 const first=app.store.project.tracks[0].items[0];await op('footage_delete_words',{sourceId:source.id,itemId:first.id,start:1,end:2});
 const suffix=app.store.project.tracks[0].items.find(i=>i.srcIn===2)!;
 await op('item_update',{itemId:suffix.id,changes:{srcIn:1,dur:4}});
 const restored=allItems(app.store.project).find(i=>i.id===suffix.id)!;
 const baked=app.store.project.sources[String(restored.props.audioSourceId)];
 record('restoreCut',{expectedAudioStart:1,expectedAudioDuration:4,actualVideoSrcIn:restored.srcIn,actualVideoDuration:restored.dur,bakedAudioDuration:(await probe(resolve(app.store.root,baked.path))).durationSec,audioSrcIn:restored.props.audioSrcIn,staleBakedAudioSource:Math.abs((await probe(resolve(app.store.root,baked.path))).durationSec-restored.dur)>.04});
 let accepted=false;try{await op('item_update',{itemId:restored.id,changes:{srcIn:1000,dur:20}});accepted=true;}catch{}
 record('sourceBounds',{acceptedOutsideSource:accepted,sourceDuration:source.durationSec});

 // Structured ten-minute fixture: this checks retrieval, not real speech recognition.
 const words=Array.from({length:1200},(_,i)=>({text:'um',start:i*.5,end:i*.5+.25}));app.state().transcripts[source.id]=words;
 const packed=await op('footage_pack',{sourceId:source.id,start:0,end:600});
 record('packedTranscript',{fixtureDurationSec:600,wordCount:words.length,utf8Bytes:Buffer.byteLength(JSON.stringify(packed)),phraseCount:packed.phrases.length,truncated:packed.truncated,fillerCandidates:packed.candidates.fillers.length});
 await op('demo_reset');const title=allItems(app.store.project).find(i=>i.clipId==='kinetic-title')!;
 const extra={id:'overlay-2',name:'Second graphics',kind:'overlay',items:[]};
 await op('project_patch',{requestId:'audit-track',baseRev:app.store.project.rev,label:'Audit second track',ops:[{op:'add',path:'/tracks/-',value:extra}]});
 const commentIds=[];for(const target of [title.id,title.id,allItems(app.store.project).find(i=>i.clipId==='lower-third')!.id]){const c=await op('comments_add',{text:'Audit anchored comment',anchor:{tStart:1,layerId:target}});commentIds.push(c.commentId);}
 const bundle=await op('comments_bundle',{commentIds});record('commentBundle',{comments:bundle.comments.length,layers:bundle.layers.length,uniqueLayerIds:new Set(bundle.layers.map((i:Item)=>i.id)).size,utf8Bytes:Buffer.byteLength(JSON.stringify(bundle))});
 await op('voice_script',{text:'First line\nSecond line\nThird line\nFourth line\nFifth line'});
 record('voiceSlots',{slots:app.store.project.voiceSlots.length,items:app.store.project.tracks.find(t=>t.kind==='voice')!.items.length});
 await op('demo_reset');

 // Child-process fault injection, explicitly not a simulated successful coding-agent run.
 const oldPath=process.env.PATH;const bin=join(temp,'fault-bin');await mkdir(bin);await writeFile(join(bin,'codex'),'#!/usr/bin/env node\nsetInterval(()=>{},1000);\n');await chmod(join(bin,'codex'),0o755);
 try{process.env.PATH=bin+':'+oldPath;app.runner.enqueue('codex','Fault injection only',[]);await wait(()=>Boolean(app.runner.child));app.runner.child!.kill('SIGKILL');await wait(()=>!app.store.currentLease());record('agentCrash',{faultInjection:true,leaseReleased:true,projectValid:ProjectSchema.safeParse(app.store.project).success});}finally{process.env.PATH=oldPath;}

 browser=await chromium.launch({executablePath:process.env.CHROME_BIN??'/usr/bin/google-chrome',headless:true,args:['--no-sandbox']});
 const page=await browser.newPage({viewport:{width:1280,height:800}});const pageErrors:string[]=[];page.on('pageerror',e=>pageErrors.push(e.message));
 await page.goto(app.url);await page.getByRole('button',{name:/Kinetic title, /}).click();
 const command=page.getByLabel('Voice or typed editing command');await command.fill('go to 1 seconds');await command.press('Enter');
 await page.waitForFunction(()=>Array.from(document.images).every(img=>img.complete&&img.naturalWidth>0));
 const snapshot=structuredClone(app.store.project.tracks);const beforeRev=app.store.project.rev;
 const clip=page.getByRole('button',{name:/Kinetic title, /});const rect=(await clip.boundingBox())!;
 await page.mouse.move(rect.x+rect.width/2,rect.y+rect.height/2);await page.mouse.down();await page.mouse.move(rect.x+rect.width/2+50,rect.y+rect.height/2,{steps:8});await page.mouse.up();
 await wait(()=>app.store.project.rev>beforeRev);const dragRevisions=app.store.project.rev-beforeRev;await op('history_undo');
 record('gesture',{revisionCount:dragRevisions,undoRestoredExactTracks:JSON.stringify(app.store.project.tracks)===JSON.stringify(snapshot)});
 await page.getByRole('button',{name:/Kinetic title, /}).click();await page.getByRole('button',{name:/Kinetic title, /}).focus();
 const count=allItems(app.store.project).length;await page.keyboard.press('d');await page.waitForTimeout(150);const bareD=allItems(app.store.project).length>count;
 if(bareD){await page.keyboard.press('Control+z');await wait(()=>allItems(app.store.project).length===count);}
 await page.keyboard.press('Control+d');await wait(()=>allItems(app.store.project).length>count);await page.keyboard.press('Control+z');await wait(()=>allItems(app.store.project).length===count);
 await page.keyboard.press('Control+Shift+z');await page.waitForTimeout(250);const redo=allItems(app.store.project).length>count;
 record('shortcuts',{bareDDuplicates:bareD,ctrlDDuplicates:true,ctrlZUndo:true,ctrlShiftZRedo:redo});
 const focus=await page.evaluate(()=>{const e=document.activeElement as HTMLElement;const s=getComputedStyle(e);return {tag:e.tagName,outlineStyle:s.outlineStyle,outlineWidth:s.outlineWidth};});record('focus',focus);
 const layouts=[];for(const width of [1280,1920]){await page.setViewportSize({width,height:900});await page.screenshot({path:join(out,`studio-${width}.png`),fullPage:true});layouts.push(await page.evaluate(()=>({width:innerWidth,scrollWidth:document.documentElement.scrollWidth,panels:Array.from(document.querySelectorAll('.library,.main,.inspector')).map(e=>{const r=e.getBoundingClientRect();return {className:e.className,left:r.left,right:r.right,width:r.width};})})));}record('layout',{layouts,pageErrors});
 await page.close();

 if(process.argv.includes('--real-agent-cycles')){
  await op('demo_reset');const item=allItems(app.store.project).find(i=>i.clipId==='kinetic-title')!;const cycles=[];
  for(let n=1;n<=3;n++){
   const before=structuredClone(app.store.project.tracks);const value=`Acceptance audit cycle ${n}`;
   const c=await op('comments_add',{text:`Set ONLY this title's text to exactly "${value}". Preserve all other properties and layers.`,anchor:{tStart:1,layerId:item.id,propPath:'text'}});
   await op('agent_run',{client:'codex',commentIds:[c.commentId]});const started=Date.now();await wait(()=>!app.state().lease,200000);
   const state=app.state();const actual=allItems(state.project).find(i=>i.id===item.id)!;for(const track of before)for(const layer of track.items)if(layer.id===item.id)layer.props.text=value;
   cycles.push({cycle:n,seconds:(Date.now()-started)/1000,resolved:state.project.comments.find(x=>x.id===c.commentId)?.status==='resolved',onlyTargetPropertyChanged:JSON.stringify(before)===JSON.stringify(state.project.tracks),valueMatches:actual.props.text===value,telemetry:state.telemetry,error:state.agentError});
   console.log(`Real Codex cycle ${n}: ${cycles.at(-1)!.resolved?'resolved':'not resolved'}`);
   if(state.agentError)break;
  }
  record('realAgentCycles',cycles);
 }
}finally{await browser?.close();await backend?.close();await rm(temp,{recursive:true,force:true});await writeFile(join(out,'evidence.json'),JSON.stringify(evidence,null,2)+'\n');}
console.log('Acceptance probes complete; review failures in .clipkit/verification/prd-audit/evidence.json.');
