import {useLayoutEffect,type ReactNode} from 'react';
import {useCurrentFrame,useVideoConfig} from 'remotion';
import {seekFrame} from '../../assets/src/frame-clock';
/** Supply a reviewed paused Web Animation or seekable timeline; never run upstream setup implicitly. */
export function FrameClockAdapter({animation,children}:{animation:{pause:()=>unknown;currentTime?:number|null;time?:(seconds:number,suppressEvents?:boolean)=>unknown}|null;children:ReactNode}){const frame=useCurrentFrame(),{fps}=useVideoConfig();useLayoutEffect(()=>{if(animation)seekFrame(animation,frame,fps)},[animation,frame,fps]);return children}
