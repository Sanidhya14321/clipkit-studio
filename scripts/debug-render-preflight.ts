import path from 'node:path';
import {spawn} from 'node:child_process';
import {writeFile,symlink,mkdir} from 'node:fs/promises';
import {freshEnvironment,sandboxCommand} from '../packages/debug/src/isolation';
const root=process.cwd(),env=await freshEnvironment();
try {
 await symlink('/deps/node_modules',path.join(env.workspace,'node_modules'),'dir');
 await writeFile(path.join(env.workspace,'entry.tsx'),`import React from 'react';import {Composition,registerRoot,AbsoluteFill} from 'remotion';const Frame=()=> <AbsoluteFill style={{backgroundColor:'#123456'}}>PREFLIGHT</AbsoluteFill>;registerRoot(()=> <Composition id="Preflight" component={Frame} width={320} height={180} fps={30} durationInFrames={30}/>);`);
 await writeFile(path.join(env.workspace,'render.cjs'),`const {bundle}=require('/deps/node_modules/@remotion/bundler');const {renderStill,selectComposition}=require('/deps/node_modules/@remotion/renderer');(async()=>{const url=await bundle({entryPoint:'/work/entry.tsx'});const c=await selectComposition({serveUrl:url,id:'Preflight',browserExecutable:'/usr/bin/google-chrome'});await renderStill({serveUrl:url,composition:c,output:'/work/frame.png',browserExecutable:'/usr/bin/google-chrome'});console.log('RENDER_PREFLIGHT_OK');})().catch(e=>{console.error(e);process.exitCode=1});`);
 const sandbox=await sandboxCommand({workspace:env.workspace,userHome:env.userHome,agent:'node',args:['/work/render.cjs'],runtime:path.join(root,'artifacts/package'),deps:path.join(root,'node_modules'),offline:true});
 const child=spawn(sandbox.command,sandbox.args,{env:sandbox.env,stdio:['ignore','pipe','pipe'],detached:true});let output='';for(const stream of [child.stdout!,child.stderr!])stream.on('data',d=>{output+=d.toString()});
 const timer=setTimeout(()=>{try{process.kill(-child.pid!,'SIGKILL')}catch{}},60000);const exit=await new Promise(r=>child.on('close',r));clearTimeout(timer);
 const receipt={exit,output:output.slice(-5000),environment:sandbox.manifest};await mkdir('.clipkit/update4',{recursive:true});await writeFile('.clipkit/update4/render-preflight.json',JSON.stringify(receipt,null,2));console.log(JSON.stringify(receipt));if(exit!==0)process.exitCode=1;
}finally{await env.cleanup()}
