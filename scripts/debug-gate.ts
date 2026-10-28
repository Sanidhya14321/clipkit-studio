import {readFile,access} from 'node:fs/promises';
import {regressionGate} from '../packages/debug/src/results';
const args=process.argv.slice(2),at=(name:string)=>{const i=args.indexOf(name);return i<0?undefined:args[i+1]};
if(!at('--current'))throw Error('Usage: tsx scripts/debug-gate.ts --current summary.json [--baseline last-good.json] [--acceptance receipts.json] [--threshold 0.2]');
const current=JSON.parse(await readFile(at('--current')!,'utf8'));
const baseline=at('--baseline')?JSON.parse(await readFile(at('--baseline')!,'utf8')):null;
const failures:string[]=[];
if(!current.groups?.length)failures.push('No experiment results');
if(!current.groups?.every((g:any)=>g.runs>=3&&g.tokens?.count===g.runs))failures.push('At least 3 valid measured runs per group required');
for(const scenario of ['S1','S2','S3','S4'])for(const arm of ['A0','A1','B']){
 const group=current.groups?.find((g:any)=>g.key===scenario+'/'+arm);
 if(!group)failures.push('Missing '+scenario+'/'+arm);
 else if(arm==='B'&&group.passRate!==1)failures.push(scenario+'/B has unmet acceptance criteria');
}
if(baseline)failures.push(...regressionGate(current,baseline,Number(at('--threshold')??.2)).failures);
else failures.push('No stored last-good baseline; no regression claim');
let receipts:Record<string,unknown>={};if(at('--acceptance'))receipts=JSON.parse(await readFile(at('--acceptance')!,'utf8'));
for(const check of ['priorP0','threeCleanDemos','liveProvider','independentRepro','blindHumanRatings','offlineFallback','securityContracts','noCriticalBugs','noUnexplainedFlakes']){
 const file=receipts[check];if(typeof file!=='string')failures.push('Missing acceptance receipt: '+check);
 else try{await access(file)}catch{failures.push('Unreadable acceptance receipt: '+check)}
}
const report={release:failures.length?'NO_GO':'GO',failures,method:'Missing measurements, human/live acceptance or a missing baseline never pass the demo gate.'};
console.log(JSON.stringify(report,null,2));if(failures.length)process.exitCode=1;
