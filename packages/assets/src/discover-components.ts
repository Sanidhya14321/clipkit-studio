import type {ComponentEntry} from './component-index';
/** Read repository trees, not full repositories. Refresh is explicit and outside film generation. */
export async function discoverComponents():Promise<{entries:ComponentEntry[];errors:Array<{repository:string;error:string}>}>{
  const families=[
    {prefix:'reactbits',repository:'DavidHDev/react-bits',folder:'src/ts-default/',preview:'https://reactbits.dev/',license:'MIT + Commons Clause'},
    {prefix:'magicui',repository:'magicuidesign/magicui',folder:'apps/www/registry/magicui/',preview:'https://magicui.design/docs/components/',license:'MIT'},
    {prefix:'primitives',repository:'ibelick/motion-primitives',folder:'components/core/',preview:'https://motion-primitives.com/docs/',license:'MIT'},
    {prefix:'animata',repository:'codse/animata',folder:'animata/',preview:'https://animata.design/docs/',license:'MIT'},
    {prefix:'paper',repository:'paper-design/shaders',folder:'packages/shaders/src/shaders/',preview:'https://shaders.paper.design/',license:'Apache-2.0'},
  ];
  const entries:ComponentEntry[]=[],errors:Array<{repository:string;error:string}>=[];
  for(const family of families)try{
    const get=async(url:string)=>{const r=await fetch(url,{headers:{'User-Agent':'ClipKit-Component-Index'},signal:AbortSignal.timeout(30000)});if(!r.ok)throw Error('GitHub '+r.status);return r.json() as Promise<any>};
    const commit=await get(`https://api.github.com/repos/${family.repository}/commits/HEAD`),tree=await get(`https://api.github.com/repos/${family.repository}/git/trees/${commit.sha}?recursive=1`);
    if(tree.truncated)throw Error('Truncated source tree; preserve the previous index');
    for(const node of tree.tree){const file=String(node.path);if(node.type!=='blob'||!file.startsWith(family.folder)||!(/\.tsx$/.test(file)||family.prefix==='paper'&&/\.ts$/.test(file))||/\/(?:utils|hooks|types|index|test)/.test(file))continue;
      const leaf=file.split('/').at(-1)!.replace(/\.(tsx|ts)$/,''),slug=leaf.replace(/([a-z\d])([A-Z])/g,'$1-$2').toLowerCase();if(slug==='index'||slug.length>70)continue;
      const category=file.slice(family.folder.length).split('/').slice(0,-1),tags=[...category,slug].join(' ').replace(/([a-z])([A-Z])/g,'$1 $2').toLowerCase().split(/[\s/-]+/);
      const clock=family.prefix==='paper'?'shader':tags.includes('scroll')?'scroll':tags.some(t=>['cursor','mouse','hover'].includes(t))?'pointer':family.prefix==='magicui'?'css':'motion';
      const route=family.prefix==='reactbits'?(category[0]??'components').replace(/([a-z])([A-Z])/g,'$1-$2').toLowerCase()+'/'+slug:family.prefix==='animata'?[...category,slug].join('/'):slug;
      entries.push({id:family.prefix+'-'+slug,name:leaf.replace(/([a-z])([A-Z])/g,'$1 $2').replace(/-/g,' '),repository:family.repository,sourcePath:file,preview:family.preview+route,mechanism:`${leaf.replace(/([a-z])([A-Z])/g,'$1 $2')} — ${category.join(' / ')||'visual component'}. Indexed source; inspect its preview before selecting.`,useWhen:tags.join(' '),tags,clock,status:'needs-adaptation',cost:clock==='shader'?'high':'medium',license:family.license,revision:commit.sha});
    }
  }catch(e){errors.push({repository:family.repository,error:(e as Error).message})}
  return {entries:[...new Map(entries.map(e=>[e.id,e])).values()],errors};
}
