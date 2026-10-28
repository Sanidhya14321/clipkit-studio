/** Six bounded real-client slots. No synthetic usage or replacement of failed attempts. */
import {mkdir,readFile,writeFile,copyFile,symlink,stat,access} from 'node:fs/promises';
import path from 'node:path';
import {homedir} from 'node:os';
import {spawn,execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {startServer} from '../packages/server/src/index';
import {toolkitInstructions} from '../packages/mcp/src/toolkit-tools';
import {probe} from '../packages/server/src/media';

const kit=process.cwd(),out=path.resolve(process.argv[2]??`.clipkit/toolkit-pilot-${new Date().toISOString().replace(/[:.]/g,'-')}`);
try{await access(path.join(out,'protocol.json'));throw Error('Pilot folder exists; preserved. Choose a new folder.')}catch(e){if((e as NodeJS.ErrnoException).code!=='ENOENT')throw e}
await mkdir(out,{recursive:true});
const brief='Create a 24-second fast-paced portrait film about turning a web idea into an editable video. Native 1080x1920 at 30 FPS, silent. Exact opening copy: "MAKE IT MOVE." Exact ending copy, fully readable for the last 3 seconds: "MAKE YOUR NEXT FRAME." Show a meaningful transformation from web-style movement into a finished film and precise control. Use bold typography, strong framing, a real media/graphic reveal and connected motion; choose your own visual direction. Avoid a sequence of generic centered title cards. Supplied assets: public/inter.ttf and public/art.svg. You may search the internet and install dependencies. Deliver a regular result.mp4 and actual editable source in this workspace. Verify the encoded output. Do not launch or delegate to another model.';
const guidance='Choose the visual argument before effects. Map key poses, readable holds and motivated handoffs to explicit frame time. Preserve exact copy. Render and inspect moving output; revise concrete weaknesses up to twice. Source fidelity, hierarchy and rhythm matter more than effect counts. Keep binary resources as files. Edit only affected code/props; preserve unrelated frames.';
const feedback=['Change only the opening headline font size to 240 pixels. Preserve its exact copy, all other styling and every later frame. Save the revised result.mp4.', 'Make the movement between 6 and 9 seconds more decisive, with a faster reveal and a clear landing. Preserve total duration, exact copy, final three-second hold and all output outside that range. Save result.mp4.'];
const protocol={brief,guidance,feedback,arms:(process.env.CLIPKIT_PILOT_ARMS??'plain,guidance,toolkit').split(','),clients:(process.env.CLIPKIT_PILOT_CLIENTS??'codex,claude').split(','),followupOf:process.env.CLIPKIT_PILOT_PARENT??null,codexModel:'gpt-6.1-sol',codexEffort:'medium',timeoutMs:480000,maxOutputBytes:20*1024*1024,network:'enabled equally',setup:'Shared pinned Node/Remotion/FFmpeg dependencies and fixtures; setup measured separately',policy:'One initial attempt per arm/client, two exact-session feedback turns only after a retained export. No retries replacing failures. No statistical or automatic taste claim.'};
await writeFile(path.join(out,'protocol.json'),JSON.stringify(protocol,null,2));
const rows:any[]=[];
const save=()=>writeFile(path.join(out,'results.json'),JSON.stringify({protocol,rows},null,2));
const requireFile=async(file:string)=>{try{return await readFile(file)}catch{return null}};
const tsx=import.meta.resolve('tsx');
const cli=path.join(kit,'packages/cli/src/index.ts');
const versions:Record<string,string>={};for(const client of protocol.clients)try{versions[client]=(await promisify(execFile)(client,['--version'],{timeout:15000})).stdout.trim()}catch{versions[client]='unavailable'}
for(const client of protocol.clients){
  if(client==='claude'){
    let authenticated=false;try{const status=JSON.parse((await promisify(execFile)('claude',['auth','status'],{timeout:15000})).stdout);authenticated=!!status.loggedIn}catch{}
    if(!authenticated){for(const arm of protocol.arms)rows.push({client,version:versions[client],arm,launched:false,status:'unavailable',reason:'Claude CLI is not authenticated; no model calls or invented results',usage:null});await save();console.log(JSON.stringify({client,status:'unauthenticated'}));continue}
  }
  let exhausted=false;
  for(const [armIndex,arm]of protocol.arms.entries()){
    if(exhausted){rows.push({client,arm,launched:false,status:'not-launched',reason:'Provider capacity/quota failure in previous slot',usage:null});await save();continue}
    const started=Date.now(),folder=path.join(out,client+'-'+arm),workspace=path.join(folder,'workspace'),home=path.join(folder,'client-home');await mkdir(path.join(workspace,'public'),{recursive:true});await mkdir(home,{recursive:true});
    await symlink(path.join(kit,'node_modules'),path.join(workspace,'node_modules'),'dir');
    await copyFile(path.join(kit,'public/vendor/inter.ttf'),path.join(workspace,'public/inter.ttf'));
    const art=await requireFile(path.join(kit,'.clipkit/rebuild-film-24/art.svg'));if(!art)throw Error('Build the demonstration film first to prepare common artwork');await writeFile(path.join(workspace,'public/art.svg'),art);
    const scaffold=`import React from 'react';import {registerRoot,Composition} from 'remotion';import Film from './Film';registerRoot(()=>React.createElement(Composition,{id:'Film',component:Film,width:1080,height:1920,fps:30,durationInFrames:720}));`;
    await writeFile(path.join(workspace,'index.tsx'),scaffold);await writeFile(path.join(workspace,'Film.tsx'),`import React from 'react';export default()=>null;`);
    await writeFile(path.join(workspace,'package.json'),JSON.stringify({private:true,type:'module'}));
    await writeFile(path.join(workspace,'render.mjs'),`import {bundle} from '@remotion/bundler';import {selectComposition,renderMedia} from '@remotion/renderer';const serveUrl=await bundle({entryPoint:'index.tsx'});const composition=await selectComposition({serveUrl,id:'Film'});await renderMedia({serveUrl,composition,codec:'h264',muted:true,pixelFormat:'yuv420p',crf:18,concurrency:2,outputLocation:'result.mp4'});`);
    let server:Awaited<ReturnType<typeof startServer>>|undefined;
    const instructions=arm==='toolkit'?toolkitInstructions:arm==='guidance'?guidance:'';
    await writeFile(path.join(workspace,'AGENTS.md'),instructions+'\nFor standalone Remotion use index.tsx + Film.tsx and node render.mjs. Deliver result.mp4 as a regular file. Do not modify outside this workspace.');await writeFile(path.join(workspace,'CLAUDE.md'),instructions);
    const mcp={command:process.execPath,args:['--import',tsx,cli,'mcp'],env:{CLIPKIT_ROOT:workspace,CLIPKIT_TOOL_PROFILE:'toolkit'}};
    const env={...process.env,CODEX_HOME:home,CLAUDE_CONFIG_DIR:client==='claude'?path.join(homedir(),'.claude'):path.join(home,'claude'),CLIPKIT_ROOT:workspace,TMPDIR:path.join(kit,'.clipkit/test-tmp')};
    const auth=await requireFile(path.join(homedir(),'.codex/auth.json'));if(auth)await writeFile(path.join(home,'auth.json'),auth,{mode:0o600});
    if(client==='codex')await writeFile(path.join(home,'config.toml'),arm==='toolkit'?`[mcp_servers.clipkit]\ncommand=${JSON.stringify(mcp.command)}\nargs=${JSON.stringify(mcp.args)}\n[mcp_servers.clipkit.env]\nCLIPKIT_ROOT=${JSON.stringify(workspace)}\nCLIPKIT_TOOL_PROFILE="toolkit"\n`:'');
    if(arm==='toolkit')server=await startServer({root:workspace,port:4470+armIndex,publicDir:path.join(kit,'public'),studioDir:path.join(kit,'dist/studio')});
    const setupMs=Date.now()-started,rounds:any[]=[];let sessionId:string|undefined;
    try{
      for(let round=0;round<3;round++){
        const prompt=round?feedback[round-1]:brief+(arm==='toolkit'?' Use the ClipKit toolkit and actual website-origin adaptations. Render with toolkit operations and copy the returned video bytes into result.mp4; preserve the ClipKit scene/project sources.':'');
        const codexArgs=['exec',...sessionId?['resume',sessionId]:[],'--json','--skip-git-repo-check','-m',protocol.codexModel,'-c','model_reasoning_effort="medium"','-c','approval_policy="never"','-c','sandbox_mode="danger-full-access"','--disable','multi_agent',prompt];
        const claudeArgs=['-p',prompt,'--output-format','stream-json','--verbose','--strict-mcp-config','--mcp-config',JSON.stringify({mcpServers:arm==='toolkit'?{clipkit:mcp}:{}}),'--setting-sources','project','--permission-mode','acceptEdits',...sessionId?['--resume',sessionId]:[]];
        const events:any[]=[];let stderr='',text='',timedOut=false,overflow=false;const roundStart=Date.now();
        console.log(JSON.stringify({event:'start',client,arm,round,setupMs}));
        const child=spawn(client,client==='codex'?codexArgs:claudeArgs,{cwd:workspace,env,stdio:['ignore','pipe','pipe'],detached:true});
        const stop=()=>{try{process.kill(-child.pid!,'SIGTERM')}catch{}setTimeout(()=>{try{process.kill(-child.pid!,'SIGKILL')}catch{}},5000).unref()};const timer=setTimeout(()=>{timedOut=true;stop()},protocol.timeoutMs);
        child.stdout.on('data',data=>{text+=data;if(Buffer.byteLength(text)>protocol.maxOutputBytes){overflow=true;stop()}});child.stderr.on('data',data=>{stderr=(stderr+data).slice(-16000)});
        const exit=await new Promise<number|null>(resolve=>{child.on('error',error=>{stderr=error.message;resolve(null)});child.on('close',resolve)});clearTimeout(timer);
        for(const line of text.split('\n'))try{events.push(JSON.parse(line))}catch{}
        sessionId=events.find(e=>e.type==='thread.started')?.thread_id??events.find(e=>e.type==='system'&&e.session_id)?.session_id??sessionId;
        const usage=events.filter(e=>e.type==='turn.completed').map(e=>e.usage);let measured:any=null;
        if(client==='codex'&&usage.length)measured={source:'client-reported',input:usage.reduce((n,u)=>n+(u.input_tokens??0),0),cached:usage.reduce((n,u)=>n+(u.cached_input_tokens??0),0),output:usage.reduce((n,u)=>n+(u.output_tokens??0),0),costUSD:null};
        const result=[...events].reverse().find((e:any)=>e.type==='result');if(client==='claude'&&result?.usage){const u=result.usage;measured={source:'client-reported',input:(u.input_tokens??0)+(u.cache_read_input_tokens??0)+(u.cache_creation_input_tokens??0),cached:u.cache_read_input_tokens??0,output:u.output_tokens??0,costUSD:result.total_cost_usd??null,raw:u}}
        if(measured){measured.uncached=measured.input-measured.cached;measured.total=measured.input+measured.output}
        const destination=path.join(folder,'round-'+round);await mkdir(destination,{recursive:true});
        const {readdir}=await import('node:fs/promises');const sources:Record<string,string>={};let sourceBytes=0;
        const snapshot=async(dir:string)=>{for(const e of await readdir(dir,{withFileTypes:true})){if(e.isSymbolicLink()||e.name.startsWith('.')||e.name==='node_modules')continue;const file=path.join(dir,e.name);if(e.isDirectory())await snapshot(file);else if(/\.(tsx?|jsx?|mjs|css|json|md|svg)$/.test(file)){const bytes=await stat(file);if(bytes.size>300000||sourceBytes+bytes.size>5000000)continue;sources[path.relative(workspace,file)]=await readFile(file,'utf8');sourceBytes+=bytes.size}}};await snapshot(workspace);await writeFile(path.join(destination,'authored-source.json'),JSON.stringify(sources),{mode:0o600});
        if(server)await writeFile(path.join(destination,'project.json'),JSON.stringify(server.store.project,null,2),{mode:0o600});await writeFile(path.join(destination,'events.jsonl'),text,{mode:0o600});await writeFile(path.join(destination,'stderr.log'),stderr,{mode:0o600});
        let video:any=null;try{const candidate=path.join(workspace,'result.mp4');const info=await stat(candidate);if(!info.isFile())throw Error('Not a regular file');await copyFile(candidate,path.join(destination,'result.mp4'));video=await probe(path.join(destination,'result.mp4'))}catch{}
        const toolCalls=events.filter(e=>e.type==='item.completed'&&['command_execution','mcp_tool_call','web_search'].includes(e.item?.type)).length||events.filter(e=>e.type==='assistant').reduce((n,e)=>n+(e.message?.content??[]).filter((c:any)=>c.type==='tool_use').length,0);
        const receipt={round,exit,timedOut,overflow,elapsedMs:Date.now()-roundStart,usage:measured,toolCalls,video,status:exit===0&&video&&!timedOut?'completed':'failed',sessionId};rounds.push(receipt);await writeFile(path.join(destination,'receipt.json'),JSON.stringify(receipt,null,2));console.log(JSON.stringify({event:'end',client,arm,...receipt}));
        if(/usage limit|rate_limit|insufficient_quota|capacity|not logged in|login required/i.test(stderr+'\n'+text.slice(-6000)))exhausted=true;
        if(receipt.status!=='completed'||!sessionId)break;
      }
    }finally{await server?.close();rows.push({client,version:versions[client],model:client==='codex'?protocol.codexModel:'CLI configured default',arm,launched:true,setupMs,rounds,complete:rounds.length===3&&rounds.every(r=>r.status==='completed')});await save()}
  }
}
await writeFile(path.join(out,'REPORT.md'),'# Bounded toolkit pilot\n\n'+protocol.policy+'\n\n'+rows.map(r=>`- ${r.client}/${r.arm}: ${r.launched?'launched; '+r.rounds.length+' recorded stages; complete='+r.complete:r.reason}`).join('\n')+'\n\nRaw client usage, retained video copies, model/version and failures are in results.json. No taste winner or token-saving claim is assigned automatically.\n');
console.log(JSON.stringify({event:'pilot-finished',out}));
