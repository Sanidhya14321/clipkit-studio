import {fieldsFor} from './property-path';
import type {Field} from './index';
export function editableFields(id:string,fields:Field[],props:Record<string,unknown>={},schema?:any):Field[]{return schema?fieldsFor(schema,props).map(field=>({...field,...fields.find(existing=>existing.key===field.key)})):fields}
export function defaultFontFamily(id:string){return 'Inter'}
export function supportsFont(id:string){return false}
