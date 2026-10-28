import {readdir} from 'node:fs/promises';
import path from 'node:path';
import {bundleTrace} from '../packages/debug/src/trace';
for(const root of [process.cwd(),path.resolve('.clipkit/e2e-v3-workspace')]){
 let runs:string[]=[];try{runs=await readdir(path.join(root,'.clipkit/trace'))}catch{}
 for(const run of runs.slice(-30))try{const bundle=await bundleTrace(root,run);console.log(JSON.stringify({run,files:bundle.files,path:bundle.path}));}catch{console.error('Could not bundle '+run);process.exitCode=1;}
}
