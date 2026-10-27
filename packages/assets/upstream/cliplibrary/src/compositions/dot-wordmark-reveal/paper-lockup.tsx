import type { DotWordmarkRevealProps } from '@/schemas';
import { settle } from '../promo/timing';
import type { Lockup } from './lockup';
import { GlyphStroke } from './strokes';

const clamp01 = (value: number): number => Math.min(1, Math.max(0, value));

const STREAKS = [
  { y: -0.3, length: 1.3 },
  { y: 0.04, length: 1.7 },
  { y: 0.3, length: 1.1 },
];

export const PaperLockup = ({
  lockup,
  look,
  accentLetter,
  scale,
  canvasWidth,
  badgeCenter,
}: {
  lockup: Lockup;
  look: DotWordmarkRevealProps['look'];
  accentLetter: number;
  scale: number;
  canvasWidth: number;
  badgeCenter: number | null;
}) => {
  const centerX = canvasWidth / 2;
  const cy = lockup.centerY;
  return (
    <g transform={`translate(${centerX} ${cy}) scale(${scale}) translate(${-centerX} ${-cy})`}>
      {lockup.letters.map((slot, index) => {
        const middle = slot.finalX + (slot.glyph.width * lockup.unit) / 2;
        const opacity =
          badgeCenter === null
            ? 1
            : clamp01((middle - badgeCenter) / (lockup.badgeRadius * 0.5) + 0.2);
        return (
          <GlyphStroke
            key={`${slot.char}-${index.toString()}`}
            glyph={slot.glyph}
            x={slot.finalX}
            top={lockup.top}
            unit={lockup.unit}
            strokeWidth={lockup.strokeWidth}
            progress={1}
            color={index === accentLetter ? look.accent : look.mark}
            opacity={opacity}
          />
        );
      })}
    </g>
  );
};

export const Streaks = ({
  lockup,
  badgeCenter,
  travelled,
  strength,
  color,
}: {
  lockup: Lockup;
  badgeCenter: number;
  travelled: number;
  strength: number;
  color: string;
}) => {
  if (strength <= 0.01) {
    return null;
  }
  const r = lockup.badgeRadius;
  return (
    <g opacity={clamp01(strength)}>
      {STREAKS.map((streak) => {
        const length = Math.min(travelled, r * 2 * streak.length);
        const end = badgeCenter - r * 0.5;
        return (
          <line
            key={streak.y}
            x1={end - length}
            x2={end}
            y1={lockup.centerY + streak.y * r * 2}
            y2={lockup.centerY + streak.y * r * 2}
            stroke={color}
            strokeWidth={3}
            strokeLinecap="round"
          />
        );
      })}
    </g>
  );
};

export const TilePulse = ({
  lockup,
  center,
  scale,
  progress,
  corner,
  strength,
  color,
}: {
  lockup: Lockup;
  center: number;
  scale: number;
  progress: number;
  corner: number;
  strength: number;
  color: string;
}) => {
  if (progress <= 0 || progress >= 1 || strength <= 0) {
    return null;
  }
  const half = lockup.badgeRadius * scale * (1.04 + 0.32 * settle(progress));
  return (
    <rect
      x={center - half}
      y={lockup.centerY - half}
      width={half * 2}
      height={half * 2}
      rx={half * corner}
      fill="none"
      stroke={color}
      strokeWidth={4.5 - 2.5 * progress}
      opacity={strength * (1 - progress) ** 1.4}
    />
  );
};
