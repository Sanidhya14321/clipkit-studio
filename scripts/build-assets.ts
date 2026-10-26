import path from 'node:path';
import {buildAssets} from '../packages/assets/src/build.ts';
const root=path.resolve(import.meta.dirname,'..');
console.log(await buildAssets(root,process.argv[2]?path.resolve(process.argv[2]):undefined));
