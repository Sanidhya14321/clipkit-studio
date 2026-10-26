import React from 'react';
import {Composition as RemotionComposition, registerRoot} from 'remotion';
import {Composition, type CompositionProps} from './Composition';
import {duration, emptyProject} from '../../core/src/index';
const defaults:CompositionProps={project:emptyProject(),mediaBaseUrl:'',assetBaseUrl:'',preview:false};
const Root=()=> <RemotionComposition id="ClipKit" component={Composition} width={720} height={1280} fps={30} durationInFrames={150} defaultProps={defaults} calculateMetadata={({props})=>({width:props.project.settings.width,height:props.project.settings.height,fps:props.project.settings.fps,durationInFrames:Math.ceil(duration(props.project)*props.project.settings.fps)})}/>;
registerRoot(Root);
