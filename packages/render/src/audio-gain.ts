import {activeTake, type Item, type Project} from '../../core/src/index';
const clamp=(n:number)=>Math.max(0,Math.min(1,n));
const smooth=(n:number)=>{const t=clamp(n);return t*t*(3-2*t);};
/** Narration only ducks music while a selected, audible take occupies the timeline. */
export function narrationDuckingGain(project:Project,time:number,rampSeconds=.12):number {
 let envelope=0;
 for(const track of project.tracks)for(const voice of track.items){
  if(voice.type!=='voice'||voice.volume<=0)continue;
  const slot=project.voiceSlots.find(s=>s.id===voice.slotId);const take=slot?activeTake(slot):undefined;
  const source=take?project.sources[take.sourceId]:undefined;
  if(!take||!source)continue;
  const audibleDuration=Math.min(voice.dur,take.durationSec-voice.srcIn,source.durationSec-voice.srcIn);
  if(audibleDuration<=0)continue;
  const elapsed=time-voice.at;const remaining=voice.at+audibleDuration-time;
  envelope=Math.max(envelope,smooth(Math.min(elapsed,remaining)/Math.max(.001,rampSeconds)));
 }
 return 1-.75*envelope;
}
/** Thirty millisecond edge fades and optional narration-aware background ducking. */
export function itemAudioGain(item:Item,frame:number,fps:number,project:Project):number {
 const frames=Math.round(item.dur*fps);
 const fade=Math.max(0,Math.min(1,frame/(fps*.03),(frames-1-frame)/(fps*.03)));
 const duck=item.type==='audio'&&item.props.duck===true?narrationDuckingGain(project,item.at+frame/fps):1;
 return item.volume*fade*duck;
}
