import { AbsoluteFill } from 'remotion';
import type { ComingSoonFlipsProps } from '@/schemas';
import { LogoMark } from './logo-mark';
import { PlasterWall } from './plaster-wall';

const REACH = 3;

const SAMPLE = 0.5;

const FLOOR = Math.exp(-(REACH * REACH) / 2);

type Beam = ComingSoonFlipsProps['beam'];

const falloff = (distance: number): number =>
  Math.max(0, (Math.exp(-(distance * distance) / 2) - FLOOR) / (1 - FLOOR));

const streakMask = (center: number, beam: Beam): string => {
  const count = Math.max(1, Math.round(beam.slats));
  const softness = Math.max(1, beam.softness);
  const offsets = Array.from(
    { length: count },
    (_, index) => (index - (count - 1) / 2) * beam.slatGap,
  );
  const reach = (offsets.at(-1) ?? 0) / softness + REACH;
  const samples = Math.ceil((2 * reach) / SAMPLE);
  const stops = Array.from({ length: samples + 1 }, (_, index) => {
    const step = -reach + (index * 2 * reach) / samples;
    const at = center + step * softness;
    const light = offsets.reduce(
      (sum, offset) => sum + falloff((at - center - offset) / softness),
      0,
    );
    const alpha = Math.min(1, beam.strength * light);
    return `rgba(0, 0, 0, ${alpha.toFixed(4)}) ${at.toFixed(1)}px`;
  });
  return `linear-gradient(${beam.angle}deg, ${stops.join(', ')})`;
};

export const LitWall = ({
  props,
  center,
}: {
  props: ComingSoonFlipsProps;
  center: number | null;
}) => {
  const { beam } = props;
  const mask = center === null ? undefined : streakMask(center, beam);
  return (
    <AbsoluteFill style={mask === undefined ? {} : { maskImage: mask, WebkitMaskImage: mask }}>
      <PlasterWall wall={props.wall} />
      <AbsoluteFill style={{ background: beam.tint, mixBlendMode: 'multiply' }} />
      <LogoMark logo={props.logo} />
    </AbsoluteFill>
  );
};
