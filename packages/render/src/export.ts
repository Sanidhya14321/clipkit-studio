import {bundle} from '@remotion/bundler';
import {renderMedia,selectComposition} from '@remotion/renderer';
import {mkdir,writeFile,readFile} from 'node:fs/promises';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import {createHash,randomUUID} from 'node:crypto';
import type {Project} from '../../core/src/index';
const bundles=new Map<string,Promise<string>>();
export async function renderProject({project,root,mediaBaseUrl,assetBaseUrl,accessToken,onProgress,entryPoint,assetDirectory}:{project:Project;root:string;mediaBaseUrl:string;assetBaseUrl:string;accessToken?:string;entryPoint?:string;assetDirectory?:string;onProgress?:(progress:number)=>void}):Promise<{path:string;creditsPath:string}>{
 const entry=entryPoint??fileURLToPath(new URL('./entry.tsx',import.meta.url));
 const entryDirectory=path.dirname(entry);
 const key=entry;
 if(!bundles.has(key))bundles.set(key,bundle({entryPoint:entry,webpackOverride:config=>({...config,resolve:{...config.resolve,alias:{...config.resolve?.alias,'@clipkit/core':path.resolve(entryDirectory,'../../core/src/index.ts'),'@clipkit/assets':path.resolve(entryDirectory,'../../assets/src/index.ts')}}})}).catch(error=>{bundles.delete(key);throw error;}));
 const serveUrl=await bundles.get(key)!;
 const inputProps={project,mediaBaseUrl,assetBaseUrl,accessToken,preview:false};
 const composition=await selectComposition({serveUrl,id:'ClipKit',inputProps,chromiumOptions:{disableWebSecurity:true}});
 const directory=path.join(root,'.clipkit','exports');await mkdir(directory,{recursive:true});
 const stem=`${project.id.replace(/[^a-zA-Z0-9_-]/g,'_')}-r${project.rev}-${randomUUID().slice(0,8)}`;const output=path.join(directory,`${stem}.mp4`);const creditsPath=path.join(directory,`${stem}.credits.json`);
 await renderMedia({serveUrl,composition,inputProps,codec:'h264',audioCodec:'aac',enforceAudioTrack:true,pixelFormat:'yuv420p',outputLocation:output,concurrency:2,chromiumOptions:{disableWebSecurity:true},onProgress:({progress})=>onProgress?.(progress)});
 let vendors:unknown=[];try{vendors=JSON.parse(await readFile(path.join(assetDirectory??path.join(root,'public'),'vendor/manifest.json'),'utf8'));}catch{}
 await writeFile(creditsPath,JSON.stringify({projectId:project.id,revision:project.rev,projectHash:createHash('sha256').update(JSON.stringify(project)).digest('hex'),renderedAt:new Date().toISOString(),sources:Object.values(project.sources).map(s=>({id:s.id,name:s.name,provenance:s.provenance??'User supplied media'})),vendors},null,2),{flag:'wx'});
 return {path:output,creditsPath};
}
