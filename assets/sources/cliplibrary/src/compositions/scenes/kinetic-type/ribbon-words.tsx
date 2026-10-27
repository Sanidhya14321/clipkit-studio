import { useId } from 'react';
import { AbsoluteFill, Composition, useCurrentFrame, useVideoConfig } from 'remotion';
import {
  type RibbonWordsProps,
  ribbonWordsDefaults,
  ribbonWordsMeta,
  ribbonWordsSchema,
} from '@/schemas';
import { Backdrop } from '../../promo/backdrop';
import { useCanvasFont } from '../../promo/desk/canvas-texture';
import { displayFont, displayFontReady } from '../../promo/fonts';
import { TransitionFrame } from '../../promo/transition-frame';
import { RibbonStrand, type StrandColors } from '../../ribbon-words/ribbon-strand';
import type { SceneModule } from '../../scene-module';

const MAIN_LANE = { offsetY: 0, shift: 0 };

const strandColors = (props: RibbonWordsProps): StrandColors => {
  const { inverted, ribbon, look } = props;
  const ribbonColor = inverted ? look.background : ribbon.color;
  const fieldColor = inverted ? ribbon.color : look.background;
  return {
    ribbon: ribbonColor,
    text: ribbon.style === 'outline' ? ribbonColor : fieldColor,
    accent: look.accent,
  };
};

const RibbonWords = (props: RibbonWordsProps) => {
  const { text, second, inverted, ribbon, words, look, motion } = props;
  useCanvasFont(`${words.weight} 100px "${displayFont}"`, displayFontReady);
  const frame = useCurrentFrame() * motion.speed;
  const { width, height } = useVideoConfig();
  const baseId = `ribbon-${useId().replace(/[^a-zA-Z0-9-]/g, '')}`;
  const colors = strandColors(props);
  const stageLook = inverted
    ? { ...look, background: ribbon.color, backgroundEdge: ribbon.color }
    : look;
  return (
    <TransitionFrame motion={motion}>
      {look.showBackdrop ? <Backdrop look={stageLook} /> : null}
      <AbsoluteFill>
        <svg width={width} height={height} viewBox={`0 0 ${width} ${height}`} aria-hidden="true">
          {second.enabled ? (
            <RibbonStrand
              props={props}
              lane={{ offsetY: second.offsetY, shift: second.shift }}
              text={second.text}
              delay={second.delay}
              colors={colors}
              frame={frame}
              frameWidth={width}
              pathId={`${baseId}-second`}
            />
          ) : null}
          <RibbonStrand
            props={props}
            lane={MAIN_LANE}
            text={text}
            delay={0}
            colors={colors}
            frame={frame}
            frameWidth={width}
            pathId={`${baseId}-main`}
          />
        </svg>
      </AbsoluteFill>
    </TransitionFrame>
  );
};

export const ribbonWordsScene: SceneModule = {
  id: 'ribbon-words',
  component: RibbonWords,
  render: (key) => (
    <Composition
      key={key}
      id="ribbon-words"
      component={RibbonWords}
      schema={ribbonWordsSchema}
      defaultProps={ribbonWordsDefaults}
      durationInFrames={ribbonWordsMeta.durationInFrames}
      fps={ribbonWordsMeta.fps}
      width={ribbonWordsMeta.width}
      height={ribbonWordsMeta.height}
    />
  ),
};
