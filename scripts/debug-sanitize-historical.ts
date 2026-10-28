import {readFile,writeFile,readdir,rename} from 'node:fs/promises';
import path from 'node:path';
import {unzipSync,zipSync} from 'fflate';
import {sha} from '../packages/debug/src/trace';
const root=path.resolve('.clipkit/update4'),audit:any[]=[];
const lease=/\b(?:lease|layers)_[a-f0-9]+secret_[a-f0-9]+\b/gi;
const scrub=(value:string,secrets:string[]=[])=>{for(const secret of secrets)value=value.split(secret).join('[redacted]');return value.replace(lease,'[redacted]').replace(/([?&](?:token|key|secret)=)[^&\s"\\]+/gi,'$1[redacted]').replace(/Bearer\s+[\w.~-]+/gi,'Bearer [redacted]');};
function credentials(value:any,found:Set<string>){if(!value||typeof value!=='object')return;if(typeof value.name==='string'&&/authorization|cookie|x-clipkit-token|x-clipkit-lease/i.test(value.name)&&typeof value.value==='string'&&value.value.length>=8)found.add(value.value);for(const [key,child]of Object.entries(value)){if(/^(?:token|leaseToken|apiKey|access_token|refresh_token|authorization)$/i.test(key)&&typeof child==='string'&&child.length>=8)found.add(child);else credentials(child,found);}}
async function replace(file:string,bytes:Uint8Array,before:Uint8Array,kind:string){if(Buffer.from(bytes).equals(Buffer.from(before)))return;await writeFile(file+'.redacted.tmp',bytes);await rename(file+'.redacted.tmp',file);audit.push({file:path.relative(root,file),kind,beforeSHA256:sha(before),afterSHA256:sha(bytes),originalRetained:false});}
async function walk(folder:string){for(const entry of await readdir(folder,{withFileTypes:true})){if(entry.isSymbolicLink()||entry.name==='.git')continue;const file=path.join(folder,entry.name);if(entry.isDirectory()){await walk(file);continue;}
 if(entry.name==='events.jsonl'){const before=await readFile(file),safe=scrub(before.toString());await replace(file,Buffer.from(safe),before,'expired lease identifiers');}
 else if(entry.name==='trace.zip'){
  const before=await readFile(file),files=unzipSync(before),textFiles:Record<string,string>={},secrets=new Set<string>();
  for(const [name,bytes]of Object.entries(files)){const text=Buffer.from(bytes).toString();if(text.includes('\0')||!Buffer.from(text).equals(Buffer.from(bytes)))continue;textFiles[name]=text;for(const line of text.split('\n'))try{credentials(JSON.parse(line),secrets);}catch{}}
  let changed=false;for(const [name,text]of Object.entries(textFiles)){const safe=scrub(text,[...secrets]);if(safe!==text){files[name]=Buffer.from(safe);changed=true;}}
  if(changed)await replace(file,zipSync(files),before,'retired native browser trace capability headers');
 }
}}
await walk(root);await writeFile(path.join(root,'historical-redaction.json'),JSON.stringify({reason:'Removed expired local edit/auth capabilities from earlier diagnostic artifacts. No raw secret backup retained. Result records, errors, usage and project revisions are unchanged.',files:audit},null,2));console.log(JSON.stringify({sanitizedFiles:audit.length,receipt:'.clipkit/update4/historical-redaction.json'}));
