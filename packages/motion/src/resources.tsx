import {useEffect,useState} from 'react';
import {continueRender,delayRender,cancelRender} from 'remotion';
export function resourceUrl(id:string,base:string,token?:string){return `${base}/resources/${encodeURIComponent(id)}${token?'?token='+encodeURIComponent(token):''}`}
export function useResourceFont(family:string,url:string){
  const [handle]=useState(()=>delayRender('Loading '+family));
  useEffect(()=>{let active=true;const font=new FontFace(family,`url(${JSON.stringify(url)})`,{weight:'100 900'});font.load().then(face=>{if(active)document.fonts.add(face);continueRender(handle)}).catch(cancelRender);return()=>{active=false;continueRender(handle)}},[family,url,handle]);
  return family;
}
