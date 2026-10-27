import { Easing, interpolate } from 'remotion';

export const smooth = Easing.bezier(0.65, 0, 0.35, 1);
export const settle = Easing.bezier(0.25, 0.8, 0.35, 1);
export const accelerate = Easing.bezier(0.7, 0, 0.84, 0);
export const turn = Easing.bezier(0.4, 0, 0.2, 1);
export const glide = Easing.inOut(Easing.sin);

export const tween = (
  frame: number,
  range: [number, number],
  output: [number, number],
  easing: (t: number) => number = smooth,
): number =>
  interpolate(frame, range, output, {
    easing,
    extrapolateLeft: 'clamp',
    extrapolateRight: 'clamp',
  });

export const pixelTween = (
  frame: number,
  range: [number, number],
  [from, to]: [number, number],
  easing: (t: number) => number = smooth,
): number => {
  const start = Math.round(from);
  const distance = Math.abs(Math.round(to) - start);
  if (distance === 0 || frame <= range[0]) {
    return start;
  }
  const direction = Math.sign(Math.round(to) - start);
  let travelled = 0;
  for (let step = Math.floor(range[0]) + 1; step <= frame && travelled < distance; step++) {
    const ideal = Math.abs(
      tween(step, range, [from, to], easing) - tween(step - 1, range, [from, to], easing),
    );
    travelled = Math.min(distance, travelled + Math.max(1, Math.round(ideal)));
  }
  return start + direction * travelled;
};

export const track = (
  frame: number,
  keys: [number, number][],
  easing: (t: number) => number = smooth,
): number => {
  const first = keys[0];
  const last = keys.at(-1);
  if (first === undefined || last === undefined) {
    return 0;
  }
  if (frame <= first[0]) {
    return first[1];
  }
  for (let index = 0; index < keys.length - 1; index++) {
    const current = keys[index];
    const next = keys[index + 1];
    if (current !== undefined && next !== undefined && frame <= next[0]) {
      return tween(frame, [current[0], next[0]], [current[1], next[1]], easing);
    }
  }
  return last[1];
};

export const TAU = Math.PI * 2;

export const swingAngle = (
  frame: number,
  at: number,
  frames: number,
  angle: number,
  peak = 0.5,
): number => {
  const top = at + frames * peak;
  return frame <= top
    ? angle * tween(frame, [at, top], [0, 1], smooth)
    : angle * tween(frame, [top, at + frames], [1, 0], smooth);
};

export const cameraPush = (frame: number, amount: number, frames: number): number =>
  1 - amount * Math.min(1, Math.max(0, frame / frames));

const hermite = (t: number, startSlope: number, endSlope: number): number => {
  const t2 = t * t;
  const t3 = t2 * t;
  return (t3 - 2 * t2 + t) * startSlope + (3 * t2 - 2 * t3) + (t3 - t2) * endSlope;
};

export const land = (
  frame: number,
  [start, end]: [number, number],
  [from, to]: [number, number],
  minSpeed = 1.5,
  startSlope?: number,
): number => {
  if (frame <= start) {
    return from;
  }
  if (frame >= end) {
    return to;
  }
  const frames = end - start;
  const distance = Math.abs(to - from);
  const endSlope = distance > 0 ? Math.min(1, (minSpeed * frames) / distance) : 1;
  const launchSlope = Math.min(3 - 2 * endSlope, startSlope ?? Number.POSITIVE_INFINITY);
  return from + (to - from) * hermite((frame - start) / frames, launchSlope, endSlope);
};

export const launch = (
  frame: number,
  [start, end]: [number, number],
  [from, to]: [number, number],
  minSpeed = 1.5,
): number => land(start + end - frame, [start, end], [to, from], minSpeed);
