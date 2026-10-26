import path from 'node:path';
import {checkAssets} from '../packages/assets/src/check.ts';
const result=await checkAssets(path.resolve(import.meta.dirname,'..'),{checkOnly:process.argv.includes('--check'),requireEvidence:!process.argv.includes('--allow-untested')});
console.log(JSON.stringify(result,null,2));process.exitCode=result.passed?0:1;
