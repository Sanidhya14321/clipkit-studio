/** Codrops TextRepetitionEffect, MIT. See toolkit-sources.json and ../sources/codrops-repeat/.
 * Retains nine spans, signed yPercent, per-span delay and power1 easing.
 * React supplies layout; a frame-sought paused timeline replaces observer/scroll/ticker.
 */
import React from 'react';
import {useGsapTimeline} from '@remotion/gsap';
import {useVideoConfig} from 'remotion';
export default function RepeatedType({text,copies=9,increment=12,delay=.1,duration=1,style={}}:{text:string;copies?:number;increment?:number;delay?:number;duration?:number;style?:React.CSSProperties}){
  const half=Math.floor(copies/2),{fps}=useVideoConfig();
  const ref=useGsapTimeline<HTMLDivElement>(({timeline,selector})=>{
    timeline.to(selector('[data-repeat]'),{duration,ease:'power1',yPercent:(_i:number,target:HTMLElement)=>Number(target.dataset.ty),delay:(_i:number,target:HTMLElement)=>Number(target.dataset.delay)});
  },{dependencies:[text,copies,increment,delay,duration,fps]});
  return <div ref={ref} style={{position:'relative',...style}} aria-label={text}>{Array.from({length:copies},(_,i)=>{
    const last=i===copies-1,upper=i<half,step=upper?half-i:half-(i-half),ty=last?0:(upper?1:-1)*step*increment,wait=last?0:delay*step-delay;
    return <span key={i} {...last?{}:{'data-repeat':'','data-ty':ty,'data-delay':wait}} aria-hidden style={{display:'block',position:last?'relative':'absolute',inset:last?undefined:0,whiteSpace:'pre',transformOrigin:'50% 50%'}}>{text}</span>;
  })}</div>;
}
