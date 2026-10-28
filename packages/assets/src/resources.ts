import {createHash} from 'node:crypto';
import {mkdir, readFile, writeFile, stat} from 'node:fs/promises';
import path from 'node:path';

export type Resource = {id:string; name:string; path:string; bytes:number; sha256:string; kind:'font'|'image'|'video'|'audio'; url:string; usage?:string};
const extensions:Record<string,Resource['kind']>={'.ttf':'font','.otf':'font','.woff':'font','.woff2':'font','.png':'image','.jpg':'image','.jpeg':'image','.webp':'image','.svg':'image','.mp4':'video','.mov':'video','.webm':'video','.wav':'audio','.mp3':'audio','.ogg':'audio','.m4a':'audio'};
/** The transport reads files, never asks a model to encode binary resources. */
export class Resources {
  constructor(readonly root:string){}
  get directory(){return path.join(this.root,'.clipkit/resources')}
  async add(name:string,bytes:Uint8Array):Promise<Resource>{
    const ext=path.extname(name).toLowerCase(),kind=extensions[ext];
    if(!kind)throw Error('Unsupported resource extension');
    if(!bytes.length||bytes.length>256*1024*1024)throw Error('Resources must be between 1 byte and 256 MB');
    const sha256=createHash('sha256').update(bytes).digest('hex'),id=sha256+ext;
    await mkdir(this.directory,{recursive:true});
    const resource:Resource={id,name:path.basename(name),path:path.join('.clipkit/resources',id),bytes:bytes.length,sha256,kind,url:'/resources/'+id,usage:'In TSX: resourceUrl(id, assetBaseUrl, accessToken) from @clipkit/motion. Authenticated URL query is ?token=accessToken.'};
    try{await writeFile(path.join(this.directory,id),bytes,{flag:'wx'})}catch(e){if((e as NodeJS.ErrnoException).code!=='EEXIST')throw e}
    await writeFile(path.join(this.directory,id+'.json'),JSON.stringify(resource));
    return resource;
  }
  async import(file:string){if((await stat(file)).size>256*1024*1024)throw Error('Resource exceeds 256 MB');return this.add(file,await readFile(file))}
  async get(id:string):Promise<Resource>{
    if(!/^[a-f0-9]{64}\.(ttf|otf|woff2?|png|jpe?g|webp|svg|mp4|mov|webm|wav|mp3|ogg|m4a)$/.test(id))throw Error('Invalid resource ID');
    const resource:Resource=JSON.parse(await readFile(path.join(this.directory,id+'.json'),'utf8'));
    const bytes=await readFile(path.join(this.directory,id));
    if(createHash('sha256').update(bytes).digest('hex')!==resource.sha256)throw Error('Resource hash mismatch');
    return resource;
  }
}
