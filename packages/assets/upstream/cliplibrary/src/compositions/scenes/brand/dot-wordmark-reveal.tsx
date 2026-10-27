import { useMemo } from 'react';
import { AbsoluteFill, Composition, Easing, useCurrentFrame, useVideoConfig } from 'remotion';
import {
  type DotWordmarkRevealProps,
  dotWordmarkRevealDefaults,
  dotWordmarkRevealMeta,
  dotWordmarkRevealSchema,
} from '@/schemas';
import { buildBeats } from '../../dot-wordmark-reveal/beats';
import { BrandField } from '../../dot-wordmark-reveal/brand-field';
import { frameState } from '../../dot-wordmark-reveal/frame-state';
import { buildLockup } from '../../dot-wordmark-reveal/lockup';
import { PaperLockup, Streaks, TilePulse } from '../../dot-wordmark-reveal/paper-lockup';
import { Tagline, useTaglineLayout } from '../../dot-wordmark-reveal/tagline';
import { tween } from '../../promo/timing';
import { TransitionFrame } from '../../promo/transition-frame';
import type { SceneModule } from '../../scene-module';

const DotWordmarkReveal = (props: DotWordmarkRevealProps) => {
  const frame = useCurrentFrame() * props.motion.speed;
  const { width, height } = useVideoConfig();
  const { copy, look, type, dot, timing, roll } = props;
  const layout = useTaglineLayout(copy.tagline, type, width);
  const lockup = useMemo(
    () => buildLockup(copy.wordmark, copy.monogram, type, width, height / 2),
    [copy.wordmark, copy.monogram, type, width, height],
  );
  const beats = useMemo(
    () => buildBeats(timing, layout.words.length, lockup.letters.length),
    [timing, layout.words.length, lockup.letters.length],
  );
  const state = frameState(props, layout, lockup, beats, frame, { width, height });
  const ink = state.inverted ? look.onBrand : look.brand;
  const { pill } = state;

  return (
    <TransitionFrame motion={props.motion}>
      <AbsoluteFill style={{ backgroundColor: look.showBackdrop ? look.paper : 'transparent' }}>
        {state.inverted ? <AbsoluteFill style={{ backgroundColor: look.flash }} /> : null}
        <Tagline
          layout={layout}
          frame={frame}
          appearAt={beats.landings}
          fadeStart={beats.centerLand}
          fadeEnd={beats.fadeEnd}
          centerY={state.textY}
          color={ink}
          type={type}
        />
        <svg
          width={width}
          height={height}
          viewBox={`0 0 ${width} ${height}`}
          style={{ position: 'absolute', inset: 0, overflow: 'visible' }}
          aria-hidden="true"
        >
          {pill ? (
            <rect
              x={-pill.length / 2}
              y={-dot.size / 2}
              width={pill.length}
              height={dot.size}
              rx={dot.size / 2}
              fill={ink}
              style={{ filter: `blur(${pill.blur.toFixed(2)}px)` }}
              transform={`translate(${pill.x} ${pill.y}) rotate(${pill.angle})`}
            />
          ) : null}
          <TilePulse
            lockup={lockup}
            center={state.badgeCenter}
            scale={state.scale}
            progress={tween(frame, [beats.irisEnd - 3, beats.irisEnd + 20], [0, 1], Easing.linear)}
            corner={look.badgeCorner}
            strength={look.pulse}
            color={look.accent}
          />
          {frame >= beats.irisStart ? (
            <PaperLockup
              lockup={lockup}
              look={look}
              accentLetter={copy.accentLetter}
              scale={state.scale}
              canvasWidth={width}
              badgeCenter={frame >= beats.rollStart ? state.badgeCenter : null}
            />
          ) : null}
          {roll.streaks && frame >= beats.rollStart ? (
            <Streaks
              lockup={lockup}
              badgeCenter={state.badgeCenter}
              travelled={state.offsetX}
              strength={
                (state.rollSpeed / 40) *
                Math.min(
                  1,
                  Math.max(
                    0,
                    (width + lockup.badgeRadius * 2 - state.badgeCenter) / lockup.badgeRadius,
                  ),
                )
              }
              color={look.streak}
            />
          ) : null}
          {state.showField ? (
            <BrandField
              frame={frame}
              beats={beats}
              lockup={lockup}
              look={look}
              accentLetter={copy.accentLetter}
              canvasWidth={width}
              canvasHeight={height}
              clip={state.clip}
              scale={state.scale}
              offsetX={state.offsetX}
              spin={state.spin}
              blur={roll.blur * Math.min(1, state.rollSpeed / 80)}
              edgeBlur={state.edgeBlur}
            />
          ) : null}
        </svg>
      </AbsoluteFill>
    </TransitionFrame>
  );
};

export const dotWordmarkRevealScene: SceneModule = {
  id: 'dot-wordmark-reveal',
  component: DotWordmarkReveal,
  render: (key) => (
    <Composition
      key={key}
      id="dot-wordmark-reveal"
      component={DotWordmarkReveal}
      schema={dotWordmarkRevealSchema}
      defaultProps={dotWordmarkRevealDefaults}
      durationInFrames={dotWordmarkRevealMeta.durationInFrames}
      fps={dotWordmarkRevealMeta.fps}
      width={dotWordmarkRevealMeta.width}
      height={dotWordmarkRevealMeta.height}
    />
  ),
};
