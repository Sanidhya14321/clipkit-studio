import {readFile,readdir,writeFile,mkdir,stat} from 'node:fs/promises';
import path from 'node:path';
import {catalog} from './index.ts';
import {parseCard,digest,type AssetCard,type CardEntry} from './cards.ts';

export const renderSources=['packages/render/src/MotionClips.tsx','packages/render/src/motion-math.ts','scripts/verify-assets.ts','packages/assets/src/check.ts','packages/assets/src/index.ts','packages/core/src/index.ts','packages/render/src/Composition.tsx','packages/render/src/external.tsx','packages/render/src/audio-gain.ts'];
export async function nativeRenderDigest(root:string){const manifest=JSON.parse(await readFile(path.join(root,'package.json'),'utf8'));return digest(JSON.stringify({versions:Object.fromEntries(['react','react-dom','remotion','@remotion/renderer'].map(name=>[name,manifest.dependencies[name]])),vendorFiles:await Promise.all((JSON.parse(await readFile(path.join(root,'public/vendor/manifest.json'),'utf8')) as Array<{file:string}>).map(async entry=>[entry.file,digest(await readFile(path.join(root,'public/vendor',entry.file)))])),files:await Promise.all(renderSources.map(async file=>[file,digest(await readFile(path.join(root,file)))]))}));}
const allowedLicenses=new Set(['MIT','ISC','ISC AND MIT','BSD-2-Clause','BSD-3-Clause','Apache-2.0','Zlib','Unlicense','CC0-1.0','CC-BY-4.0','OFL-1.1']);
const caps:Record<string,number>={'.json':150000,'.png':500000,'.jpg':500000,'.jpeg':500000,'.webp':500000,'.svg':500000,'.glb':2000000,'.hdr':4000000,'.woff2':150000,'.ttf':300000,'.wav':10000000,'.mp3':5000000,'.m4a':5000000,'.mp4':15000000,'.webm':15000000,'.cube':1000000};
async function walk(directory:string):Promise<string[]>{let entries;try{entries=await readdir(directory,{withFileTypes:true})}catch(e){if((e as NodeJS.ErrnoException).code==='ENOENT')return [];throw e;}const result:string[]=[];for(const entry of entries.sort((a,b)=>a.name.localeCompare(b.name))){const full=path.join(directory,entry.name);if(entry.isSymbolicLink())throw new Error(`Asset symlink is not allowed: ${full}`);if(entry.isDirectory())result.push(...await walk(full));else result.push(full);}return result;}
export type CheckOptions={checkOnly?:boolean;requireEvidence?:boolean;now?:number};
export async function checkAssets(root:string,options:CheckOptions={}){
 const errors:string[]=[],warnings:string[]=[],entries:CardEntry[]=[];const cards=new Map<string,{card:AssetCard;markdown:string;file:string;bytes:number}>();const pendingWrites=new Map<string,string>();
 const files=[...(await walk(path.join(root,'packages/assets/cards'))).filter(file=>path.basename(file)==='card.md'),...(await walk(path.join(root,'public/vendor'))).filter(file=>/\.(?:ttf|woff2|svg|wav|png|jpe?g|webp)\.md$/.test(file)),...(await walk(path.join(root,'public/previews'))).filter(file=>file.endsWith('.png.md')),...(await walk(path.join(root,'public/assets'))).filter(file=>file.endsWith('.md'))];
 let exceptions:Record<string,{sha256:string;maxBytes:number;reason:string}>={};try{exceptions=JSON.parse(await readFile(path.join(root,'packages/assets/budget-exceptions.json'),'utf8'))}catch{}
 let proof:any;try{proof=JSON.parse(await readFile(path.join(root,'public/packs/native/determinism.json'),'utf8'))}catch{}
 const sourceDigest=await nativeRenderDigest(root);let publicBytes=0;
 for(const file of files){try{
  const markdown=await readFile(file,'utf8');const {card,summary}=parseCard(markdown);if(cards.has(card.id))throw new Error(`Duplicate asset ID ${card.id}`);
  if(!allowedLicenses.has(card.license))throw new Error(`License is not in the asset allowlist: ${card.license}`);
  let bytes=0;
  if(card['media-path']){
   const relative=card['media-path'];if(!relative.startsWith('public/')||relative.split('/').includes('..'))throw new Error('Media path must stay inside public/');
   const full=path.join(root,relative);const data=await readFile(full);bytes=data.byteLength;publicBytes+=bytes;
   if(!card.sha256||digest(data)!==card.sha256)throw new Error('Media checksum changed; reverify provenance and update its card');
   const extension=path.extname(full).toLowerCase(),cap=card.kind==='audio'&&card.tags.includes('sfx')?300000:caps[extension];if(!cap)throw new Error(`No size budget for ${extension}`);
   if(bytes>cap){const exception=exceptions[relative];if(!exception||exception.sha256!==card.sha256||bytes>exception.maxBytes)throw new Error(`Size ${bytes} exceeds ${cap} byte budget`);warnings.push(`${card.id}: legacy size exception (${bytes} bytes): ${exception.reason}`);}
  }
  if(card['provenance-status']==='human-attributed'&&(!card['provenance-evidence'].length||!card['license-file']))throw new Error('Human attribution requires source evidence and a local license file');
  if(card['license-file'])await stat(path.join(root,card['license-file']));
  if(card['schema-path'])await stat(path.join(root,card['schema-path']));
  for(const file of card['source-files']){if(file.startsWith('/')||file.split('/').includes('..'))throw new Error('Source file escapes project');await stat(path.join(root,file));}
  if(catalog.some(asset=>asset.id===card.id)){
   const asset=catalog.find(asset=>asset.id===card.id)!;const tested=proof?.assets?.find((entry:any)=>entry.id===card.id);const expectedFrames=[0,Math.floor(asset.durationSec*30/2),Math.ceil(asset.durationSec*30)-1];
   const valid=proof?.sourceDigest===sourceDigest&&tested?.defaultsHash===digest(JSON.stringify(asset.defaults))&&expectedFrames.every(frame=>tested?.frames?.some((f:any)=>f.frame===frame&&/^[a-f0-9]{64}$/.test(f.first)&&f.first===f.second));
   if(card.tested.status==='render-verified'&&!valid)throw new Error('Render evidence is stale or missing; run assets:verify');
   if(options.requireEvidence!==false&&(!valid||card.tested.status!=='render-verified'))throw new Error('Native asset has not passed current start/mid/end determinism checks');
  }
  cards.set(card.id,{card,markdown,file,bytes});const internal=!file.startsWith(path.join(root,'public')+path.sep);const cardPath=internal?`packs/native/cards/${card.id}.md`:path.relative(path.join(root,'public'),file).split(path.sep).join('/');
  entries.push({id:card.id,name:card.name,kind:card.kind,summary,duration:card.duration,tags:card.tags,mood:card.mood,bestFor:card['best-for'],avoid:card.avoid,provenanceStatus:card['provenance-status'],cardPath,sha256:digest(markdown),schemaPath:card['schema-path']});
  if(internal)pendingWrites.set(path.join(root,'public',cardPath),markdown);
 }catch(e){errors.push(`${path.relative(root,file)}: ${(e as Error).message}`);}}
 for(const asset of catalog)if(!cards.has(asset.id))errors.push(`Native clip ${asset.id} is missing a valid card`);
 for(const directory of ['public/vendor','public/previews','public/assets'])for(const file of await walk(path.join(root,directory))){const relative=path.relative(root,file).split(path.sep).join('/');if(relative.includes('/references/')||file.endsWith('.md')||/(?:^|-)LICENSE(?:\.txt)?$/i.test(path.basename(file))||['manifest.json','source-audit.json'].includes(path.basename(file)))continue;if(![...cards.values()].some(({card})=>card['media-path']===relative))errors.push(`Uncarded asset file ${relative}`);}
 const usage=JSON.parse(await readFile(path.join(root,'packages/assets/usage.json'),'utf8')) as Array<{scene:string;file:string;assets:string[]}>;const referenced=new Set<string>();for(const scene of usage){try{await stat(path.join(root,scene.file))}catch{errors.push(`Usage scene ${scene.scene} is missing ${scene.file}`);}for(const id of scene.assets){if(!cards.has(id))errors.push(`Scene ${scene.scene} references unknown card ${id}`);referenced.add(id);}}
 const follow=(id:string,visited=new Set<string>())=>{if(visited.has(id))return;visited.add(id);for(const used of cards.get(id)?.card.uses??[]){if(!cards.has(used))errors.push(`${id} references unknown dependency ${used}`);else{referenced.add(used);follow(used,visited);}}};for(const id of [...referenced])follow(id);
 for(const [id,{file}] of cards)if(!referenced.has(id)){const age=(options.now??Date.now())-(await stat(file)).mtimeMs;warnings.push(`${id}: ${age>=48*3600000?'unreferenced candidate (card age at least 48 hours)':'not yet used in a registered scene'}; no file was deleted`);}
 if(publicBytes>60000000)errors.push(`Carded public asset total ${publicBytes} exceeds 60MB`);
 for(const source of ['packages/render/src/Composition.tsx','packages/render/src/MotionClips.tsx','packages/render/src/motion-math.ts']){const code=await readFile(path.join(root,source),'utf8');if(/\b(?:Math\.random|Date\.now|setTimeout|setInterval|requestAnimationFrame)\s*\(|new\s+Date\s*\(/.test(code))errors.push(`Wall-clock/random animation in ${source}`);}
 entries.sort((a,b)=>a.kind.localeCompare(b.kind)||a.id.localeCompare(b.id));
 const indexPath=path.join(root,'packages/assets/cards/_INDEX.md');const index=await readFile(indexPath,'utf8');if(!/<!-- AUTO:BEGIN -->[\s\S]*<!-- AUTO:END -->/.test(index))errors.push('Asset index is missing AUTO markers');
 const table=['| Asset | Kind | Media bytes | License | Provenance | Tags |','|---|---|---:|---|---|---|',...entries.map(entry=>{const {card,bytes}=cards.get(entry.id)!;return `| ${entry.id} | ${entry.kind} | ${card['media-path']?bytes:'shared code'} | ${card.license} | ${entry.provenanceStatus} | ${entry.tags.join(', ')} |`;})].join('\n');
 pendingWrites.set(indexPath,index.replace(/<!-- AUTO:BEGIN -->[\s\S]*?<!-- AUTO:END -->/,`<!-- AUTO:BEGIN -->\n${table}\n<!-- AUTO:END -->`));pendingWrites.set(path.join(root,'public/packs/native/asset-cards.json'),JSON.stringify(entries,null,2)+'\n');
 if(!errors.length)for(const [file,contents] of pendingWrites){if(options.checkOnly){let old='';try{old=await readFile(file,'utf8')}catch{}if(old!==contents)errors.push(`Generated inventory is stale: ${path.relative(root,file)}`);}else{await mkdir(path.dirname(file),{recursive:true});await writeFile(file,contents);}}
 return {passed:!errors.length,errors,warnings,cardCount:cards.size,publicBytes,sourceDigest};
}
