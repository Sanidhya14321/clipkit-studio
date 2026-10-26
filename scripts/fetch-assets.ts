import {mkdir,writeFile} from 'node:fs/promises';
import {createHash} from 'node:crypto';
import path from 'node:path';
const target=path.resolve(import.meta.dirname,'../public/vendor');
// Immutable raw URLs avoid mutable branches, API quotas, and repository clones.
const files=[
  {
    "file": "inter.ttf",
    "url": "https://raw.githubusercontent.com/rsms/inter/353b61b9f4430d5f420d56605a6e7993e0941470/docs/font-files/InterVariable.ttf",
    "sha256": "4989b125924991b90d05b2d16e0e388c48f7d5bb8b30539bbf9c755278d0ccaf"
  },
  {
    "file": "inter-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/rsms/inter/353b61b9f4430d5f420d56605a6e7993e0941470/LICENSE.txt",
    "sha256": "262481e844521b326f5ecd053e59b98c8b2da78c8ee1bdbb6e8174305e54935a"
  },
  {
    "file": "space-grotesk.ttf",
    "url": "https://raw.githubusercontent.com/floriankarsten/space-grotesk/03507d024a01282884232081fc6011c09ff4e849/fonts/ttf/SpaceGrotesk[wght].ttf",
    "sha256": "acad6de1fc93436f5c0f1f4137751ef04f1aea3063e7036535970ffcfbd79f72"
  },
  {
    "file": "space-grotesk-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/floriankarsten/space-grotesk/03507d024a01282884232081fc6011c09ff4e849/OFL.txt",
    "sha256": "564ce565c371c5e5bbf286006565a7c9aa55a9f56e7ca58d56e05d649dd61a72"
  },
  {
    "file": "illustration.svg",
    "url": "https://raw.githubusercontent.com/realvjy/illlustrations/b6d6f3286995e6efbbf397eff21b572245c336fd/content/illlustrations/101/101-gym-guy.svg",
    "sha256": "1ed6d84a8e4aff43d280dcc95470c505c869c140f32220d65224b4e7c17f9caa"
  },
  {
    "file": "illlustrations-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/realvjy/illlustrations/b6d6f3286995e6efbbf397eff21b572245c336fd/LICENSE",
    "sha256": "b1418458e6ea8c65567ce82af3c724fb16eeaa1b29e39c7074bc4751a8d9194b"
  },
  {
    "file": "ui-click.wav",
    "url": "https://raw.githubusercontent.com/Calinou/kenney-ui-audio/8c3d81b9159d058c444f89d12d518276b0b09345/addons/kenney_ui_audio/click1.wav",
    "sha256": "dd90b97f38b0ec7d52161d7c3bf8d61a66655766fd0af06439963351b4853a83"
  },
  {
    "file": "kenney-ui-audio-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/Calinou/kenney-ui-audio/8c3d81b9159d058c444f89d12d518276b0b09345/LICENSE.txt",
    "sha256": "4f88ab3c885c87874834441a0d009cea8942f57461d7b870be65cf4e31362073"
  },
  {
    "file": "lucide-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/lucide-icons/lucide/1802f33f6c270395fcd981067f22719140e2ffbc/LICENSE",
    "sha256": "b495047bd93a9b06913511076f504daba17d5bbeb3e0650f3bb53a4220329c57"
  },
  {
    "file": "references/RenderComp--free-remotion-templates-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/RenderComp/free-remotion-templates/f648980e528defb7594e24c6447055f0540f340b/LICENSE",
    "sha256": "3e0f0f2895a6bf2b5aa9d03aa4a6b68679a7d037d54f24177f99748084a58444"
  },
  {
    "file": "references/lucide-icons--lucide-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/lucide-icons/lucide/1802f33f6c270395fcd981067f22719140e2ffbc/LICENSE",
    "sha256": "b495047bd93a9b06913511076f504daba17d5bbeb3e0650f3bb53a4220329c57"
  },
  {
    "file": "references/rsms--inter-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/rsms/inter/353b61b9f4430d5f420d56605a6e7993e0941470/LICENSE.txt",
    "sha256": "262481e844521b326f5ecd053e59b98c8b2da78c8ee1bdbb6e8174305e54935a"
  },
  {
    "file": "references/floriankarsten--space-grotesk-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/floriankarsten/space-grotesk/03507d024a01282884232081fc6011c09ff4e849/OFL.txt",
    "sha256": "564ce565c371c5e5bbf286006565a7c9aa55a9f56e7ca58d56e05d649dd61a72"
  },
  {
    "file": "references/realvjy--illlustrations-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/realvjy/illlustrations/b6d6f3286995e6efbbf397eff21b572245c336fd/LICENSE",
    "sha256": "b1418458e6ea8c65567ce82af3c724fb16eeaa1b29e39c7074bc4751a8d9194b"
  },
  {
    "file": "references/boringdesigners--boring-avatars-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/boringdesigners/boring-avatars/d0ff2582a8921b643a89de4a4912be28938a828b/LICENSE",
    "sha256": "528d05b9822f5bfe55d99d68521d12e79fb02cbf40909402fd1c11a488831ecc"
  },
  {
    "file": "references/marvelapp--devices.css-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/marvelapp/devices.css/a5448c0ab819d7f3050e80b60208f891e92fe60b/LICENSE",
    "sha256": "0c1057f1f0e2ea88da22df6c3f53c3de28dfb7ce2d8a02792836a1d6dafd194e"
  },
  {
    "file": "references/jdecked--twemoji-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/jdecked/twemoji/b6b55fef1e8636b540a6d016a4729ca8cdf2e60b/LICENSE-GRAPHICS",
    "sha256": "8ae9438818c26e4873b91d8c6ad620526c011e27e125677f13031eda903f007c"
  },
  {
    "file": "references/useAnimations--react-useanimations-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/useAnimations/react-useanimations/035f164df779da512f06f1de4b0b59f7bf5624ba/LICENSE",
    "sha256": "8df8d669d974995cd174ca0cd72c7fd5275e0da893f81c3dc2e1fed25c034535"
  },
  {
    "file": "references/Calinou--kenney-ui-audio-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/Calinou/kenney-ui-audio/8c3d81b9159d058c444f89d12d518276b0b09345/LICENSE.txt",
    "sha256": "4f88ab3c885c87874834441a0d009cea8942f57461d7b870be65cf4e31362073"
  },
  {
    "file": "references/facebookincubator--Meta-open-sound-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/facebookincubator/Meta-open-sound/251c0be57d5d1b3242b91f53e0cde62ab1cb42d9/LICENSE",
    "sha256": "a2010f343487d3f7618affe54f789f5487602331c0a8d03f49e9a7c547cf0499"
  },
  {
    "file": "references/Meta-open-sound-Oculus-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/facebookincubator/Meta-open-sound/251c0be57d5d1b3242b91f53e0cde62ab1cb42d9/corpora/oculus-audio-pack/LICENSE",
    "sha256": "9e5f1b3c610b9c2da5c313bf81d577a7d1acec686bdb0384edefa6df0f90cd94"
  },
  {
    "file": "references/tabler--tabler-icons-LICENSE.txt",
    "url": "https://raw.githubusercontent.com/tabler/tabler-icons/a4ce1404bc6d24d3c365afe7b258d6bf6f48d62d/LICENSE",
    "sha256": "b740a1d46122672da62833e97f7e7c8a13fa85cbc7445b584b297cc00dde93db"
  }
];
for(const file of files){
 const response=await fetch(file.url);if(!response.ok)throw new Error(`Fetch ${file.url}: ${response.status}`);
 const bytes=Buffer.from(await response.arrayBuffer());
 if(createHash('sha256').update(bytes).digest('hex')!==file.sha256)throw new Error(`SHA256 mismatch: ${file.file}`);
 const output=path.join(target,file.file);await mkdir(path.dirname(output),{recursive:true});await writeFile(output,bytes);
}
console.log(`Verified and fetched ${files.length} pinned files to ${target}`);
