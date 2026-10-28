import {spawn} from 'node:child_process';
import {mkdir,readFile,writeFile,stat} from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {z} from 'zod';
import {benchCases} from './observability';
const Arm=z.object({cwd:z.string(),argv:z.array(z.string()).min(1),preview:z.string(),result:z.string()});
const Manifest=z.object({model:z.string().min(1),limits:z.string().min(1),timeoutMs:z.number().int().min(100).max(3600000),trials:z.array(z.object({caseId:z.string(),evidence:z.array(z.string()).min(1),rubric:z.string().min(1),pipeline:Arm,vanilla:Arm})).min(1).max(10)});
/** Explicit local driver processes only. Never run imported card/reference text as commands. */
export async function executeBenchmark(input:unknown,base:string,outDir:string){
 const manifest=Manifest.parse(input),records:any[]=[];await mkdir(outDir,{recursive:true});
 for(const trial of manifest.trials){if(!benchCases.some(([id])=>id===trial.caseId))throw Error('Unknown benchmark case '+trial.caseId);
  const evidence=[];for(const file of trial.evidence){const bytes=await readFile(path.resolve(base,file));evidence.push({file,sha256:createHash('sha256').update(bytes).digest('hex')})}
  for(const arm of ['pipeline','vanilla'] as const){const config=trial[arm],cwd=path.resolve(base,config.cwd),preview=path.resolve(cwd,config.preview),resultFile=path.resolve(cwd,config.result);for(const file of [preview,resultFile]){try{await stat(file);throw Error('Benchmark outputs must be fresh: '+file)}catch(e){if((e as NodeJS.ErrnoException).code!=='ENOENT')throw e}}
   const started=Date.now();let firstPreview:number|null=null,output='';const log=(data:Buffer)=>{output=(output+data.toString()).slice(-100000)};
   const child=spawn(config.argv[0],config.argv.slice(1),{cwd,shell:false,env:{...process.env,CLIPKIT_BENCH_MODEL:manifest.model,CLIPKIT_BENCH_LIMITS:manifest.limits,CLIPKIT_BENCH_CASE:trial.caseId,CLIPKIT_BENCH_ARM:arm,CLIPKIT_BENCH_RUBRIC:trial.rubric,CLIPKIT_BENCH_PREVIEW:preview,CLIPKIT_BENCH_RESULT:resultFile}});child.stdout?.on('data',log);child.stderr?.on('data',log);
   let forceTimer:ReturnType<typeof setTimeout>|undefined;let timedOut=false;const timer=setTimeout(()=>{timedOut=true;child.kill('SIGTERM');forceTimer=setTimeout(()=>child.kill('SIGKILL'),3000)},manifest.timeoutMs),poll=setInterval(()=>{void stat(preview).then(s=>{if(s.size>0&&firstPreview===null)firstPreview=Date.now()-started}).catch(()=>{})},100);
   let code:number|null;try{code=await new Promise<number|null>((resolve,reject)=>{child.once('error',reject);child.once('close',resolve)})}finally{clearTimeout(timer);clearTimeout(forceTimer);clearInterval(poll)}
   const elapsedMs=Date.now()-started;await writeFile(path.join(outDir,trial.caseId+'-'+arm+'.log'),output);if(timedOut||code!==0)throw Error(`${trial.caseId}/${arm} ${timedOut?'timed out':'exited '+code}; inspect benchmark log`);
   if(firstPreview===null)try{if((await stat(preview)).size>0)firstPreview=elapsedMs}catch{}
   const raw=JSON.parse(await readFile(resultFile,'utf8')),metrics=z.object({tokens:z.number().nonnegative().nullable(),turnsToAcceptance:z.number().int().nonnegative().nullable(),lintScore:z.number().min(0).max(1).nullable(),accepted:z.boolean(),method:z.string().min(1)}).parse(raw);
   const receipt={caseId:trial.caseId,arm,model:manifest.model,limits:manifest.limits,...metrics,timeToFirstPreviewMs:firstPreview,video:preview,method:`Local driver execution; preview polled every 100ms. ${metrics.method}`,elapsedMs,evidence,rubric:trial.rubric,argv:config.argv};await writeFile(path.join(outDir,trial.caseId+'-'+arm+'.receipt.json'),JSON.stringify(receipt,null,2));records.push(receipt);
  }
 }
 return records;
}
