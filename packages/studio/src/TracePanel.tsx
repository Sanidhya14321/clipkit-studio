import React,{useEffect,useState,useRef} from 'react';
import jsonPatch from 'fast-json-patch';

type Props={op:(name:string,args?:any)=>Promise<any>;currentRun?:string;harness?:boolean;onVoice?:(text:string)=>void};
export function TracePanel({op,currentRun,harness,onVoice}:Props){
 const inspection=useRef(0);
 const [runs,setRuns]=useState<string[]>([]),[run,setRun]=useState(currentRun??'');
 const [events,setEvents]=useState<any[]>([]),[next,setNext]=useState<number|null>(null);
 const [replay,setReplay]=useState<any>(),[diff,setDiff]=useState<any[]>([]),[index,setIndex]=useState(0);
 const [error,setError]=useState(''),[voice,setVoice]=useState(''),[filter,setFilter]=useState(''),[bundle,setBundle]=useState('');
 async function invoke(name:string,args:any={}){
  try{setError('');return await op(name,args);}catch(error){setError((error as Error).message);}
 }
 async function loadEvents(offset=0){
  const result=await invoke('trace_events',{runId:run,offset});
  if(result){setEvents(result.events??[]);setNext(result.next??null);}
 }
 useEffect(()=>{void invoke('trace_list').then(result=>setRuns([...new Set([currentRun,...result?.items??[]].filter(Boolean))] as string[]));},[currentRun]);
 useEffect(()=>{inspection.current++;setReplay(undefined);setDiff([]);setBundle('');if(run)void loadEvents();},[run]);
 async function inspect(revision?:number){
  const request=++inspection.current;
  const result=await invoke('trace_replay',{runId:run,...revision===undefined?{}:{rev:revision}});
  if(request!==inspection.current||!result?.revisions?.length)return;setDiff([]);
  const position=result.revisions.indexOf(result.rev);setReplay(result);setIndex(position);
  if(position>0){const previous=await invoke('trace_replay',{runId:run,rev:result.revisions[position-1]});if(request===inspection.current&&previous?.project)setDiff(jsonPatch.compare(previous.project,result.project));}
  else setDiff([]);
 }
 async function copyRepro(){
  const result=await invoke('trace_repro',{runId:run});if(!result?.path)return;setBundle(result.path);
  try{await navigator.clipboard.writeText(result.path);}catch{setError('Bundle created. Select and copy the path below.');}
 }
 return <section aria-label="Flight recorder">
  <h3>Flight recorder</h3>
  <label>Run <select aria-label="Trace run" value={run} onChange={event=>setRun(event.target.value)}>{runs.map(id=><option key={id}>{id}</option>)}</select></label>
  <button onClick={()=>void loadEvents()}>Refresh trace events</button>
  <button onClick={()=>void copyRepro()}>Copy repro bundle path</button>
  <button onClick={()=>void inspect()}>Inspect recorded revisions</button>
  {bundle&&<p><code>{bundle}</code></p>}
  <input aria-label="Layer history filter" value={filter} onChange={event=>setFilter(event.target.value)} placeholder="Layer ID (current page)"/>
  {events.filter(event=>!filter||event.layerIds?.includes(filter)).map((event,n)=><article key={n} style={{borderLeft:event.resultCode?.startsWith('E_')?'3px solid #ff8c8c':'3px solid #4f7a68',padding:'6px 12px',margin:'6px 0'}}>
   <b>{event.event} · rev {String(event.rev??'?')}</b><span> · {event.traceId}</span>
   {event.resultCode&&<p>{event.resultCode}: {event.hint}</p>}<small>{event.operation} · {event.duration}ms</small>
  </article>)}
  {next!==null&&<button onClick={()=>void loadEvents(next)}>Next trace events</button>}
  {replay&&<>
   <p>Recorded revision {replay.rev} · {replay.hash}</p>
   <input aria-label="Recorded revision" type="range" min="0" max={replay.revisions.length-1} value={index} onChange={event=>void inspect(replay.revisions[Number(event.target.value)])}/>
   <details open><summary>Patch from previous recorded revision</summary><pre style={{maxHeight:220,overflow:'auto'}}>{JSON.stringify(diff,null,2)}</pre></details>
   <details><summary>Anonymized project at this revision</summary><pre style={{maxHeight:220,overflow:'auto'}}>{JSON.stringify(replay.project,null,2)}</pre></details>
  </>}
  {harness&&<form onSubmit={event=>{event.preventDefault();onVoice?.(voice);}}>
   <label>Simulated voice transcript<input aria-label="Simulated voice transcript" value={voice} onChange={event=>setVoice(event.target.value)}/></label>
   <button>Send simulated voice</button><p>Synthetic harness only. Calls the same local command/comment router; no microphone or speech provider.</p>
  </form>}
  {error&&<p role="alert">{error}</p>}
 </section>;
}
