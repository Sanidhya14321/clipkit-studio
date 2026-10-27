import React from 'react';
import {useCurrentFrame, useVideoConfig} from 'remotion';

const W=1080,H=1920;
const clamp=(v:number)=>Math.max(0,Math.min(1,v));
const ease=(v:number)=>{const t=clamp(v);return t*t*t*(t*(t*6-15)+10);};
const progress=(f:number,a:number,b:number)=>ease((f-a)/(b-a));
const mix=(a:number,b:number,t:number)=>a+(b-a)*t;
const ink='#141412',ivory='#F3EEDF';
const type:React.CSSProperties={fontFamily:'Arial, Helvetica, sans-serif',fontWeight:900,letterSpacing:'-0.075em',lineHeight:.84};
const small:React.CSSProperties={fontFamily:'Arial, Helvetica, sans-serif',fontSize:23,fontWeight:700,letterSpacing:'0.16em'};

export default function Scene({project,item,headline,accent}:{project:any;item:any;headline:string;accent:string}){
 const frame=useCurrentFrame();
 const {width,height}=useVideoConfig();
 const coral=accent||'#FF604B';
 const title=headline||'CLIPKIT. MAKE IT YOURS.';
 const c1=progress(frame,92,114),c2=progress(frame,205,229),c3=progress(frame,320,344);
 const camera=H*(c1+c2+c3);
 const a=frame,b=frame-106,c=frame-220,d=frame-334;
 const panel=(top:number,bg:string):React.CSSProperties=>({position:'absolute',left:0,top,width:W,height:H,background:bg,overflow:'hidden'});
 const label=(left:number,top:number,color:string):React.CSSProperties=>({...small,position:'absolute',left,top,color});
 const strike=progress(a,12,23);
 const release=progress(b,4,62);
 const trim=progress(c,14,54);
 const settle=progress(d,5,35);
 return <div style={{position:'absolute',inset:0,overflow:'hidden',background:ink}}>
  <div style={{position:'absolute',width:W,height:H,transform:`scale(${width/W},${height/H})`,transformOrigin:'0 0',overflow:'hidden'}}>
   <div style={{position:'absolute',width:W,height:4*H,transform:`translateY(${-camera}px)`}}>
    <section style={panel(0,ivory)}>
     <div style={label(66,78,ink)}>CLIPKIT / A NEW WAY TO CREATE</div>
     <div style={{position:'absolute',right:66,top:76,width:40,height:40,background:coral,transform:`rotate(${mix(-45,0,progress(a,0,18))}deg)`}}/>
     <div style={{position:'absolute',left:65,top:307,...type,color:ink,fontSize:284,transform:`translateX(${mix(-65,0,progress(a,0,10))}px)`}}>STOP</div>
     <div style={{position:'absolute',left:69,top:561,...type,color:ink,fontSize:217,transform:`translateX(${mix(100,0,progress(a,5,18))}px)`}}>MAKING</div>
     <div style={{position:'absolute',left:61,top:767,...type,color:coral,fontSize:244,transform:`translateY(${mix(45,0,progress(a,10,24))}px)`}}>SLIDES.</div>
     <div style={{position:'absolute',left:70,top:1003,width:940*strike,height:19,background:ink,transformOrigin:'left'}}/>
     <div style={{position:'absolute',left:70,top:1177,width:940,height:433}}>
      {[2,1,0].map((n)=><div key={n} style={{position:'absolute',inset:0,border:`3px solid ${ink}`,background:n===0?ivory:coral,transform:`translate(${n*mix(0,14,progress(a,25,55))}px,${n*mix(0,26,progress(a,25,55))}px) rotate(${n*mix(0,-2,progress(a,25,55))}deg)`}}>
       {n===0&&<><div style={{position:'absolute',left:33,top:30,width:121,height:14,background:ink}}/><div style={{position:'absolute',left:33,top:66,width:204,height:14,background:ink}}/><div style={{position:'absolute',left:33,bottom:30,...small,fontSize:19}}>A THOUGHT IS NOT A STATIC THING.</div><svg width="180" height="180" viewBox="0 0 180 180" style={{position:'absolute',right:40,top:90}}><path d="M15 140 L15 40 L115 40 M15 140 L140 15 M90 15 H140 V65" fill="none" stroke={ink} strokeWidth="13"/></svg></>}
      </div>)}
     </div>
     <div style={label(70,1780,ink)}>01 / THE THOUGHT</div>
     <div style={{...label(865,1780,ink),letterSpacing:0}}>↓</div>
    </section>

    <section style={panel(H,coral)}>
     <div style={label(66,78,ink)}>02 / THE DIRECTION</div>
     <svg width="1080" height="1920" viewBox="0 0 1080 1920" style={{position:'absolute',inset:0}}>
      <path d="M-180 1560 H210 Q360 1560 360 1410 V1180 Q360 1040 500 1040 H770 Q920 1040 920 890 V280" fill="none" stroke={ivory} strokeWidth="7" strokeDasharray="2600" strokeDashoffset={2600*(1-release)}/>
      <path d="M878 330 L920 280 L962 330" fill="none" stroke={ivory} strokeWidth="7" opacity={release}/>
     </svg>
     <div style={{position:'absolute',left:61,top:337,...type,fontSize:226,color:ink,transform:`translateX(${mix(-850,0,progress(b,0,25))}px)`}}>DIRECT</div>
     <div style={{position:'absolute',left:67,top:546,...type,fontSize:255,color:ink,transform:`translateX(${mix(850,0,progress(b,5,30))}px)`}}>THE</div>
     <div style={{position:'absolute',left:59,top:790,...type,fontSize:215,color:ivory,transform:`translateX(${mix(-900,0,progress(b,10,35))}px)`}}>MOTION.</div>
     <div style={{position:'absolute',left:70,top:1220,width:180,height:180,background:ink,transform:`translate(${mix(0,548,release)}px,${mix(120,-76,release)}px) rotate(${mix(-22,0,release)}deg)`}}>
      <svg width="180" height="180" viewBox="0 0 180 180"><path d="M55 125 L125 55 M55 55 H125 V125" fill="none" stroke={ivory} strokeWidth="11"/></svg>
     </div>
     <div style={{position:'absolute',left:68,top:1630,...small,fontSize:31,letterSpacing:'-.02em',lineHeight:1.3,color:ink}}>Give the thought a direction.<br/>Let every move mean something.</div>
     <div style={{position:'absolute',left:68,bottom:110,width:940,height:3,background:ink}}/>
    </section>

    <section style={panel(2*H,ivory)}>
     <div style={label(66,78,ink)}>03 / THE MOMENT</div>
     <div style={{position:'absolute',left:65,top:320,...type,fontSize:271,color:ink,transform:`translateY(${mix(70,0,progress(c,0,22))}px)`}}>EDIT</div>
     <div style={{position:'absolute',left:68,top:565,...type,fontSize:255,color:ink}}>THE</div>
     <div style={{position:'absolute',left:61,top:807,...type,fontSize:186,color:ink}}>MOMENT.</div>
     <div style={{position:'absolute',left:0,top:1100,width:1080,height:480,background:ink,overflow:'hidden'}}>
      <div style={{position:'absolute',left:mix(-150,-36,trim),top:115,...type,fontSize:236,color:ivory,whiteSpace:'nowrap',letterSpacing:'-.065em'}}>MOMENT</div>
      <div style={{position:'absolute',left:0,top:0,bottom:0,width:mix(0,225,trim),background:ivory,opacity:.91}}/>
      <div style={{position:'absolute',right:0,top:0,bottom:0,width:mix(0,225,trim),background:ivory,opacity:.91}}/>
      {[225,855].map((x,i)=><div key={x} style={{position:'absolute',left:mix(i===0?0:1072,x-4,trim),top:0,width:8,height:480,background:coral}}><div style={{position:'absolute',left:i===0?0:-28,top:0,width:36,height:39,background:coral}}/><div style={{position:'absolute',left:i===0?0:-28,bottom:0,width:36,height:39,background:coral}}/></div>)}
      <div style={{position:'absolute',left:mix(280,591,progress(c,55,73)),top:0,width:3,height:480,background:coral,opacity:progress(c,48,57)}}/>
     </div>
     <div style={{position:'absolute',top:1640,left:70,...small,fontSize:31,letterSpacing:'-.02em',color:ink}}>A little earlier. A little later.<br/><span style={{display:'inline-block',marginTop:12,color:coral}}>Exactly yours.</span></div>
     <div style={{position:'absolute',left:70,bottom:89,display:'flex',gap:13}}>{Array.from({length:31},(_,i)=><div key={i} style={{width:17,height:i%5===0?31:13,background:i===17?coral:ink,opacity:i===17?1:.3}}/>)}</div>
    </section>

    <section style={panel(3*H,ink)}>
     <div style={label(66,78,ivory)}>YOUR THOUGHT. YOUR TIMING. YOUR CALL.</div>
     <div style={{position:'absolute',left:70,top:286,width:184,height:184,background:coral,transform:`rotate(${mix(-90,0,settle)}deg) scale(${mix(.5,1,settle)})`,opacity:progress(d,0,12)}}>
      <svg viewBox="0 0 184 184" width="184" height="184"><path d="M119 43 H65 L39 69 V115 L65 141 H119 V113 H79 L67 101 V83 L79 71 H119 Z M119 71 L147 43 V141 L119 113 Z" fill={ink}/></svg>
     </div>
     <div data-ck-layer={item?.id} data-ck-path="headline" data-ck-element="headline" style={{position:'absolute',left:61,right:50,top:595,...type,color:ivory,transform:`translateY(${mix(90,0,settle)}px)`,opacity:progress(d,3,20)}}>
      {title==='CLIPKIT. MAKE IT YOURS.'?<><div style={{fontSize:230}}>CLIPKIT.</div><div style={{fontSize:181,marginTop:61}}>MAKE IT</div><div style={{fontSize:239,color:coral,marginTop:19}}>YOURS.</div></>:<div style={{fontSize:title.length>32?135:185,lineHeight:.96,overflowWrap:'anywhere'}}>{title}</div>}
     </div>
     <div style={{position:'absolute',left:70,top:1295,height:13,width:940*progress(d,22,44),background:coral}}/>
     <div style={{position:'absolute',left:70,top:1400,...small,fontSize:32,letterSpacing:'-.025em',color:ivory,opacity:progress(d,30,48),transform:`translateY(${mix(20,0,progress(d,30,48))}px)`}}>From a thought to a film.<br/><span style={{display:'inline-block',marginTop:13,opacity:.6}}>Stay in creative control.</span></div>
     <div style={{position:'absolute',left:70,right:70,bottom:106,display:'flex',alignItems:'center',justifyContent:'space-between',opacity:progress(d,35,52)}}><span style={{...small,color:ivory,fontSize:22}}>CLIPKIT</span><svg width="63" height="39" viewBox="0 0 63 39"><path d="M0 19.5 H58 M41 3 L58 19.5 L41 36" fill="none" stroke={coral} strokeWidth="5"/></svg></div>
    </section>
   </div>
  </div>
 </div>;
}
