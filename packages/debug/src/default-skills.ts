import {readFile,readdir} from 'node:fs/promises';
import path from 'node:path';
import codexBuiltins from '../../../bench/codex-builtins-0.162.1.json';
import {sha} from './trace';
export {codexBuiltins};
// These are CLI-seeded vanilla defaults, not optional owner-installed skills.
export async function verifyDefaultSkills(folder:string){
 const actual:Record<string,string>={};
 async function walk(dir:string){for(const entry of await readdir(dir,{withFileTypes:true})){const file=path.join(dir,entry.name);if(entry.isSymbolicLink())throw Error('symlink');if(entry.isDirectory())await walk(file);else actual[path.relative(folder,file)]=sha(new Uint8Array(await readFile(file)));}}
 try{await walk(folder);return Object.keys(actual).length===Object.keys(codexBuiltins).length&&Object.entries(actual).every(([file,hash])=>(codexBuiltins as Record<string,string>)[file]===hash);}catch{return false;}
}
