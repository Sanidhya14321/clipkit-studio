import path from 'node:path';
import {spawn} from 'node:child_process';
import {mkdir,writeFile,readFile} from 'node:fs/promises';
import {freshEnvironment,copyAgentAuth,sandboxCommand} from '../packages/debug/src/isolation';
import {verifyDefaultSkills} from '../packages/debug/src/default-skills';
const env=await freshEnvironment(),root=process.cwd(),rounds:any[]=[];
try{
 if(!await copyAgentAuth(env.userHome))throw Error('No Codex authentication');
 let session='';
 for(let round=0;round<3;round++){
  const args=['exec','--json','--skip-git-repo-check','--sandbox','danger-full-access','--ignore-user-config','-m','gpt-6.1-sol','-c','model_reasoning_effort="low"','--disable','plugins','--disable','apps','--disable','multi_agent'];
  if(round)args.push('resume',session);args.push('Return exactly PREFLIGHT_OK. Do not call tools.');
  const sandbox=await sandboxCommand({workspace:env.workspace,userHome:env.userHome,agent:'codex',args,runtime:path.join(root,'artifacts/package'),deps:path.join(root,'node_modules')});
  const child=spawn(sandbox.command,sandbox.args,{env:sandbox.env,stdio:['ignore','pipe','pipe'],detached:true});let output='',stderr='';child.stdout.on('data',d=>output+=d);child.stderr.on('data',d=>stderr+=d);
  const timer=setTimeout(()=>{try{process.kill(-child.pid!,'SIGKILL')}catch{}},45000),exit=await new Promise(r=>child.on('close',r));clearTimeout(timer);
  const events=output.split('\n').flatMap(l=>{try{return [JSON.parse(l)]}catch{return []}});session=events.find(e=>e.type==='thread.started')?.thread_id??session;
  const proof=JSON.parse(await readFile(path.join(env.userHome,'.harness/baseline-clean.json'),'utf8')),defaultsMatch=await verifyDefaultSkills(path.join(env.userHome,'.codex/skills/.system'));
  rounds.push({round,exit,proof,defaultsMatch,usage:events.find(e=>e.type==='turn.completed')?.usage??null,stderr:stderr.slice(-500)});
  if(exit!==0||!defaultsMatch||!proof.ok||!session)throw Error('Agent/resume preflight failed: '+JSON.stringify(rounds.at(-1)));
 }
}finally{await mkdir('.clipkit/update4',{recursive:true});await writeFile('.clipkit/update4/agent-preflight.json',JSON.stringify({rounds},null,2));console.log(JSON.stringify({rounds}));await env.cleanup();}
