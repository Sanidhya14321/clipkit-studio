import React,{createContext,useContext} from 'react';
import {useVideoConfig} from 'remotion';
export const DesignCanvas=createContext<{width:number;height:number}|undefined>(undefined);
/** Asset layout uses project coordinates even when the export preset has different output dimensions. */
export function useDesignVideoConfig(){const config=useVideoConfig(),design=useContext(DesignCanvas);return {...config,...design};}
