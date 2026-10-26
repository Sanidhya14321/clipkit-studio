import {it,expect} from 'vitest';
import {starPoints,confettiPieces,confettiPosition,typedText,chartRatios} from '../packages/render/src/motion-math';
import {validateClipProps} from '../packages/assets/src/index';
it('seeds particle layouts reproducibly and changes them with the seed',()=>{for(const generate of [starPoints,confettiPieces]){expect(generate(20,'demo')).toEqual(generate(20,'demo'));expect(generate(20,'demo')).not.toEqual(generate(20,'other'));}for(const star of starPoints(240,'demo')){expect(star.x).toBeGreaterThanOrEqual(0);expect(star.x).toBeLessThan(1);expect(star.y).toBeLessThan(1);}});
it('computes confetti positions from elapsed time with delayed visibility',()=>{const p=confettiPieces(1,'demo')[0];expect(confettiPosition(p,-1,{width:720,height:1280},600).visible).toBe(false);expect(confettiPosition(p,1,{width:720,height:1280},600)).toEqual(confettiPosition(p,1,{width:720,height:1280},600));});
it('types Unicode without splitting a surrogate pair and clamps progress',()=>{expect(typedText('A🚀B',.7)).toBe('A🚀');expect(typedText('abc',-1)).toBe('');expect(typedText('abc',2)).toBe('abc');});
it('keeps zero charts finite with a shared nonnegative domain',()=>{expect(chartRatios([0,0,0,0])).toEqual([0,0,0,0]);expect(chartRatios([20,40,10,0])).toEqual([.5,1,.25,0]);});
it('rejects excessive particle loads and invalid chart inputs',()=>{expect(()=>validateClipProps('starfield',{count:241})).toThrow();expect(()=>validateClipProps('confetti-burst',{seed:''})).toThrow();expect(()=>validateClipProps('bar-chart',{value4:-1})).toThrow();expect(()=>validateClipProps('bar-chart',{label1:'a'.repeat(13)})).toThrow();});
it('preserves every original native ID and default property',async()=>{
 const {readFile}=await import('node:fs/promises');const {catalog}=await import('../packages/assets/src/index');
 // Preserve the released defaults independently of the current generated catalog.
 const previous=JSON.parse(await readFile(new URL('./fixtures/native-1.0.0.json',import.meta.url),'utf8'));
 for(const old of previous.clips){const asset=catalog.find(item=>item.id===old.clipId);expect(asset?.defaults).toEqual(old.defaults);}
});
it('credits fonts only when optional particle titles contain text',async()=>{
 const {exportLicenses}=await import('../packages/render/src/licenses');const {emptyProject}=await import('../packages/core/src/index');const {makeClip}=await import('../packages/assets/src/index');
 const p=emptyProject();const item=makeClip('starfield');p.tracks[1].items.push(item);
 expect((await exportLicenses(p,'public')).assets).toHaveLength(0);
 item.props.text='Across the stars';expect((await exportLicenses(p,'public')).assets.map(a=>a.name)).toEqual(expect.arrayContaining(['rsms/inter','floriankarsten/space-grotesk']));
});
