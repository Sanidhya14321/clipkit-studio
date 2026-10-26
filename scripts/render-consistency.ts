import assert from 'node:assert/strict';
import {createServer} from 'node:http';
import {readFile,mkdir,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {bundle} from '@remotion/bundler';
import {openBrowser,renderStill,selectComposition} from '@remotion/renderer';
import {catalog,makeClip} from '../packages/assets/src/index.ts';
import {emptyProject} from '../packages/core/src/index.ts';
const root=path.resolve(import.meta.dirname,'..');
const distribution=process.env.CK_DISTRIBUTION;
const assetDirectory=distribution?path.join(path.resolve(distribution),'public'):path.join(root,'public');
const entryPoint=distribution?path.join(path.resolve(distribution),'runtime/packages/render/src/entry.tsx'):path.join(root,'packages/render/src/entry.tsx');
const vendorManifest=JSON.parse(await readFile(path.join(assetDirectory,'vendor/manifest.json'),'utf8')) as Array<{file:string;sha256:string}>;
for(const asset of vendorManifest){const bytes=await readFile(path.join(assetDirectory,'vendor',asset.file));assert.equal(createHash('sha256').update(bytes).digest('hex'),asset.sha256,`Bundled asset hash mismatch: ${asset.file}`);}
const audit=JSON.parse(await readFile(path.join(assetDirectory,'vendor/source-audit.json'),'utf8'));assert.equal(audit.sources.length,12,'Twelve licensed source records must ship');
const outDirectory=path.join(root,'.clipkit/render-consistency');await mkdir(outDirectory,{recursive:true});
const server=createServer(async(req,res)=>{try{const name=new URL(req.url!,'http://localhost').pathname;const data=await readFile(path.join(assetDirectory,name));res.setHeader('Access-Control-Allow-Origin','*');res.setHeader('Content-Type',name.endsWith('.svg')?'image/svg+xml':name.endsWith('.ttf')?'font/ttf':'application/octet-stream');res.end(data);}catch{res.statusCode=404;res.end();}});
server.listen(0,'127.0.0.1');await new Promise<void>(resolve=>server.once('listening',resolve));
const base=`http://127.0.0.1:${(server.address() as {port:number}).port}`;
let browser:Awaited<ReturnType<typeof openBrowser>>|undefined;
const results:Array<{clipId:string;frame:number;previewHash:string;exportHash:string;editedHash:string}>=[];
try{
 const serveUrl=await bundle({entryPoint});browser=await openBrowser('chrome');
 for(const asset of catalog){
  const project=emptyProject();project.tracks[1].items.push(makeClip(asset.id));const frame=asset.id==='wipe'?18:45;
  const inputProps={project,assetBaseUrl:base,mediaBaseUrl:`${base}/media`,preview:true};
  const composition=await selectComposition({serveUrl,id:'ClipKit',inputProps,puppeteerInstance:browser});
  const capture=async(label:string,props:Record<string,unknown>)=>{const output=path.join(outDirectory,`${asset.id}-${label}.png`);const {buffer}=await renderStill({serveUrl,composition:{...composition,props},inputProps:props,frame,puppeteerInstance:browser,imageFormat:'png',output});const bytes=buffer??await readFile(output);assert(bytes.length>1000,`${asset.id}: empty render`);return createHash('sha256').update(bytes).digest('hex');};
  const previewHash=await capture('preview',inputProps);
  const exportHash=await capture('export',{...inputProps,preview:false});
  assert.equal(previewHash,exportHash,`${asset.id}: native preview differs from export`);
  const editedProject=structuredClone(project);editedProject.tracks[1].items[0].props.color='#ff007f';
  const editedHash=await capture('edited',{...inputProps,project:editedProject});
  assert.notEqual(previewHash,editedHash,`${asset.id}: editable color made no visible change`);
  results.push({clipId:asset.id,frame,previewHash,exportHash,editedHash});console.log(`PASS ${asset.id}: preview/export pixels match; color edit changes frame`);
 }
 await writeFile(path.join(outDirectory,'report.json'),JSON.stringify({distribution:distribution??null,entryPoint,assetDirectory,results},null,2));
}finally{await browser?.close({silent:true});server.close();}
