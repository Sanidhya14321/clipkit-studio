import {readFile,mkdir,writeFile,access} from 'node:fs/promises';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createHash} from 'node:crypto';
import {createRequire} from 'node:module';
import path from 'node:path';

const allowed=new Set(['gsap','@remotion/gsap','motion','@paper-design/shaders','lucide-react']);
const pending=new Map<string,Promise<{nodePaths:string[];hash:string}>>();
/** Exact versions, no lifecycle scripts. React/Remotion stay on the shared runtime. */
export async function sceneDependencies(root:string,deps:Record<string,string>={}){
  for(const [name,version]of Object.entries(deps)){
    if(!allowed.has(name)||!/^\d+\.\d+\.\d+(?:-[\w.-]+)?$/.test(version))throw Error('Declare an exact supported scene dependency: '+name);
    if(name==='@remotion/gsap'&&version!=='4.0.538')throw Error('All Remotion packages must use 4.0.538');
  }
  const resolver=createRequire(import.meta.url);let lockBytes=Buffer.from('runtime-lock-unavailable');
  for(const modules of resolver.resolve.paths('gsap')??[]) {let found=false;for(const name of ['npm-shrinkwrap.json','pnpm-lock.yaml','package-lock.json'])try{lockBytes=await readFile(path.join(path.dirname(modules),name));found=true;break}catch{}if(found)break}
  const runtimeHash=createHash('sha256').update(lockBytes).digest('hex');
  const sorted=Object.fromEntries(Object.entries(deps).sort(([a],[b])=>a.localeCompare(b))),hash=createHash('sha256').update(JSON.stringify({dependencies:sorted,runtimeHash,renderer:'4.0.538'})).digest('hex');
  const folder=path.join(root,'.clipkit/dependencies',hash),key=folder;
  if(!pending.has(key))pending.set(key,(async()=>{
    const require=createRequire(import.meta.url);let local=true;
    for(const [name,version]of Object.entries(sorted))try{
      let dir=path.dirname(require.resolve(name));let found=false;
      for(let i=0;i<5;i++,dir=path.dirname(dir))try{const p=JSON.parse(await readFile(path.join(dir,'package.json'),'utf8'));if(p.name===name){if(p.version!==version)throw Error('Version differs');found=true;break}}catch(e){if((e as NodeJS.ErrnoException).code!=='ENOENT')throw e}
      if(!found)local=false;
    }catch{local=false}
    const nodePaths=require.resolve.paths('gsap')??[];
    if(!Object.keys(sorted).length||local){await mkdir(folder,{recursive:true});await writeFile(path.join(folder,'dependency-lock.json'),JSON.stringify({dependencies:sorted,runtimeHash,renderer:'4.0.538'},null,2));await writeFile(path.join(folder,'runtime.lock'),lockBytes);return {hash,nodePaths}}
    await mkdir(folder,{recursive:true});
    try{await access(path.join(folder,'package-lock.json'));await access(path.join(folder,'node_modules'))}catch{
      await writeFile(path.join(folder,'package.json'),JSON.stringify({private:true,dependencies:sorted}));
      await promisify(execFile)('npm',['install','--ignore-scripts','--no-audit','--no-fund'],{cwd:folder,timeout:120000,maxBuffer:1024*1024});
    }
    const installedLock=await readFile(path.join(folder,'package-lock.json'));return {hash:createHash('sha256').update(hash).update(installedLock).digest('hex'),nodePaths:[path.join(folder,'node_modules'),...nodePaths]};
  })().catch(e=>{pending.delete(key);throw e}));
  return pending.get(key)!;
}
