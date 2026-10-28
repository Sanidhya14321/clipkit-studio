import {redact} from '../../packages/debug/src/trace';
import type {Page,TestInfo} from '@playwright/test';
import {appendFile,mkdir} from 'node:fs/promises';
import path from 'node:path';
export async function recordDriver(page:Page,info:TestInfo){
 const folder=info.outputPath('driver');
 await mkdir(folder,{recursive:true});
 const log=(event:unknown)=>void appendFile(path.join(folder,'events.jsonl'),JSON.stringify(event)+'\n');
 page.on('console',message=>log({type:'console',level:message.type(),bytes:message.text().length,detail:redact(message.text())}));
 page.on('requestfinished',request=>{
  if(request.url().includes('/api/op/'))log({type:'request',operation:new URL(request.url()).pathname,method:request.method(),traceId:request.headers()['x-clipkit-trace']??null});
 });
 page.on('pageerror',error=>log({type:'pageerror',name:error.name}));
 await info.attach('driver-log-directory',{body:Buffer.from(folder),contentType:'text/plain'});
 return {step:async(name:string,action:()=>Promise<void>)=>{
  log({type:'step',name});await action();
  await page.screenshot({path:path.join(folder,name.replace(/[^a-z0-9-]/gi,'-')+'.png')});
 }};
}
