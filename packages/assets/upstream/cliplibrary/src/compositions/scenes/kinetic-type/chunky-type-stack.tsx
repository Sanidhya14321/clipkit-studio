import { Composition, Easing, useCurrentFrame } from 'remotion';
import {
  type ChunkyTypeStackProps,
  chunkyTypeStackDefaults,
  chunkyTypeStackMeta,
  chunkyTypeStackSchema,
} from '@/schemas';
import { chalkFilterStyle } from '../../connectors/badge';
import { ChalkLayer } from '../../connectors/chalk-board';
import { CONNECTOR_FONTS } from '../../connectors/fonts';
import { signedHash } from '../../promo/ink-geometry';
import { smooth } from '../../promo/timing';
import type { SceneModule } from '../../scene-module';

const FILTER_ID = 'chunky-stack-ink';

const clamp01 = (value: number): number => Math.min(1, Math.max(0, value));

const Letter = ({
  letter,
  pop,
  bounce,
  gone,
  tilt,
  lift,
  grow,
}: {
  letter: string;
  pop: number;
  bounce: number;
  gone: number;
  tilt: number;
  lift: number;
  grow: number;
}) => {
  const overshoot = Easing.out(Easing.back(bounce));
  const scale = (0.25 + 0.75 * overshoot(pop)) * (1 - smooth(gone)) * grow;
  return (
    <span
      style={{
        display: 'inline-block',
        whiteSpace: 'pre',
        opacity: pop <= 0 || gone >= 1 ? 0 : Math.min(1, pop * 1.6),
        transform: `translateY(${(lift - 0.25 * gone).toFixed(3)}em) rotate(${tilt.toFixed(2)}deg) scale(${Math.max(0, scale).toFixed(4)})`,
      }}
    >
      {letter}
    </span>
  );
};

export const ChunkyTypeStack = ({
  lines,
  color,
  font,
  lineGap,
  jitter,
  seed,
  delay,
  letterFrames,
  popFrames,
  popBounce,
  bar,
  exitAt,
  exitFrames,
  lineStagger,
  exitLetterStagger,
  burst,
  endTag,
  endTagSize,
  endTagColor,
  endTagSpacing,
  chalk,
  look,
  motion,
}: ChunkyTypeStackProps) => {
  const frame = useCurrentFrame() * motion.speed;
  const fontFamily = CONNECTOR_FONTS[font];
  let cursor = 0;
  const laidOut = lines.map((line) =>
    [...line.text].map((letter) => {
      const order = cursor;
      if (letter.trim() !== '') {
        cursor += 1;
      }
      return { letter, order };
    }),
  );
  const typedEnd = delay + cursor * letterFrames + popFrames * 0.5;
  const longest = Math.max(...lines.map((line) => line.text.length));
  const exitEnd =
    exitAt + (lines.length - 1) * lineStagger + (longest - 1) * exitLetterStagger + exitFrames;
  const burstStart = exitEnd - 2;
  const burstProgress = clamp01((frame - burstStart) / burst.frames);
  const tagStart = burstStart + burst.frames * 0.95;
  return (
    <ChalkLayer look={look} motion={motion} chalk={chalk} filterId={FILTER_ID}>
      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          flexDirection: 'column',
          alignItems: 'center',
          justifyContent: 'center',
          fontFamily,
          color,
          filter: chalkFilterStyle(FILTER_ID, chalk),
        }}
      >
        {laidOut.map((letters, lineIndex) => {
          const line = lines[lineIndex];
          if (line === undefined) {
            return null;
          }
          const exitStart = exitAt + lineIndex * lineStagger;
          const showBar = bar.show && Math.round(bar.line) === lineIndex;
          const barGrow = smooth(clamp01((frame - typedEnd) / bar.frames));
          const barGone = smooth(clamp01((frame - exitStart) / exitFrames));
          return (
            <div
              key={`${lineIndex.toString()}-${line.text}`}
              style={{
                position: 'relative',
                display: 'flex',
                fontSize: line.size,
                lineHeight: 0.92,
                marginTop: lineIndex === 0 ? 0 : line.size * lineGap,
                transform: `translateX(${line.offset}px)`,
              }}
            >
              {letters.map(({ letter, order }, index) => (
                <Letter
                  key={`${index.toString()}${letter}`}
                  letter={letter}
                  pop={clamp01((frame - delay - order * letterFrames) / popFrames)}
                  bounce={popBounce}
                  gone={clamp01((frame - exitStart - index * exitLetterStagger) / exitFrames)}
                  tilt={signedHash(seed, order) * 6 * jitter}
                  lift={signedHash(seed + 1, order) * 0.05 * jitter}
                  grow={1 + signedHash(seed + 2, order) * 0.07 * jitter}
                />
              ))}
              {showBar && barGrow > 0 && barGone < 1 ? (
                <div
                  style={{
                    position: 'absolute',
                    left: 0,
                    bottom: -bar.height * 1.4,
                    width: `${bar.width}%`,
                    height: bar.height,
                    background: bar.color,
                    transformOrigin: '0% 50%',
                    opacity: Math.min(1, barGrow * 5) * Math.min(1, (1 - barGone) * 5),
                    transform: `scaleX(${(barGrow * (1 - barGone)).toFixed(4)})`,
                  }}
                />
              ) : null}
            </div>
          );
        })}
      </div>
      {burstProgress > 0 && burstProgress < 1 ? (
        <div
          style={{
            position: 'absolute',
            inset: 0,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            fontFamily,
            fontSize: burst.size,
            lineHeight: 1,
            color: burst.color,
            opacity:
              Math.min(1, burstProgress * 5) * (1 - smooth(clamp01((burstProgress - 0.62) / 0.38))),
            transform: `rotate(${(-10 + 14 * burstProgress).toFixed(2)}deg) scale(${(0.06 + 3.4 * burstProgress ** 2.2).toFixed(4)})`,
          }}
        >
          {burst.text}
        </div>
      ) : null}
      <div
        style={{
          position: 'absolute',
          inset: 0,
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'center',
          fontFamily,
          fontSize: endTagSize,
          letterSpacing: `${endTagSpacing}em`,
          color: endTagColor,
          filter: chalkFilterStyle(FILTER_ID, chalk),
        }}
      >
        {[...endTag].map((letter, index) => (
          <Letter
            key={`${index.toString()}${letter}`}
            letter={letter}
            pop={clamp01((frame - tagStart - index * letterFrames * 0.6) / popFrames)}
            bounce={popBounce}
            gone={0}
            tilt={0}
            lift={0}
            grow={1}
          />
        ))}
      </div>
    </ChalkLayer>
  );
};

export const chunkyTypeStackScene: SceneModule = {
  id: 'chunky-type-stack',
  component: ChunkyTypeStack,
  render: (key) => (
    <Composition
      key={key}
      id="chunky-type-stack"
      component={ChunkyTypeStack}
      schema={chunkyTypeStackSchema}
      defaultProps={chunkyTypeStackDefaults}
      durationInFrames={chunkyTypeStackMeta.durationInFrames}
      fps={chunkyTypeStackMeta.fps}
      width={chunkyTypeStackMeta.width}
      height={chunkyTypeStackMeta.height}
    />
  ),
};
