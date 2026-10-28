import {readFile,realpath,stat} from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import type {SceneDraft} from './service';

export const sourceName=(name:string)=>!name.includes('\\')&&!path.isAbsolute(name)&&!name.split('/').some(p=>!p||p==='.'||p==='..'||p==='node_modules')&&/\.(tsx|jsx|ts|js|css|svg|json)$/.test(name);
export async function readDiskScene(root:string,sceneId:string):Promise<SceneDraft>{
  if(!/^[a-z][a-z0-9-]{0,60}$/.test(sceneId))throw Error('Invalid scene ID');
  const base=path.join(root,'scenes',sceneId),actual=await realpath(base);
  if(actual!==path.resolve(base))throw Error('Scene directories cannot be symlinks');
  const draft=JSON.parse(await readFile(path.join(base,'scene.json'),'utf8')) as SceneDraft;
  const names=Object.keys(draft.files??{});if(!names.length||names.length>(draft.manifestVersion===2?128:16))throw Error('Too many or missing scene files');
  let total=0;const files:Record<string,string>={};
  for(const name of names){
    if(!sourceName(name))throw Error('Invalid scene filename');
    const file=await realpath(path.join(base,name));if(!file.startsWith(actual+path.sep))throw Error('Scene file escapes directory');
    const size=(await stat(file)).size;total+=size;if(size>200000||total>2000000)throw Error('Scene source budget exceeded; store binary assets as resources');
    files[name]=await readFile(file,'utf8');
  }
  return {...draft,sceneId,files};
}
export function sourceHashes(files:Record<string,string>){return Object.fromEntries(Object.entries(files).map(([name,code])=>[name,createHash('sha256').update(code).digest('hex')]))}
