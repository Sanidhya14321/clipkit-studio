import {readFile} from 'node:fs/promises';
import path from 'node:path';
import {activeTake,allItems,type Project} from '../../core/src/index';
import {catalog} from '../../assets/src/index';
import type {AssetRegistry} from '../../assets/src/runtime';
export type LicenseSummary={clips:Array<{id:string;name:string;license:string}>;assets:Array<{name:string;license:string;sourceUrl?:string}>;notices:string[]};
export async function exportLicenses(project:Project,assetDirectory?:string,registry?:AssetRegistry):Promise<LicenseSummary>{
 const items=allItems(project);const ids=new Set(items.filter(i=>i.type==='clip').map(i=>i.clipId!));if(items.some(i=>i.type==='caption'))ids.add('captions');
 const assets:LicenseSummary['assets']=[];const clips:LicenseSummary['clips']=[];const vendorFiles=new Set<string>();
 const definitions=[...catalog,...(registry?.clips??[])];
 for(const id of ids){const clip=definitions.find(a=>a.id===id);if(!clip)continue;clips.push({id,name:clip.name,license:clip.license});const external=registry?.clips.find(c=>c.id===id);const nativeId=external?.basedOn??id;if(external?.basedOn){const base=catalog.find(c=>c.id===external.basedOn);if(base&&!clips.some(c=>c.id===base.id))clips.push({id:base.id,name:base.name,license:base.license});}
  const optionalTextOnly=['starfield','confetti-burst'].includes(nativeId);const hasText=items.some(item=>item.clipId===id&&String(item.props.text??'').length>0);
  if(!['gradient','wipe'].includes(nativeId)&&(!optionalTextOnly||hasText)&&catalog.some(a=>a.id===nativeId)){vendorFiles.add('space-grotesk.ttf');if(nativeId!=='captions')vendorFiles.add('inter.ttf');}
  if(nativeId==='illustration-card')vendorFiles.add('illustration.svg');
  if(['icon-callout','cta'].includes(nativeId))vendorFiles.add('lucide-LICENSE.txt');
  for(const asset of external?.assets??[])assets.push(asset);
 }
 let vendors:Array<{file:string;repo:string;license?:string;url:string}>=[];if(assetDirectory)try{vendors=JSON.parse(await readFile(path.join(assetDirectory,'vendor/manifest.json'),'utf8'));}catch{}
 for(const vendor of vendors)if(vendorFiles.has(vendor.file))assets.push({name:vendor.repo,license:vendor.license??'See bundled license notice',sourceUrl:vendor.url});
 const sourceIds=new Set(items.flatMap(i=>[...(i.src?[i.src]:[]),...(typeof i.props.audioSourceId==='string'?[i.props.audioSourceId]:[]),...(typeof i.props.imageSourceId==='string'&&i.props.imageSourceId?[i.props.imageSourceId]:[])]));
 for(const item of items.filter(i=>i.type==='voice')){const slot=project.voiceSlots.find(s=>s.id===item.slotId);const take=slot?activeTake(slot):undefined;if(take)sourceIds.add(take.sourceId);}
 const notices:string[]=[];for(const use of project.assetsUsed??[]){assets.push({name:use.sourceAsset,license:use.license});notices.push(`${use.sourceAsset}: ${use.mode}; ${use.adaptations.join('; ')}; source hash ${use.sourceHash}`);}
for(const sourceId of sourceIds){const source=project.sources[sourceId];if(!source)continue;const vendor=vendors.find(v=>source.path.endsWith(v.file)&&!v.file.includes('LICENSE'));if(vendor)assets.push({name:source.name,license:vendor.license??'See bundled license',sourceUrl:vendor.url});else if(source.provenance?.startsWith('{')){try{const info=JSON.parse(source.provenance);if(info.license){assets.push({name:source.name,license:info.license,sourceUrl:info.sourceUrl});notices.push(`${source.name}: ${info.author??'See source'}; ${info.modifications??'Used as timeline media'}`);}else notices.push(source.provenance);}catch{notices.push(source.provenance)}}else notices.push(`${source.name}: ${source.provenance??'User supplied media; rights are not established by ClipKit.'}`);}
 return {clips,assets:[...new Map(assets.map(a=>[`${a.name}|${a.license}|${a.sourceUrl??''}`,a])).values()],notices};
}
