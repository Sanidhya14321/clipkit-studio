import type { DotWordmarkRevealProps } from '@/schemas';
import type { Point } from './dot-path';
import {
  glyphEnd,
  glyphFor,
  glyphMetrics,
  glyphStart,
  layoutWord,
  type StrokeGlyph,
} from './glyphs';

type TypeProps = DotWordmarkRevealProps['type'];

export interface LetterSlot {
  char: string;
  glyph: StrokeGlyph;
  tightX: number;
  finalX: number;
  width: number;
}

export interface Lockup {
  letters: LetterSlot[];
  unit: number;
  strokeWidth: number;
  top: number;
  centerY: number;
  wordRight: number;
  badgeX: number;
  badgeRadius: number;
  monogram: StrokeGlyph;
  monogramUnit: number;
  monogramStroke: number;
  monogramX: number;
  monogramTop: number;
}

export const buildLockup = (
  wordmark: string,
  monogramText: string,
  type: TypeProps,
  canvasWidth: number,
  centerY: number,
): Lockup => {
  const text = wordmark.length > 0 ? wordmark : ' ';
  const tight = layoutWord(text, type.wordmarkSize, type.stroke, type.tightGap);
  const final = layoutWord(text, type.wordmarkSize, type.stroke, type.gap);
  const { unit, strokeWidth } = glyphMetrics(type.wordmarkSize, type.stroke);
  const lockupWidth = type.badgeSize + type.badgeGap + final.width;
  const left = (canvasWidth - lockupWidth) / 2;
  const wordLeft = left + type.badgeSize + type.badgeGap;
  const tightLeft = (canvasWidth - tight.width) / 2;
  const letters = final.glyphs.map((placed, index) => ({
    char: placed.char,
    glyph: placed.glyph,
    tightX: tightLeft + (tight.glyphs[index]?.x ?? placed.x),
    finalX: wordLeft + placed.x,
    width: placed.width,
  }));
  const monogram = glyphFor([...monogramText][0] ?? 'm');
  const monogramMetrics = glyphMetrics(type.badgeSize * type.monogramSize, type.stroke * 1.15);
  const badgeX = left + type.badgeSize / 2;
  return {
    letters,
    unit,
    strokeWidth,
    top: centerY - unit / 2,
    centerY,
    wordRight: wordLeft + final.width,
    badgeX,
    badgeRadius: type.badgeSize / 2,
    monogram,
    monogramUnit: monogramMetrics.unit,
    monogramStroke: monogramMetrics.strokeWidth,
    monogramX: badgeX - (monogram.width * monogramMetrics.unit) / 2,
    monogramTop: centerY - monogramMetrics.unit / 2,
  };
};

export const letterX = (slot: LetterSlot, spread: number): number =>
  slot.tightX + (slot.finalX - slot.tightX) * spread;

const monogramStartPoint = (lockup: Lockup): Point => {
  const start = glyphStart(lockup.monogram);
  return {
    x: lockup.monogramX + start.x * lockup.monogramUnit,
    y: lockup.monogramTop + start.y * lockup.monogramUnit,
  };
};

const firstLetterStart = (lockup: Lockup): Point => {
  const first = lockup.letters[0];
  if (first === undefined) {
    return { x: lockup.badgeX, y: lockup.centerY };
  }
  const start = glyphStart(first.glyph);
  return { x: first.tightX + start.x * lockup.unit, y: lockup.top + start.y * lockup.unit };
};

const lastLetterEnd = (lockup: Lockup, spread: number): Point => {
  const last = lockup.letters.at(-1);
  if (last === undefined) {
    return { x: lockup.wordRight, y: lockup.centerY };
  }
  const end = glyphEnd(last.glyph);
  return { x: letterX(last, spread) + end.x * lockup.unit, y: lockup.top + end.y * lockup.unit };
};

export const entryCourse = (lockup: Lockup): Point[] => {
  const target = firstLetterStart(lockup);
  return [
    { x: -60, y: lockup.centerY + 230 },
    { x: target.x - 170, y: lockup.centerY + 170 },
    { x: target.x - 60, y: lockup.centerY + 70 },
    target,
  ];
};

export const swooshCourse = (lockup: Lockup, spread: number, canvasWidth: number): Point[] => {
  const { badgeX, centerY: cy } = lockup;
  const start = lastLetterEnd(lockup, spread);
  return [
    start,
    { x: Math.max(start.x + 26, lockup.wordRight + 20), y: cy - 36 },
    { x: canvasWidth / 2 + 80, y: cy - 196 },
    { x: badgeX + 34, y: cy - 192 },
    { x: badgeX - 120, y: cy - 96 },
    { x: badgeX - 168, y: cy + 48 },
    { x: badgeX - 130, y: cy + 106 },
    { x: badgeX - 72, y: cy + 26 },
    monogramStartPoint(lockup),
  ];
};

export const orbitPath = (lockup: Lockup): string => {
  const { badgeX: x, centerY: y, badgeRadius: r } = lockup;
  return `M${x} ${y + r} A${r} ${r} 0 1 1 ${x} ${y - r} A${r} ${r} 0 1 1 ${x} ${y + r}`;
};
