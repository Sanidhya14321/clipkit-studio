import {compareDecodedVideo} from './compare-decoded';
/** Real Chromium evidence; never labels unexecuted checks as passed. */
import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,writeFile,mkdir,readdir} from 'node:fs/promises';
import {promisify} from 'node:util';
import {execFile} from 'node:child_process';
import path from 'node:path';
import {parseDocument} from 'yaml';
import {openBrowser,renderStill,selectComposition} from '@remotion/renderer';
import {catalog,makeClip} from '../packages/assets/src/index.ts';
import {emptyProject} from '../packages/core/src/index.ts';
import {nativeRenderDigest,checkAssets} from '../packages/assets/src/check.ts';
import {digest,parseCard} from '../packages/assets/src/cards.ts';
import {getRenderBundle,renderProject} from '../packages/render/src/export.ts';
import {ffmpegBinary,probe} from '../packages/server/src/media.ts';
const root=path.resolve(import.meta.dirname,'..'),out=path.join(root,'.clipkit/asset-verification');await mkdir(out,{recursive:true});
const sourceDigest=await nativeRenderDigest(root);const server=createServer(async(req,res)=>{try{const name=new URL(req.url!,'http://localhost').pathname;const file=path.resolve(root,'public','.'+(name==='/media/smoke-ui-click'?'/vendor/ui-click.wav':name));if(!file.startsWith(path.join(root,'public')+path.sep))throw new Error('Invalid asset path');const data=await readFile(file);res.setHeader('Access-Control-Allow-Origin','*');res.setHeader('Content-Type',file.endsWith('.svg')?'image/svg+xml':file.endsWith('.ttf')?'font/ttf':file.endsWith('.wav')?'audio/wav':'application/octet-stream');res.end(data);}catch{res.statusCode=404;res.end();}});
server.listen(0,'127.0.0.1');await new Promise<void>(resolve=>server.once('listening',resolve));const base=`http://127.0.0.1:${(server.address() as {port:number}).port}`;
let browser:Awaited<ReturnType<typeof openBrowser>>|undefined;const assets:Array<{id:string;defaultsHash:string;frames:Array<{frame:number;first:string;second:string}>;previewHash:string}>=[];
const locateCard=async(id:string)=>{for(const kind of await readdir(path.join(root,'packages/assets/cards'),{withFileTypes:true}))if(kind.isDirectory()){const file=path.join(root,'packages/assets/cards',kind.name,id,'card.md');try{await readFile(file);return file;}catch{}}throw new Error(`Missing native card ${id}`);};
const updateCard=async(file:string,updates:Record<string,unknown>)=>{const markdown=await readFile(file,'utf8');parseCard(markdown);const match=markdown.match(/^---\r?\n([\s\S]*?)\r?\n---([\s\S]*)$/)!;const document=parseDocument(match[1]);for(const [key,value]of Object.entries(updates))document.set(key,value);const result='---\n'+document.toString({lineWidth:0})+'---'+match[2];parseCard(result);await writeFile(file,result);};
try{
 const serveUrl=await getRenderBundle();browser=await openBrowser('chrome');const previews=[];
 for(const asset of catalog){
  const project=emptyProject();project.tracks[1].items.push(makeClip(asset.id));const inputProps={project,assetBaseUrl:base,mediaBaseUrl:`${base}/media`,preview:false};const composition=await selectComposition({serveUrl,id:'ClipKit',inputProps,puppeteerInstance:browser});const frames=[];
  for(const frame of [0,Math.floor(asset.durationSec*30/2),Math.ceil(asset.durationSec*30)-1]){const hashes=[];for(let pass=0;pass<2;pass++){const {buffer}=await renderStill({serveUrl,composition,inputProps,frame,puppeteerInstance:browser,imageFormat:'png'});assert(buffer,'Missing rendered PNG');hashes.push(digest(buffer));}assert.equal(hashes[0],hashes[1],`${asset.id} at frame ${frame} is not deterministic`);frames.push({frame,first:hashes[0],second:hashes[1]});}
  const previewPath=path.join(root,'public/previews',`${asset.id}.png`);const frame=asset.id==='wipe'?18:asset.id.startsWith('adapted-')||asset.id.startsWith('lottie-')?75:45;await renderStill({serveUrl,composition,inputProps,frame,puppeteerInstance:browser,imageFormat:'png',scale:.4,output:previewPath});const previewHash=digest(await readFile(previewPath));
  assets.push({id:asset.id,defaultsHash:digest(JSON.stringify(asset.defaults)),frames,previewHash});previews.push({clipId:asset.id,frame,sha256:previewHash,defaultsSha256:digest(JSON.stringify(asset.defaults))});console.log(`PASS ${asset.id}: start/middle/end twice; preview regenerated`);
 }
 let fullSmoke:unknown={status:'not-run',reason:'Use --full to execute the two complete video renders.'};
 if(process.argv.includes('--full')){
  const project=emptyProject();project.name='ClipKit asset smoke — native synthetic graphics';project.tracks[1].items=catalog.map((asset,index)=>({...makeClip(asset.id,index*3),dur:3}));
  const clickPath=path.join(root,'public/vendor/ui-click.wav');const click=await probe(clickPath);project.sources['smoke-ui-click']={id:'smoke-ui-click',kind:'audio',name:'Kenney UI click',path:clickPath,durationSec:click.durationSec,provenance:'Kenney Vleugels; CC0; pinned vendor card'};project.tracks.find(track=>track.kind==='audio')!.items.push({id:'smoke-click-item',type:'audio',src:'smoke-ui-click',at:3,dur:Math.max(1/30,Math.floor(click.durationSec*30)/30),srcIn:0,volume:.35,props:{}});
  const outputs=[];for(let pass=0;pass<2;pass++)outputs.push(await renderProject({project,root,assetBaseUrl:base,mediaBaseUrl:`${base}/media`,assetDirectory:path.join(root,'public'),preset:{width:360,height:640},quality:20}));
  const ffmpeg=await ffmpegBinary();const videoHashes=[];const audioHashes=[];for(const output of outputs){videoHashes.push((await promisify(execFile)(ffmpeg,['-v','error','-i',output.path,'-map','0:v:0','-f','framemd5','-'],{maxBuffer:10*1024*1024})).stdout);audioHashes.push((await promisify(execFile)(ffmpeg,['-v','error','-i',output.path,'-map','0:a:0','-f','md5','-'])).stdout);}
  const videoComparison=await compareDecodedVideo(ffmpeg,outputs.map(output=>output.path),videoHashes);assert.equal(audioHashes[0],audioHashes[1],'Full smoke decoded audio differs');fullSmoke={status:'passed',...videoComparison,durationSec:catalog.length*3,width:360,height:640,videoDigest:digest(videoHashes[0]),audioDigest:digest(audioHashes[0]),audioComparison:'Exact decoded audio hashes; container metadata excluded'};await writeFile(path.join(out,'full-smoke.json'),JSON.stringify({outputs,fullSmoke},null,2));
 }
 if(await nativeRenderDigest(root)!==sourceDigest)throw new Error('Asset sources changed during verification; no evidence recorded');
 const proof={format:'clipkit.asset-proof/v1',sourceDigest,environment:{platform:process.platform,arch:process.arch,node:process.version,renderer:'Local Chromium; no second-machine claim'},assets,fullSmoke};
 for(const asset of assets){await updateCard(await locateCard(asset.id),{tested:{status:'render-verified',method:'Start/middle/end rendered twice with equal PNG hashes on local Chromium.',evidence:'public/packs/native/determinism.json'}});await updateCard(path.join(root,'public/previews',`${asset.id}.png.md`),{sha256:asset.previewHash,perf:`Static PNG thumbnail, ${(await readFile(path.join(root,'public/previews',`${asset.id}.png`))).byteLength} bytes.`,tested:{status:'integrity-verified',method:'Preview regenerated with the current native render proof.',evidence:'public/packs/native/determinism.json'}});}
 await writeFile(path.join(root,'public/previews/manifest.json'),JSON.stringify(previews,null,2)+'\n');await mkdir(path.join(root,'public/packs/native'),{recursive:true});await writeFile(path.join(root,'public/packs/native/determinism.json'),JSON.stringify(proof,null,2)+'\n');
 const result=await checkAssets(root);assert(result.passed,result.errors.join('\n'));await writeFile(path.join(out,'check.json'),JSON.stringify(result,null,2));console.log(JSON.stringify(result,null,2));
}finally{await browser?.close({silent:true});await new Promise<void>(resolve=>server.close(()=>resolve()));}
