import type { DotWordmarkRevealProps } from '@/schemas';

export interface Beats {
  enter: number;
  landings: number[];
  centerLand: number;
  fadeEnd: number;
  growStart: number;
  growEnd: number;
  writeStart: number;
  writeEnd: number;
  letterStarts: number[];
  letterFrames: number;
  spreadStart: number;
  spreadEnd: number;
  swooshStart: number;
  swooshEnd: number;
  accentStart: number;
  monogramStart: number;
  monogramEnd: number;
  orbitStart: number;
  orbitEnd: number;
  irisStart: number;
  irisEnd: number;
  settleEnd: number;
  rollStart: number;
  rollEnd: number;
}

export const buildBeats = (
  timing: DotWordmarkRevealProps['timing'],
  hops: number,
  letters: number,
): Beats => {
  const enter = timing.enterDelay;
  const first = enter + timing.approachFrames;
  const landings = Array.from(
    { length: Math.max(1, hops) },
    (_, index) => first + index * timing.hopFrames + (timing.hopSlowdown * index * (index - 1)) / 2,
  );
  const lastLanding = landings.at(-1) ?? first;
  const centerLand = lastLanding + timing.loopFrames;
  const growStart = centerLand + timing.settleFrames;
  const growEnd = growStart + timing.growFrames;
  const writeStart = growEnd;
  const writeEnd = writeStart + timing.writeFrames;
  const letterFrames = Math.min(timing.letterFrames, timing.writeFrames);
  const stagger = letters > 1 ? (timing.writeFrames - letterFrames) / (letters - 1) : 0;
  const letterStarts = Array.from({ length: letters }, (_, index) => writeStart + index * stagger);
  const swooshStart = writeEnd;
  const swooshEnd = swooshStart + timing.swooshFrames;
  const monogramEnd = swooshEnd + timing.monogramFrames;
  const orbitEnd = monogramEnd + timing.orbitFrames;
  const irisEnd = orbitEnd + timing.irisFrames;
  const rollStart = irisEnd + timing.holdFrames;
  return {
    enter,
    landings,
    centerLand,
    fadeEnd: centerLand + timing.fadeFrames,
    growStart,
    growEnd,
    writeStart,
    writeEnd,
    letterStarts,
    letterFrames,
    spreadStart: writeEnd - Math.min(8, timing.writeFrames / 2),
    spreadEnd: writeEnd - Math.min(8, timing.writeFrames / 2) + timing.spreadFrames,
    swooshStart,
    swooshEnd,
    accentStart: swooshEnd - Math.min(6, timing.swooshFrames),
    monogramStart: swooshEnd,
    monogramEnd,
    orbitStart: monogramEnd - Math.min(2, timing.monogramFrames),
    orbitEnd,
    irisStart: orbitEnd,
    irisEnd,
    settleEnd: irisEnd + Math.min(10, timing.holdFrames),
    rollStart,
    rollEnd: rollStart + timing.rollFrames,
  };
};
