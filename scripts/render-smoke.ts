import {createServer} from 'node:http';
import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {emptyProject} from '../packages/core/src/index.ts';
import {makeClip} from '../packages/assets/src/index.ts';
import {renderProject} from '../packages/render/src/export.ts';
const root=path.resolve(import.meta.dirname,'..');
const distribution=process.env.CK_DISTRIBUTION?path.resolve(process.env.CK_DISTRIBUTION):undefined;
const assetDirectory=distribution?path.join(distribution,'public'):path.join(root,'public');
const entryPoint=distribution?path.join(distribution,'runtime/packages/render/src/entry.tsx'):undefined;
const server=createServer(async(req,res)=>{try{const name=new URL(req.url!,'http://localhost').pathname;const data=await readFile(path.join(assetDirectory,name));res.setHeader('Content-Type',name.endsWith('.svg')?'image/svg+xml':name.endsWith('.ttf')?'font/ttf':'application/octet-stream');res.end(data);}catch{res.statusCode=404;res.end();}});
server.listen(0,'127.0.0.1');await new Promise<void>(resolve=>server.once('listening',resolve));
const port=(server.address() as {port:number}).port;const project=emptyProject();project.name='Render proof';
project.tracks[0].items.push(makeClip('gradient'));
project.tracks[1].items.push(makeClip('kinetic-title'));
try{console.log(await renderProject({project,root,entryPoint,assetDirectory,mediaBaseUrl:`http://127.0.0.1:${port}/media`,assetBaseUrl:`http://127.0.0.1:${port}`,onProgress:p=>{if(Math.round(p*100)%25===0)console.log(`${Math.round(p*100)}%`);}}));}finally{server.close();}
