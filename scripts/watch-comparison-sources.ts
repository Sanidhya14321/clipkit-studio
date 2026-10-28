import {readdir,readFile,writeFile,mkdir,stat} from 'node:fs/promises';
import path from 'node:path';
import {tmpdir} from 'node:os';
import {sha,redact,registerSecrets} from '../packages/debug/src/trace';
const out=path.resolve(process.argv[2]??'.clipkit/video-comparison-20261028'),log='/tmp/clipkit-video-comparison.log';
const protocol=JSON.parse(await readFile(path.join(out,'protocol.json'),'utf8')),seen=new Map<string,string>(),index:any[]=[];
async function sample(){let event:any;for(const line of (await readFile(log,'utf8')).split('\n'))try{const parsed=JSON.parse(line);if(parsed.event==='run-start')event=parsed;}catch{}
 if(!event)return;const target=path.join(out,`rep-${event.repetition+1}-${event.arm}`,'source-history');
 for(const name of await readdir(tmpdir())){if(!name.startsWith('ck-e2e-'))continue;const workspace=path.join(tmpdir(),name,'workspace');try{if(await readFile(path.join(workspace,'BRIEF.md'),'utf8')!==protocol.prompt+'\n')continue;}catch{continue;}
  try{const descriptor=JSON.parse(await readFile(path.join(workspace,'.clipkit/server.json'),'utf8'));registerSecrets([descriptor.token]);}catch{}
  async function walk(dir:string){for(const item of await readdir(dir,{withFileTypes:true})){if(item.name.startsWith('.')||item.isSymbolicLink()||['node_modules','fixtures','public','media'].includes(item.name))continue;const file=path.join(dir,item.name);if(item.isDirectory())await walk(file);else if(/\.(?:tsx?|jsx?|[cm]js|py|sh|svg|html|json|md|css|srt|txt|ya?ml|toml)$/.test(item.name)&&!['AGENTS.md','BRIEF.md','package.json'].includes(item.name)){
   const size=(await stat(file)).size;if(size>200000)continue;const text=redact(await readFile(file,'utf8'),{synthetic:true}),hash=sha(text),relative=path.relative(workspace,file),key=target+'/'+relative;if(seen.get(key)===hash)continue;seen.set(key,hash);await mkdir(path.join(target,'files'),{recursive:true});const stored='files/'+hash+path.extname(file);await writeFile(path.join(target,stored),text);index.push({repetition:event.repetition,arm:event.arm,relative,sha256:hash,stored:path.relative(out,path.join(target,stored)),observedAt:new Date().toISOString()});await writeFile(path.join(out,'source-history.json'),JSON.stringify(index,null,2));
  }}}
  try{await walk(workspace);}catch{}
 }
}
while(true){await sample();if(await stat(path.join(out,'completion.json')).then(()=>true).catch(()=>false))break;await new Promise(resolve=>setTimeout(resolve,1000));}
console.log(JSON.stringify({capturedVersions:index.length,notice:'Read-only source observation; code never executed by this observer.'}));
