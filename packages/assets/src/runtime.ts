import {validateValue} from './property-path';
import {validateClipProps,catalog,type Asset,type Field} from './index';
export type PropsSchema={type:'object';properties:Record<string,any>;required?:string[];additionalProperties?:false;[key:string]:unknown};
export type RuntimeClipDefinition=Asset & {sourceDurationFrames?:number;sceneId?:string;sceneVersion?:string;moduleUrl?:string;resourceBase?:string;compatibility?:boolean;assets?:Array<{name:string;license:string;sourceUrl?:string}>;packId:string;card?:string;basedOn?:string;component?:string;propsSchema:PropsSchema;deterministic:true;editable:true};
export type AssetRegistry={clips:RuntimeClipDefinition[];moduleUrl?:string;version?:string};
export function validateRegisteredProps(clipId:string,props:unknown,registry?:AssetRegistry):Record<string,unknown>{
 const clip=registry?.clips.find(c=>c.id===clipId);if(!clip)return validateClipProps(clipId,props);
 if(!props||typeof props!=='object'||Array.isArray(props))throw new Error('Clip props must be an object');
 const result={...clip.defaults,...props};const schema=clip.propsSchema;
 validateValue(schema,result,clipId);
 if(clip.basedOn)return validateClipProps(clip.basedOn,result);
 return result;
}
export function registeredCatalog(registry?:AssetRegistry):Asset[]{return [...catalog,...(registry?.clips.filter(c=>!c.compatibility)??[])];}
export function searchRegisteredAssets(query:string,limit=5,registry?:AssetRegistry){const terms=query.toLowerCase().split(/\W+/).filter(Boolean);return registeredCatalog(registry).map(a=>({a,score:terms.reduce((n,t)=>n+(a.id.includes(t)?4:0)+(a.name.toLowerCase().includes(t)?3:0)+(`${a.description} ${a.tags.join(' ')}`.toLowerCase().includes(t)?1:0),0)})).filter(r=>!terms.length||r.score>0).sort((a,b)=>b.score-a.score||a.a.id.localeCompare(b.a.id)).slice(0,Math.max(0,limit)).map(r=>r.a);}
export type PackField=Field;
