import {spawnSync} from 'node:child_process';import {checkAssets} from '../packages/assets/src/check';
const mode=process.argv[2]??'all';const run=(cmd:string,args:string[])=>{const r=spawnSync(cmd,args,{stdio:'inherit',env:process.env});if(r.status!==0)process.exit(r.status??1)};
if(mode==='assets'||mode==='all'){const r=await checkAssets(process.cwd());console.log(JSON.stringify(r,null,2));if(!r.passed)process.exit(1)}
if(mode==='all'||mode==='unit'){run('pnpm',['typecheck']);run('pnpm',['test'])}
const files:Record<string,string>={components:'components-render.ts',scenes:'scenes.ts',install:'installation.ts'};if(files[mode])await import('../tests/integration/'+files[mode]);if(mode==='browser')run('pnpm',['exec','playwright','test']);
if(!['all','assets','unit','browser',...Object.keys(files)].includes(mode))throw Error('Use all, assets, unit, browser, components, scenes or install');
