import { useId } from 'react';
import { AbsoluteFill } from 'remotion';
import type { ComingSoonFlipsProps } from '@/schemas';
import { svgId } from '../printer/geometry';

const ALPHA_FROM_RED = '0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  1 0 0 0 0';

const EDGE_SLOPE = 12;

const SPECK_SLOPE = 7;

const SPECK_FREQUENCY = 0.22;

export const PlasterWall = ({ wall }: { wall: ComingSoonFlipsProps['wall'] }) => {
  const id = svgId(useId());
  const threshold = 0.66 - wall.blotchAmount * 0.3;
  const speckThreshold = 0.62 - wall.speckAmount * 0.24;
  return (
    <AbsoluteFill style={{ background: wall.base }}>
      <svg width="100%" height="100%" style={{ position: 'absolute', inset: 0 }} aria-hidden="true">
        <defs>
          <filter id={`${id}-blotch`} x="0" y="0" width="100%" height="100%">
            <feTurbulence
              type="fractalNoise"
              baseFrequency={wall.blotchScale}
              numOctaves={6}
              seed={wall.seed}
              result="noise"
            />
            <feColorMatrix in="noise" type="matrix" values={ALPHA_FROM_RED} result="field" />
            <feComponentTransfer in="field" result="shape">
              <feFuncA type="linear" slope={EDGE_SLOPE} intercept={-EDGE_SLOPE * threshold} />
            </feComponentTransfer>
            <feTurbulence
              type="fractalNoise"
              baseFrequency={SPECK_FREQUENCY}
              numOctaves={2}
              seed={wall.seed + 5}
              result="grain"
            />
            <feColorMatrix in="grain" type="matrix" values={ALPHA_FROM_RED} result="grainField" />
            <feComponentTransfer in="grainField" result="specks">
              <feFuncA
                type="linear"
                slope={SPECK_SLOPE}
                intercept={-SPECK_SLOPE * speckThreshold}
              />
            </feComponentTransfer>
            <feFlood floodColor={wall.blotch} result="patchTone" />
            <feComposite in="patchTone" in2="shape" operator="in" result="patch" />
            <feFlood floodColor={wall.speck} result="speckTone" />
            <feComposite in="speckTone" in2="specks" operator="in" result="speckLayer" />
            <feComposite in="speckLayer" in2="shape" operator="in" result="speckInPatch" />
            <feMerge>
              <feMergeNode in="patch" />
              <feMergeNode in="speckInPatch" />
            </feMerge>
          </filter>
          <filter id={`${id}-grit`} x="0" y="0" width="100%" height="100%">
            <feTurbulence
              type="fractalNoise"
              baseFrequency={0.6}
              numOctaves={2}
              seed={wall.seed + 3}
            />
            <feColorMatrix type="saturate" values="0" />
          </filter>
        </defs>
        <rect width="100%" height="100%" filter={`url(#${id}-blotch)`} />
        <rect
          width="100%"
          height="100%"
          filter={`url(#${id}-grit)`}
          opacity={wall.grit}
          style={{ mixBlendMode: 'overlay' }}
        />
      </svg>
    </AbsoluteFill>
  );
};
