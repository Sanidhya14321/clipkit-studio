export type AudioMode='auto'|'silent'|'required';
export function exportOptions(a:Record<string,any>,settings:{width:number;height:number;fps:number}){
  const presets:Record<string,{width:number;height:number}>={'vertical-720':{width:720,height:1280},'vertical-1080':{width:1080,height:1920},square:{width:1080,height:1080},landscape:{width:1920,height:1080}};
  if(a.preset!==undefined&&!presets[a.preset])throw Error('Unknown export preset');
  if((a.width===undefined)!==(a.height===undefined))throw Error('Specify both width and height');
  const size=a.width!==undefined?{width:a.width,height:a.height}:a.preset?presets[a.preset]:settings;
  for(const v of [size.width,size.height])if(!Number.isInteger(v)||v<64||v>3840||v%2)throw Error('Dimensions must be even integers from 64 to 3840');
  const fps=a.fps??settings.fps;if(!Number.isInteger(fps)||fps<1||fps>120)throw Error('FPS must be an integer from 1 to 120');
  const audio:AudioMode=a.audio??'auto';if(!['auto','silent','required'].includes(audio))throw Error('Audio mode must be auto, silent or required');
  const quality=a.crf??({draft:28,standard:20,high:16} as Record<string,number>)[a.quality??'standard'];
  if(!Number.isInteger(quality)||quality<0||quality>51)throw Error('CRF must be an integer from 0 to 51');
  return {preset:{width:size.width,height:size.height},fps,audio,quality};
}
