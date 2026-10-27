import { useId } from 'react';
import type { FlipTone } from '@/schemas';
import { svgId } from '../printer/geometry';

export const FlipGrain = ({
  amount,
  live,
  frame,
  tone,
}: {
  amount: number;
  live: boolean;
  frame: number;
  tone: FlipTone;
}) => {
  const id = svgId(useId());
  if (amount <= 0) {
    return null;
  }
  return (
    <svg
      width="100%"
      height="100%"
      style={{
        position: 'absolute',
        inset: 0,
        opacity: amount,
        mixBlendMode: tone === 'dark' ? 'screen' : 'multiply',
      }}
      aria-hidden="true"
    >
      <filter id={`${id}-grain`}>
        <feTurbulence
          type="fractalNoise"
          baseFrequency="0.9"
          numOctaves={2}
          seed={live ? 7 + Math.floor(frame) : 7}
          stitchTiles="stitch"
        />
        <feColorMatrix type="saturate" values="0" />
      </filter>
      <rect width="100%" height="100%" filter={`url(#${id}-grain)`} />
    </svg>
  );
};
