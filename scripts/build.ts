import {build as viteBuild} from 'vite';
import {build} from 'esbuild';
import {mkdir,cp,readFile,writeFile,chmod,rm} from 'node:fs/promises';
import {resolve,join} from 'node:path';
const root=resolve(import.meta.dirname,'..');
await viteBuild({configFile:join(root,'packages/studio/vite.config.ts')});
await build({entryPoints:[join(root,'packages/cli/src/index.ts')],outfile:join(root,'dist/cli.js'),bundle:true,platform:'node',target:'node22',format:'esm',packages:'external',banner:{js:"import {createRequire as __createRequire} from 'node:module';const require=__createRequire(import.meta.url);"},sourcemap:true});
const distribution=join(root,'artifacts','package');await rm(distribution,{recursive:true,force:true});await mkdir(join(distribution,'dist'),{recursive:true});
await cp(join(root,'dist'),join(distribution,'dist'),{recursive:true});
await cp(join(root,'public'),join(distribution,'public'),{recursive:true});
for(const name of ['core','assets','render'])await cp(join(root,'packages',name,'src'),join(distribution,'runtime','packages',name,'src'),{recursive:true});
const manifest=JSON.parse(await readFile(join(root,'package.json'),'utf8'));
await writeFile(join(distribution,'package.json'),JSON.stringify({name:'clipkit-studio-local',version:'0.1.0',description:'Local, revisioned video collaboration for humans and coding agents',type:'module',bin:{ck:'dist/cli.js'},engines:{node:'>=22'},files:['dist','public','runtime','README.md','LICENSE','THIRD_PARTY_NOTICES.md'],dependencies:manifest.dependencies},null,2));
for(const file of ['README.md','LICENSE','THIRD_PARTY_NOTICES.md'])try{await cp(join(root,file),join(distribution,file));}catch{}
await chmod(join(distribution,'dist','cli.js'),0o755);
console.log('Built studio, CLI, renderer runtime and self-contained package at artifacts/package. Run npm pack ./artifacts/package --pack-destination ./artifacts.');
