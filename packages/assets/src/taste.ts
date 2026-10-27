import {readdir} from 'node:fs/promises';
import path from 'node:path';
import {parse} from 'yaml';
import {z} from 'zod';
import {boundedRead} from './knowledge';
const Constraints=z.object({minimumTextSize:z.number().min(12).max(200),maxWords:z.number().int().min(1).max(20),maxWps:z.number().min(1).max(10),maxAnimated:z.number().int().min(1).max(50),maxHeavy:z.number().int().min(0).max(10),maxUses:z.number().int().min(1).max(10),safeArea:z.number().min(0).max(.4),excludeTags:z.array(z.string()).max(20)});
export class TasteLibrary {
 constructor(readonly folder:string){}
 async search(query=''){const terms=query.toLowerCase().split(/\W+/).filter(Boolean),files=(await readdir(this.folder)).filter(f=>f.endsWith('.md')&&f!=='INDEX.md');const rows=await Promise.all(files.map(async file=>{const text=await boundedRead(this.folder,file,6000);return {id:file.slice(0,-3),summary:text.split('\n').find(l=>l&&!l.startsWith('#'))?.slice(0,120)??'',score:terms.reduce((s,t)=>s+(text.toLowerCase().includes(t)?1:0),0)}}));return rows.sort((a,b)=>b.score-a.score||a.id.localeCompare(b.id)).slice(0,5).map(({score,...row})=>row)}
 async read(id:string){if(!/^[a-z0-9-]+$/.test(id))throw Error('Invalid taste ID');const text=await boundedRead(this.folder,id+'.md',6000);if(text.split('\n').length>60)throw Error('Taste file exceeds 60 lines');return {id,text,dataOnly:true}}
 async constraints(){return Constraints.parse(parse(await boundedRead(this.folder,'constraints.yaml',4000),{maxAliasCount:0}))}
}
