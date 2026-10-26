import type {Word} from './index.ts';
export const byteLength=(value:unknown)=>new TextEncoder().encode(JSON.stringify(value)).byteLength;
/** Compact phrases plus a bounded preview of candidates. Cursor is a word index. */
export function packTranscript(words:Word[],options:{start:number;end:number;cursor?:number;maxBytes?:number;silences?:Array<{start:number;end:number}>}){
 const {start,end}=options,maxBytes=Math.min(12000,Math.max(1500,options.maxBytes??12000));
 const ranged=words.filter(w=>w.end>=start&&w.start<=end);const cursor=Math.max(0,Math.floor(options.cursor??0));
 const fillers=ranged.filter(w=>/^(um+|uh+|erm+|hmm+)[,.!?]?$/i.test(w.text));
 const silences=(options.silences??[]).filter(s=>s.end>=start&&s.start<=end);
 const result={range:{start,end},cursor,phrases:[] as Word[],candidates:{fillers:fillers.slice(0,8),silences:silences.slice(0,8),totalFillers:fillers.length,totalSilences:silences.length},nextCursor:null as number|null,totalWords:ranged.length,truncated:false};
 let i=cursor;for(;i<ranged.length;i++){
  const w=ranged[i];if(w.text.length>2000)throw new Error(`Transcript word ${i} is too large; inspect the source transcript`);
  const old=result.phrases.at(-1);const merge=old&&old.text.length+w.text.length<230&&w.start-old.end<.7;
  const oldText=old?.text,oldEnd=old?.end;
  if(merge){old.text+=' '+w.text;old.end=w.end;}else result.phrases.push({...w});
  if(byteLength(result)>maxBytes-100){if(merge){old.text=oldText!;old.end=oldEnd!;}else result.phrases.pop();break;}
 }
 if(i<ranged.length){result.nextCursor=i;result.truncated=true;}
 if(byteLength(result)>maxBytes)throw new Error('Transcript response exceeds its byte budget');return result;
}
