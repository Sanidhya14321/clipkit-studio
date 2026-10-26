import {mkdir,writeFile,readFile} from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {z} from 'zod';
import {catalog,clipSchemas,validateClipProps} from './index';
const editableKeys=['text','color','size','x','y','opacity','revealDuration'];
/** Only this compiled registry can provide executable components. Pack JSON is metadata. */
export const nativePackManifest={format:'clipkit.asset-pack/v1',id:'clipkit-native',version:'1.1.0',license:'MIT',runtime:{kind:'compiled',registry:'clipkit/native-v1'},editableContract:{layerAttribute:'data-ck-layer',propertyAttribute:'data-ck-path',elementAttribute:'data-ck-element',requiredProps:editableKeys},clips:catalog.map(a=>({clipId:a.id,license:a.license,defaults:a.defaults,fields:a.fields,previewPath:a.previewPath,schemaPath:`schemas/${a.id}.json`}))};
/** Build deterministic metadata from the trusted in-process native registry; never load pack code. */
export async function buildAssets(root:string,outputDir=path.join(root,'public','packs','native')){
 const previewManifest=JSON.parse(await readFile(path.join(root,'public/previews/manifest.json'),'utf8')) as Array<{clipId:string;sha256:string;defaultsSha256:string}>;
 const ids=new Set<string>();const categories:Record<string,string[]>={};
 const rows:Array<Record<string,unknown>>=[];const schemas:Array<{id:string;value:unknown}>=[];
 for(const asset of catalog){
  if(ids.has(asset.id)||!/^[a-z0-9-]+$/.test(asset.id))throw new Error(`Invalid or duplicate native clip ID: ${asset.id}`);ids.add(asset.id);
  if(!asset.license?.trim())throw new Error(`${asset.id}: license is required`);
  const parsed=validateClipProps(asset.id,asset.defaults);
  for(const key of editableKeys)if(!(key in parsed)||!asset.fields.some(f=>f.key===key))throw new Error(`${asset.id}: missing editable property ${key}`);
  for(const field of asset.fields){
   if(!(field.key in parsed))throw new Error(`${asset.id}: field ${field.key} has no default`);
   if(field.type==='number'&&typeof parsed[field.key]!=='number')throw new Error(`${asset.id}: numeric field ${field.key} has wrong default`);
   if(field.type!=='number'&&typeof parsed[field.key]!=='string')throw new Error(`${asset.id}: text field ${field.key} has wrong default`);
  }
  const preview=await readFile(path.join(root,'public',asset.previewPath.slice(1)));
  if(!preview.subarray(0,8).equals(Buffer.from([137,80,78,71,13,10,26,10])))throw new Error(`${asset.id}: missing or invalid PNG preview`);
  const proof=previewManifest.find(p=>p.clipId===asset.id);
  if(!proof||proof.sha256!==createHash('sha256').update(preview).digest('hex')||proof.defaultsSha256!==createHash('sha256').update(JSON.stringify(asset.defaults)).digest('hex'))throw new Error(`${asset.id}: thumbnail proof does not match current default props`);
  categories[asset.category]??=[];categories[asset.category].push(asset.id);
  const schemaPath=`schemas/${asset.id}.json`;
  rows.push({id:asset.id,name:asset.name,category:asset.category,description:asset.description,durationSec:asset.durationSec,tags:asset.tags,aspect:asset.aspect,whenToUse:asset.whenToUse,license:asset.license,previewPath:asset.previewPath,previewSha256:createHash('sha256').update(preview).digest('hex'),schemaPath});
  const propsSchema=z.toJSONSchema(clipSchemas[asset.id]);
  for(const [key,value] of Object.entries(parsed)){const property=propsSchema.properties?.[key];if(property&&typeof property==='object')property.default=value;}
  schemas.push({id:asset.id,value:{clipId:asset.id,defaults:parsed,fields:asset.fields,propsSchema,editableContract:nativePackManifest.editableContract}});
 }
 await mkdir(path.join(outputDir,'schemas'),{recursive:true});
 for(const schema of schemas)await writeFile(path.join(outputDir,'schemas',`${schema.id}.json`),JSON.stringify(schema.value,null,2)+'\n');
 await writeFile(path.join(outputDir,'catalog.jsonl'),rows.map(row=>JSON.stringify(row)).join('\n')+'\n');
 await writeFile(path.join(outputDir,'categories.json'),JSON.stringify(categories,null,2)+'\n');
 await writeFile(path.join(outputDir,'pack.json'),JSON.stringify(nativePackManifest,null,2)+'\n');
 return {packId:nativePackManifest.id,count:rows.length,outputDir:path.resolve(outputDir),catalogPath:path.join(outputDir,'catalog.jsonl'),packPath:path.join(outputDir,'pack.json')};
}
