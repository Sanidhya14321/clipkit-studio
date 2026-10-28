import {readFile,realpath} from 'node:fs/promises';
import path from 'node:path';
import {parse} from 'yaml';
import {sha} from './trace';
import {inventory} from '../../assets/src/source-vetting';
// A path flag selects a verified pin; it never grants vetting or license approval.
export async function verifiedGuidance(root:string,selected:string){
 const entries=parse(await readFile(path.join(root,'.clipkit/sources.yaml'),'utf8'));
 const entry=entries.find((e:any)=>e.id==='remotion-skills'&&e.repo==='remotion-dev/skills');
 const lock=JSON.parse(await readFile(path.join(root,'.clipkit/sources.lock'),'utf8'));
 const pin=lock['remotion-skills'];
 if(entry?.status!=='verified'||!entry.enabled||pin?.vet?.result!=='pass'||!['permissive','attribution-required'].includes(pin?.license?.licenseClass)||!pin?.license?.text)throw Error('E_SOURCE_VET: official guidance is not verified; keep quarantined');
 const expected=path.resolve(root,'.clipkit/source-hub/snapshots/remotion-skills',pin.commit);
 if(await realpath(selected)!==await realpath(expected))throw Error('E_SOURCE_VET: guidance must be the verified original snapshot');
 const files=await inventory(expected);
 if(files.length!==Object.keys(pin.inventory).length)throw Error('E_SOURCE_VET: guidance inventory changed');
 for(const file of files)if(sha((await readFile(path.join(expected,file))).toString('base64'))!==pin.inventory[file])throw Error('E_SOURCE_VET: guidance hash changed');
 return path.join(expected,'skills/remotion-best-practices');
}
