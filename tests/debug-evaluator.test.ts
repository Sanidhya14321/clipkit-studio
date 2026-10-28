import {it,expect} from 'vitest';
import {mkdtemp,rm} from 'node:fs/promises';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {generateFixtures} from '../packages/debug/src/fixtures';
import {ffmpegBinary} from '../packages/server/src/media';
import {evaluateOutput} from '../packages/debug/src/evaluator';
it('detects actual clipped audio and keeps missing required visual evidence unmeasured',async()=>{
 const root=await mkdtemp(path.join(tmpdir(),'ck-eval-negative-'));
 try{
  await generateFixtures(root);const video=path.join(root,'clipped.mkv');
  await promisify(execFile)(await ffmpegBinary(),['-y','-v','error','-i',path.join(root,'counter.mp4'),'-f','lavfi','-i','aevalsrc=2*sin(2*PI*440*t):s=48000:d=6','-map','0:v','-map','1:a','-c:v','copy','-c:a','pcm_s16le','-t','6',video],{timeout:15000});
  const result=await evaluateOutput({video,expected:{duration:6,aspect:16/9,requiredCriteria:['text-presence']}});
  expect(result.checks.find(c=>c.id==='aspect')?.status).toBe('pass');
  expect(result.checks.find(c=>c.id==='peak')?.status).toBe('fail');
  expect(result.checks.find(c=>c.id==='text-presence')?.status).toBe('unmeasured');expect(result.passed).toBe(false);
 }finally{await rm(root,{recursive:true,force:true});}
},60000);
