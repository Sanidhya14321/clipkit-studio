import { Composition, useCurrentFrame, useVideoConfig } from 'remotion';
import {
  type LetterFlipProps,
  letterFlipDefaults,
  letterFlipMeta,
  letterFlipSchema,
} from '@/schemas';
import { glowShadow } from '../../promo/glow';
import { displayType, KineticLayer } from '../../promo/kinetic';
import { land, settle, tween } from '../../promo/timing';
import type { SceneModule } from '../../scene-module';

export const LetterFlip = ({
  text,
  size,
  weight,
  tracking,
  startAngle,
  slide,
  delay,
  stagger,
  flipFrames,
  depth,
  highlight,
  glowOut,
  placement,
  look,
  motion,
}: LetterFlipProps) => {
  const clipFrame = useCurrentFrame();
  const frame = clipFrame * motion.speed;
  const letters = Array.from(text);
  const reach = size * (0.3 * startAngle * (Math.PI / 180) + slide);
  const { durationInFrames } = useVideoConfig();
  const burnAt = durationInFrames - motion.exitFrames - Math.round(letters.length * 2.5) - 10;
  const burn = glowOut * tween(clipFrame, [burnAt, burnAt + 12], [0, 1], (t) => t);
  return (
    <KineticLayer look={look} motion={motion} placement={placement}>
      <div
        style={{
          ...displayType(size, weight, tracking),
          color: look.ink,
          display: 'flex',
          textShadow: glowShadow(look.ink, burn * 1.4, size),
          filter: burn > 0 ? `blur(${(burn * 5).toFixed(2)}px)` : undefined,
        }}
      >
        {letters.map((letter, index) => {
          const start = delay + index * stagger;
          const turn = land(frame, [start, start + flipFrames], [1, 0], 1 / Math.max(1, reach));
          return (
            <span
              key={`${index}:${letter}`}
              style={{
                display: 'inline-block',
                color: index + 1 === highlight ? look.accent : undefined,
                opacity:
                  tween(frame, [delay, delay + 3], [0, 1], settle) *
                  (1 -
                    glowOut *
                      tween(
                        clipFrame,
                        [burnAt + 12 + index * 2.5, burnAt + 16 + index * 2.5],
                        [0, 1],
                        (t) => t,
                      )),
                transform: `perspective(${size * depth}px) translateX(${(-slide * size * turn).toFixed(2)}px) rotateY(${(-startAngle * turn).toFixed(3)}deg)`,
                transformOrigin: '50% 50%',
                willChange: 'transform',
              }}
            >
              {letter}
            </span>
          );
        })}
      </div>
    </KineticLayer>
  );
};

export const letterFlipScene: SceneModule = {
  id: 'letter-flip',
  component: LetterFlip,
  render: (key) => (
    <Composition
      key={key}
      id="letter-flip"
      component={LetterFlip}
      schema={letterFlipSchema}
      defaultProps={letterFlipDefaults}
      durationInFrames={letterFlipMeta.durationInFrames}
      fps={letterFlipMeta.fps}
      width={letterFlipMeta.width}
      height={letterFlipMeta.height}
    />
  ),
};
