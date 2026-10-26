import {describe,it,expect} from 'vitest';
import {catalog,makeClip,searchAssets,validateClipProps} from '../packages/assets/src/index';
import {ItemSchema} from '../packages/core/src/index';
describe('native clip catalog',()=>{
 it('provides ten valid clips and independent editable defaults',()=>{expect(catalog).toHaveLength(10);for(const asset of catalog){const a=makeClip(asset.id);expect(ItemSchema.safeParse(a).success).toBe(true);expect(validateClipProps(asset.id,a.props)).toEqual(asset.defaults);expect(a.props).not.toBe(asset.defaults);}});
 it('finds captions and rejects out of range controls',()=>{expect(searchAssets('caption')[0].id).toBe('captions');expect(()=>validateClipProps('counter',{value:Infinity})).toThrow();expect(()=>validateClipProps('cta',{opacity:2})).toThrow();expect(()=>makeClip('missing')).toThrow();});
});
// Opt-in real Chromium test: 30 PNG renders, with font/art serving and pixel hashes.
it.runIf(process.env.CK_RENDER_TEST==='1')('matches native preview/export frames and visibly applies edits',async()=>{
 const {execFile}=await import('node:child_process');const {promisify}=await import('node:util');
 const result=await promisify(execFile)(process.execPath,['--import','tsx','scripts/render-consistency.ts'],{cwd:process.cwd(),timeout:300000,maxBuffer:1024*1024});
 expect(result.stdout.match(/PASS /g)).toHaveLength(10);
},310000);
