import {readFile,readdir} from 'node:fs/promises';
import path from 'node:path';
import {parse} from 'yaml';
import {TokenSchema,PacingSchema,type DesignTokens} from '../../core/src/brief';
import {getProperty,setProperty} from './property-path';
import type {Item} from '../../core/src/index';
export class DirectionLibrary {
 constructor(readonly root:string){}
 async look(id:string):Promise<DesignTokens>{if(!/^[a-z][a-z0-9-]{0,40}$/.test(id))throw Error('Invalid look');return TokenSchema.parse(JSON.parse(await readFile(path.join(this.root,'assets/taste/looks',id+'.tokens.json'),'utf8')))}
 async looks(){return Promise.all((await readdir(path.join(this.root,'assets/taste/looks'))).filter(f=>f.endsWith('.tokens.json')).map(f=>this.look(f.slice(0,-12))))}
 async pacing(id:string){const data=parse(await readFile(path.join(this.root,'assets/taste/pacing.yaml'),'utf8'),{maxAliasCount:0});if(!data[id])throw Error('Unknown pacing profile');return PacingSchema.parse({id,...data[id]})}
}
export function tokenProps(item:Item,tokens?:DesignTokens){let props=item.props;if(tokens)for(const [property,token]of Object.entries(item.tokenBindings??{})){const value=getProperty(tokens,token);if(value===undefined)throw Error(`Unknown token ${token}`);props=setProperty(props,property,value)}return props;}
export type LayoutIntent={id:string;role:string;text?:string;maxChars?:number};
export function solveLayout(intents:LayoutIntent[],width:number,height:number,minimum=24){if(intents.length>8)throw Error('At most eight layout boxes');const margin=Math.ceil(Math.min(width,height)*.08),grid=8,gap=32,usableH=height-margin*2,boxH=Math.floor((usableH-gap*Math.max(0,intents.length-1))/Math.max(1,intents.length)/grid)*grid;const boxes=intents.map((i,n)=>{if(i.text&&i.maxChars&&i.text.length>i.maxChars)throw Error(`${i.id}: shorten copy to ${i.maxChars} characters`);const w=width-margin*2;const headline=i.role==='headline';let fontSize=Math.min(headline?104:52,Math.floor(boxH*.6));const estimate=(size:number)=>{const columns=Math.max(1,Math.floor(w/(size*.62)));return Math.ceil((i.text?.length??1)/columns)*size*1.25};while(fontSize>=minimum&&estimate(fontSize)>boxH)fontSize-=2;if(fontSize<minimum)throw Error(`${i.id}: copy cannot fit above ${minimum}px; shorten it`);return {id:i.id,role:i.role,x:margin,y:margin+n*(boxH+gap),width:w,height:boxH,fontSize}});return {width,height,minimum,boxes,pass:boxes.every(b=>b.x>=margin&&b.y>=margin&&b.x+b.width<=width-margin&&b.y+b.height<=height-margin)};}
