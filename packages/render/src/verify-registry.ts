import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {createHash} from 'node:crypto';
import {renderStill,selectComposition,openBrowser} from '@remotion/renderer';
import {getRenderBundle} from './export';
import {emptyProject} from '../../core/src/index';
import type {AssetRegistry} from '../../assets/src/runtime';
/** Executes only explicitly trusted compiled components, then checks actual tags and frame determinism. */
export async function verifyAssetRegistry({root,registry,entryPoint,assetDirectory}:{root:string;registry:AssetRegistry;entryPoint?:string;assetDirectory?:string}){
 const server=createServer(async(req,res)=>{try{const name=new URL(req.url!,'http://localhost').pathname;let file:string;if(name.startsWith('/asset-registry/'))file=path.join(root,'.clipkit/assets/generated',path.basename(name));else if(name.startsWith('/asset-packs/'))file=path.join(root,'.clipkit/assets/packs',name.slice('/asset-packs/'.length));else file=path.join(assetDirectory??path.join(root,'public'),name);const bytes=await readFile(file);res.setHeader('Access-Control-Allow-Origin','*');res.setHeader('Content-Type',file.endsWith('.js')?'text/javascript':file.endsWith('.svg')?'image/svg+xml':file.endsWith('.ttf')?'font/ttf':'application/octet-stream');res.end(bytes);}catch{res.statusCode=404;res.end();}});server.listen(0,'127.0.0.1');await new Promise<void>(resolve=>server.once('listening',resolve));const base=`http://127.0.0.1:${(server.address() as {port:number}).port}`;
 const browser=await openBrowser('chrome');
 // Remotion's error callback starts page.close without awaiting it. Drain those
 // protocol requests before closing Chromium, including invalid-pack failures.
 const pendingCloses:Promise<void>[]=[];const newPage=browser.newPage.bind(browser);browser.newPage=async options=>{const page=await newPage(options);const close=page.close.bind(page);let closing:Promise<void>|undefined;page.close=options=>{if(!closing){closing=close(options);pendingCloses.push(closing);void closing.catch(()=>{});}return closing;};return page;};
 const results:Array<{id:string;hash:string}>=[];
 try{const serveUrl=await getRenderBundle(entryPoint);for(const clip of registry.clips.filter(c=>c.component)){
  const project=emptyProject();project.tracks[1].items.push({id:`verify-${clip.id}`,type:'clip',clipId:clip.id,at:0,dur:clip.durationSec,srcIn:0,volume:1,props:clip.defaults});const inputProps={project,mediaBaseUrl:base,assetBaseUrl:base,preview:false,assetRegistry:{...registry,moduleUrl:registry.moduleUrl}};const composition=await selectComposition({serveUrl,id:'ClipKit',inputProps,puppeteerInstance:browser});let previous='';for(let i=0;i<2;i++){const {buffer}=await renderStill({serveUrl,composition,inputProps,frame:Math.min(20,Math.floor(clip.durationSec*30)-1),imageFormat:'png',puppeteerInstance:browser});if(!buffer)throw new Error('Missing rendered frame');const digest=createHash('sha256').update(buffer).digest('hex');if(previous&&previous!==digest)throw new Error(`${clip.id}: same-frame renders differ; deterministic contract failed`);previous=digest;}results.push({id:clip.id,hash:previous});}
 return results;}finally{await Promise.allSettled(pendingCloses);await browser.close({silent:true});server.close();}
}
