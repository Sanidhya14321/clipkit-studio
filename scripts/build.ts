import {buildComponents} from '../packages/assets/src/components-build';
import {execFile} from 'node:child_process';
import {promisify} from 'node:util';
import {rename} from 'node:fs/promises';
import {build as viteBuild} from 'vite';
import {build} from 'esbuild';
import {mkdir,cp,readFile,writeFile,chmod,rm} from 'node:fs/promises';
import {resolve,join} from 'node:path';
import {checkAssets} from '../packages/assets/src/check.ts';
const root=resolve(import.meta.dirname,'..');
const cardCheck=await checkAssets(root);if(!cardCheck.passed)throw new Error(cardCheck.errors.join('\n'));
await buildComponents(root);
await viteBuild({configFile:join(root,'packages/studio/vite.config.ts')});
await build({entryPoints:[join(root,'packages/cli/src/index.ts')],outfile:join(root,'dist/cli.js'),bundle:true,platform:'node',target:'node22',format:'esm',packages:'external',banner:{js:"import {createRequire as __createRequire} from 'node:module';const require=__createRequire(import.meta.url);"},sourcemap:true});
const distribution=join(root,'artifacts','package');await rm(distribution,{recursive:true,force:true});await mkdir(join(distribution,'dist'),{recursive:true});
await cp(join(root,'dist'),join(distribution,'dist'),{recursive:true});
await cp(join(root,'public'),join(distribution,'public'),{recursive:true});
for(const name of ['core','assets','render','motion','scenes'])await cp(join(root,'packages',name,'src'),join(distribution,'runtime','packages',name,'src'),{recursive:true});
await cp(join(root,'packages/assets/components'),join(distribution,'runtime/packages/assets/components'),{recursive:true});
await cp(join(root,'packages/scenes/compatibility.json'),join(distribution,'runtime/packages/scenes/compatibility.json'));
await cp(join(root,'packages/motion'),join(distribution,'runtime/packages/motion'),{recursive:true});

const manifest=JSON.parse(await readFile(join(root,'package.json'),'utf8'));
await writeFile(join(distribution,'package.json'),JSON.stringify({name:'clipkit-studio-local',version:manifest.version,description:'Local, revisioned video collaboration for humans and coding agents',type:'module',bin:{ck:'dist/cli.js'},engines:{node:'>=22'},files:['npm-shrinkwrap.json','dist','public','runtime','README.md','LICENSE','THIRD_PARTY_NOTICES.md','docs','assets','DECISIONS.md','sources.yaml','.env.example'],dependencies:manifest.dependencies,overrides:manifest.overrides},null,2));
for(const file of ['README.md','LICENSE','THIRD_PARTY_NOTICES.md','DECISIONS.md','sources.yaml','.env.example'])try{await cp(join(root,file),join(distribution,file));}catch{}
await cp(join(root,'docs'),join(distribution,'docs'),{recursive:true});
await cp(join(root,'assets'),join(distribution,'assets'),{recursive:true});
await chmod(join(distribution,'dist','cli.js'),0o755);
// A dependency's overrides are ignored by npm consumers. Ship a shrinkwrap so
// the tarball also retains the patched transitive dependency versions.
await promisify(execFile)('npm',['install','--package-lock-only','--ignore-scripts','--no-audit','--no-fund'],{cwd:distribution,maxBuffer:4*1024*1024,timeout:180000});
await rename(join(distribution,'package-lock.json'),join(distribution,'npm-shrinkwrap.json'));
// npm ignores dependency-level overrides even with a shrinkwrap when it
// reconciles exact upstream constraints. Bundle the overridden Remotion tree
// so consumers receive the tested patched versions rather than re-resolving it.
await promisify(execFile)('npm',['ci','--omit=dev','--ignore-scripts','--no-audit','--no-fund'],{cwd:distribution,maxBuffer:4*1024*1024,timeout:180000});
const packagedManifest=JSON.parse(await readFile(join(distribution,'package.json'),'utf8'));packagedManifest.bundledDependencies=Object.keys(manifest.dependencies).filter(name=>name.startsWith('@remotion/')||name==='remotion'||name==='lottie-web');await writeFile(join(distribution,'package.json'),JSON.stringify(packagedManifest,null,2));
console.log('Built studio, CLI, renderer runtime and self-contained package at artifacts/package. Run npm pack ./artifacts/package --pack-destination ./artifacts.');
