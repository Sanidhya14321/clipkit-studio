import type { DotWordmarkRevealProps } from '@/schemas';
import type { Beats } from './beats';

export interface Point {
  x: number;
  y: number;
}

export interface DotCourse {
  beats: Beats;
  landingsX: number[];
  maxX: number;
  landY: number;
  restY: number;
  centerX: number;
  dot: DotWordmarkRevealProps['dot'];
}

const lerp = (from: number, to: number, t: number): number => from + (to - from) * t;

const clamp01 = (value: number): number => Math.min(1, Math.max(0, value));

const progress = (frame: number, start: number, end: number): number =>
  end > start ? clamp01((frame - start) / (end - start)) : 1;

const catmull = (p0: Point, p1: Point, p2: Point, p3: Point, t: number): Point => {
  const t2 = t * t;
  const t3 = t2 * t;
  const axis = (a: number, b: number, c: number, d: number) =>
    0.5 * (2 * b + (-a + c) * t + (2 * a - 5 * b + 4 * c - d) * t2 + (-a + 3 * b - 3 * c + d) * t3);
  return { x: axis(p0.x, p1.x, p2.x, p3.x), y: axis(p0.y, p1.y, p2.y, p3.y) };
};

const mirror = (from: Point, about: Point): Point => ({
  x: 2 * about.x - from.x,
  y: 2 * about.y - from.y,
});

const splineAt = (points: Point[], keys: number[], t: number): Point => {
  const last = points.length - 1;
  let segment = 0;
  while (segment < last - 1 && t > (keys[segment + 1] ?? 1)) {
    segment++;
  }
  const from = keys[segment] ?? 0;
  const to = keys[segment + 1] ?? 1;
  const local = to > from ? clamp01((t - from) / (to - from)) : 1;
  const p1 = points[segment] ?? { x: 0, y: 0 };
  const p2 = points[segment + 1] ?? p1;
  const p0 = points[segment - 1] ?? mirror(p2, p1);
  const p3 = points[segment + 2] ?? mirror(p1, p2);
  return catmull(p0, p1, p2, p3, local);
};

const loopPoints = (course: DotCourse): Point[] => {
  const { landingsX, landY, centerX, dot } = course;
  const startX = landingsX.at(-1) ?? centerX;
  const reachX = Math.min(course.maxX, startX + dot.loopReach);
  return [
    { x: startX, y: landY },
    { x: reachX, y: landY - dot.loopHeight * 0.66 },
    { x: lerp(centerX, reachX, 0.53), y: landY - dot.loopHeight },
    { x: centerX, y: landY },
  ];
};

const LOOP_KEYS = [0, 0.27, 0.57, 1];

export const dotPosition = (course: DotCourse, frame: number): Point | null => {
  const { beats, landingsX, landY, restY, centerX, dot } = course;
  const first = beats.landings[0] ?? beats.enter;
  if (frame < beats.enter) {
    return null;
  }
  if (frame < first) {
    const t = progress(frame, beats.enter, first);
    const targetX = landingsX[0] ?? centerX;
    return {
      x: lerp(-dot.size, targetX, t * (1.2 - 0.2 * t)),
      y: landY - dot.approachDrop * (1 - t * t),
    };
  }
  const lastIndex = beats.landings.length - 1;
  for (let index = 0; index < lastIndex; index++) {
    const start = beats.landings[index] ?? 0;
    const end = beats.landings[index + 1] ?? start;
    if (frame < end) {
      const t = progress(frame, start, end);
      return {
        x: lerp(landingsX[index] ?? centerX, landingsX[index + 1] ?? centerX, t),
        y: landY - dot.hopHeight * 4 * t * (1 - t),
      };
    }
  }
  const lastLanding = beats.landings[lastIndex] ?? first;
  if (frame < beats.centerLand) {
    return splineAt(loopPoints(course), LOOP_KEYS, progress(frame, lastLanding, beats.centerLand));
  }
  if (frame < beats.growStart) {
    const t = progress(frame, beats.centerLand, beats.growStart);
    return { x: centerX, y: lerp(landY, restY, t) - dot.riseHeight * 4 * t * (1 - t) };
  }
  return { x: centerX, y: restY };
};

export const pathThrough = (points: Point[]): string => {
  const head = points[0];
  if (head === undefined) {
    return '';
  }
  const parts = [`M${head.x.toFixed(2)} ${head.y.toFixed(2)}`];
  for (let index = 0; index < points.length - 1; index++) {
    const p1 = points[index] ?? head;
    const p2 = points[index + 1] ?? p1;
    const p0 = points[index - 1] ?? mirror(p2, p1);
    const p3 = points[index + 2] ?? mirror(p1, p2);
    const c1 = { x: p1.x + (p2.x - p0.x) / 6, y: p1.y + (p2.y - p0.y) / 6 };
    const c2 = { x: p2.x - (p3.x - p1.x) / 6, y: p2.y - (p3.y - p1.y) / 6 };
    parts.push(
      `C${c1.x.toFixed(2)} ${c1.y.toFixed(2)} ${c2.x.toFixed(2)} ${c2.y.toFixed(2)} ${p2.x.toFixed(2)} ${p2.y.toFixed(2)}`,
    );
  }
  return parts.join(' ');
};
