import {describe,it,expect} from 'vitest';
import {catalog,makeClip,searchAssets,validateClipProps} from '../packages/assets/src/index';
import {ItemSchema} from '../packages/core/src/index';
describe('native clip catalog',()=>{
 it('provides forty-six valid clips and independent editable defaults',()=>{expect(catalog).toHaveLength(46);for(const asset of catalog){const a=makeClip(asset.id);expect(ItemSchema.safeParse(a).success).toBe(true);expect(validateClipProps(asset.id,a.props)).toEqual(asset.defaults);expect(a.props).not.toBe(asset.defaults);}});
 it('finds captions and rejects out of range controls',()=>{expect(searchAssets('caption')[0].id).toBe('captions');expect(()=>validateClipProps('counter',{value:Infinity})).toThrow();expect(()=>validateClipProps('cta',{opacity:2})).toThrow();expect(()=>makeClip('missing')).toThrow();});
});
// Opt-in real Chromium test: preview/export/edit renders for every native clip, with font/art serving and pixel hashes.
it.runIf(process.env.CK_RENDER_TEST==='1')('matches native preview/export frames and visibly applies edits',async()=>{
 const {execFile}=await import('node:child_process');const {promisify}=await import('node:util');
 const result=await promisify(execFile)(process.execPath,['--import','tsx','scripts/render-consistency.ts'],{cwd:process.cwd(),timeout:300000,maxBuffer:1024*1024});
 expect(result.stdout.match(/PASS /g)).toHaveLength(46);
},310000);

import {itemAudioGain,narrationDuckingGain} from '../packages/render/src/audio-gain';
import {emptyProject,type Item} from '../packages/core/src/index';
it('ducks opted-in background audio smoothly only beneath the audible active narration take',()=>{
 const p=emptyProject();p.sources.take={id:'take',kind:'audio',path:'take.wav',name:'Take',durationSec:3};
 p.voiceSlots.push({id:'slot',text:'Hello',mode:'recorded',takes:[{id:'t',sourceId:'take',mode:'recorded',durationSec:3,createdAt:'2026-10-26T00:00:00Z'}],activeTakeByMode:{recorded:'t'},targetWindow:{start:2,end:7},status:'ready'});
 const voice:Item={id:'v',type:'voice',slotId:'slot',at:2,dur:5,srcIn:1,volume:1,props:{}};
 p.tracks.find(t=>t.kind==='voice')!.items.push(voice);
 const music:Item={id:'m',type:'audio',src:'take',at:0,dur:10,srcIn:0,volume:.8,props:{duck:true}};
 expect(narrationDuckingGain(p,1.9)).toBe(1);
 expect(narrationDuckingGain(p,2)).toBe(1);
 expect(narrationDuckingGain(p,2.06)).toBeCloseTo(.625);
 expect(itemAudioGain(music,90,30,p)).toBeCloseTo(.2);
 expect(narrationDuckingGain(p,3.94)).toBeCloseTo(.625);
 expect(narrationDuckingGain(p,4.01)).toBe(1); // take ends before the longer voice item
 expect(itemAudioGain({...music,props:{}},90,30,p)).toBe(.8);
 expect(itemAudioGain(music,0,30,p)).toBe(0);
 expect(itemAudioGain(music,299,30,p)).toBe(0);
 voice.volume=0;expect(narrationDuckingGain(p,3)).toBe(1);
 voice.volume=1;p.voiceSlots[0].activeTakeByMode.recorded=undefined;
 expect(narrationDuckingGain(p,3)).toBe(1);
});

it('builds native pack metadata, category index and full schemas from the trusted registry',async()=>{
 const {buildAssets}=await import('../packages/assets/src/build');
 const {mkdtemp,readFile,rm}=await import('node:fs/promises');const {tmpdir}=await import('node:os');const {join}=await import('node:path');
 const output=await mkdtemp(join(tmpdir(),'ck-pack-'));
 try{
  const result=await buildAssets(process.cwd(),output);expect(result.count).toBe(46);
  const rows=(await readFile(join(output,'catalog.jsonl'),'utf8')).trim().split('\n').map(line=>JSON.parse(line));expect(rows.map(r=>r.id)).toEqual(catalog.map(a=>a.id));
  const schema=JSON.parse(await readFile(join(output,'schemas/kinetic-title.json'),'utf8'));expect(schema.propsSchema.properties.text.default).toBe(catalog[0].defaults.text);expect(schema.editableContract.requiredProps).toContain('opacity');
  const pack=JSON.parse(await readFile(join(output,'pack.json'),'utf8'));expect(pack.runtime).toEqual({kind:'compiled',registry:'clipkit/native-v1'});expect(pack.clips).toHaveLength(46);
  const originalLicense=catalog[0].license;try{catalog[0].license='';await expect(buildAssets(process.cwd(),output)).rejects.toThrow('license is required');}finally{catalog[0].license=originalLicense;}
  const originalFields=catalog[0].fields;try{catalog[0].fields=originalFields.filter(field=>field.key!=='opacity');await expect(buildAssets(process.cwd(),output)).rejects.toThrow('missing editable property opacity');}finally{catalog[0].fields=originalFields;}
  const original=catalog[0].defaults.opacity;try{catalog[0].defaults.opacity=2;await expect(buildAssets(process.cwd(),output)).rejects.toThrow();}finally{catalog[0].defaults.opacity=original;}
 }finally{await rm(output,{recursive:true,force:true});}
});
