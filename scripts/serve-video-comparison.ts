import {createServer} from 'node:http';
import {createReadStream} from 'node:fs';
import {readFile,stat,writeFile} from 'node:fs/promises';
import path from 'node:path';
const root=path.resolve(process.argv[2]??'.clipkit/video-comparison-20261028'),port=Number(process.argv[3]??4335);
const data=JSON.parse(await readFile(path.join(root,'report.json'),'utf8'));
const allowed=new Map<string,{file:string;type:string}>([['/',{file:'index.html',type:'text/html; charset=utf-8'}],['/index.html',{file:'index.html',type:'text/html; charset=utf-8'}],['/report.json',{file:'report.json',type:'application/json'}],['/comparison.md',{file:'COMPARISON.md',type:'text/plain; charset=utf-8'}]]);
for(const run of data.runs)for(const phase of run.phases){if(!phase.metrics||phase.metrics.error)continue;allowed.set('/media/'+run.id+'-round-'+phase.round+'.mp4',{file:phase.metrics.video,type:'video/mp4'});if(phase.metrics.contactSheet)allowed.set('/contact/'+phase.metrics.sha256.slice(0,16)+'.png',{file:phase.metrics.contactSheet,type:'image/png'});}
const server=createServer(async(req,res)=>{try{
 if(!['GET','HEAD'].includes(req.method??'')){res.writeHead(405).end();return;}
 if(!['127.0.0.1:'+port,'localhost:'+port].includes(req.headers.host??'')){res.writeHead(403).end();return;}
 const item=allowed.get(new URL(req.url??'/','http://127.0.0.1').pathname);if(!item){res.writeHead(404).end();return;}
 const file=path.resolve(root,item.file);if(!file.startsWith(root+path.sep)){res.writeHead(403).end();return;}
 const info=await stat(file),range=req.headers.range;res.setHeader('Content-Type',item.type);res.setHeader('X-Content-Type-Options','nosniff');res.setHeader('Cache-Control','no-store');res.setHeader('Content-Security-Policy',"default-src 'self'; script-src 'self' 'unsafe-inline'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; media-src 'self'; frame-ancestors 'none'");
 if(range){const match=/^bytes=(\d+)-(\d*)$/.exec(range);if(!match){res.writeHead(416).end();return;}const start=Number(match[1]),end=match[2]?Math.min(Number(match[2]),info.size-1):info.size-1;if(start>end||start>=info.size){res.writeHead(416).end();return;}res.writeHead(206,{'Content-Range':`bytes ${start}-${end}/${info.size}`,'Accept-Ranges':'bytes','Content-Length':end-start+1});if(req.method==='HEAD')res.end();else createReadStream(file,{start,end}).pipe(res);}
 else{res.writeHead(200,{'Content-Length':info.size,'Accept-Ranges':'bytes'});if(req.method==='HEAD')res.end();else createReadStream(file).pipe(res);}
 }catch{res.writeHead(404).end();}});
server.listen(port,'127.0.0.1',async()=>{await writeFile(path.join(root,'viewer-server.json'),JSON.stringify({pid:process.pid,url:`http://127.0.0.1:${port}/`}));console.log(`Comparison viewer: http://127.0.0.1:${port}/`);});
