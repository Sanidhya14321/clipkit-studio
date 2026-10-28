import {useEffect,useState} from 'react';
type Candidate={id:string;name:string;status:string;clock:string;cost:string;mechanism:string;preview:string;reason:string};
export function ComponentDiscovery({op,sceneId='film'}:{op:(name:string,args?:Record<string,unknown>)=>Promise<any>;sceneId?:string}){
  const [query,setQuery]=useState(''),[results,setResults]=useState<Candidate[]>([]),[status,setStatus]=useState(''),[error,setError]=useState(''),[busy,setBusy]=useState(false);
  useEffect(()=>{let active=true;const timer=setTimeout(()=>{setBusy(true);void op('component_search',{query,limit:8}).then(value=>{if(active){setResults(value.results);setError('')}}).catch(e=>{if(active)setError(e.message)}).finally(()=>{if(active)setBusy(false)})},180);return()=>{active=false;clearTimeout(timer)}},[query,op]);
  const add=async(id:string)=>{setBusy(true);setError('');try{const result=await op('component_add',{id,sceneId});setStatus(result.status==='ready'?`Adapter added to ${sceneId}. Your agent can now compose it.`:'Pinned source fetched. Your agent can adapt its clock and inputs for this film.')}catch(e){setError((e as Error).message)}finally{setBusy(false)}};
  return <div className="panel-scroll" style={{padding:16}}>
    <h3>Find the movement</h3><p className="muted">Search by action: text reflow, media aperture, speed, glass. Your agent chooses how it belongs in the film.</p>
    <input aria-label="Search web effects" placeholder="What should move?" value={query} onChange={e=>setQuery(e.target.value)} style={{width:'100%',marginBottom:16}}/>
    {busy&&<p role="status">Loading effects…</p>}{error&&<p role="alert">{error}</p>}{status&&<p role="status">{status}</p>}
    {!busy&&!results.length&&<p>No matching effect. Try a visual action or material.</p>}
    {results.map(effect=><article key={effect.id} style={{border:'1px solid #353535',borderRadius:12,padding:14,marginBottom:12}}>
      <div style={{display:'flex',justifyContent:'space-between',gap:10}}><strong>{effect.name}</strong><span className="muted">{effect.status==='ready'?'Ready':'Needs adaptation'}</span></div>
      <p>{effect.mechanism}</p>
      {effect.preview.startsWith('/')&&effect.preview.endsWith('.mp4')?<video src={effect.preview} muted loop controls preload="none" style={{width:'100%',maxHeight:220,background:'#121212'}} aria-label={`${effect.name} moving preview`}/>:<a href={effect.preview} target="_blank" rel="noreferrer">See source preview ↗</a>}
      <p className="muted">{effect.clock} · {effect.cost} render cost</p>
      <button disabled={busy} onClick={()=>void add(effect.id)}>{effect.status==='ready'?'Add adapter to film':'Fetch source for film'}</button>
    </article>)}
  </div>;
}
