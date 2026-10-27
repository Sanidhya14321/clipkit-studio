import {it,expect,describe,beforeEach,afterEach} from 'vitest';
import {mkdtemp,cp,rm,readFile,writeFile,mkdir,utimes,symlink} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import {tmpdir} from 'node:os';
import path from 'node:path';
import {parseDocument} from 'yaml';
import {parseCard,readInside,nativeCard,listCards} from '../packages/assets/src/cards.ts';
import {checkAssets} from '../packages/assets/src/check.ts';
const repo=process.cwd();let root:string;
const changeCard=async(file:string,fn:(doc:ReturnType<typeof parseDocument>)=>void)=>{const markdown=await readFile(file,'utf8');const match=markdown.match(/^---\n([\s\S]*?)\n---([\s\S]*)$/)!;const doc=parseDocument(match[1]);fn(doc);await writeFile(file,'---\n'+doc.toString({lineWidth:0})+'---'+match[2]);};
it('rejects malformed/ambiguous YAML and absent real usage guidance',async()=>{
 const markdown=await readFile('packages/assets/cards/text/kinetic-title/card.md','utf8');expect(parseCard(markdown).card['provenance-status']).toBe('agent-authored');
 expect(()=>parseCard(markdown.replace('version: 1','version: 1\nversion: 2'))).toThrow();
 expect(()=>parseCard(markdown.replace(/best-for:.*\n/,'best-for: ""\n'))).toThrow();
 expect(()=>parseCard('---\n'+('x'.repeat(13000))+'\n---\nDescription')).toThrow('12KB');
 expect(()=>parseCard(markdown.replace(/uses:.*\n/,'uses: &loop [*loop]\n'))).toThrow();
});
it('offers bounded creative filters and a complete on-demand native card without changing project content',async()=>{
 const page=await listCards(path.join(repo,'public'),{kind:'text',q:'opening',limit:1});expect(page.items).toHaveLength(1);expect(page.items[0].id).toBe('kinetic-title');expect(page.items[0].provenanceStatus).toBe('agent-authored');
 const first=await listCards(path.join(repo,'public'),{limit:2});expect(first.items).toHaveLength(2);expect(first.next).toBe(2);const next=await listCards(path.join(repo,'public'),{limit:2,offset:first.next!});expect(next.items[0].id).not.toBe(first.items[0].id);
 const card=await nativeCard(path.join(repo,'public'),'kinetic-title');expect(card.markdown).toContain('avoid:');expect(Buffer.byteLength(card.markdown)).toBeLessThanOrEqual(12000);
 await expect(nativeCard(path.join(repo,'public'),'../.env')).rejects.toThrow();
});
describe('asset enforcement against real source fixtures',()=>{
 beforeEach(async()=>{root=await mkdtemp(path.join(tmpdir(),'ck-card-check-'));for(const relative of ['package.json','packages/assets','packages/core/src','packages/render/src','public','scripts/verify-assets.ts','scripts/compare-decoded.ts','scripts/verify-library.ts','docs/ASSET-SYSTEM.md']){const target=path.join(root,relative);await mkdir(path.dirname(target),{recursive:true});await cp(path.join(repo,relative),target,{recursive:true});}},30000);
 afterEach(async()=>{await rm(root,{recursive:true,force:true});});
 it('passes current proofs and exposes the one pinned legacy font exception',async()=>{const report=await checkAssets(root,{checkOnly:true});expect(report.errors).toEqual([]);expect(report.cardCount).toBe(121);expect(report.warnings).toHaveLength(1);expect(report.warnings[0]).toContain('inter-font: legacy size exception');});
 it('fails a missing card and a disallowed license',async()=>{await rm(path.join(root,'packages/assets/cards/text/kinetic-title/card.md'));await changeCard(path.join(root,'packages/assets/cards/text/cta/card.md'),doc=>doc.set('license','GPL-3.0'));const result=await checkAssets(root);expect(result.passed).toBe(false);expect(result.errors.join('\n')).toContain('kinetic-title is missing');expect(result.errors.join('\n')).toContain('License is not in the asset allowlist');});
 it('fails checksum changes and stale render evidence without rewriting the inventory',async()=>{const inventory=path.join(root,'public/packs/native/asset-cards.json');const old=await readFile(inventory,'utf8');await writeFile(path.join(root,'public/vendor/illustration.svg'),'<svg>changed</svg>');const result=await checkAssets(root);expect(result.errors.join('\n')).toContain('Media checksum changed');expect(result.errors.join('\n')).toContain('Render evidence is stale');expect(await readFile(inventory,'utf8')).toBe(old);});
 it('rejects uncarded files and real oversized media',async()=>{const file=path.join(root,'public/vendor/illustration.svg');const bytes=Buffer.alloc(600000,32);await writeFile(file,bytes);await changeCard(file+'.md',doc=>{doc.set('sha256',createHash('sha256').update(bytes).digest('hex'));});await writeFile(path.join(root,'public/previews/orphan.png'),Buffer.from([1,2,3]));const result=await checkAssets(root);expect(result.errors.join('\n')).toContain('Size 600000 exceeds 500000');expect(result.errors.join('\n')).toContain('Uncarded asset file public/previews/orphan.png');});
 it('rejects unknown dependency references and stale generated output',async()=>{await changeCard(path.join(root,'packages/assets/cards/text/cta/card.md'),doc=>doc.set('uses',['unknown-asset']));expect((await checkAssets(root)).errors.join('\n')).toContain('references unknown dependency');await cp(path.join(repo,'packages/assets/cards/text/cta/card.md'),path.join(root,'packages/assets/cards/text/cta/card.md'));await writeFile(path.join(root,'public/packs/native/asset-cards.json'),'[]');const result=await checkAssets(root,{checkOnly:true});expect(result.errors.join('\n')).toContain('Generated inventory is stale');expect(await readFile(path.join(root,'public/packs/native/asset-cards.json'),'utf8')).toBe('[]');});
 it('warns about 48-hour orphans without deleting files',async()=>{const file=path.join(root,'packages/assets/cards/media/unused/card.md');await mkdir(path.dirname(file),{recursive:true});await cp(path.join(root,'packages/assets/cards/overlays/lucide-icons/card.md'),file);await changeCard(file,doc=>doc.set('id','unused-art'));const old=new Date(Date.now()-49*3600000);await utimes(file,old,old);const result=await checkAssets(root);expect(result.passed).toBe(true);expect(result.warnings.join('\n')).toContain('unused-art: unreferenced candidate (card age at least 48 hours)');expect((await readFile(file,'utf8')).length).toBeGreaterThan(0);});
 it('blocks symlink reads outside the registered public root',async()=>{const outside=path.join(root,'private.txt');await writeFile(outside,'private');await symlink(outside,path.join(root,'public/escape.txt'));await expect(readInside(path.join(root,'public'),'escape.txt')).rejects.toThrow('symlink escapes');});
});
