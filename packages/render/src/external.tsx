import React,{useEffect,useLayoutEffect,useRef,useState} from 'react';
import * as Remotion from 'remotion';
import {delayRender,continueRender,cancelRender} from 'remotion';
import type {Item} from '../../core/src/index';
import type {AssetRegistry,RuntimeClipDefinition} from '../../assets/src/runtime';
type Component=React.ComponentType<Record<string,unknown>>;
type RuntimeWindow=typeof globalThis & {__CLIPKIT_PACK_RUNTIME__?:unknown;ClipKitPackRegistry?:{components:Record<string,Component>}};
const loads=new Map<string,Promise<Record<string,Component>>>();
function loadRegistry(url:string){if(!loads.has(url))loads.set(url,new Promise((resolve,reject)=>{const runtime=globalThis as RuntimeWindow;runtime.__CLIPKIT_PACK_RUNTIME__={react:React,remotion:Remotion,jsx:(type:React.ElementType,props:Record<string,unknown>,key?:string)=>React.createElement(type,{...props,key})};const script=document.createElement('script');script.src=url;script.onload=()=>resolve(runtime.ClipKitPackRegistry?.components??{});script.onerror=()=>{loads.delete(url);reject(new Error('Trusted component registry could not load'));};document.head.appendChild(script);}));return loads.get(url)!;}
export function ExternalClip({item,definition,registry,assetBaseUrl,context}:{item:Item;definition:RuntimeClipDefinition;registry:AssetRegistry;assetBaseUrl:string;context:Record<string,unknown>}){
 const [components,setComponents]=useState<Record<string,Component>>();const [handle]=useState(()=>delayRender('Loading trusted clip registry'));const container=useRef<HTMLDivElement>(null);
 useEffect(()=>{if(!registry.moduleUrl){cancelRender(new Error('Missing trusted compiled component registry'));return;}const url=new URL(registry.moduleUrl,assetBaseUrl||location.origin).href;let alive=true;void loadRegistry(url).then(value=>{if(alive){setComponents(value);}}).catch(cancelRender);return()=>{alive=false;continueRender(handle);};},[registry.moduleUrl,assetBaseUrl,handle]);
 useLayoutEffect(()=>{if(!components)return;const tagged=Array.from(container.current?.querySelectorAll<HTMLElement>('[data-ck-layer]')??[]); // Check each property against the declared schema; invalid tag paths cannot become inspector edits.
 const valid=tagged.some(el=>el.dataset.ckLayer===item.id&&!!el.dataset.ckElement&&!!el.dataset.ckPath&&Object.prototype.hasOwnProperty.call(definition.propsSchema.properties,el.dataset.ckPath));
 if(!valid||tagged.some(el=>el.dataset.ckLayer!==item.id||!el.dataset.ckElement||!el.dataset.ckPath||!Object.prototype.hasOwnProperty.call(definition.propsSchema.properties,el.dataset.ckPath)))cancelRender(new Error(`${item.clipId}: rendered editable layer/property/element tags are missing or invalid`));else continueRender(handle);
 },[components,item,definition]);
 const Component=components?.[item.clipId!];if(components&&!Component)throw new Error(`Trusted component ${item.clipId} is not registered`);
 return <div ref={container} style={{position:'absolute',inset:0}}>{Component?<Component {...item.props} {...context} item={item} props={item.props}/>:null}</div>;
}
