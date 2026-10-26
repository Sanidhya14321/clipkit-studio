import {describe,it,expect} from 'vitest';
import {captureAnchor} from '../packages/studio/src/anchor';
describe('studio anchor capture',()=>{
  it('targets an unselected hovered element rather than the selected layer',()=>{
    expect(captureAnchor(2,'background',{layerId:'headline',propPath:'text',elementId:'headline-text',point:{x:.3,y:.4}},{layerId:'background',propPath:'color'})).toEqual({tStart:2,layerId:'headline',propPath:'text',elementId:'headline-text',point:{x:.3,y:.4}});
  });
  it('preserves the explicitly clicked element when leaving the canvas',()=>{
    expect(captureAnchor(4,'headline',undefined,{layerId:'headline',propPath:'subtitle',elementId:'subtitle'},6)).toEqual({tStart:4,tEnd:6,layerId:'headline',propPath:'subtitle',elementId:'subtitle'});
  });
  it('does not attach an old property to a different timeline selection',()=>{
    expect(captureAnchor(1,'footage',undefined,{layerId:'headline',propPath:'text',elementId:'headline-text'})).toEqual({tStart:1,layerId:'footage'});
  });
  it('copies the frozen pointer point and excludes invalid ranges',()=>{
    const hover={layerId:'headline',point:{x:.4,y:.2}};const frozen=captureAnchor(3,undefined,hover,{},1);hover.point.x=.9;expect(frozen).toEqual({tStart:3,layerId:'headline',point:{x:.4,y:.2}});
  });
});
