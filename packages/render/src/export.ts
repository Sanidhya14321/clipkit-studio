import {debugEvent} from '../../debug/src/trace';
import {enforceExportLicenses} from '../../assets/src/export-policy';
import {bundle} from '@remotion/bundler';
import {renderMedia,selectComposition} from '@remotion/renderer';
import {mkdir,writeFile,readFile} from 'node:fs/promises';
import path from 'node:path';
import {homedir} from 'node:os';
import {fileURLToPath} from 'node:url';
import {createHash,randomUUID} from 'node:crypto';
import {exportLicenses} from './licenses';
import type {AssetRegistry} from '../../assets/src/runtime';
import type {Project} from '../../core/src/index';
const bundles=new Map<string,Promise<string>>();
export async function getRenderBundle(entryPoint?:string){
 const entry=entryPoint??fileURLToPath(new URL('./entry.tsx',import.meta.url));
 const entryDirectory=path.dirname(entry);
 const key=entry;
 if(!bundles.has(key))bundles.set(key,bundle({entryPoint:entry,webpackOverride:config=>({...config,cache:config.cache&&typeof config.cache==='object'?{...config.cache,cacheDirectory:path.join(process.env.XDG_CACHE_HOME??path.join(homedir(),'.cache'),'clipkit','webpack')}:config.cache,resolve:{...config.resolve,alias:{...config.resolve?.alias,'@clipkit/core':path.resolve(entryDirectory,'../../core/src/index.ts'),'@clipkit/assets':path.resolve(entryDirectory,'../../assets/src/index.ts')}}})}).catch(error=>{bundles.delete(key);throw error;}));
 return await bundles.get(key)!;
}
export async function renderProject({project,root,mediaBaseUrl,assetBaseUrl,accessToken,onProgress,entryPoint,assetDirectory,assetRegistry,preset,quality,frameRange,layerCache}:{layerCache?:Record<string,{url:string;hash:string}>;project:Project;root:string;mediaBaseUrl:string;assetBaseUrl:string;assetRegistry?:AssetRegistry;frameRange?:[number,number];preset?:'portrait'|'landscape'|'square'|{width:number;height:number;crf?:number};quality?:number;accessToken?:string;entryPoint?:string;assetDirectory?:string;onProgress?:(progress:number)=>void}):Promise<{path:string;creditsPath:string}>{
 const renderStarted=Date.now();debugEvent('render.start',{component:'render',rev:project.rev});enforceExportLicenses(project,assetRegistry);project=structuredClone(project);const designSize={width:project.settings.width,height:project.settings.height};
 const presets={portrait:{width:720,height:1280},landscape:{width:1280,height:720},square:{width:1080,height:1080}};const dimensions=typeof preset==='string'?presets[preset]:preset;
 if(dimensions){if(!Number.isInteger(dimensions.width)||!Number.isInteger(dimensions.height)||dimensions.width<64||dimensions.height<64||dimensions.width>3840||dimensions.height>3840||dimensions.width%2||dimensions.height%2)throw new Error('Export dimensions must be even integers between64and3840');project.settings={...project.settings,width:dimensions.width,height:dimensions.height};}
 const crf=quality??(typeof preset==='object'?preset.crf:undefined)??18;if(!Number.isInteger(crf)||crf<0||crf>51)throw new Error('CRF must be0–51');
 const serveUrl=await getRenderBundle(entryPoint);
 const inputProps={project,mediaBaseUrl,assetBaseUrl,accessToken,preview:false,assetRegistry,designSize,layerCache};
 const composition=await selectComposition({serveUrl,id:'ClipKit',inputProps,chromiumOptions:{disableWebSecurity:true}});
 const directory=path.join(root,'.clipkit','exports');await mkdir(directory,{recursive:true});
 const stem=`${project.id.replace(/[^a-zA-Z0-9_-]/g,'_')}-r${project.rev}-${randomUUID().slice(0,8)}`;const output=path.join(directory,`${stem}.mp4`);const creditsPath=path.join(directory,`${stem}.credits.json`);
 await renderMedia({serveUrl,composition,inputProps,frameRange,codec:'h264',crf,audioCodec:'aac',enforceAudioTrack:true,pixelFormat:'yuv420p',outputLocation:output,concurrency:2,chromiumOptions:{disableWebSecurity:true},onProgress:({progress})=>onProgress?.(progress)});
 const licenses=await exportLicenses(project,assetDirectory??path.join(root,'public'),assetRegistry);
 await writeFile(creditsPath,JSON.stringify({projectId:project.id,revision:project.rev,projectHash:createHash('sha256').update(JSON.stringify(project)).digest('hex'),renderedAt:new Date().toISOString(),settings:project.settings,crf,licenses},null,2),{flag:'wx'});
 debugEvent('render.end',{component:'render',rev:project.rev,duration:Date.now()-renderStarted,resultCode:'OK'});return {path:output,creditsPath};
}
