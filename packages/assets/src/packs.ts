import {readFile,writeFile,mkdir,readdir,cp,rm,realpath,stat} from 'node:fs/promises';
import {watch} from 'node:fs';
import path from 'node:path';
import {createHash,randomUUID} from 'node:crypto';
import {build} from 'esbuild';
import * as React from 'react';
import * as Remotion from 'remotion';
import {catalog,type Field} from './index';
import {validateRegisteredProps,type AssetRegistry,type RuntimeClipDefinition,type PropsSchema} from './runtime';
const installations=new Map<string,{promise:Promise<void>}>();
const EDITABLE=['text','color','size','x','y','opacity','revealDuration'];
const slug=/^[a-z0-9][a-z0-9-]{0,79}$/;
const hash=(value:string|Buffer)=>createHash('sha256').update(value).digest('hex');
const safe=(folder:string,file:string)=>{const result=path.resolve(folder,file);if(!result.startsWith(path.resolve(folder)+path.sep))throw new Error('Pack path escapes its folder');return result;};
const nonempty=(value:unknown,label:string)=>{if(typeof value!=='string'||!value.trim())throw new Error(`${label} is required`);return value;};
export type PackManifest={format:'clipkit.asset-pack/v1';id:string;version:string;license:string;clips:Array<{assets?:Array<{name:string;license:string;sourceUrl?:string}>;id:string;name:string;category:string;description:string;license:string;durationSec:number;tags?:string[];aspect?:string;whenToUse?:string;defaults:Record<string,unknown>;fields:Field[];propsSchema:PropsSchema;basedOn?:string;component?:string;preview:string;deterministic:true;editable:true}>};
export type DiscoveredPack={folder:string;manifest:PackManifest;clips:RuntimeClipDefinition[];codeFiles:string[];contentHash:string;trusted:boolean;errors?:string[]};
export async function validatePack(folder:string):Promise<DiscoveredPack>{
 folder=await realpath(folder);if((await stat(safe(folder,'pack.json'))).size>256*1024)throw new Error('Pack manifest exceeds256KB');const manifest=JSON.parse(await readFile(safe(folder,'pack.json'),'utf8')) as PackManifest;
 if(manifest.format!=='clipkit.asset-pack/v1'||!slug.test(manifest.id))throw new Error('Invalid pack format/id');nonempty(manifest.version,'Pack version');nonempty(manifest.license,'Pack license');
 if(!Array.isArray(manifest.clips)||manifest.clips.length===0||manifest.clips.length>100)throw new Error('Pack needs 1–100 clips');
 const ids=new Set<string>();const codeFiles:string[]=[];const clips:RuntimeClipDefinition[]=[];const content=[JSON.stringify(manifest)];
 for(const clip of manifest.clips){
  if(!slug.test(clip.id)||ids.has(clip.id)||catalog.some(c=>c.id===clip.id))throw new Error(`Invalid/duplicate/native clip ID ${clip.id}`);ids.add(clip.id);
  for(const key of ['name','category','description','license'] as const)nonempty(clip[key],`${clip.id} ${key}`);
  for(const asset of clip.assets??[]){nonempty(asset.name,'Asset name');nonempty(asset.license,'Asset license');}
  if(!Number.isFinite(clip.durationSec)||clip.durationSec<=0||clip.durationSec>3600)throw new Error('Invalid clip duration');
  if(clip.deterministic!==true||clip.editable!==true)throw new Error(`${clip.id}: deterministic and editable contract required`);
  if(Boolean(clip.basedOn)===Boolean(clip.component))throw new Error(`${clip.id}: choose basedOn or component`);
  if(clip.basedOn&&!catalog.some(c=>c.id===clip.basedOn))throw new Error('Declarative wrapper must use a compiled native clip');
  if(!clip.defaults||typeof clip.defaults!=='object'||Array.isArray(clip.defaults))throw new Error('Defaults object required');
  if(clip.basedOn&&Object.keys(clip.defaults).some(key=>!(key in catalog.find(c=>c.id===clip.basedOn)!.defaults)))throw new Error('Wrapper defaults must match its native base schema');
  const schema=clip.propsSchema;if(!schema||schema.type!=='object'||!schema.properties||schema.additionalProperties!==false)throw new Error('Explicit object propsSchema with additionalProperties:false required');
  if(Object.keys(schema.properties).some(k=>['__proto__','constructor','prototype'].includes(k)))throw new Error('Unsafe property');
  for(const rule of Object.values(schema.properties))if(!['string','number','integer','boolean'].includes(rule.type)||Object.keys(rule).some(k=>!['type','enum','minimum','maximum','minLength','maxLength','default','description','title','pattern'].includes(k)))throw new Error('Unsupported JSON schema keyword/type');
  for(const rule of Object.values(schema.properties))if(rule.pattern&&rule.pattern!=='^#[0-9a-fA-F]{3,8}$')throw new Error('Only the bounded native color pattern is supported; use enum for other string constraints');
  if(!Array.isArray(clip.fields))throw new Error('Editable fields required');
  for(const key of EDITABLE)if(!(key in clip.defaults)||!clip.fields.some(f=>f.key===key)||!(schema.required??[]).includes(key))throw new Error(`${clip.id}: missing editable ${key}`);
  const fieldKeys=new Set<string>();for(const field of clip.fields){const rule=schema.properties[field.key];if(!rule||fieldKeys.has(field.key)||!['text','textarea','color','number','select','image'].includes(field.type))throw new Error('Field does not match props schema');fieldKeys.add(field.key);if(field.type==='number'?!['number','integer'].includes(rule.type):rule.type!=='string')throw new Error('Field type conflicts with schema');if(field.type==='select'&&(!field.options?.length||!field.options.includes(String(clip.defaults[field.key]))))throw new Error('Select field needs options containing its default');}
  const preview=await readFile(safe(folder,nonempty(clip.preview,'PNG preview')));if(!preview.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10])))throw new Error('Preview must be a real PNG');content.push(hash(preview));
  if(clip.component){if(!/\.tsx$/.test(clip.component))throw new Error('Trusted component must be .tsx');const file=safe(folder,clip.component);const source=await readFile(file,'utf8');if(/\b(?:Math\.random|Date\.now|performance\.now|setInterval|setTimeout)\s*\(|new\s+Date\s*\(/.test(source))throw new Error('Component uses a non-frame deterministic clock/random source');codeFiles.push(file);}
  const runtime:RuntimeClipDefinition={...clip,id:clip.id,packId:manifest.id,previewPath:`/asset-packs/${manifest.id}/${clip.preview}`,propsSchema:schema,tags:clip.tags??[],aspect:clip.aspect??'any',whenToUse:clip.whenToUse??clip.description};
  validateRegisteredProps(clip.id,clip.defaults,{clips:[runtime]});clips.push(runtime);
 }
 // Hash all local files: changing a trusted dependency also requires renewed explicit trust.
 let fileCount=0,totalBytes=0;const walk=async(dir:string):Promise<void>=>{for(const entry of (await readdir(dir,{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){if(entry.name==='.trust.json')continue;if(entry.isSymbolicLink())throw new Error('Pack symlinks are not permitted');const full=path.join(dir,entry.name);if(entry.isDirectory())await walk(full);else{const info=await stat(full);fileCount++;totalBytes+=info.size;if(info.size>20*1024*1024||totalBytes>50*1024*1024||fileCount>1000)throw new Error('Pack exceeds file/count/50MB limits');const bytes=await readFile(full);if(/\.[cm]?[jt]sx?$/.test(full)&&/\b(?:Math\.random|Date\.now|performance\.now|crypto\.randomUUID|crypto\.getRandomValues|setInterval|setTimeout)\s*\(|new\s+Date\s*\(/.test(bytes.toString()))throw new Error('Pack dependency violates frame deterministic contract');content.push(path.relative(folder,full),hash(bytes));}}};await walk(folder);
 const trusted=false; // Source-provided receipts never establish user trust.
 return {folder,manifest,clips,codeFiles,contentHash:hash(content.join('\n')),trusted};
}
export async function discoverPacks(root:string){const base=path.join(root,'.clipkit/assets/packs');await mkdir(base,{recursive:true});let approvals:Record<string,string>={};try{approvals=JSON.parse(await readFile(path.join(root,'.clipkit/assets/trust.json'),'utf8'));}catch{}const packs:DiscoveredPack[]=[];const errors:Array<{folder:string;error:string}>=[];for(const entry of (await readdir(base,{withFileTypes:true})).sort((a,b)=>a.name.localeCompare(b.name))){if(!entry.isDirectory()||entry.name.startsWith('.'))continue;const folder=path.join(base,entry.name);try{const checked=await validatePack(folder);checked.trusted=approvals[checked.manifest.id]===checked.contentHash;packs.push(checked);}catch(error){errors.push({folder,error:String(error)});}}return {packs,errors};}
export async function installPack({root,path:source,trust=false,entryPoint,assetDirectory}:{root:string;path:string;trust?:boolean;entryPoint?:string;assetDirectory?:string}){
 await installations.get(root)?.promise;let release!:()=>void;const promise=new Promise<void>(resolve=>{release=resolve;});installations.set(root,{promise});try{
 const pack=await validatePack(source);if(pack.codeFiles.length&&!trust)throw new Error('Executable TSX pack requires explicit --trust. Validation did not execute code.');
 const base=path.join(root,'.clipkit/assets/packs');await mkdir(base,{recursive:true});const destination=path.join(base,pack.manifest.id);const staging=path.join(base,`.staging-${randomUUID()}`);await cp(pack.folder,staging,{recursive:true,filter:file=>path.basename(file)!=='.trust.json'});
 if(pack.codeFiles.length)await writeFile(path.join(staging,'.trust.json'),JSON.stringify({contentHash:pack.contentHash,approvedAt:new Date().toISOString(),explicitTrust:true}));
 if(pack.codeFiles.length){const trustFile=path.join(root,'.clipkit/assets/trust.json');let approvals:Record<string,string>={};try{approvals=JSON.parse(await readFile(trustFile,'utf8'));}catch{}approvals[pack.manifest.id]=pack.contentHash;await writeFile(trustFile,JSON.stringify(approvals),{mode:0o600});}
 await rm(destination,{recursive:true,force:true});await cp(staging,destination,{recursive:true});await rm(staging,{recursive:true,force:true});
 try{const registry=await refreshPacks(root,true);
 if(pack.codeFiles.length){const {verifyAssetRegistry}=await import('../../render/src/verify-registry');await verifyAssetRegistry({root,registry,entryPoint,assetDirectory});}
 return registry;}catch(error){const trustFile=path.join(root,'.clipkit/assets/trust.json');try{const approvals=JSON.parse(await readFile(trustFile,'utf8'));delete approvals[pack.manifest.id];await writeFile(trustFile,JSON.stringify(approvals),{mode:0o600});}catch{}await rm(path.join(destination,'.trust.json'),{force:true});await refreshPacks(root,true);throw error;}
 }finally{installations.delete(root);release();}
}
export async function refreshPacks(root:string,skipInstallWait=false):Promise<AssetRegistry & {errors:Array<{folder:string;error:string}>;generatedDir:string}>{
 if(!skipInstallWait)await installations.get(root)?.promise;
 const discovery=await discoverPacks(root);const errors=[...discovery.errors];const clips:RuntimeClipDefinition[]=[];const imports:string[]=[];const mappings:string[]=[];const seen=new Set<string>();
 for(const pack of discovery.packs){if(pack.codeFiles.length&&!pack.trusted){errors.push({folder:pack.folder,error:'Executable files changed or lack explicit trust; reinstall with --trust'});continue;}for(const clip of pack.clips){if(seen.has(clip.id)){errors.push({folder:pack.folder,error:`Duplicate clip ${clip.id}`});continue;}seen.add(clip.id);clips.push(clip);if(clip.component){const alias=`Clip${imports.length}`;imports.push(`import ${alias} from ${JSON.stringify(safe(pack.folder,clip.component))};`);mappings.push(`${JSON.stringify(clip.id)}:${alias}`);}}}
 const generatedDir=path.join(root,'.clipkit/assets/generated');await mkdir(generatedDir,{recursive:true});
 const registrySource=`${imports.join('\n')}\nexport const components={${mappings.join(',')}};`;
 await writeFile(path.join(generatedDir,'registry.tsx'),registrySource);
 const bridge=(key:string,module:Record<string,unknown>)=>`const runtime=globalThis.__CLIPKIT_PACK_RUNTIME__.${key};\nexport default runtime;\n${Object.keys(module).filter(k=>k!=='default'&&/^[A-Za-z_$][\w$]*$/.test(k)).map(k=>`export const ${k}=runtime[${JSON.stringify(k)}];`).join('\n')}`;
 await build({stdin:{contents:registrySource,resolveDir:generatedDir,loader:'tsx'},bundle:true,format:'iife',globalName:'ClipKitPackRegistry',platform:'browser',target:'es2022',loader:{'.png':'dataurl','.jpg':'dataurl','.jpeg':'dataurl','.webp':'dataurl','.svg':'dataurl','.woff2':'dataurl','.ttf':'dataurl'},outfile:path.join(generatedDir,'registry.js'),plugins:[{name:'shared-runtime',setup(builder){builder.onResolve({filter:/^(react(?:\/jsx(?:-dev)?-runtime)?|remotion)$/},args=>({path:args.path,namespace:'runtime'}));builder.onResolve({filter:/.*/},args=>{if(args.namespace==='runtime')return;if(!args.path.startsWith('.')&&!path.isAbsolute(args.path))throw new Error('Trusted pack imports are limited to local files, react and remotion');const target=path.resolve(args.resolveDir,args.path);const base=path.join(root,'.clipkit/assets/packs');const own=args.importer.startsWith(base+path.sep)?path.join(base,path.relative(base,args.importer).split(path.sep)[0]):base;if(!target.startsWith(own+path.sep))throw new Error('Trusted component import escapes its installed pack folder');if(/\.(css|scss)$/.test(target))throw new Error('Trusted components currently require inline styles; CSS imports are not supported');return undefined;});builder.onLoad({filter:/.*/,namespace:'runtime'},args=>({contents:args.path==='remotion'?bridge('remotion',Remotion):args.path==='react/jsx-runtime'||args.path==='react/jsx-dev-runtime'?"export const jsx=globalThis.__CLIPKIT_PACK_RUNTIME__.jsx;export const jsxs=jsx;export const jsxDEV=jsx;export const Fragment=globalThis.__CLIPKIT_PACK_RUNTIME__.react.Fragment;":bridge('react',React),loader:'js'}));}}]});
 const version=hash(Buffer.concat([await readFile(path.join(generatedDir,'registry.js')),Buffer.from(JSON.stringify(clips))])).slice(0,16);
 await cp(path.join(generatedDir,'registry.js'),path.join(generatedDir,`registry-${version}.js`));
 await writeFile(path.join(generatedDir,'registry.json'),JSON.stringify({clips,version},null,2));
 await writeFile(path.join(generatedDir,'catalog.jsonl'),clips.map(({id,name,category,description,durationSec,tags,aspect,whenToUse,license,previewPath})=>JSON.stringify({id,name,category,description,durationSec,tags,aspect,whenToUse,license,previewPath})).join('\n')+'\n');
 const categories:Record<string,string[]>={};for(const c of clips)(categories[c.category]??=[]).push(c.id);await writeFile(path.join(generatedDir,'categories.json'),JSON.stringify(categories,null,2));
 const schemas=path.join(generatedDir,'schemas');await rm(schemas,{recursive:true,force:true});await mkdir(schemas);for(const c of clips)await writeFile(path.join(schemas,`${c.id}.json`),JSON.stringify({clipId:c.id,defaults:c.defaults,fields:c.fields,propsSchema:c.propsSchema},null,2));
 return {clips,version,moduleUrl:clips.some(c=>c.component)?`/asset-registry/registry.js?version=${version}`:undefined,errors,generatedDir};
}
export async function watchPacks(root:string,onChange:(registry:Awaited<ReturnType<typeof refreshPacks>>)=>void){const folder=path.join(root,'.clipkit/assets/packs');await mkdir(folder,{recursive:true});let pending:ReturnType<typeof setTimeout>|undefined;let running=false,dirty=false,closed=false;const refresh=async()=>{if(closed)return;if(running){dirty=true;return;}running=true;try{do{dirty=false;onChange(await refreshPacks(root));}while(dirty&&!closed);}finally{running=false;}};const watcher=watch(folder,{recursive:true},()=>{dirty=true;clearTimeout(pending);pending=setTimeout(()=>void refresh(),150);});return {close(){closed=true;clearTimeout(pending);watcher.close();}};}
