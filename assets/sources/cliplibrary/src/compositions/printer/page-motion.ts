import { Easing, random } from 'remotion';
import type { PrinterPagesProps } from '@/schemas';
import { tween } from '../promo/timing';

export const PAGE_RATIO = Math.SQRT2;
export const REST_GAP = 26;
export const SLOT_DEPTH = 18;

const eject = Easing.bezier(0.5, 0, 0.2, 1);

const MAX_SHIFT = 34;
const MAX_LIFT = 12;
const MAX_TURN = 2.4;

export type PagePose = {
  visible: boolean;
  x: number;
  y: number;
  turn: number;
  landed: number;
  speed: number;
};

const progressAt = (frame: number, start: number, frames: number): number =>
  tween(frame, [start, start + frames], [0, 1], eject);

const scatterFor = (index: number) => ({
  x: (random(`printer-page-x-${index}`) - 0.5) * 2 * MAX_SHIFT,
  y: (random(`printer-page-y-${index}`) - 0.5) * 2 * MAX_LIFT,
  turn: (random(`printer-page-turn-${index}`) - 0.5) * 2 * MAX_TURN,
});

const pageStart = (timing: PrinterPagesProps['timing'], index: number): number =>
  timing.delay + index * timing.interval;

export const pagePose = (
  frame: number,
  index: number,
  timing: PrinterPagesProps['timing'],
  pageHeight: number,
  scatter: number,
): PagePose => {
  const start = pageStart(timing, index);
  const travel = pageHeight + REST_GAP + SLOT_DEPTH;
  const progress = progressAt(frame, start, timing.ejectFrames);
  const previous = progressAt(frame - 1, start, timing.ejectFrames);
  const landed = tween(progress, [0.62, 1], [0, 1]);
  const offset = scatterFor(index);
  return {
    visible: frame > start,
    x: offset.x * scatter * landed,
    y: -pageHeight - SLOT_DEPTH + travel * progress + offset.y * scatter * landed,
    turn: offset.turn * scatter * landed,
    landed,
    speed: (progress - previous) * travel,
  };
};
