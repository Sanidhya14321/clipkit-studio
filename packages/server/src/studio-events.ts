import {EventEmitter} from 'node:events';
import {appendFile,readFile,mkdir} from 'node:fs/promises';
import path from 'node:path';
export type StudioEvent={cursor:number;type:string;data:Record<string,unknown>};
export class StudioEvents {
 private bus=new EventEmitter();private events:StudioEvent[]=[];private sequence=0;private closed=false;private writes:Promise<unknown>=Promise.resolve();
 constructor(private root:string){this.bus.setMaxListeners(100)}
 async load(){try{this.events=(await readFile(path.join(this.root,'.clipkit/events.jsonl'),'utf8')).split('\n').filter(Boolean).slice(-500).map(l=>JSON.parse(l));this.sequence=this.events.at(-1)?.cursor??0}catch(e){if((e as NodeJS.ErrnoException).code!=='ENOENT')throw e}return this}
 get cursor(){return this.sequence}
 emit(type:string,data:Record<string,unknown>={}){const event={cursor:++this.sequence,type,data};this.events.push(event);this.events=this.events.slice(-500);this.writes=this.writes.catch(()=>{}).then(async()=>{await mkdir(path.join(this.root,'.clipkit'),{recursive:true});await appendFile(path.join(this.root,'.clipkit/events.jsonl'),JSON.stringify(event)+'\n')});this.bus.emit('event');return event}
 async wait(after:number,timeout=45000,types?:string[]){const read=()=>this.events.filter(e=>e.cursor>after&&(!types||types.includes(e.type))).slice(0,10);let events=read();if(!events.length&&timeout>0)await new Promise<void>(resolve=>{const finish=()=>{if(read().length||this.closed){clearTimeout(timer);this.bus.off('event',finish);resolve()}};const timer=setTimeout(()=>{this.bus.off('event',finish);resolve()},Math.max(0,Math.min(45000,timeout)));this.bus.on('event',finish);finish()});events=read();return {cursor:events.at(-1)?.cursor??this.sequence,events,timeout:events.length===0,closed:this.closed}}
 async close(){this.closed=true;this.bus.emit('event');await this.writes}
}
