import type {Anchor} from '../../core/src/index';
export type AnchorTarget = Pick<Anchor,'layerId'|'propPath'|'elementId'|'point'>;
/** Prefer the element under the pointer, otherwise the last explicitly selected element. */
export function captureAnchor(time:number,selectedLayerId:string|undefined,hover:AnchorTarget|undefined,selectedDetail:AnchorTarget,rangeEnd?:number):Anchor {
  const target=hover?.layerId?hover:selectedDetail.layerId===selectedLayerId?selectedDetail:{layerId:selectedLayerId};
  return {tStart:time,...(rangeEnd!==undefined&&rangeEnd>time?{tEnd:rangeEnd}:{}),...(target.layerId?{layerId:target.layerId}:{}),...(target.propPath?{propPath:target.propPath}:{}),...(target.elementId?{elementId:target.elementId}:{}),...(target.point?{point:{...target.point}}:{})};
}
