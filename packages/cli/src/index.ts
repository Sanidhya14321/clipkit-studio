#!/usr/bin/env node
import {readFile,writeFile,mkdir,access} from 'node:fs/promises';
import {resolve,join,dirname} from 'node:path';
import {fileURLToPath} from 'node:url';
import {execFileSync} from 'node:child_process';
import {createHash} from 'node:crypto';
import {createServer as createProbe} from 'node:net';
import {spawn} from 'node:child_process';
import {createRequire} from 'node:module';
import {ProjectStore} from '../../core/src/store.ts';
import {id} from '../../core/src/index.ts';
import {serveMcp} from '../../mcp/src/index.ts';
import {startServer} from '../../server/src/index.ts';
import {agentInstructions} from '../../server/src/runner.ts';
import {MediaService} from '../../server/src/media.ts';
import {buildAssets} from '../../assets/src/build.ts';

const cliPath=fileURLToPath(import.meta.url);
const packaged=cliPath.endsWith('.js');
const installRoot=packaged?resolve(dirname(cliPath),'..'):resolve(dirname(cliPath),'../../..');
let args=process.argv.slice(2);const rootIndex=args.indexOf('--root');const root=resolve(rootIndex>=0?args.splice(rootIndex,2)[1]:process.env.CLIPKIT_ROOT??process.cwd());
const flags:Record<string,string|boolean>={};args=args.filter((a,i,all)=>{if(a.startsWith('--')){const key=a.slice(2);flags[key]=all[i+1]&&!all[i+1].startsWith('--')?all[i+1]:true;return false;}return i===0||!all[i-1].startsWith('--')||all[i-1]==='--summary'||all[i-1]==='--open'||all[i-1]==='--compact';});
try{process.loadEnvFile(join(root,'.env'));}catch{/* Optional local provider credentials. */}
async function connection(){
  if(process.env.CLIPKIT_URL&&process.env.CLIPKIT_TOKEN)return {url:process.env.CLIPKIT_URL,token:process.env.CLIPKIT_TOKEN};
  try{return JSON.parse(await readFile(join(root,'.clipkit','server.json'),'utf8')) as {url:string;token:string};}catch{throw new Error('Studio is not running. Start it with ck studio.');}
}
async function invoke(name:string,data:Record<string,unknown>={}){
  const {url,token}=await connection();if(process.env.CLIPKIT_TURN&&typeof data.requestId==='string')data={...data,requestId:process.env.CLIPKIT_TURN+':'+data.requestId};let response:Response;
  try{response=await fetch(`${url}/api/op/${name}`,{method:'POST',headers:{'Content-Type':'application/json','X-ClipKit-Token':token,...(process.env.CLIPKIT_LEASE?{'X-ClipKit-Lease':process.env.CLIPKIT_LEASE,'X-ClipKit-Turn':process.env.CLIPKIT_TURN??''}:{})},body:JSON.stringify(data),signal:AbortSignal.timeout(600000)});}catch{throw new Error('Cannot reach the studio. Run ck studio and retry.');}
  const body=await response.json() as Record<string,unknown>;if(!response.ok)throw new Error(JSON.stringify(body));return body;
}
const output=(value:unknown)=>{const serialized=JSON.stringify(value,null,2);process.stdout.write(Buffer.byteLength(serialized)>20000?JSON.stringify({error:'CONTEXT_TOO_LARGE',message:'Use a smaller page, one layer, or a shorter range.'})+'\n':serialized+'\n');};
async function managedText(path:string,start:string,end:string,content:string){let old='';try{old=await readFile(path,'utf8');}catch{}const block=`${start}\n${content.trim()}\n${end}`;const first=old.indexOf(start),last=old.indexOf(end);const next=first>=0&&last>=first?old.slice(0,first)+block+old.slice(last+end.length):`${old.trimEnd()}${old.trim()?'\n\n':''}${block}\n`;await mkdir(dirname(path),{recursive:true});await writeFile(path,next);}
async function initialize(){
  const commandArgs=packaged?[cliPath,'mcp']:['--import',fileURLToPath(import.meta.resolve('tsx')),cliPath,'mcp'];
  const mcpConfig={command:process.execPath,args:commandArgs,env:{CLIPKIT_ROOT:root}};
  const optional=async(path:string)=>{try{return await readFile(path,'utf8')}catch(e){if((e as NodeJS.ErrnoException).code!=='ENOENT')throw e;return ''}};
  const priorCodex=await optional(join(root,'.codex/config.toml'));const priorAgents=await optional(join(root,'AGENTS.md'));
  if(/\[mcp_servers\.clipkit(?:\]|\.)/.test(priorCodex)&&!priorCodex.includes('# clipkit:start'))throw new Error('An unmanaged ClipKit server exists in .codex/config.toml; merge or rename it before ck init. Existing files were preserved.');
  let mcp:Record<string,any>={};const existingMcp=await optional(join(root,'.mcp.json'));if(existingMcp){try{mcp=JSON.parse(existingMcp)}catch{throw new Error('Existing .mcp.json is not valid JSON; repair it before init.')}}
  let receipt:any;try{receipt=JSON.parse(await optional(join(root,'.clipkit/adapters.json')))}catch{}
  const existing=mcp.mcpServers?.clipkit;const owned=existing&&(JSON.stringify(existing)===JSON.stringify(mcpConfig)||JSON.stringify(existing)===JSON.stringify(receipt?.mcpConfig));
  if(existing&&!owned)throw new Error('An unmanaged clipkit entry exists in .mcp.json; merge or rename it before ck init. Existing files were preserved.');
  const cursorPath=join(root,'.cursor/rules/clipkit.mdc');const priorCursor=await optional(cursorPath);if(priorCursor&&!priorCursor.includes('<!-- clipkit:start -->'))throw new Error('An unmanaged .cursor/rules/clipkit.mdc exists; rename it before ck init. Existing files were preserved.');
  await mkdir(join(root,'scenes'),{recursive:true});await mkdir(join(root,'.clipkit'),{recursive:true});try{await access(join(root,'.clipkit','project.json'));}catch{await new ProjectStore(root).load();}
  await managedText(join(root,'AGENTS.md'),'<!-- clipkit:start -->','<!-- clipkit:end -->',agentInstructions);
  mcp.mcpServers={...mcp.mcpServers,clipkit:mcpConfig};await writeFile(join(root,'.mcp.json'),JSON.stringify(mcp,null,2)+'\n');
  const config=`[mcp_servers.clipkit]\ncommand = ${JSON.stringify(process.execPath)}\nargs = ${JSON.stringify(commandArgs)}\ndefault_tools_approval_mode = "approve"\n[mcp_servers.clipkit.env]\nCLIPKIT_ROOT = ${JSON.stringify(root)}`;
  await managedText(join(root,'.codex','config.toml'),'# clipkit:start','# clipkit:end',config);
  const skill=join(root,'.claude','skills','clipkit');await mkdir(skill,{recursive:true});await writeFile(join(skill,'SKILL.md'),`---\nname: clipkit\ndescription: Edit the local ClipKit video timeline with precise revisioned patches.\n---\n${agentInstructions}`);
  if(!priorCursor.startsWith('---')){await mkdir(dirname(cursorPath),{recursive:true});await writeFile(cursorPath,'---\ndescription: Edit the shared ClipKit timeline through revisioned MCP tools.\nalwaysApply: true\n---\n'+priorCursor);}
  await managedText(cursorPath,'<!-- clipkit:start -->','<!-- clipkit:end -->',agentInstructions);
  await writeFile(join(root,'.clipkit/adapters.json'),JSON.stringify({mcpConfig}));
  await mkdir(join(root,'assets'),{recursive:true});try{await writeFile(join(root,'assets/README.md'),'# Project assets\n\nAdd media or reviewed pack sources here, then select Library → Rescan. Raw source files are never automatically executed.\n',{flag:'wx'});}catch(error){if((error as NodeJS.ErrnoException).code!=='EEXIST')throw error;}
  await doctor();
  output({initialized:true,root,adapters:['Codex','Claude Code','Cursor','MCP'],next:'ck studio',note:'Agent CLIs must be installed and signed in. Fish Audio is optional until voice generation/transcription.'});
}
async function studio(){
  const port=Number(flags.port??process.env.CLIPKIT_PORT??4310);
  const publicDir=join(installRoot,'public');const studioDir=join(installRoot,'dist','studio');
  const server=await startServer({root,port,cliPath,publicDir,studioDir,renderEntry:packaged?join(installRoot,'runtime','packages','render','src','entry.tsx'):undefined});
  let vite:ReturnType<typeof spawn>|undefined;let browserUrl=server.url;
  if(!packaged&&!flags['no-vite']){const require=createRequire(import.meta.url);const viteEntry=join(dirname(require.resolve('vite/package.json')),'bin','vite.js');vite=spawn(process.execPath,[viteEntry,'--config',join(installRoot,'packages/studio/vite.config.ts')],{cwd:installRoot,stdio:'inherit',env:{...process.env,CLIPKIT_SERVER_PORT:String(port)}});browserUrl='http://127.0.0.1:5173';}
  console.log(`\n  ClipKit Studio\n  ${browserUrl}\n  Project: ${root}\n  Ctrl+C to stop.\n`);
  if(flags.open)spawn(process.platform==='darwin'?'open':process.platform==='win32'?'explorer':'xdg-open',[browserUrl],{stdio:'ignore'}).on('error',()=>{});
  let closing=false;const close=async()=>{if(closing)return;closing=true;vite?.kill('SIGTERM');await server.close();process.exit(0);};process.on('SIGINT',()=>void close());process.on('SIGTERM',()=>void close());
}
async function doctor(){
  const media=new MediaService({root,getProject:()=>({}) as any,getTranscripts:()=>({}),setTranscript:()=>{},commit:async()=>{},job:()=>({}) as any});
  const result=await media.doctor();const clients:Record<string,string>={};for(const name of ['codex','claude']){clients[name]=await new Promise<string>(res=>{const child=spawn(name,['--version'],{stdio:['ignore','pipe','ignore']});let value='';child.stdout.on('data',d=>value+=d);child.on('error',()=>res('not installed'));child.on('close',code=>res(code===0?value.trim():'unavailable'));});}output({node:process.version,...result,clients,notes:['Fish Audio requires FISH_API_KEY in .env.','Use codex login / claude auth login for real agent turns.','With a Fish key configured, media imports automatically schedule transcription uploads.']});
}
async function main(){
  const [command,sub,...rest]=args;
  if(!command){console.log('ClipKit Studio\n\n  ck init     Connect your coding agents\n  ck studio   Open the local video editor\n  ck doctor   Check media and voice dependencies\n\nOther commands: assets, search, schema, project, status, footage, comments, voice, history, export, demo, mcp.');return;}
  if(command==='init')return initialize();
  if(command==='studio')return studio();
  if(command==='doctor')return doctor();
  if(command==='ensure'){try{const c=await connection();const r=await fetch(c.url+'/api/bootstrap',{signal:AbortSignal.timeout(1000)});if(r.ok)return output({url:c.url,running:true});}catch{}const port=flags.port?Number(flags.port):await new Promise<number>(done=>{const probe=createProbe();probe.listen(0,'127.0.0.1',()=>{const p=(probe.address() as any).port;probe.close(()=>done(p))})});const commandArgs=packaged?[cliPath,'studio','--no-vite','--port',String(port),'--root',root]:['--import',fileURLToPath(import.meta.resolve('tsx')),cliPath,'studio','--no-vite','--port',String(port),'--root',root];const child=spawn(process.execPath,commandArgs,{cwd:root,detached:true,stdio:'ignore',env:{...process.env}});child.unref();for(let i=0;i<100;i++){await new Promise(r=>setTimeout(r,150));try{const c=await connection();if((await fetch(c.url+'/api/bootstrap')).ok)return output({url:c.url,running:true,started:true})}catch{}}throw Error('Studio did not start. Run ck studio to see diagnostics.');}
  if(command==='motion')return output(await invoke(sub==='read'?'motion_read':'motion_search',sub==='read'?{id:rest[0]}:{query:rest.join(' ')}));
  if(command==='scene'){if(sub==='read')return output(await invoke('scene_read',{itemId:rest[0]}));if(sub==='status')return output(await invoke('scene_status'));if(sub==='draft')return output(await invoke('scene_draft',JSON.parse(await readFile(resolve(rest[0]),'utf8'))));}
  if(command==='reel'&&sub==='read')return output(await invoke('reel_read',rest[0]?{chapterId:rest[0]}:{}));
  if(command==='reel'&&sub==='score')return output(await invoke('reel_chapter_score_set',{chapterId:rest[0],score:JSON.parse(await readFile(resolve(rest[1]),'utf8'))}));
  if(command==='reel'&&sub==='plan')return output(await invoke('reel_plan_set',{plan:JSON.parse(await readFile(resolve(rest[0]),'utf8'))}));
  if(command==='compat'&&sub==='install'){const file=resolve(rest[0]);const expected=JSON.parse(await readFile(join(installRoot,packaged?'runtime/packages/scenes/compatibility.json':'packages/scenes/compatibility.json'),'utf8'));if(createHash('sha256').update(await readFile(file)).digest('hex')!==expected.sha256)throw Error('Compatibility archive checksum does not match the pinned release');const entries=execFileSync('tar',['-tzf',file],{encoding:'utf8'}).trim().split('\n');if(entries.some(e=>!e.startsWith('0.2.0/')&&e!=='0.2.0'||e.split('/').includes('..')))throw Error('Invalid compatibility archive paths');const destination=join(root,'.clipkit/compat');await mkdir(destination,{recursive:true});execFileSync('tar',['-xzf',file,'-C',destination]);return output({installed:'0.2.0',restart:true});}
  if(command==='library'){if(sub==='list')return output(await invoke('library_list',{q:flags.q,kind:flags.kind,origin:flags.origin,offset:Number(flags.offset??0),limit:Number(flags.limit??20)}));if(sub==='inspect')return output(await invoke('library_inspect',{assetId:rest[0]}));if(sub==='rescan')return output(await invoke('library_rescan'));if(sub==='import')return output(await invoke('library_import',{path:resolve(rest[0])}));if(sub==='place')return output(await invoke('library_place',{assetId:rest[0],at:Number(flags.at??0),trackId:flags.track}));}
  if(command==='assets'&&sub==='build'){await buildAssets(installRoot,join(root,'.clipkit','assets'));const {refreshPacks}=await import('../../assets/src/packs.ts');return output(await refreshPacks(root));}
  if(command==='assets'&&sub==='install')return output(await invoke('assets_install',{path:resolve(rest[0]),trust:flags.trust===true}));
  if(command==='mcp'){await serveMcp(invoke);return;}
  if(command==='assets'&&sub==='list')return output(await invoke('assets_list',{kind:typeof flags.kind==='string'?flags.kind:undefined,tags:typeof flags.tags==='string'?flags.tags.split(','):undefined,mood:typeof flags.mood==='string'?flags.mood:undefined,q:typeof flags.q==='string'?flags.q:undefined,offset:flags.offset?Number(flags.offset):0,limit:flags.limit?Number(flags.limit):5}));
  if(command==='assets'&&sub==='card')return output(await invoke('asset_card',{id:rest[0]}));
  if(command==='search')return output(await invoke('assets_search',{query:[sub,...rest].filter(Boolean).join(' '),limit:flags.limit?Number(flags.limit):5}));
  if(command==='schema')return output(await invoke('asset_schema',{clipId:sub}));
  if(command==='project'&&sub==='get')return output(await invoke('project_get',{layerId:flags.layer,offset:flags.offset?Number(flags.offset):0,limit:flags.limit?Number(flags.limit):15}));
  if(command==='project'&&sub==='patch'){const raw=rest[0]&&rest[0]!=='-'?await readFile(resolve(rest[0]),'utf8'):await new Promise<string>(resolve=>{let s='';process.stdin.on('data',d=>s+=d);process.stdin.on('end',()=>resolve(s));});return output(await invoke('project_patch',JSON.parse(raw)));}
  if(command==='status'){if(sub==='working')return output(await invoke('status_working',{owner:'external-agent',label:rest.join(' ')||'Editing'}));if(sub==='idle')return output(await invoke('status_idle',{leaseToken:flags.lease}));return output(await invoke('project_get'));}
  if(command==='footage'){if(sub==='add')return output(await invoke('footage_add',{path:resolve(rest[0])}));if(sub==='transcribe')return output(await invoke('footage_transcribe',{sourceId:rest[0]}));if(sub==='candidates')return output(await invoke('footage_candidates',{sourceId:rest[0]}));if(sub==='view'){const rangeIndex=process.argv.indexOf('--range');return output(await invoke('footage_view',{sourceId:rest[0],start:rangeIndex>=0?Number(process.argv[rangeIndex+1]):0,end:rangeIndex>=0?Number(process.argv[rangeIndex+2]):5}));}if(sub==='pack'){const rangeIndex=process.argv.indexOf('--range');return output(await invoke('footage_pack',{sourceId:rest[0],start:rangeIndex>=0?Number(process.argv[rangeIndex+1]):0,end:rangeIndex>=0?Number(process.argv[rangeIndex+2]):36000}));}}
  if(command==='comments'){if(sub==='list')return output(await invoke('comments_list',{open:Boolean(flags.open),limit:flags.limit?Number(flags.limit):10}));if(sub==='resolve')return output(await invoke('comments_resolve',{commentId:rest[0],reply:rest.slice(1).join(' ')}));if(sub==='reply')return output(await invoke('comments_reply',{commentId:rest[0],text:rest.slice(1).join(' ')}));}
  if(command==='voice'){if(sub==='script')return output(await invoke('voice_script',{text:rest.join(' ')}));if(sub==='speak')return output(await invoke('voice_generate',{slotId:rest[0],voiceRef:flags.voice}));if(sub==='mode')return output(await invoke('voice_mode',{slotId:rest[0],mode:rest[1]}));if(sub==='clone')return output(await invoke('voice_clone',{sampleSourceId:rest[0],consent:flags.consent===true}));}
  if(command==='history'){if(sub==='list')return output(await invoke('history_list'));if(sub==='undo')return output(await invoke('history_undo'));if(sub==='redo')return output(await invoke('history_redo'));if(sub==='revert')return output(await invoke(rest[0]?'history_revert':'history_revert_turn',rest[0]?{rev:Number(rest[0])}:{}));}
  if(command==='export')return output(await invoke('export_start',{preset:flags.preset,quality:flags.quality}));
  if(command==='demo'&&sub==='save')return output(await invoke('demo_save'));
  if(command==='demo'&&sub==='reset')return output(await invoke('demo_reset',{graphics:flags.graphics===true}));
  if(command==='agent')return output(await invoke('agent_run',{client:flags.client==='claude'?'claude':'codex',prompt:[sub,...rest].join(' ')}));
  if(command==='op')return output(await invoke(sub,rest[0]?JSON.parse(rest[0]):{}));
  throw new Error(`Unknown command: ${args.join(' ')}. Run ck for help.`);
}
main().catch(e=>{console.error((e as Error).message);process.exitCode=1;});
