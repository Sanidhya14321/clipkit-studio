/** Three-way revert merges by stable object IDs, never stale JSON array indices. */
const absent=Symbol('absent');
type Value=unknown|typeof absent;
const equal=(a:Value,b:Value)=>a===absent||b===absent?a===b:JSON.stringify(a)===JSON.stringify(b);
const object=(v:Value):v is Record<string,unknown>=>v!==absent&&v!==null&&typeof v==='object'&&!Array.isArray(v);
const idArray=(v:Value):v is Array<Record<string,unknown>&{id:string}>=>Array.isArray(v)&&v.every(x=>object(x)&&typeof x.id==='string');
export class RevertConflict extends Error {constructor(public paths:string[]){super(`Later edits conflict at ${paths.slice(0,5).join(', ')}`);}}
export function mergeRevert(before:unknown,after:unknown,current:unknown):unknown{
 const conflicts:string[]=[];
 function visit(b:Value,a:Value,c:Value,path:string):Value{
  if(equal(b,a)||equal(b,c))return c;
  if(equal(a,c))return b;
  if(object(b)&&object(a)&&object(c)){
   const result:Record<string,unknown>={};for(const key of new Set([...Object.keys(b),...Object.keys(a),...Object.keys(c)])){
    const v=visit(Object.hasOwn(b,key)?b[key]:absent,Object.hasOwn(a,key)?a[key]:absent,Object.hasOwn(c,key)?c[key]:absent,`${path}/${key}`);if(v!==absent)result[key]=v;
   }return result;
  }
  if(idArray(b)&&idArray(a)&&idArray(c)){
   const bm=new Map(b.map(x=>[x.id,x])),am=new Map(a.map(x=>[x.id,x])),cm=new Map(c.map(x=>[x.id,x]));const values=new Map<string,unknown>();
   for(const key of new Set([...bm.keys(),...am.keys(),...cm.keys()])){const v=visit(bm.get(key)??absent,am.get(key)??absent,cm.get(key)??absent,`${path}/@${key}`);if(v!==absent)values.set(key,v);}
   const common=b.filter(x=>am.has(x.id)).map(x=>x.id),afterOrder=a.filter(x=>bm.has(x.id)).map(x=>x.id),currentOrder=c.filter(x=>bm.has(x.id)&&am.has(x.id)).map(x=>x.id);
   let order=c.map(x=>x.id).filter(x=>values.has(x));
   if(JSON.stringify(common)!==JSON.stringify(afterOrder)){
    if(JSON.stringify(currentOrder)!==JSON.stringify(afterOrder)){conflicts.push(`${path}/order`);}else{let i=0;order=order.map(x=>common.includes(x)?common[i++]:x);}
   }
   for(let i=0;i<b.length;i++){const key=b[i].id;if(values.has(key)&&!order.includes(key)){const next=b.slice(i+1).find(x=>order.includes(x.id));const pos=next?order.indexOf(next.id):order.length;order.splice(pos,0,key);}}
   return order.map(key=>values.get(key));
  }
  conflicts.push(path||'/');return c;
 }
 const result=visit(before,after,current,'');if(conflicts.length)throw new RevertConflict(conflicts);return result;
}
