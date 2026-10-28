import {ProjectStore} from '../../packages/core/src/store';
import {startServer} from '../../packages/server/src/index';
const [mode,root,rawPort]=process.argv.slice(2);
if(mode==='lock'){
 const store=await new ProjectStore(root).load();store.acquire('synthetic-agent','Interrupted edit');
 process.send?.({ready:true});setInterval(()=>{},1000);
}else{
 const server=await startServer({root,port:Number(rawPort),publicDir:process.cwd()+'/public',debug:{runId:'chaos-server',synthetic:true}});
 (server.store as any).options.prepare=async()=>{process.send?.({prepared:true});await new Promise(()=>{});};
 process.send?.({ready:true,url:server.url,token:server.token});
}
