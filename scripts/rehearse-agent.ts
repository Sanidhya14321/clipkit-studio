import {chromium} from '@playwright/test';
import {readFile,writeFile,mkdir,cp} from 'node:fs/promises';
import {resolve,join} from 'node:path';
const root=resolve(import.meta.dirname,'..');const config=JSON.parse(await readFile(join(root,'.clipkit/server.json'),'utf8'));const url=config.url as string;
const op=async(name:string,args:Record<string,unknown>={})=>{const response=await fetch(`${url}/api/op/${name}`,{method:'POST',headers:{'Content-Type':'application/json','X-ClipKit-Token':config.token},body:JSON.stringify(args)});const result=await response.json() as any;if(!response.ok)throw new Error(result.error);return result;};
const state=async()=>await (await fetch(`${url}/api/state`,{headers:{'X-ClipKit-Token':config.token}})).json() as any;
const before=await state();const title=before.project.tracks.flatMap((t:any)=>t.items).find((i:any)=>i.clipId==='kinetic-title');if(!title)throw new Error('Load the graphics fixture before this optional real-agent rehearsal');
const folder=join(root,'.clipkit/verification');await mkdir(folder,{recursive:true});
const browser=await chromium.launch({executablePath:process.env.CHROME_BIN??'/usr/bin/google-chrome',headless:true,args:['--no-sandbox']});
const context=await browser.newContext({viewport:{width:1440,height:1000},recordVideo:{dir:join(folder,'recordings'),size:{width:1440,height:1000}}});const page=await context.newPage();
try{
 await page.goto(url);await page.getByRole('button',{name:/Kinetic title, /}).click();
 const command=page.getByLabel('Voice or typed editing command');await command.fill('go to 2 seconds');await command.press('Enter');
 await page.getByRole('button',{name:'Comments',exact:true}).click();await page.getByLabel('New comment').fill('Change only this title’s accent color to #e4a4ff. Keep all other properties and layers unchanged.');await page.getByRole('button',{name:'Add comment',exact:true}).click();
 let current=await state();for(let n=0;n<30&&!current.project.comments.some((c:any)=>c.status==='open');n++){await new Promise(r=>setTimeout(r,100));current=await state();}
 const comment=current.project.comments.filter((c:any)=>c.status==='open').at(-1);if(!comment?.anchor.layerId)throw new Error('UI did not persist an anchored comment');
 await page.getByRole('button',{name:'Apply open',exact:true}).click();
 let observedLock=false;const deadline=Date.now()+180000;let after:any;
 while(Date.now()<deadline){after=await state();if(after.lease)observedLock=true;if(observedLock&&!after.lease)break;await new Promise(r=>setTimeout(r,500));}
 if(!observedLock||after?.lease)throw new Error('Agent did not complete its real editing turn');
 if(after.agentError)throw new Error(after.agentError);
 const finalComment=after.project.comments.find((c:any)=>c.id===comment.id);if(finalComment?.status!=='resolved')throw new Error('Agent did not resolve its comment');
 const expected=structuredClone(before.project.tracks);for(const t of expected)for(const item of t.items)if(item.id===title.id)item.props.color='#e4a4ff';
 if(JSON.stringify(expected)!==JSON.stringify(after.project.tracks))throw new Error('Agent modified unexpected layer data');
 if(!finalComment.diff?.length)throw new Error('Resolved comment is missing its real diff');
 const reverted=await op('history_revert_turn');const revertedState=await state();if(JSON.stringify(revertedState.project.tracks)!==JSON.stringify(before.project.tracks))throw new Error('Grouped agent revert failed');
 for(let n=0;n<reverted.reverted;n++)await op('history_redo');
 await page.screenshot({path:join(folder,'live-codex-comment.png'),fullPage:true});
 const result={passed:true,client:'codex',model:'gpt-6.1-sol',effort:'medium',commentId:comment.id,layerId:title.id,observedLock,onlyTargetLayerChanged:true,resolvedWithDiff:true,revertRestoredExactTracks:true,revision:after.project.rev,telemetry:after.telemetry,activity:after.activity};
 await writeFile(join(folder,'live-codex-comment.json'),JSON.stringify(result,null,2));console.log(JSON.stringify({...result,activity:undefined},null,2));
}finally{const video=page.video();await context.close();if(video)await cp(await video.path(),join(folder,'live-codex-workflow.webm'));await browser.close();}
