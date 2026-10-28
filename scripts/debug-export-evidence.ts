import {readFile,writeFile,mkdir,cp,mkdtemp,rm} from 'node:fs/promises';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {unzipSync} from 'fflate';
import {replayTrace,bundleTrace,reproTrace,sha} from '../packages/debug/src/trace';
import {ProjectSchema} from '../packages/core/src/index';
const out=path.resolve(process.argv[2]??'.clipkit/update4/results');
const rows=(await readFile(path.join(out,'results.jsonl'),'utf8')).split('\n').filter(Boolean).map(line=>JSON.parse(line));
const aliases=[];
for(const row of rows.filter(row=>row.arm==='B')){
 const source=path.join(out,'.clipkit/trace',row.runId,'server'),alias=row.runId+'-studio';
 try{await cp(source,path.join(out,'.clipkit/trace',alias),{recursive:true});await writeFile(path.join(out,'.clipkit/trace',alias,'derived-origin.json'),JSON.stringify({derived:true,originalRunId:row.runId,source:'server/ nested recorder; raw source retained'}));aliases.push({runId:row.runId,studioTrace:alias});}catch(error){if((error as NodeJS.ErrnoException).code!=='ENOENT')throw error;}
}
await writeFile(path.join(out,'derived-traces.json'),JSON.stringify(aliases,null,2));
const selected=aliases.find(alias=>rows.some(row=>row.runId===alias.runId&&row.valid))??aliases[0];
if(!selected)throw Error('No actual Studio recorder available');
const original=await replayTrace(out,selected.studioTrace),bundle=await bundleTrace(out,selected.studioTrace);
const temp=await mkdtemp(path.join(tmpdir(),'ck-exported-replay-'));
let proof:any;
try{
 const files=unzipSync(await readFile(bundle.path)),destination=path.join(temp,'.clipkit/trace',selected.studioTrace);
 for(const [relative,bytes]of Object.entries(files)){const file=path.resolve(destination,relative);if(!file.startsWith(destination+path.sep))throw Error('Unsafe bundle path');await mkdir(path.dirname(file),{recursive:true});await writeFile(file,bytes);}
 const replay=await replayTrace(temp,selected.studioTrace);
 proof={runId:selected.runId,studioTrace:selected.studioTrace,rawRevisions:original.revisions.length,anonymizedRevisions:replay.revisions.length,allProjectsSchemaValid:replay.revisions.every(revision=>ProjectSchema.safeParse(revision.project).success),lastRevision:replay.project.rev,bundle:path.relative(out,bundle.path),sha256:sha(new Uint8Array(await readFile(bundle.path))),independentHumanOrMachine:false,method:'Local extraction of actual failing Studio trace; every anonymized revision hash verified without an agent or provider.'};
 const layers=original.project.tracks.flatMap((track:any)=>track.items).slice(0,2).map((item:any)=>item.id);
 const repro=await reproTrace(out,selected.studioTrace,layers);proof.minimalRepro=path.relative(out,repro.path);proof.selectedLayers=layers;
}finally{await rm(temp,{recursive:true,force:true});}
await writeFile(path.join(out,'replay-proof.json'),JSON.stringify(proof,null,2));console.log(JSON.stringify(proof,null,2));
