export const techniques=[
{id:'kinetic-phrase',name:'Kinetic phrase',summary:'Keep phrase identity while letters scale, stagger and change spatial relationships.',tags:['type','bold','rhythm']},
{id:'match-movement',name:'Match movement',summary:'Carry direction, scale or velocity across a cut; preserve a focal object.',tags:['cut','continuity']},
{id:'mask-handoff',name:'Mask handoff',summary:'A letterform or foreground object occludes one scene and reveals the next.',tags:['mask','transition']},
{id:'depth-travel',name:'Depth traversal',summary:'Move through CSS perspective planes to reveal information at a useful scale.',tags:['camera','spatial']},
{id:'evidence-punch',name:'Evidence punch-in',summary:'Travel to one real product action, then hold it large enough to read.',tags:['product','proof','hold']},
{id:'editorial-collage',name:'Editorial collage',summary:'Compose cutouts, type and crop changes around one argument; vary density and scale.',tags:['bold','collage']},
{id:'caption-emphasis',name:'Caption emphasis',summary:'Preserve measured word timing and emphasize a decisive phrase without animating every word.',tags:['caption','speech']},
{id:'material-reveal',name:'Material reveal',summary:'Use a frame-seeked shader as a motivated material/reveal, not decorative filler.',tags:['shader','material']}
];
export function motionSearch(query=''){const words=query.toLowerCase().split(/\W+/).filter(Boolean);return techniques.filter(t=>!words.length||words.some(w=>`${t.id} ${t.summary} ${t.tags.join(' ')}`.includes(w))).slice(0,8)}
