import React from 'react';
import {AbsoluteFill,Sequence,useCurrentFrame,interpolate} from 'remotion';
import {resourceUrl,useResourceFont} from '@clipkit/motion';
import Morph from './components/web-gooey-morphing-text';
import Aperture from './components/web-aperture-reveal';
import Build from './components/web-kinetic-center-build';
import Repeat from './components/codrops-repeat-text';
import Stagger from './components/primitives-text-effect';

const clamp=(x:number)=>Math.max(0,Math.min(1,x));
const ease=(x:number)=>1-(1-clamp(x))**4;
const between=(f:number,a:number,b:number)=>ease((f-a)/(b-a));
const ink='#171717',paper='#f5f1e8';
const type:React.CSSProperties={fontWeight:900,letterSpacing:'-.075em',lineHeight:.86};
function Grain(){return <svg style={{position:'absolute',inset:0,width:'100%',height:'100%',opacity:.035,pointerEvents:'none'}}><filter id="grain"><feTurbulence type="fractalNoise" baseFrequency=".73" numOctaves="2" seed="8"/></filter><rect width="100%" height="100%" filter="url(#grain)"/></svg>}
function Corners({color}:{color:string}){return <>{[0,1,2,3].map(i=><div key={i} style={{position:'absolute',width:28,height:28,left:i%2?undefined:-2,right:i%2?-2:undefined,top:i<2?-2:undefined,bottom:i>=2?-2:undefined,borderTop:i<2?'5px solid '+color:undefined,borderBottom:i>=2?'5px solid '+color:undefined,borderLeft:i%2?undefined:'5px solid '+color,borderRight:i%2?'5px solid '+color:undefined}}/>)}</>}
function Bar({index,label,color=ink}:{index:string;label:string;color?:string}){return <div style={{position:'absolute',top:94,left:80,right:80,color,fontSize:21,fontWeight:650,letterSpacing:'.12em',display:'flex',justifyContent:'space-between'}}><span>CLIPKIT / {index}</span><span>{label}</span></div>}
function Shot({from,to,children}:{from:number;to:number;children:React.ReactNode}){return <Sequence from={from} durationInFrames={to-from}>{children}</Sequence>}
export default function Film(p:any){
  const f=useCurrentFrame(),accent=p.accent,font=useResourceFont('FilmSans',resourceUrl(p.fontResourceId,p.assetBaseUrl,p.accessToken));
  const tag=(key:string,element=key)=>({'data-ck-layer':p.item.id,'data-ck-path':key,'data-ck-element':element});
  const morph={texts:p.morphWords,background:ink,color:paper,size:260,morphFrames:14,holdFrames:20};
  const aperture={lead:p.apertureLead,trail:p.apertureTrail,caption:p.apertureCaption,size:128,captionSize:76,background:paper,color:ink,delayFrames:0,expandFrames:78,imageSourceId:p.artResourceId};
  const build={words:p.kineticWords,background:paper,color:ink,size:96,gap:20,pushFrames:16};
  return <AbsoluteFill style={{background:ink,color:paper,fontFamily:font,overflow:'hidden'}} {...tag('headline','film')}>
    <Shot from={0} to={78}>
      <AbsoluteFill style={{background:accent,color:ink}}>
        <Bar index="01" label="A DIFFERENT CLOCK"/>
        <div style={{position:'absolute',left:78,top:320,width:950,transform:`translateY(${(1-between(f,0,14))*160}px) rotate(${-5+5*between(f,0,32)}deg)`,...type,fontSize:p.headlineSize,whiteSpace:'pre-line'}} {...tag('headline')}>{p.headline}</div>
        <div style={{position:'absolute',left:80,bottom:250,fontSize:40,fontWeight:500,lineHeight:1.15,letterSpacing:'-.035em'}}>The web already knows<br/>how to move.</div>
        <svg viewBox="0 0 120 120" style={{position:'absolute',right:78,bottom:245,width:150,height:150,transform:`rotate(${90*between(f,38,65)}deg)`}}><path d="M12 60H108M62 14L108 60 62 106" fill="none" stroke={ink} strokeWidth="12"/></svg>
        <div style={{position:'absolute',left:0,right:0,bottom:0,height:140*between(f,62,77),background:ink}}/>
      </AbsoluteFill>
    </Shot>
    <Shot from={78} to={164}>
      <AbsoluteFill style={{background:ink}}>
        <Bar index="02" label="THE ACTUAL EFFECT" color={paper}/>
        <div style={{position:'absolute',left:-120,top:440,width:1320,height:650,transform:`rotate(${-7+5*between(f,78,164)}deg)`,...type}}>
          <Morph {...p} {...morph} bindPath="morphWords"/>
        </div>
        <div style={{position:'absolute',left:86,bottom:280,fontSize:43,lineHeight:1.2,letterSpacing:'-.035em',color:paper}}>Keep the mechanism.<br/><span style={{color:accent}}>Change its time.</span></div>
        <div style={{position:'absolute',left:80,right:80,top:1230,height:2,background:'#ffffff33'}}><div style={{height:6,width:`${100*between(f,84,156)}%`,background:accent}}/></div>
      </AbsoluteFill>
    </Shot>
    <Shot from={164} to={266}>
      <AbsoluteFill>
        <Aperture {...p} {...aperture} mediaBaseUrl={p.assetBaseUrl+'/resources'} bindPath="apertureLead"/>
        <Bar index="03" label="OPEN THE FRAME" color={f>210?paper:ink}/>
      </AbsoluteFill>
    </Shot>
    <Shot from={266} to={362}>
      <AbsoluteFill style={{background:accent,color:ink,justifyContent:'center'}}>
        <Bar index="04" label="BORROW / RECOMPOSE"/>
        <div style={{transform:`rotate(-9deg) scale(${1+.15*between(f,266,352)})`,...type,fontSize:260,textAlign:'center',lineHeight:.83}} {...tag('repeatText')}><Repeat text={p.repeatText} increment={105} delay={.06} duration={1.15}/></div>
        <div style={{position:'absolute',bottom:210,left:80,fontSize:38,background:accent,padding:'16px 20px 16px 0',lineHeight:1.2}}>Real source.<br/>New choreography.</div>
      </AbsoluteFill>
    </Shot>
    <Shot from={362} to={458}>
      <AbsoluteFill style={{background:paper,color:ink}}>
        <Bar index="05" label="YOUR AGENT DIRECTS"/>
        <div style={{position:'absolute',left:80,top:400,width:920,...type,fontSize:208}} {...tag('staggerText')}><Stagger text={p.staggerText} per="line" preset="fade-in-blur" duration={.48} stagger={.14}/></div>
        <div style={{position:'absolute',bottom:260,left:80,right:80,display:'flex',alignItems:'center',gap:24,fontSize:27,fontWeight:650}}><span style={{background:accent,width:70,height:70,borderRadius:'50%'}}/>CLAUDE / CODEX / YOUR AGENT</div>
      </AbsoluteFill>
    </Shot>
    <Shot from={458} to={566}>
      <AbsoluteFill style={{background:paper,color:ink}}>
        <Build {...p} {...build} bindPath="kineticWords"/>
        <Bar index="06" label="EDIT THE INTENTION"/>
        <div style={{position:'absolute',left:78,right:78,top:820,height:280,border:'2px solid '+accent,transform:`scale(${.95+.05*between(f,458,478)})`}}><Corners color={accent}/></div>
        <div style={{position:'absolute',left:80,bottom:250,fontSize:40,lineHeight:1.25}}>Change the words.<br/>Keep the movement.</div>
        <div style={{position:'absolute',right:80,bottom:247,padding:'20px 28px',background:ink,color:paper,fontSize:28,borderRadius:40}}>SOURCE INTACT ↗</div>
      </AbsoluteFill>
    </Shot>
    <Shot from={566} to={630}>
      <AbsoluteFill style={{background:accent,color:ink}}>
        <Bar index="07" label="MADE TO KEEP"/>
        <div style={{position:'absolute',left:80,top:400,...type,fontSize:206,transform:`translateX(${(1-between(f,566,578))*900}px)`}}>SHIP<br/>THE<br/>SOURCE.</div>
        <div style={{position:'absolute',left:80,top:1160,fontSize:30,fontFamily:'monospace',lineHeight:1.7,borderTop:'2px solid '+ink,paddingTop:32}}>Film.tsx<br/>components/<br/>assets/ + lockfile</div>
      </AbsoluteFill>
    </Shot>
    <Shot from={630} to={720}>
      <AbsoluteFill style={{background:ink,color:paper}}>
        <Bar index="08" label="WEB ENERGY. FILM CONTROL." color={paper}/>
        <div style={{position:'absolute',left:80,top:360,...type,fontSize:190}}>WEB<br/><span style={{color:accent}}>ENERGY.</span><br/>FILM<br/>CONTROL.</div>
        <div style={{position:'absolute',left:84,right:80,bottom:280,borderTop:'3px solid '+accent,paddingTop:38,fontSize:37,fontWeight:650,letterSpacing:'-.03em'}} {...tag('cta')}>{p.cta}</div>
        <div style={{position:'absolute',bottom:130,right:80,fontSize:22,letterSpacing:'.13em',color:'#b9b5ae'}}>CLIPKIT STUDIO</div>
      </AbsoluteFill>
    </Shot>
    <Grain/>
  </AbsoluteFill>;
}
