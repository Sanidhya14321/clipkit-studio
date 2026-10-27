import React from 'react';
import {AbsoluteFill,useCurrentFrame} from 'remotion';
import {TextEffect} from '@clipkit/motion';
const clamp=(x:number)=>Math.max(0,Math.min(1,x));
const p=(f:number,a:number,b:number)=>clamp((f-a)/(b-a));
const ease=(x:number)=>1-Math.pow(1-x,3);
const smooth=(x:number)=>x*x*(3-2*x);
export default function Scene(props:any){
 const f=useCurrentFrame();
 const v={ink:'#111214',ivory:'#FFF2DC',coral:'#FF553F',ribbonWidth:90,glyphScale:1,openingWord:'STOP',signature:'CLIPKIT.',...props,...props.item?.props};
 const {ink,ivory,coral}=v;
 const tag=(name:string,path:string='coral')=>({'data-ck-layer':props.item?.id,'data-ck-path':path,'data-ck-element':name});
 const text=(s:string,x:number,y:number,size:number,width:number,color:string,key:string)=> <text {...tag(key,'ivory')} x={x} y={y} fontSize={size} fontFamily="Impact, 'Arial Narrow', 'Arial Black', sans-serif" fontWeight="900" letterSpacing="-5" textLength={width} lengthAdjust="spacingAndGlyphs" fill={color}>{s}</text>;
 const glyph=(fold:number,color:string,id:string)=> <g {...tag(id,'glyphScale')} fill="none" stroke={color} strokeWidth={56} strokeLinecap="square" strokeLinejoin="miter"><path d="M-120 -170 V170"/><g transform={`rotate(${(1-fold)*-64})`}><path d="M-95 0 L130 -170"/></g><g transform={`rotate(${(1-fold)*64})`}><path d="M-95 0 L130 170"/></g></g>;
 const enter=ease(p(f,0,18)), peel=smooth(p(f,78,108));
 const direct=ease(p(f,105,126)), seq=smooth(p(f,194,225));
 const sel=ease(p(f,210,240)), expand=smooth(p(f,312,348));
 const final=ease(p(f,330,366));
 const openScale=2.45-1.45*enter;
 const ribbonD=`M 925 1060 C ${925-260*peel} ${1060+350*peel}, ${1080-650*peel} ${1300+200*peel}, ${1300-900*peel} 2050`;
 return <AbsoluteFill style={{backgroundColor:ink,overflow:'hidden'}}>
 <svg viewBox="0 0 1080 1920" width="100%" height="100%" style={{position:'absolute',inset:0}}>
 <defs><clipPath id="aperture"><rect x="350" y="1035" width="380" height="530"/></clipPath></defs>
 <rect width="1080" height="1920" fill={ink}/>
 {f<109&&<g>
 <g transform={`translate(${-630*(1-enter)} ${-250*(1-enter)}) translate(540 900) scale(${openScale}) rotate(${-18*p(f,78,108)}) translate(-540 -900)`} opacity={1-p(f,90,105)}>
 <path d="M80 410 H1000 V1100 H80 Z" fill="none" stroke={ivory} strokeOpacity=".15" strokeWidth="2"/>
 {text(v.openingWord,72,674,310,920,ivory,'stop')}
 {text('MAKING',80,876,207,920,ivory,'making')}
 {text('SLIDES',80,1080,222,795,ivory,'slides')}
 <rect {...tag('terminal-period')} x="910" y="1006" width="90" height="74" fill={coral}/>
 <path d="M80 1140 H1000" stroke={coral} strokeWidth="3"/>
 </g>
 {f>=78&&<path {...tag('period-to-ribbon','ribbonWidth')} d={ribbonD} fill="none" stroke={coral} strokeWidth={v.ribbonWidth+1450*p(f,91,105)}/>}
 </g>}
 {f>=96&&f<330&&<g>
 <path d={`M-200 ${1920-2300*smooth(p(f,96,113))} L1280 ${2200-2300*smooth(p(f,96,113))} V2200 H-200Z`} fill={ivory}/>
 {f<222&&<g opacity={1-p(f,198,218)}>
 <g transform={`translate(${90*(1-direct)} ${210*(1-direct)})`} opacity={direct}>
 {text('DIRECT',78,468,222,920,ink,'direct')}
 {text('THE',82,638,165,360,ink,'the')}
 {text('MOTION.',82,842,211,920,ink,'motion')}
 </g>
 <path {...tag('directed-ribbon','ribbonWidth')} d={`M-200 1770 L380 1770 Q760 1770 760 1430 L760 1290 Q760 1100 1130 1060`} fill="none" stroke={coral} strokeWidth={v.ribbonWidth} transform={`translate(0 ${240*(1-direct)})`}/>
 <path d="M-200 1817 L380 1817 Q807 1817 807 1430" fill="none" stroke={ink} strokeWidth="7" opacity=".16"/>
 <g transform={`translate(${460+140*direct} ${1350-40*direct}) scale(${(0.72+0.28*direct)*v.glyphScale})`}>
 {glyph(ease(p(f,106,126)),ink,'directed-fold')}
 </g>
 </g>}
 {f>=194&&<g>
 <path {...tag('ribbon-broadens')} d={`M-250 ${2250-1650*seq} L1330 ${1900-1650*seq} V2300 H-250Z`} fill={coral}/>
 {f>=210&&<rect x="0" y="0" width="1080" height={1920*p(f,210,226)} fill={coral}/>}
 {f>=210&&<g opacity={ease(p(f,218,240))*(1-p(f,310,326))} transform={`translate(0 ${70*(1-sel)})`}>
 {text('EDIT THE',80,445,191,920,ink,'edit-the')}
 {text('MOMENT.',80,653,218,920,ink,'moment')}
 </g>}
 {f>=210&&<g transform={`translate(${450*(1-sel)} 0)`} opacity={1-p(f,312,329)}>
 <path d="M-450 1010 H1600 V1590 H-450Z" fill={ink}/>
 <path d="M-450 990 H1600 M-450 1610 H1600" stroke={ink} strokeWidth="6"/>
 {[-2,-1,0,1,2].map((n)=><g key={n} transform={`translate(${540+n*435} 1300) scale(.68)`} opacity={n===0?1:.28}>{glyph(clamp(1+n*.34),n===0?ivory:coral,'sample-'+n)}</g>)}
 <rect {...tag('selection-aperture','ivory')} x="350" y="1035" width="380" height="530" fill="none" stroke={ivory} strokeWidth="10"/>
 <path d="M350 970 V900 M730 970 V900 M350 1630 V1700 M730 1630 V1700" stroke={ivory} strokeWidth="6"/>
 </g>}
 </g>}
 </g>}
 {f>=312&&<g>
 <g transform={`translate(${540-180*expand} ${1300-340*expand}) scale(${(.68+6.4*expand)*v.glyphScale})`}>
 {glyph(1,ink,'selected-expansion')}
 </g>
 {f>=330&&<rect width="1080" height="1920" fill={ink} opacity={smooth(p(f,330,345))}/>}
 {f>=330&&<g {...tag('authored-frame')} transform={`translate(${1160-300*final} ${1050-90*final}) scale(${2.8-1.3*final})`} opacity={p(f,330,340)}>
 <g fill="none" stroke={coral} strokeWidth="38"><path d="M-510 -490 V490"/><path d={`M-320 0 L${330-220*final} -490 M-320 0 L${330-220*final} 490`}/></g>
 </g>}
 </g>}
 </svg>
 {f>=348&&<div {...tag('closing-signature','ivory')} style={{position:'absolute',left:100,top:660,width:850,color:ivory,fontFamily:"Impact, 'Arial Narrow', 'Arial Black', sans-serif",fontWeight:900,letterSpacing:-6,lineHeight:.94}}>
 <TextEffect text={v.signature} frame={f} start={348} duration={14} stagger={0} per="line" preset="blur-slide" style={{fontSize:185,lineHeight:1}}/>
 <div style={{height:60}}/>
 <TextEffect text={'MAKE IT\nYOURS.'} frame={f} start={352} duration={14} stagger={0} per="line" preset="slide" style={{fontSize:143,lineHeight:.95,whiteSpace:'pre-line'}}/>
 </div>}
 </AbsoluteFill>;
}
