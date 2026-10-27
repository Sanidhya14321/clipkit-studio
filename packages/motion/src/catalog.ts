export const techniques=[
{id:'text-morph',name:'Shared-glyph morph',summary:'Words change meaning while matching letters carry position across the cut.',tags:['web','type','shared-element']},
{id:'focus-stagger',name:'Focus-pull typography',summary:'Stagger a phrase through depth and blur, then leave it sharp and still long enough to read.',tags:['web','blur','type']},
{id:'scroll-story',name:'Scripted scroll journey',summary:'Turn a website scroll story into an authored camera path through content planes.',tags:['web','camera','depth']},
{id:'gradient-blinds',name:'Gradient blind handoff',summary:'Strip geometry becomes a cut between subjects, instead of looping decoration behind text.',tags:['web','transition','mask']},
{id:'outlined-trails',name:'Outlined type trails',summary:'Outlined copies catch up to the solid subject and settle into a clean reading pose.',tags:['web','type','trails']},
{id:'perspective-flip',name:'Perspective cluster',summary:'A related group arrives through staggered perspective, then hands off as one object.',tags:['web','perspective','stagger']},
{id:'kinetic-phrase',name:'Kinetic phrase',summary:'Keep phrase identity while letters scale, stagger and change spatial relationships.',tags:['type','bold','rhythm']},
{id:'match-movement',name:'Match movement',summary:'Carry direction, scale or velocity across a cut; preserve a focal object.',tags:['cut','continuity']},
{id:'mask-handoff',name:'Mask handoff',summary:'A letterform or foreground object occludes one scene and reveals the next.',tags:['mask','transition']},
{id:'depth-travel',name:'Depth traversal',summary:'Move through CSS perspective planes to reveal information at a useful scale.',tags:['camera','spatial']},
{id:'evidence-punch',name:'Evidence punch-in',summary:'Travel to one real product action, then hold it large enough to read.',tags:['product','proof','hold']},
{id:'editorial-collage',name:'Editorial collage',summary:'Compose cutouts, type and crop changes around one argument; vary density and scale.',tags:['bold','collage']},
{id:'caption-emphasis',name:'Caption emphasis',summary:'Preserve measured word timing and emphasize a decisive phrase without animating every word.',tags:['caption','speech']},
{id:'material-reveal',name:'Material reveal',summary:'Use a frame-seeked shader as a motivated material/reveal, not decorative filler.',tags:['shader','material']}
];
export function motionSearch(query=''){const words=query.toLowerCase().split(/\W+/).filter(Boolean);return techniques.filter(t=>!words.length||words.some(w=>`${t.id} ${t.summary} ${t.tags.join(' ')}`.includes(w))).sort((a,b)=>{const score=(t:typeof techniques[number])=>words.reduce((n,w)=>n+(t.id.includes(w)?4:0)+(t.tags.includes(w)?2:0),0);return score(b)-score(a)}).slice(0,14)}
