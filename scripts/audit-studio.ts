/** Browser acceptance probes against disposable state, with no external AI calls. */
import {chromium} from '@playwright/test';
import {mkdir,mkdtemp,writeFile,rm} from 'node:fs/promises';
import {join,resolve} from 'node:path';
import {tmpdir} from 'node:os';
import {startServer} from '../packages/server/src/index.ts';
import {catalog} from '../packages/assets/src/index.ts';
import {allItems} from '../packages/core/src/index.ts';
const repo=resolve(import.meta.dirname,'..'),temp=await mkdtemp(join(tmpdir(),'clipkit-studio-audit-'));
const out=join(repo,'.clipkit/verification/prd-audit');await mkdir(out,{recursive:true});
const app=await startServer({root:temp,port:4341,publicDir:join(repo,'public'),studioDir:join(repo,'dist/studio')});
const browser=await chromium.launch({executablePath:process.env.CHROME_BIN??'/usr/bin/google-chrome',headless:true,args:['--no-sandbox']});
const evidence:Record<string,unknown>={};
const wait=async(fn:()=>boolean)=>{const end=Date.now()+5000;while(!fn()&&Date.now()<end)await new Promise(r=>setTimeout(r,30));if(!fn())throw new Error('Audit condition timed out');};
try{
 const op=async(name:string,args:Record<string,unknown>={})=>{const r=await fetch(app.url+'/api/op/'+name,{method:'POST',headers:{'Content-Type':'application/json','X-ClipKit-Token':app.token},body:JSON.stringify(args)});const value=await r.json() as any;if(!r.ok)throw new Error(value.error);return value;};
 const page=await browser.newPage({viewport:{width:1440,height:1000}});const errors:string[]=[];page.on('pageerror',e=>errors.push(e.message));await page.goto(app.url);await page.getByRole('button',{name:'Explore demo'}).waitFor({state:'visible'});
 evidence.emptyState={demoVisible:await page.getByRole('button',{name:'Explore demo'}).isVisible(),commandVisible:await page.getByLabel('Voice or typed editing command').isVisible(),libraryCards:await page.locator('.asset-card').count()};
 const hoverChecks=[];
 for(const asset of catalog){
  await op('project_clear');await op('assets_add',{clipId:asset.id,at:0});const item=allItems(app.store.project)[0];
  await wait(()=>Boolean(item));const command=page.getByLabel('Voice or typed editing command');await command.fill(`go to ${asset.id==='wipe'?'.3':'2'} seconds`);
  // Seek syntax accepts numbers with a leading digit.
  if(asset.id==='wipe')await command.fill('go to 0.3 seconds');await command.press('Enter');await page.waitForTimeout(120);
  const nodes=page.locator('[data-ck-layer]').filter({hasNot:page.locator('[data-ck-layer]')});
  const count=await nodes.count();const results=[];
  for(let n=0;n<count;n++){
   const node=nodes.nth(n);const rect=await node.boundingBox();if(!rect||rect.width<=0||rect.height<=0)continue;
   const stage=(await page.locator('.stage').boundingBox())!;const x=Math.max(stage.x+2,Math.min(stage.x+stage.width-2,rect.x+rect.width/2)),y=Math.max(stage.y+2,Math.min(stage.y+stage.height-2,rect.y+rect.height/2));
   await page.mouse.move(x,y);await page.waitForTimeout(40);const label=await page.locator('.layer-outline span').textContent().catch(()=>null);const path=await node.getAttribute('data-ck-path');
   results.push({path,highlighted:label?.includes(item.id)??false,pathInSchema:Boolean(path&&Object.hasOwn(asset.defaults,path))});
  }
  hoverChecks.push({clipId:asset.id,visibleTaggedLeaves:results.length,results});
 }
 evidence.hover=hoverChecks;
 await op('project_clear');await op('assets_add',{clipId:'kinetic-title',at:0});const item=allItems(app.store.project)[0];
 const command=page.getByLabel('Voice or typed editing command');await command.fill('go to 2 seconds');await command.press('Enter');
 const textNode=page.locator('[data-ck-path="text"]').first();await textNode.click();
 const canvasSelectsTimeline=await page.locator('.timeline-clip.selected').count()===1;
 const explicitAsk=await page.getByRole('button',{name:/Ask agent/}).count()>0;if(explicitAsk)await page.getByRole('button',{name:/Ask agent/}).click();else await page.getByRole('button',{name:'Add anchored comment'}).click();await page.getByLabel('New comment').fill('Audit exact selection anchor');await page.getByRole('button',{name:'Add comment',exact:true}).click();await wait(()=>app.store.project.comments.length>0);
 const anchor=app.store.project.comments.at(-1)!.anchor;evidence.askAgent={explicitAskButton:explicitAsk,correctLayer:anchor.layerId===item.id,propPath:anchor.propPath,elementId:anchor.elementId};
 await page.getByRole('button',{name:/Kinetic title, /}).click();await page.mouse.move(20,20);await page.waitForTimeout(60);
 evidence.selectionSync={canvasSelectsTimeline,timelineSelected:await page.locator('.timeline-clip.selected').count()===1,canvasOutlineAfterTimelineSelection:await page.locator('.layer-outline').count()>0};
 await page.getByRole('button',{name:'Properties',exact:true}).click();const field=page.getByRole('textbox',{name:'Text',exact:true});await field.fill('Audit changed title');
 const beforeBlur=allItems(app.store.project).find(i=>i.id===item.id)!.props.text;await field.blur();await wait(()=>allItems(app.store.project).find(i=>i.id===item.id)!.props.text==='Audit changed title');
 const color=page.locator('input[type="color"]').first();await color.fill('#0faaff');await color.blur();await wait(()=>allItems(app.store.project).find(i=>i.id===item.id)!.props.color==='#0faaff');
 evidence.inspector={liveBeforeBlur:beforeBlur==='Audit changed title',committedTextAndColor:true,author:app.store.history.at(-1)!.author,floatingInspector:false};
 await page.screenshot({path:join(out,'canvas-selection.png'),fullPage:true});
 // Reproduce the baseline's inclusion of transcripts no longer referenced by the project.
 const baselineBefore=app.state().telemetry.baselineTokens;app.state().transcripts.orphan=Array.from({length:100},(_,n)=>({text:'unused word',start:n,end:n+.5}));
 evidence.baseline={projectSources:Object.keys(app.store.project.sources).length,orphanTranscriptIncluded:true,before:baselineBefore,after:app.state().telemetry.baselineTokens};
 evidence.pageErrors=errors;
}finally{await browser.close();await app.close();await rm(temp,{recursive:true,force:true});await writeFile(join(out,'studio-evidence.json'),JSON.stringify(evidence,null,2)+'\n');}
console.log('Studio acceptance observations saved to .clipkit/verification/prd-audit/studio-evidence.json');
