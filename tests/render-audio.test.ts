import {it,expect} from 'vitest';
import {emptyProject,type Item} from '../packages/core/src/index';
import {audibleVoiceWindows,voiceCrossfadeGain,itemAudioGain} from '../packages/render/src/audio-gain';
it('bounds trimmed active takes and crossfades overlapping voice slots at equal power',()=>{
 const project=emptyProject();for(const [id,at,srcIn]of [['a',0,1],['b',2,0]] as const){project.sources[id]={id,name:id,path:`${id}.wav`,kind:'audio',durationSec:4};project.voiceSlots.push({id,text:id,mode:'recorded',status:'ready',targetWindow:{start:at,end:at+4},takes:[{id,sourceId:id,mode:'recorded',durationSec:4,createdAt:''}],activeTakeByMode:{recorded:id}});project.tracks[3].items.push({id,type:'voice',slotId:id,at,dur:4,srcIn,volume:1,props:{}});}
 const [a,b]=project.tracks[3].items;expect(audibleVoiceWindows(project).map(w=>[w.start,w.end])).toEqual([[0,3],[2,6]]);
 const outgoing=voiceCrossfadeGain(project,a,2.5),incoming=voiceCrossfadeGain(project,b,2.5);expect(outgoing**2+incoming**2).toBeCloseTo(1);expect(outgoing).toBeCloseTo(Math.SQRT1_2);expect(itemAudioGain(a,0,30,project)).toBe(0);expect(itemAudioGain(a,90,30,project)).toBe(0);expect(itemAudioGain(b,15,30,project)).toBeCloseTo(Math.SQRT1_2);
 const audio:Item={id:'m',type:'audio',src:'a',at:0,dur:7,srcIn:2,volume:1,props:{duck:true}};expect(itemAudioGain(audio,0,30,project)).toBe(0);expect(itemAudioGain(audio,120,30,project)).toBe(.25);
});

it('retains only used project media provenance in export credits',async()=>{
 const {exportLicenses}=await import('../packages/render/src/licenses');const p=emptyProject();p.sources.used={id:'used',kind:'audio',name:'Original sound',path:'sound.wav',durationSec:2,provenance:JSON.stringify({license:'MIT',author:'Project author',sourceUrl:'https://example.com/original'})};p.sources.orphan={id:'orphan',kind:'audio',name:'Unused',path:'unused.wav',durationSec:2};p.tracks[4].items.push({id:'sound',type:'audio',src:'used',at:0,dur:2,srcIn:0,volume:1,props:{}});const result=await exportLicenses(p);expect(result.assets.map(a=>a.name)).toEqual(['Original sound']);expect(result.notices.join(' ')).not.toContain('Unused');
});
it('rejects invalid export dimensions/quality without mutating the project snapshot',async()=>{
 const {renderProject}=await import('../packages/render/src/export');const project=emptyProject();const before=structuredClone(project);
 await expect(renderProject({project,root:process.cwd(),mediaBaseUrl:'',assetBaseUrl:'',preset:{width:721,height:1280}})).rejects.toThrow('even');
 await expect(renderProject({project,root:process.cwd(),mediaBaseUrl:'',assetBaseUrl:'',preset:'landscape',quality:52})).rejects.toThrow('CRF');expect(project).toEqual(before);
});
