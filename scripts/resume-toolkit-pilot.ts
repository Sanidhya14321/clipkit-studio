/** Explicit recovery of a retained timed-out session; never replaces its original receipt. */
import {readFile,writeFile,mkdir,copyFile} from 'node:fs/promises';
import path from 'node:path';
import {spawn} from 'node:child_process';
import {startServer} from '../packages/server/src/index';
import {probe} from '../packages/server/src/media';
const base=path.resolve(process.argv[2]??'.clipkit/toolkit-pilot-postfix-20261028'),slot=path.join(base,'codex-toolkit'),workspace=path.join(slot,'workspace'),prior=JSON.parse(await readFile(path.join(slot,'round-0/receipt.json'),'utf8'));
const out=path.join(slot,'recovery');await mkdir(out,{recursive:true});
const server=await startServer({root:workspace,port:4470,publicDir:path.resolve('public'),studioDir:path.resolve('dist/studio')});
const prompts=[
  'Resume finishing the existing task. The retained result.mp4 and source export already exist. Inspect their availability and encoded format; regenerate only if you find a concrete defect. Finish with their paths. The ClipKit server was restarted with the same project and updated resource-URL compatibility; use its current connection. Do not delegate to another model.',
  'Change only the opening headline font size to 240 pixels. Preserve its exact copy, all other styling and every later frame. Prefer property_patch for an exposed control. Save the revised result.mp4 and finish with its path.',
  'Make the movement between 6 and 9 seconds more decisive, with a faster reveal and a clear landing. Preserve total duration, exact copy, final three-second hold and all output outside that range. Save result.mp4 and finish with its path.',
];
const rows:any[]=[];
try{for(const [round,prompt]of prompts.entries()){
  const started=Date.now(),args=['exec','resume',prior.sessionId,'--json','--skip-git-repo-check','-m','gpt-6.1-sol','-c','model_reasoning_effort="medium"','-c','approval_policy="never"','-c','sandbox_mode="danger-full-access"','--disable','multi_agent',prompt];
  let text='',stderr='',timedOut=false;const child=spawn('codex',args,{cwd:workspace,env:{...process.env,CODEX_HOME:path.join(slot,'client-home'),CLIPKIT_ROOT:workspace,TMPDIR:path.resolve('.clipkit/test-tmp')},detached:true,stdio:['ignore','pipe','pipe']});
  const stop=()=>{try{process.kill(-child.pid!,'SIGTERM')}catch{}};const timer=setTimeout(()=>{timedOut=true;stop()},360000);
  child.stdout.on('data',data=>{text+=data;if(text.length>20*1024*1024)stop()});child.stderr.on('data',data=>{stderr=(stderr+data).slice(-8000)});
  console.log(JSON.stringify({event:'resume-start',round}));const exit=await new Promise<number|null>(resolve=>{child.on('error',error=>{stderr=error.message;resolve(null)});child.on('close',resolve)});clearTimeout(timer);
  const events=text.split('\n').flatMap(line=>{try{return [JSON.parse(line)]}catch{return []}}),usage=events.filter(e=>e.type==='turn.completed').at(-1)?.usage??null;
  const folder=path.join(out,'round-'+round);await mkdir(folder,{recursive:true});await writeFile(path.join(folder,'events.jsonl'),text,{mode:0o600});await writeFile(path.join(folder,'stderr.log'),stderr,{mode:0o600});
  let video=null;try{await copyFile(path.join(workspace,'result.mp4'),path.join(folder,'result.mp4'));video=await probe(path.join(folder,'result.mp4'))}catch{}
  await copyFile(path.join(workspace,'.clipkit/project.json'),path.join(folder,'project.json'));
  const receipt={round,prompt,exit,timedOut,elapsedMs:Date.now()-started,cumulativeClientUsage:usage,toolCalls:events.filter(e=>e.type==='item.completed'&&['mcp_tool_call','command_execution'].includes(e.item?.type)).length,video,status:exit===0&&!timedOut&&video?'completed':'incomplete'};rows.push(receipt);await writeFile(path.join(out,'receipts.json'),JSON.stringify({originalAttempt:'../round-0/receipt.json',classification:'Post-timeout recovery and feedback, not a replacement or independent initial-generation sample',rows},null,2));console.log(JSON.stringify({event:'resume-end',...receipt}));if(receipt.status!=='completed')break;
}}finally{await server.close()}
