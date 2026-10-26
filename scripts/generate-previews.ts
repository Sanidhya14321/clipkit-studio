import {readFile,mkdir,writeFile,unlink} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {createRequire} from 'node:module';
import path from 'node:path';
import {catalog} from '../packages/assets/src/index.ts';
const root=path.resolve(import.meta.dirname,'..');
const report=JSON.parse(await readFile(path.join(root,'.clipkit/render-consistency/report.json'),'utf8')) as {results:Array<{clipId:string;frame:number;previewHash:string;defaultsSha256:string}>};
const require=createRequire(import.meta.url);const ffmpeg=require('ffmpeg-static') as string;
const directory=path.join(root,'public/previews');await mkdir(directory,{recursive:true});
const manifest=[];
for(const asset of catalog){
 const proof=report.results.find(r=>r.clipId===asset.id);if(!proof)throw new Error(`No verified preview proof for ${asset.id}`);
 if(proof.defaultsSha256!==createHash('sha256').update(JSON.stringify(asset.defaults)).digest('hex'))throw new Error(`Default props changed for ${asset.id}; regenerate actual render proofs first`);
 const input=path.join(root,'.clipkit/render-consistency',`${asset.id}-preview.png`);const original=await readFile(input);
 if(createHash('sha256').update(original).digest('hex')!==proof.previewHash)throw new Error(`Proof image changed: ${asset.id}`);
 const output=path.join(directory,`${asset.id}.png`);
 await promisify(execFile)(ffmpeg,['-y','-i',input,'-vf','scale=240:-1','-frames:v','1','-compression_level','9',output,'-loglevel','error']);
 const png=await readFile(output);manifest.push({clipId:asset.id,previewPath:asset.previewPath,frame:proof.frame,sourcePngSha256:proof.previewHash,sha256:createHash('sha256').update(png).digest('hex'),defaultsSha256:createHash('sha256').update(JSON.stringify(asset.defaults)).digest('hex'),width:240,height:427,source:'Real Remotion render from the native composition; scaled without design changes'});
 try{await unlink(path.join(directory,`${asset.id}-preview.png`));}catch{}
}
await writeFile(path.join(directory,'manifest.json'),JSON.stringify(manifest,null,2)+'\n');
console.log(`Generated ${manifest.length} actual native clip thumbnails`);
