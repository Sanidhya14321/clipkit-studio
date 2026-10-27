import { useId } from 'react';
import { Easing } from 'remotion';
import type { DotWordmarkRevealProps } from '@/schemas';
import { glide, settle, smooth, tween } from '../promo/timing';
import type { Beats } from './beats';
import { pathThrough } from './dot-path';
import { glyphEnd } from './glyphs';
import { entryCourse, type Lockup, letterX, orbitPath, swooshCourse } from './lockup';
import { Comet, GlyphStroke } from './strokes';

export interface FieldClip {
  x: number;
  y: number;
  r: number;
  corner: number;
  angle: number;
}

const COMET_TAIL = 0.3;
const SWOOSH_TAIL = COMET_TAIL * 0.35;
const ORBIT_EASE = Easing.bezier(0.45, 0, 0.55, 1);

const cometHead = (
  frame: number,
  start: number,
  arrive: number,
  retract: number,
  tail: number,
  retractEase: (t: number) => number,
): number =>
  frame < arrive
    ? tween(frame, [start, arrive], [0, 1], Easing.bezier(0.4, 0, 0.3, 1))
    : 1 + tail * tween(frame, [arrive, arrive + retract], [0, 1], retractEase);

export const BrandField = ({
  frame,
  beats,
  lockup,
  look,
  accentLetter,
  canvasWidth,
  canvasHeight,
  clip,
  scale,
  offsetX,
  spin,
  blur,
  edgeBlur,
}: {
  frame: number;
  beats: Beats;
  lockup: Lockup;
  look: DotWordmarkRevealProps['look'];
  accentLetter: number;
  canvasWidth: number;
  canvasHeight: number;
  clip: FieldClip;
  scale: number;
  offsetX: number;
  spin: number;
  blur: number;
  edgeBlur: number;
}) => {
  const id = useId().replace(/[^a-zA-Z0-9_-]/g, '');
  const spread = tween(frame, [beats.spreadStart, beats.spreadEnd], [0, 1], smooth);
  const centerX = canvasWidth / 2;
  const cy = lockup.centerY;
  const lettersShown = frame < beats.irisEnd;
  const accentFlip = tween(frame, [beats.accentStart, beats.accentStart + 7], [0, 1], smooth);
  const iris = tween(frame, [beats.irisStart, beats.irisEnd], [0, 1], smooth);
  const glowIn = tween(frame, [beats.growStart + 2, beats.growEnd], [0, 1], smooth);
  const accentDot = tween(frame, [beats.irisStart + 6, beats.irisEnd], [0, 1], settle);
  const monogramDrawn = tween(frame, [beats.monogramStart, beats.monogramEnd], [0, 1], glide);
  const entryHead = cometHead(
    frame,
    beats.writeStart - 6,
    beats.writeStart,
    3,
    COMET_TAIL,
    Easing.linear,
  );
  const swooshHead = cometHead(frame, beats.swooshStart, beats.swooshEnd, 6, SWOOSH_TAIL, smooth);
  const orbitHead = tween(frame, [beats.orbitStart, beats.irisStart + 4], [0, 1.12], ORBIT_EASE);
  const orbitFade = tween(frame, [beats.irisStart - 1, beats.irisStart + 4], [1, 0], Easing.linear);
  const markEnd = glyphEnd(lockup.monogram);
  const dotR = lockup.monogramStroke * 0.55;
  const dotX = lockup.monogramX + markEnd.x * lockup.monogramUnit + lockup.monogramStroke * 1.35;
  const dotY = lockup.monogramTop + lockup.monogramUnit - dotR * 0.2;
  const groupTransform = `translate(${offsetX} 0) translate(${centerX} ${cy}) scale(${scale}) translate(${-centerX} ${-cy})`;
  const spinTransform = `rotate(${spin} ${lockup.badgeX} ${cy})`;
  const clipHalf = Math.max(0, clip.r);
  return (
    <g
      filter={blur > 0.05 ? `url(#${id}-blur)` : undefined}
      style={edgeBlur > 0.05 ? { filter: `blur(${edgeBlur.toFixed(2)}px)` } : undefined}
    >
      <defs>
        <clipPath id={`${id}-clip`}>
          <rect
            x={clip.x - clipHalf}
            y={clip.y - clipHalf}
            width={clipHalf * 2}
            height={clipHalf * 2}
            rx={clipHalf * clip.corner}
            transform={`rotate(${clip.angle} ${clip.x} ${clip.y})`}
          />
        </clipPath>
        <radialGradient id={`${id}-sheen`} cx="0.36" cy="0.3" r="0.75">
          <stop offset="0" stopColor="#ffffff" stopOpacity={0.32} />
          <stop offset="0.6" stopColor="#ffffff" stopOpacity={0.06} />
          <stop offset="1" stopColor="#000000" stopOpacity={0.12} />
        </radialGradient>
        <radialGradient id={`${id}-glow`} cx="0.5" cy="0.5" r="0.5">
          <stop offset="0" stopColor={look.onBrand} stopOpacity={1} />
          <stop offset="1" stopColor={look.onBrand} stopOpacity={0} />
        </radialGradient>
        <filter id={`${id}-blur`} x="-60%" y="-10%" width="220%" height="120%">
          <feGaussianBlur stdDeviation={`${blur.toFixed(2)} 0`} />
        </filter>
      </defs>
      <g clipPath={`url(#${id}-clip)`}>
        <rect
          x={-canvasWidth}
          y={-canvasHeight}
          width={canvasWidth * 3}
          height={canvasHeight * 3}
          fill={look.brand}
        />
        {look.glow > 0 && iris < 1 ? (
          <circle
            cx={centerX}
            cy={cy}
            r={canvasHeight * 0.42}
            fill={`url(#${id}-glow)`}
            opacity={look.glow * glowIn * (1 - iris)}
          />
        ) : null}
        <g transform={groupTransform}>
          {look.sheen > 0 && iris > 0 ? (
            <rect
              x={lockup.badgeX - lockup.badgeRadius}
              y={cy - lockup.badgeRadius}
              width={lockup.badgeRadius * 2}
              height={lockup.badgeRadius * 2}
              rx={lockup.badgeRadius * look.badgeCorner}
              fill={`url(#${id}-sheen)`}
              opacity={look.sheen * iris}
              transform={spinTransform}
            />
          ) : null}
          {lettersShown
            ? lockup.letters.map((slot, index) => {
                const start = beats.letterStarts[index] ?? beats.writeStart;
                const isAccent = index === accentLetter;
                const flipScale = isAccent ? Math.abs(1 - 2 * accentFlip) : 1;
                return (
                  <GlyphStroke
                    key={`${slot.char}-${index.toString()}`}
                    glyph={slot.glyph}
                    x={letterX(slot, spread)}
                    top={lockup.top}
                    unit={lockup.unit}
                    strokeWidth={lockup.strokeWidth}
                    progress={tween(frame, [start, start + beats.letterFrames], [0, 1], glide)}
                    color={isAccent && accentFlip >= 0.5 ? look.accent : look.onBrand}
                    scale={flipScale}
                  />
                );
              })
            : null}
          <Comet
            d={pathThrough(entryCourse(lockup))}
            head={entryHead}
            tail={COMET_TAIL}
            width={lockup.strokeWidth}
            color={look.onBrand}
          />
          <Comet
            d={pathThrough(swooshCourse(lockup, spread, canvasWidth))}
            head={swooshHead}
            tail={SWOOSH_TAIL}
            width={lockup.strokeWidth}
            color={look.onBrand}
          />
          <Comet
            d={orbitPath(lockup)}
            head={orbitHead}
            opacity={orbitFade}
            tail={0.35}
            width={lockup.strokeWidth * 0.8}
            color={look.onBrand}
          />
          <g transform={spinTransform}>
            <GlyphStroke
              glyph={lockup.monogram}
              x={lockup.monogramX}
              top={lockup.monogramTop}
              unit={lockup.monogramUnit}
              strokeWidth={lockup.monogramStroke}
              progress={monogramDrawn}
              color={frame < beats.irisStart ? look.onBrand : look.monogramInk}
            />
            {look.monogramDot && accentDot > 0 ? (
              <circle
                cx={dotX}
                cy={dotY}
                r={dotR * accentDot}
                fill={look.accent}
                stroke={look.brand}
                strokeWidth={dotR * 0.6 * accentDot}
                paintOrder="stroke"
              />
            ) : null}
          </g>
        </g>
      </g>
    </g>
  );
};
