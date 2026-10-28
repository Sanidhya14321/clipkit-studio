import {readdir,readFile,writeFile,mkdir,stat} from 'node:fs/promises';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {redact,registerSecrets,sha} from '../packages/debug/src/trace';
const root=path.resolve('.clipkit/video-comparison-20261028'),target=path.join(root,'rep-3-B');
const protocol=JSON.parse(await readFile(path.join(root,'protocol.json'),'utf8'));
for(const name of await readdir(tmpdir())){
 if(!name.startsWith('ck-e2e-'))continue;const base=path.join(tmpdir(),name),workspace=path.join(base,'workspace');
 try{if(await readFile(path.join(workspace,'BRIEF.md'),'utf8')!==protocol.prompt+'\n')continue;}catch{continue;}
 try{const descriptor=JSON.parse(await readFile(path.join(workspace,'.clipkit/server.json'),'utf8'));registerSecrets([descriptor.token]);}catch{}
 const destination=path.join(target,'recovered-workspace');await mkdir(destination,{recursive:true});let files=0;
 async function walk(folder:string){for(const item of await readdir(folder,{withFileTypes:true})){if(item.isSymbolicLink()||['node_modules','fixtures','public','media','server.json','.mcp.json'].includes(item.name))continue;const file=path.join(folder,item.name),relative=path.relative(workspace,file);if(item.isDirectory()){if(/(?:render-cache|layer-cache|cache|proxies)/.test(item.name))continue;await walk(file);}else if(/\.(?:jsonl?|tsx?|jsx?|[cm]js|py|css|md|srt|ya?ml|svg)$/.test(item.name)){
 const size=(await stat(file)).size;if(size>4*1024**2)continue;const text=await readFile(file,'utf8');let safe:string;
 if(item.name.endsWith('.jsonl'))safe=text.split('\n').filter(Boolean).map(line=>{try{return JSON.stringify(redact(JSON.parse(line),{synthetic:true}));}catch{return JSON.stringify({corruptLineSHA256:sha(line),notice:'Incomplete line preserved by hash after storage failure'});}}).join('\n')+'\n';
 else if(item.name.endsWith('.json'))try{safe=JSON.stringify(redact(JSON.parse(text),{synthetic:true}),null,2);}catch{safe=JSON.stringify({corruptFileSHA256:sha(text),notice:'Storage write failure'});}
 else safe=redact(text,{synthetic:true});const out=path.join(destination,relative);await mkdir(path.dirname(out),{recursive:true});await writeFile(out,safe);files++;
 }}}
 await walk(workspace);await writeFile(path.join(target,'recovery.json'),JSON.stringify({files,credentialsCopied:false,originalFixturesCopied:false,notice:'Read-only recovery after harness storage crash; not a successful agent export.'},null,2));
}
const rounds=[];for(let i=0;i<3;i++)try{rounds.push(JSON.parse(await readFile(path.join(target,'rounds','round-'+i,'receipt.json'),'utf8')));}catch{}
const traceDir=path.join(target,'.clipkit/trace'),id=(await readdir(traceDir)).find(name=>name.startsWith('codex-'))!;let environment:any={};try{environment=JSON.parse(await readFile(path.join(traceDir,id,'environment.json'),'utf8'));}catch{}
const last=rounds.at(-1)?.usage??{source:'agent-reported',complete:false,input:null,output:null,cached:null,costUSD:null};
const record={runId:id,scenario:'S1',arm:'B',agent:'codex',agentVersion:environment.version??'codex-cli 0.162.1',model:protocol.model,repetition:0,valid:false,passed:false,usage:{...last,complete:false},wallMs:null,setupMs:null,totalMs:null,firstPreviewMs:null,feedbackRounds:rounds.length?rounds.length-1:0,toolCalls:null,clientExits:rounds.map((round:any)=>({round:round.round,exit:0,signal:null,derivedFrom:'onRound callback after successful child exit'})),errors:[{code:'E_ENV',message:'Harness interrupted by local write quota failure (errno -122); final child exit and remaining token usage were not recorded.'}],artifacts:[],derived:true,provenance:'Derived from preserved successful round callbacks, environment receipt and uncaught-error log. The missing original result record is not fabricated or overwritten.'};
await writeFile(path.join(target,'crash-record.json'),JSON.stringify(record,null,2));
const originals=(await readdir(root)).filter(name=>/^rep-/.test(name));let recorded=0;for(const name of originals)if(await stat(path.join(root,name,'results.jsonl')).then(()=>true).catch(()=>false))recorded++;
await writeFile(path.join(root,'completion.json'),JSON.stringify({plannedRuns:6,recordedOriginalRuns:recorded,derivedCrashRecords:1,interrupted:true,reason:'Local write quota failure (errno -122); no further model launches.',fullAcceptance:false},null,2));console.log(JSON.stringify({recordedOriginalRuns:recorded,derivedCrashRecords:1}));
