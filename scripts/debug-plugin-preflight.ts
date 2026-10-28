import {createServer} from 'node:net';
import path from 'node:path';
import {writeFile,mkdir} from 'node:fs/promises';
import {Client} from '@modelcontextprotocol/sdk/client/index.js';
import {StdioClientTransport} from '@modelcontextprotocol/sdk/client/stdio.js';
import {freshEnvironment,sandboxCommand} from '../packages/debug/src/isolation';
import {startServer} from '../packages/server/src/index';
const env=await freshEnvironment(),root=process.cwd();
const port=await new Promise<number>(resolve=>{const s=createServer();s.listen(0,'127.0.0.1',()=>{const port=(s.address() as any).port;s.close(()=>resolve(port))})});
const server=await startServer({root:env.workspace,port,publicDir:path.join(root,'public'),debug:{synthetic:true}});
const client=new Client({name:'clipkit-namespace-preflight',version:'1'});
try{
 const sandbox=await sandboxCommand({workspace:env.workspace,userHome:env.userHome,agent:'node',args:['/opt/clipkit/dist/cli.js','mcp'],plugin:true,runtime:path.join(root,'artifacts/package'),deps:path.join(root,'node_modules')});
 await client.connect(new StdioClientTransport({command:sandbox.command,args:sandbox.args,env:sandbox.env as Record<string,string>,stderr:'pipe'}));
 const tools=await client.listTools();
 const project=await client.callTool({name:'project_get',arguments:{summary:true}});
 const replay=tools.tools.some(t=>t.name==='trace_replay');
 if(project.isError||!replay)throw Error('MCP namespace preflight failed');
 const receipt={ok:true,tools:tools.tools.length,replayTool:replay,projectResponseBytes:Buffer.byteLength(JSON.stringify(project)),environment:sandbox.manifest};
 await mkdir('.clipkit/update4',{recursive:true});await writeFile('.clipkit/update4/plugin-preflight.json',JSON.stringify(receipt,null,2));console.log(JSON.stringify(receipt));
}finally{await client.close();await server.close();await env.cleanup()}
