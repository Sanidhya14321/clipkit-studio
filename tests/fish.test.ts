import {describe,it,expect,vi} from 'vitest';
import {FishClient,parseTranscript} from '../packages/server/src/fish.ts';

describe('Fish provider contract',()=>{
 it('uses authentic word timing and refuses invented phrase subdivisions',()=>{
  expect(parseTranscript({text:'Hello world',segments:[{text:'Hello',start:0,end:.3},{text:'world',start:.4,end:.8}]}).words).toHaveLength(2);
  expect(parseTranscript({text:'Hello world',segments:[{text:'Hello world',start:0,end:.8}]})).toMatchObject({words:[],wordTiming:false});
  expect(parseTranscript({text:'Hello',segments:[{text:'Hello',start:-1,end:0}]}).words).toEqual([]);
 });
 it('sends TTS model, reference and speed in the documented fields',async()=>{
  const transport=vi.fn(async()=>new Response(new Uint8Array([82,73,70,70]),{status:200}));
  const client=new FishClient('server-test-key',transport as typeof fetch);
  await client.tts('Hello','voice123',1.05);
  const [url,init]=transport.mock.calls[0] as unknown as [string,RequestInit];
  expect(url).toBe('https://api.fish.audio/v1/tts');expect(init.headers).toMatchObject({Authorization:'Bearer server-test-key',model:process.env.FISH_TTS_MODEL||'s2-pro'});
  expect(JSON.parse(String(init.body))).toMatchObject({text:'Hello',reference_id:'voice123',format:'wav',prosody:{speed:1.05}});
 });
 it('explains missing credentials without trying a fake provider',async()=>{
  const client=new FishClient('');await expect(client.tts('hello','voice')).rejects.toThrow('FISH_API_KEY');
 });
 it('does not expose provider error bodies or keys',async()=>{
  const client=new FishClient('secret',async()=>new Response('secret raw provider detail',{status:401}));
  await expect(client.tts('Hello','voice')).rejects.toThrow('Fish 401: check FISH_API_KEY');
 });
});
