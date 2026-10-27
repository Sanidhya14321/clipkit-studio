import { Easing } from 'remotion';
import type { DotWordmarkRevealProps } from '@/schemas';
import { smooth, tween } from '../promo/timing';
import type { Beats } from './beats';
import type { FieldClip } from './brand-field';
import { type DotCourse, dotPosition } from './dot-path';
import type { Lockup } from './lockup';
import type { TaglineLayout } from './tagline';

const ROLL_EASE = Easing.bezier(0.55, 0, 0.45, 1);
const GROW_EASE = Easing.bezier(0.3, 0.1, 0.6, 0.55);
const IRIS_EASE = Easing.bezier(0.55, 0, 0.3, 1);

interface Pill {
  x: number;
  y: number;
  length: number;
  angle: number;
  blur: number;
}

export interface FrameState {
  textY: number;
  inverted: boolean;
  pill: Pill | null;
  scale: number;
  offsetX: number;
  rollSpeed: number;
  spin: number;
  badgeCenter: number;
  clip: FieldClip;
  edgeBlur: number;
  showField: boolean;
}

interface Canvas {
  width: number;
  height: number;
}

const farthestCorner = (x: number, y: number, { width, height }: Canvas): number =>
  Math.max(
    Math.hypot(x, y),
    Math.hypot(width - x, y),
    Math.hypot(x, height - y),
    Math.hypot(width - x, height - y),
  );

const isInverted = (beats: Beats, frame: number): boolean =>
  beats.landings.some((landing, index) => {
    const next = beats.landings[index + 1];
    return index % 2 === 0 && next !== undefined && frame > landing && frame <= next;
  });

const pillAt = (course: DotCourse, frame: number): Pill | null => {
  const position = dotPosition(course, frame);
  if (position === null || frame >= course.beats.growStart) {
    return null;
  }
  const before = dotPosition(course, frame - 0.5) ?? position;
  const after = dotPosition(course, frame + 0.5) ?? position;
  const velocityX = after.x - before.x;
  const velocityY = after.y - before.y;
  const speed = Math.hypot(velocityX, velocityY);
  const { size, stretch } = course.dot;
  return {
    x: position.x,
    y: position.y,
    length: Math.min(size * 3, size + stretch * speed * 0.6),
    angle: (Math.atan2(velocityY, velocityX) * 180) / Math.PI,
    blur: Math.min(2.5, speed * 0.05),
  };
};

const courseFor = (
  props: DotWordmarkRevealProps,
  layout: TaglineLayout,
  beats: Beats,
  canvas: Canvas,
): DotCourse => {
  const { type, dot } = props;
  const textY = canvas.height / 2 + type.taglineY;
  return {
    beats,
    landingsX: layout.lefts.map(
      (left, index) => left + Math.min((layout.widths[index] ?? 0) * 0.35, type.taglineSize * 1.5),
    ),
    maxX: canvas.width - dot.size * 1.4,
    landY: textY - type.taglineSize * 0.4 - dot.size * 0.3,
    restY: canvas.height / 2,
    centerX: canvas.width / 2,
    dot,
  };
};

export const frameState = (
  props: DotWordmarkRevealProps,
  layout: TaglineLayout,
  lockup: Lockup,
  beats: Beats,
  frame: number,
  canvas: Canvas,
): FrameState => {
  const { type, dot, look, roll } = props;
  const course = courseFor(props, layout, beats, canvas);
  const scale =
    1 + type.settleScale * tween(frame, [beats.irisStart, beats.settleEnd], [1, 0], smooth);
  const badgeScreenX = canvas.width / 2 + (lockup.badgeX - canvas.width / 2) * scale;
  const rollDistance = roll.away ? canvas.width + type.badgeSize * 0.75 - badgeScreenX : 0;
  const rollAt = (at: number) =>
    rollDistance * tween(at, [beats.rollStart, beats.rollEnd], [0, 1], ROLL_EASE);
  const offsetX = rollAt(frame);
  const badgeCenter = badgeScreenX + offsetX;
  const growCover = farthestCorner(course.centerX, course.restY, canvas) + 4;
  const growAt = (at: number) =>
    dot.size / 2 +
    (growCover - dot.size / 2) * tween(at, [beats.growStart, beats.growEnd], [0, 1], GROW_EASE);
  const irisCover = farthestCorner(badgeScreenX, lockup.centerY, canvas) + 4;
  const irisT = tween(frame, [beats.irisStart, beats.irisEnd], [0, 1], IRIS_EASE);
  const spin = (offsetX / lockup.badgeRadius) * roll.spin * (180 / Math.PI);
  const clip: FieldClip =
    frame < beats.irisStart
      ? {
          x: course.centerX,
          y: course.restY,
          r: growAt(frame),
          corner: 1,
          angle: 0,
        }
      : {
          x: badgeCenter,
          y: lockup.centerY,
          r: irisCover + (lockup.badgeRadius * scale - irisCover) * irisT,
          corner: 1 + (look.badgeCorner - 1) * irisT,
          angle: spin,
        };
  return {
    textY: canvas.height / 2 + type.taglineY,
    inverted: dot.invertOnLanding && isInverted(beats, frame),
    pill: pillAt(course, frame),
    scale,
    offsetX,
    rollSpeed: rollAt(frame + 0.5) - rollAt(frame - 0.5),
    spin,
    badgeCenter,
    clip,
    edgeBlur:
      frame < beats.growEnd ? Math.min(4, (growAt(frame + 0.5) - growAt(frame - 0.5)) * 0.05) : 0,
    showField: frame >= beats.growStart && (!roll.away || frame < beats.rollEnd + 2),
  };
};
