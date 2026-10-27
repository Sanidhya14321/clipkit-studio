import {z} from 'zod';
import {id,type Item} from '../../core/src/index';
export type Field = {key:string;label:string;type:'text'|'textarea'|'color'|'number'|'select'|'image';min?:number;max?:number;step?:number;options?:string[]};
export type Asset = {aspect:string;whenToUse:string;previewPath:string;id:string;name:string;category:string;durationSec:number;description:string;tags:string[];license:string;defaults:Record<string,unknown>;fields:Field[]};

export const catalog:Asset[]=[];
export const clipSchemas:Record<string,z.ZodType<Record<string,unknown>>>={};
export function validateClipProps(clipId:string,props:unknown):Record<string,unknown>{throw Error(`Unknown scene ${clipId}. Author a scene or restore its compatibility archive.`)}
export function searchAssets(query:string,limit=5):Asset[]{return []}
export function makeClip(clipId:string,at=0):Item{throw Error('The old template catalog was removed. Use motion_search and scene_draft.')}
