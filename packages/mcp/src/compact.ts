/** Mutation journals belong in durable history, not echoed back into model context. */
export function compactToolResult(value:any):any {
 if(!value||typeof value!=='object'||Array.isArray(value))return value;
 if(value.kind==='component'&&value.schema?.schema)return {id:value.id,name:value.name,version:value.version,preview:value.preview,sourceUrl:value.sourceUrl,license:value.license,usage:value.usage,defaults:value.schema.defaults,schema:value.schema.schema,...value.componentSource?{source:value.componentSource}:{}};
 if(Array.isArray(value.ops)&&Array.isArray(value.inverse)&&typeof value.rev==='number'){
  return Object.fromEntries(['rev','label','changedItemIds','itemIds','version','clipId','requestId','commentId'].filter(key=>value[key]!==undefined).map(key=>[key,value[key]]));
 }
 if(value.result&&typeof value.result==='object')return {...value,result:compactToolResult(value.result)};
 return value;
}
export const videoRunnerFeatureArgs=['--disable','shell_tool','--disable','apps','--disable','plugins','--disable','multi_agent','--disable','browser_use','--disable','computer_use'];
