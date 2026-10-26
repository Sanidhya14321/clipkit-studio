import {readFile} from 'node:fs/promises';
import {basename} from 'node:path';
import type {Word} from '../../core/src/index.ts';

// Contract: https://docs.fish.audio/api-reference/openapi.json
export function parseTranscript(value: unknown): {text:string;words:Word[];wordTiming:boolean} {
  const data=value as {text?:unknown;segments?:unknown};
  if(!data || typeof data.text!=='string') throw new Error('Fish returned an invalid transcript');
  const segments=Array.isArray(data.segments)? data.segments as Array<Record<string,unknown>>:[];
  const valid=segments.every(w=>typeof w.text==='string'&&typeof w.start==='number'&&Number.isFinite(w.start)&&w.start>=0&&typeof w.end==='number'&&Number.isFinite(w.end)&&w.end>w.start);
  // ASR segments may be phrases. Never fabricate individual word boundaries.
  const wordTiming=valid && segments.length>0 && segments.every((w,i)=>Boolean(String(w.text).trim())&&!/\s/.test(String(w.text).trim())&&(i===0||Number(w.start)>=Number(segments[i-1].end)));
  return {text:data.text, words:wordTiming?segments.map(w=>({text:String(w.text).trim(),start:Number(w.start),end:Number(w.end)})):[],wordTiming};
}
export class FishClient {
  constructor(private token=process.env.FISH_API_KEY, private transport:typeof fetch=fetch) {}
  get enabled(){return Boolean(this.token);}
  private async request(path:string,init:RequestInit={},model?:string){
    if(!this.token) throw new Error('FISH_API_KEY is missing. Add it to .env and restart the studio.');
    let response:Response;
    try {response=await this.transport(`https://api.fish.audio${path}`,{...init,signal:AbortSignal.timeout(180000),headers:{...init.headers,Authorization:`Bearer ${this.token}`,...(model?{model}:{})}});}catch{throw new Error('Fish request failed. Check your connection and retry.');}
    if(!response.ok) throw new Error(`Fish ${response.status}: ${response.status===401?'check FISH_API_KEY':response.status===402?'insufficient credits':response.status===429?'rate limit; retry shortly':'provider rejected request; check input and retry'}`);
    return response;
  }
  async asr(path:string){
    const body=new FormData();body.append('audio',new Blob([new Uint8Array(await readFile(path))]),basename(path));body.append('ignore_timestamps','false');
    return parseTranscript(await (await this.request('/v1/asr',{method:'POST',body},'transcribe-1-pro')).json());
  }
  async tts(text:string,referenceId:string,speed=1){
    if(!text.trim())throw new Error('Add narration text first');
    if(!referenceId)throw new Error('Set FISH_VOICE_ID or choose a Fish voice');
    if(!Number.isFinite(speed)||speed<0.5||speed>2)throw new Error('Voice speed must be between 0.5 and 2');
    return Buffer.from(await (await this.request('/v1/tts',{method:'POST',headers:{'Content-Type':'application/json'},body:JSON.stringify({text,reference_id:referenceId,format:'wav',prosody:{speed}})},process.env.FISH_TTS_MODEL||'s2-pro')).arrayBuffer());
  }
  async clone(path:string,title='My recorded voice'){
    const body=new FormData();for(const [k,v] of Object.entries({type:'tts',train_mode:'fast',visibility:'private',title,enhance_audio_quality:'true'}))body.append(k,v);
    body.append('voices',new Blob([new Uint8Array(await readFile(path))]),basename(path));
    const data=await (await this.request('/model',{method:'POST',body})).json() as {id?:string;_id?:string;state?:string;status?:string};
    const id=data.id??data._id;if(!id)throw new Error('Fish clone response did not contain a model ID');
    return {id,status:data.state??data.status??'unknown'};
  }
  async model(id:string){return await (await this.request(`/model/${encodeURIComponent(id)}`)).json() as Record<string,unknown>;}
}
