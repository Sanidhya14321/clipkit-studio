/** Frame adaptation of MIT Motion Primitives TextEffect. Originals in packages/motion/upstream.
 * Retains segmentation, wrapper hierarchy and default hidden/visible poses (blur12, y20, scale0).
 * Explicit frame interpolation replaces AnimatePresence's browser clock and callbacks.
 */
import React from 'react';
import {useCurrentFrame,useVideoConfig} from 'remotion';
export default function StaggerType({text,per='word',preset='fade-in-blur',delay=0,duration=.4,stagger,style={}}:{text:string;per?:'char'|'word'|'line';preset?:'blur'|'fade-in-blur'|'scale'|'fade'|'slide';delay?:number;duration?:number;stagger?:number;style?:React.CSSProperties}){
  const frame=useCurrentFrame(),{fps}=useVideoConfig(),segments=per==='line'?text.split('\n'):per==='char'?Array.from(text):text.split(/(\s+)/),step=stagger??({char:.03,word:.05,line:.1}[per]);
  return <div aria-label={text} style={style}>{segments.map((segment,i)=>{
    const t=Math.max(0,Math.min(1,(frame/fps-delay-i*step)/duration)),p=1-(1-t)**3;
    return <span key={i} aria-hidden style={{display:per==='line'?'block':'inline-block',whiteSpace:'pre',overflow:'hidden'}}><span style={{display:'inline-block',opacity:p,filter:preset.includes('blur')?`blur(${12*(1-p)}px)`:undefined,transform:`translateY(${['fade-in-blur','slide'].includes(preset)?20*(1-p):0}px) scale(${preset==='scale'?p:1})`}}>{segment}</span></span>;
  })}</div>;
}
