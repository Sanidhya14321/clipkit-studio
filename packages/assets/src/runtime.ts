import {validateClipProps,catalog,type Asset,type Field} from './index';
export type PropsSchema={type:'object';properties:Record<string,{type:'string'|'number'|'integer'|'boolean';enum?:Array<string|number|boolean>;minimum?:number;maximum?:number;pattern?:string;minLength?:number;maxLength?:number;default?:unknown}>;required?:string[];additionalProperties?:false};
export type RuntimeClipDefinition=Asset & {sourceDurationFrames?:number;sceneId?:string;sceneVersion?:string;moduleUrl?:string;resourceBase?:string;compatibility?:boolean;assets?:Array<{name:string;license:string;sourceUrl?:string}>;packId:string;card?:string;basedOn?:string;component?:string;propsSchema:PropsSchema;deterministic:true;editable:true};
export type AssetRegistry={clips:RuntimeClipDefinition[];moduleUrl?:string;version?:string};
export function validateRegisteredProps(clipId:string,props:unknown,registry?:AssetRegistry):Record<string,unknown>{
 const clip=registry?.clips.find(c=>c.id===clipId);if(!clip)return validateClipProps(clipId,props);
 if(!props||typeof props!=='object'||Array.isArray(props))throw new Error('Clip props must be an object');
 const result={...clip.defaults,...props};const schema=clip.propsSchema;
 for(const key of Object.keys(result))if(!schema.properties[key])throw new Error(`${clipId}: unknown property ${key}`);
 for(const key of schema.required??[])if(result[key]===undefined)throw new Error(`${clipId}: missing ${key}`);
 for(const [key,value]of Object.entries(result)){
  const rule=schema.properties[key];if(value===undefined)continue;
  if(rule.type==='integer'?!Number.isInteger(value):typeof value!==rule.type)throw new Error(`${clipId}: ${key} must be ${rule.type}`);
  if(typeof value==='number'&&(!Number.isFinite(value)||(rule.minimum!==undefined&&value<rule.minimum)||(rule.maximum!==undefined&&value>rule.maximum)))throw new Error(`${clipId}: ${key} out of range`);
  if(typeof value==='string'&&((rule.minLength!==undefined&&value.length<rule.minLength)||(rule.maxLength!==undefined&&value.length>rule.maxLength)))throw new Error(`${clipId}: ${key} length invalid`);
  if(typeof value==='string'&&rule.pattern&&!new RegExp(rule.pattern).test(value))throw new Error(`${clipId}: ${key} pattern mismatch`);
  if(rule.enum&&!rule.enum.includes(value as string|number|boolean))throw new Error(`${clipId}: ${key} not in enum`);
 }
 if(clip.basedOn)return validateClipProps(clip.basedOn,result);
 return result;
}
export function registeredCatalog(registry?:AssetRegistry):Asset[]{return [...catalog,...(registry?.clips.filter(c=>!c.compatibility)??[])];}
export function searchRegisteredAssets(query:string,limit=5,registry?:AssetRegistry){const terms=query.toLowerCase().split(/\W+/).filter(Boolean);return registeredCatalog(registry).map(a=>({a,score:terms.reduce((n,t)=>n+(a.id.includes(t)?4:0)+(a.name.toLowerCase().includes(t)?3:0)+(`${a.description} ${a.tags.join(' ')}`.toLowerCase().includes(t)?1:0),0)})).filter(r=>!terms.length||r.score>0).sort((a,b)=>b.score-a.score||a.a.id.localeCompare(b.a.id)).slice(0,Math.max(0,limit)).map(r=>r.a);}
export type PackField=Field;
