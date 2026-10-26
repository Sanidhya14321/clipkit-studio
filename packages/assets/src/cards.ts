import {readFile,realpath} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
import {parseDocument} from 'yaml';
import {z} from 'zod';

const permittedLicenses=new Set(['MIT','ISC','ISC AND MIT','BSD-2-Clause','BSD-3-Clause','Apache-2.0','Zlib','Unlicense','CC0-1.0','CC-BY-4.0','OFL-1.1']);
/** An allowlist decision, not a certification of a publisher's ownership claim. */
export function normalizeAssetLicense(value:string,legacyNative=false){
 if(legacyNative&&value==='MIT (native ClipKit component; embedded source assets retain their licenses)')return 'MIT';
 if(!permittedLicenses.has(value))throw new Error(`Asset license is not approved: ${value}. Use a verified SPDX identifier from the allowlist.`);return value;
}
export const cardKinds=['backgrounds','text','overlays','transitions','fx','media','data','audio','font','image','model','texture','video','lottie'] as const;
const nonempty=z.string().trim().min(1);
export const CardSchema=z.object({
 id:z.string().regex(/^[a-z0-9][a-z0-9-]{0,99}$/),name:nonempty,kind:z.enum(cardKinds),version:z.number().int().positive(),
 source:nonempty,author:nonempty,license:nonempty,provenance:nonempty,
 'provenance-status':z.enum(['agent-authored','human-attributed','unverified']),
 'best-for':nonempty,avoid:nonempty,integration:nonempty,tags:z.array(nonempty).min(1).max(24),mood:nonempty,
 duration:z.string().optional(),perf:nonempty,'source-files':z.array(nonempty).default([]),uses:z.array(nonempty).default([]),
 'license-file':nonempty.optional(),'provenance-evidence':z.array(z.string().url()).default([]),'schema-path':nonempty.optional(),
 'media-path':nonempty.optional(),sha256:z.string().regex(/^[a-f0-9]{64}$/).optional(),
 tested:z.object({status:z.enum(['not-run','integrity-verified','render-verified']),method:nonempty,evidence:nonempty.optional()}).strict(),
}).strict();
export type AssetCard=z.infer<typeof CardSchema>;
export type CardEntry={id:string;kind:AssetCard['kind'];name:string;summary:string;duration?:string;tags:string[];mood:string;bestFor:string;avoid:string;provenanceStatus:AssetCard['provenance-status'];cardPath:string;sha256:string;schemaPath?:string};
export const digest=(value:string|Uint8Array)=>createHash('sha256').update(value).digest('hex');
export function parseCard(markdown:string):{card:AssetCard;summary:string}{
 if(Buffer.byteLength(markdown)>12000)throw new Error('Asset card exceeds the 12KB context budget');
 if(markdown.split(/\r?\n/).length>45)throw new Error('Asset card must stay within 45 lines');
 const match=markdown.match(/^---\r?\n([\s\S]*?)\r?\n---(?:\r?\n|$)([\s\S]*)$/);if(!match)throw new Error('Asset card requires YAML frontmatter');
 const document=parseDocument(match[1],{uniqueKeys:true});if(document.errors.length)throw new Error(document.errors[0].message);
 const card=CardSchema.parse(document.toJS({maxAliasCount:0}));
 const summary=match[2].split(/\r?\n/).map(line=>line.trim()).find(line=>line&&!line.startsWith('#'));
 if(!summary)throw new Error('Asset card needs a short prose summary');return {card,summary};
}
export async function readInside(root:string,relative:string){
 const resolved=path.resolve(root,relative);if(!resolved.startsWith(path.resolve(root)+path.sep))throw new Error('Asset path escapes its root');
 const actual=await realpath(resolved);const base=await realpath(root);if(!actual.startsWith(base+path.sep))throw new Error('Asset symlink escapes its root');return readFile(actual);
}
export async function readCardInventory(publicDir:string):Promise<CardEntry[]>{
 const value=JSON.parse((await readInside(publicDir,'packs/native/asset-cards.json')).toString('utf8'));if(!Array.isArray(value))throw new Error('Invalid asset card inventory');return value;
}
export async function nativeCard(publicDir:string,id:string){
 const entry=(await readCardInventory(publicDir)).find(entry=>entry.id===id);if(!entry)throw new Error(`No reviewed card for asset ${id}`);
 const markdown=(await readInside(publicDir,entry.cardPath)).toString('utf8');if(digest(markdown)!==entry.sha256)throw new Error('Asset card changed; run assets:check to refresh the inventory');const {card}=parseCard(markdown);if(card.id!==id)throw new Error('Asset card ID mismatch');return {entry,card,markdown,hash:entry.sha256};
}
export async function listCards(publicDir:string,filter:{kind?:string;tags?:string[];mood?:string;q?:string;offset?:number;limit?:number}={},additional:CardEntry[]=[]){
 const entries=[...await readCardInventory(publicDir),...additional];const terms=(filter.q??'').toLowerCase().split(/\W+/).filter(Boolean);
 const matches=entries.filter(e=>(filter.kind?true:!!e.schemaPath)&&(!filter.kind||e.kind===filter.kind)&&(!filter.mood||e.mood===filter.mood)&&(filter.tags??[]).every(tag=>e.tags.includes(tag))&&terms.every(term=>`${e.id} ${e.name} ${e.summary} ${e.tags.join(' ')} ${e.bestFor}`.toLowerCase().includes(term)));
 const offset=Math.max(0,Math.floor(filter.offset??0)),limit=Math.min(10,Math.max(1,Math.floor(filter.limit??5)));
 return {items:matches.slice(offset,offset+limit).map(({id,kind,name,summary,duration,tags,mood,bestFor,avoid,provenanceStatus,schemaPath})=>({id,kind,name,summary,duration,tags,mood,bestFor,avoid,provenanceStatus,schemaPath})),total:matches.length,next:offset+limit<matches.length?offset+limit:null};
}
