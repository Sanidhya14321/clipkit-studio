import {mkdir,readFile,writeFile,stat,cp,readdir} from 'node:fs/promises';
import path from 'node:path';
import {runAgentMatrix,type BenchObservation} from '../packages/debug/src/agent-bench';
import {scenarios} from '../packages/debug/src/config';
import {sha,redact} from '../packages/debug/src/trace';
const root=process.cwd(),out=path.resolve(process.argv[2]??'.clipkit/video-comparison-20261028');
if(await stat(path.join(out,'protocol.json')).then(()=>true).catch(()=>false))throw Error('Existing comparison preserved; choose a fresh output folder');
const raw=JSON.parse(await readFile('bench/video-comparison.json','utf8')),spec=raw.comparison;
const scenario:any=scenarios.find(s=>s.id==='S1');
scenario.prompt=`Create a 15 second silent launch teaser. Exact opening headline: "${spec.title}". Supporting copy: "${spec.supportingText}". Final call to action: "${spec.cta}". Resolution ${spec.width}x${spec.height}, ${spec.fps} fps, 16:9. Show the opening headline in the first 2 seconds and the CTA in the final 3 seconds. Audience: beginners. Initial pacing: balanced. Look: editorial, with original frame-driven typography and vector/shape motion. Background ${spec.palette[0]}, foreground ${spec.palette[1]}, accent ${spec.palette[2]}. Use ${spec.font}; the licensed font file is fixtures/brand-inter.ttf. Keep text within an 8 percent safe area on each edge, with no overlapping or clipped type. No audio stream, music, narration, captions, footage, external downloads or new dependencies. Keep all existing fixtures unchanged. Deliver an actual playable result.mp4 and editable source. result.mp4 must be a regular file containing video bytes inside /work, never a symlink to temporary storage. Do not ask questions. Work independently without another model or agent.`;
scenario.scriptedAnswers={audience:'beginners',pacing:'balanced',length:15,look:'editorial',voice:'none',captions:'off',aspect:'16:9',width:1920,height:1080,fps:30,font:'Inter',palette:spec.palette.join(', '),safeArea:'8 percent on each edge',audio:'none'};
scenario.feedback=[{round:2,text:'Change only the opening headline font size to 64 pixels. Preserve its exact copy and all unrelated design and timing. Render the updated video.'},{round:3,text:'Make the pacing fast while keeping the film exactly 15 seconds, preserving all exact copy and safe areas, and keeping the final CTA readable for the final 3 seconds. Render the final version.'}];
await mkdir(out,{recursive:true});
const protocol={...raw,fixtures:path.resolve(raw.fixtures),prompt:scenario.prompt,answers:scenario.scriptedAnswers,feedback:scenario.feedback,policy:'Alternating arm order; fresh namespace/home/workspace/private cache each run. Same client/model/render facts/active budget/feedback. Only original/local verified assets. Provider failures stop subsequent launches. All round outputs and reported cumulative usage preserved; no selection by visual quality.',fixtureManifestHash:sha(await readFile(path.join(raw.fixtures,'manifest.json'),'utf8'))};
await writeFile(path.join(out,'protocol.json'),JSON.stringify(protocol,null,2));
const rows:any[]=[];let blocked:string|undefined;
for(let repetition=0;repetition<raw.repetitions&&!blocked;repetition++)for(const arm of spec.order[repetition]){
 if(await stat(path.join(out,'STOP_AFTER_RUN')).then(()=>true).catch(()=>false)){blocked='Operator stopped further launches';break;}
 const folder=path.join(out,`rep-${repetition+1}-${arm}`),rounds:any[]=[];
 const prepared=async(c:BenchObservation)=>{
  await writeFile(path.join(c.workspace,'BRIEF.md'),scenario.prompt+'\n');
  if(c.server){const fields:any={goal:scenario.prompt,audience:'beginners',length:15,aspect:'16:9',pacing:'balanced',look:'editorial',voiceMode:'none',captions:false,music:'none',sfxLevel:'none',cta:spec.cta,brand:{colors:spec.palette,fonts:['Inter','Inter']},mustInclude:[spec.title,spec.supportingText,spec.cta],avoid:['External downloads','Clipped or overlapping text','Any audio stream'],licenseMode:'commercial-safe'};await c.server.operate('brief_propose',{prompt:scenario.prompt,brief:{fields:Object.fromEntries(Object.entries(fields).map(([key,value])=>[key,{value,source:'user',confidence:1}]))}});await c.server.operate('brief_confirm',{rev:c.server.store.project.brief.rev});}
 };
 const observed=async(c:BenchObservation)=>{
  const destination=path.join(folder,'rounds','round-'+c.round);await mkdir(destination,{recursive:true});
  let video=false;try{await cp(path.join(c.workspace,'result.mp4'),path.join(destination,'result.mp4'),{dereference:true});video=(await stat(path.join(destination,'result.mp4'))).isFile();}catch{}
  const sources:Record<string,string>={};let bytes=0;
  async function walk(dir:string){for(const item of await readdir(dir,{withFileTypes:true})){if(item.name.startsWith('.')||['node_modules','fixtures','public','media'].includes(item.name)||item.isSymbolicLink())continue;const file=path.join(dir,item.name);if(item.isDirectory())await walk(file);else if(/\.(tsx?|jsx?|[cm]js|py|sh|svg|html|json|md|css|srt|txt|ya?ml|toml)$/.test(item.name)){const size=(await stat(file)).size;if(size<200000&&bytes+size<2000000){bytes+=size;sources[path.relative(c.workspace,file)]=await readFile(file,'utf8');}}}}
  await walk(c.workspace);await writeFile(path.join(destination,'authored-sources.json'),JSON.stringify(redact(sources,{synthetic:true}),null,2));
  if(c.server){await writeFile(path.join(destination,'project.json'),JSON.stringify(redact(c.server.store.project,{synthetic:true}),null,2));await writeFile(path.join(destination,'retrieval-estimate.json'),JSON.stringify(c.server.state().telemetry.retrieval,null,2));}
  const receipt={round:c.round,video,elapsedMs:c.elapsedMs,usage:c.usage,sourceBytes:bytes};rounds.push(receipt);await writeFile(path.join(destination,'receipt.json'),JSON.stringify(receipt,null,2));
  process.stdout.write(JSON.stringify({event:'round-completed',repetition,arm,...receipt})+'\n');
 };
 process.stdout.write(JSON.stringify({event:'run-start',repetition,arm,timeoutMs:raw.timeoutMs})+'\n');
 await runAgentMatrix({...raw,comparison:undefined,arms:[arm],repetitions:1,fixtures:protocol.fixtures},{root,out:folder,onPrepared:prepared,onRound:observed,onProgress:e=>process.stdout.write(JSON.stringify(e)+'\n')});
 const row=JSON.parse((await readFile(path.join(folder,'results.jsonl'),'utf8')).trim());rows.push({...row,repetition,sourceRecord:path.relative(out,path.join(folder,'results.jsonl')),rounds});await writeFile(path.join(out,'comparison-runs.json'),JSON.stringify(rows,null,2));
 if(row.errors.some((e:any)=>/hit your usage limit|rate_limit_exceeded|insufficient_quota/i.test(e.message??'')))blocked='Provider quota exhausted; no further model launches';
 if(blocked)break;
}
await writeFile(path.join(out,'completion.json'),JSON.stringify({plannedRuns:raw.repetitions*2,recordedRuns:rows.length,blocked:blocked??null,completeSessions:rows.filter(r=>r.usage.complete).length,actualVideoArtifacts:rows.filter(r=>r.artifacts.some((f:string)=>f.endsWith('.mp4'))).length},null,2));
console.log(JSON.stringify({event:'comparison-finished',out,runs:rows.length,blocked}));
