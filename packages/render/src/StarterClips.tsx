/** Reviewed adaptations. Original notices and unmodified source files live in packages/assets/vendor. */
import React,{useMemo} from 'react';
import {AbsoluteFill,random,useCurrentFrame} from 'remotion';
import {Lottie} from '@remotion/lottie';
import AvatarBeam from '../../assets/vendor/boringdesigners/src/lib/components/avatar-beam';
import AvatarBauhaus from '../../assets/vendor/boringdesigners/src/lib/components/avatar-bauhaus';
import {useDesignVideoConfig} from './DesignCanvas';
import {starterComponents} from './StarterRegistry';
import {starterSpecs} from '../../assets/src/starter-specs';
import type {Item} from '../../core/src/index';
import {motionEase,unit} from './motion-math';
import checkmark from '../../assets/vendor/useAnimations/icons/checkmark.json';
import loading from '../../assets/vendor/useAnimations/icons/loading.json';
import heart from '../../assets/vendor/useAnimations/icons/heart.json';
import notification from '../../assets/vendor/useAnimations/icons/notification.json';
import download from '../../assets/vendor/useAnimations/icons/download.json';
import upload from '../../assets/vendor/useAnimations/icons/upload.json';
import playPause from '../../assets/vendor/useAnimations/icons/playPause.json';
import microphone from '../../assets/vendor/useAnimations/icons/microphone.json';
const animations:Record<string,any>={checkmark,loading,heart,notification,download,upload,playpause:playPause,microphone};
const annotate=(item:Item,key:string)=>({'data-ck-layer':item.id,'data-ck-path':key,'data-ck-element':key});
export const isStarterClip=(id:string)=>starterSpecs.some(asset=>asset.id===id);
function TintLottie({name,color,speed}:{name:string;color:string;speed:number}){const animation=useMemo(()=>{const data=structuredClone(animations[name]);const hex=color.replace('#','');const normalized=hex.length===3?hex.split('').map(c=>c+c).join(''):hex.padEnd(6,'0');const rgb=[0,2,4].map(i=>parseInt(normalized.slice(i,i+2),16)/255);const visit=(value:any)=>{if(!value||typeof value!=='object')return;if(['st','fl'].includes(value.ty)&&value.c)value.c={a:0,k:[...rgb,1]};for(const child of Object.values(value))if(typeof child==='object')visit(child)};visit(data);return data;},[name,color]);return <Lottie animationData={animation} playbackRate={speed} loop style={{width:280,height:280}}/>;}
export function StarterClip({item,p}:{item:Item;p:Record<string,any>}){
 const frame=useCurrentFrame(),{fps,width,height}=useDesignVideoConfig();const progress=motionEase(frame/(fps*p.revealDuration),p.easing);const spec=starterSpecs.find(asset=>asset.id===item.clipId)!;const entry=starterComponents[item.clipId!];const key=item.clipId!.replace('adapted-','');
 const text=<div {...annotate(item,'text')} style={{whiteSpace:'pre-wrap',fontSize:p.size,fontWeight:650,lineHeight:1.12,textAlign:'center',color:p.color,fontFamily:p.fontFamily??'Space Grotesk'}}>{p.text}</div>;
 let content:React.ReactNode;
 if(item.clipId!.startsWith('lottie-'))content=<div style={{height:'100%',display:'flex',alignItems:'center',justifyContent:'center',flexDirection:'column',gap:36}}><div {...annotate(item,'color')}><TintLottie name={item.clipId!.slice(7)} color={p.color} speed={p.animationSpeed??1}/></div>{text}</div>;
 else if(entry){const props={...entry.defaults};if(entry.textKey?.startsWith('@'))props[entry.textKey.slice(1)]=String(p.text).split(/\n+/).filter(Boolean);else if(entry.textKey)props[entry.textKey]=p.text;if(entry.colorKey)props[entry.colorKey]=p.color;for(const bg of ['background','backgroundColor','bgColor'])if(bg in props)props[bg]=p.background;if('fontSize' in props)props.fontSize=p.size;if('titleSize' in props)props.titleSize=p.size;if(entry.subtitleKey)props[entry.subtitleKey]=p.subtitle;if('duration' in props)props.duration=item.dur;const dataKey='dataKey' in spec?spec.dataKey:undefined;if(dataKey&&p.dataJson)props[dataKey]=JSON.parse(p.dataJson);const Component=entry.Component;content=<div {...annotate(item,'text')} style={{position:'absolute',inset:0}}><Component {...props}/>{!entry.textKey&&<div style={{position:'absolute',left:20,right:20,bottom:'20%'}}>{text}</div>}</div>}
 else if(key.includes('avatar')){const colors=[p.color,'#f0bf84','#344b6b','#98c5bb','#e4e9f2'];content=<div style={{height:'100%',display:'flex',justifyContent:'center',alignItems:'center',flexDirection:'column',gap:40}}><div {...annotate(item,'color')} style={{display:'flex',gap:24}}>{key==='speaker-avatar'?<AvatarBeam name={p.text} colors={colors} size={240}/>:['Ada','Linus','Grace'].map((name,index)=><div key={name} style={{transform:`translateY(${(1-motionEase(frame/(fps*p.revealDuration)-index*.1,p.easing))*50}px)`}}><AvatarBauhaus name={name+p.text} colors={colors} size={145}/></div>)}</div>{text}<div style={{fontSize:24,color:'#b5bfd0'}}>{p.subtitle}</div></div>}
 else if(key==='svg-draw')content=<div style={{height:'100%',display:'flex',flexDirection:'column',justifyContent:'center',gap:48}}><svg {...annotate(item,'color')} viewBox="0 0 600 300" aria-label="Animated path"><path d="M 35 225 C 90 15 150 15 200 190 S 315 295 350 130 S 470 15 555 150" pathLength={1} fill="none" stroke={p.color} strokeWidth={8} strokeLinecap="round" strokeDasharray={1} strokeDashoffset={1-progress}/><circle cx={555} cy={150} r={9*progress} fill={p.color}/></svg>{text}</div>;
 else if(key==='grid-stagger')content=<div style={{height:'100%',display:'flex',flexDirection:'column',justifyContent:'center',gap:55}}><div {...annotate(item,'color')} style={{display:'grid',gridTemplateColumns:'repeat(7,1fr)',gap:14,padding:28}}>{Array.from({length:49},(_,i)=>{const x=i%7,y=Math.floor(i/7),delay=Math.sqrt((x-3)**2+(y-3)**2)*3;const reveal=motionEase((frame-delay)/(fps*p.revealDuration),p.easing);return <div key={i} style={{height:55,borderRadius:9,background:p.color,opacity:.2+reveal*.8,transform:`scale(${.2+reveal*.8}) rotate(${(1-reveal)*45}deg)`}}/>})}</div>{text}</div>;
 else content=<div {...annotate(item,'text')} style={{height:'100%',display:'flex',alignItems:'center',justifyContent:'center',fontSize:p.size,color:p.color,fontWeight:650,textAlign:'center',whiteSpace:'pre-wrap'}}>{Array.from(String(p.text)).map((char,i)=>{const reveal=unit(frame/(fps*p.revealDuration)-i*.018);const scrambled=key==='scramble-title'&&reveal<.8?String.fromCharCode(65+Math.floor(random(`scramble:${i}:${Math.floor(frame/3)}`)*26)):char;return <span key={i} style={{display:'inline-block',opacity:reveal,filter:key==='morph-text'?`blur(${(1-reveal)*12}px)`:undefined,transform:`translateY(${(1-reveal)*25}px)`}}>{scrambled}</span>})}</div>;
 return <AbsoluteFill {...annotate(item,'opacity')} style={{background:p.backgroundMode==='transparent'?'transparent':p.background,overflow:'hidden',opacity:p.opacity*progress,fontFamily:p.fontFamily??'Inter'}}><div style={{position:'absolute',height,width,left:'50%',top:'50%',transform:`translate(-50%,-50%) translate(${p.x}px,${p.y}px) scale(${p.scale*p.width/width})`}}>{content}</div></AbsoluteFill>;
}
