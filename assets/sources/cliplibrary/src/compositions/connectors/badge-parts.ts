import type { Point } from '../promo/ink-geometry';
import { type Knot, spline } from './brush';
import type { ConnectorFont } from './fonts';

export type Tone = 'line' | 'fill' | 'text' | 'shade' | 'deep';

export type Window = readonly [number, number];

export interface StrokePart {
  kind: 'stroke';
  points: Point[];
  width: number;
  tone: Tone;
  at: Window;
  out?: Window;
  rotate?: number;
}

export interface ShapePart {
  kind: 'shape';
  d: string;
  tone: Tone;
  at: Window;
  reveal: 'pop' | 'unroll' | 'grow';
  out?: Window;
  origin: Knot;
  span?: readonly [number, number];
  spin?: number;
  rotate?: number;
}

export interface WordPart {
  kind: 'word';
  font: ConnectorFont;
  size: number;
  x: number;
  y: number;
  tone: Tone;
  at: Window;
  reveal: 'wipe' | 'pop';
  out?: Window;
  rotate?: number;
  tracking?: number;
  angle?: number;
  outline?: number;
  anchor?: 'start' | 'center' | 'end';
  unwrite?: boolean;
}

export type BadgePart = StrokePart | ShapePart | WordPart;

export const stroke = (
  knots: readonly Knot[],
  at: Window,
  tone: Tone = 'line',
  width = 1,
  rotate?: number,
): StrokePart => ({
  kind: 'stroke',
  points: spline(knots),
  width,
  tone,
  at,
  ...(rotate === undefined ? {} : { rotate }),
});

export const word = (
  font: ConnectorFont,
  size: number,
  at: Window,
  options: Partial<Omit<WordPart, 'kind' | 'font' | 'size' | 'at'>> = {},
): WordPart => ({
  kind: 'word',
  font,
  size,
  at,
  x: 0,
  y: 0,
  tone: 'text',
  reveal: 'pop',
  ...options,
});

export const shape = (
  d: string,
  at: Window,
  options: Partial<Omit<ShapePart, 'kind' | 'd' | 'at'>> = {},
): ShapePart => ({
  kind: 'shape',
  d,
  at,
  tone: 'fill',
  reveal: 'pop',
  origin: [0, 0],
  ...options,
});

const fmt = (value: number): string => value.toFixed(1);

export const polygon = (points: readonly Knot[]): string =>
  `M ${points.map(([x, y]) => `${fmt(x)} ${fmt(y)}`).join(' L ')} Z`;

export const sparkle = (cx: number, cy: number, radius: number): string => {
  const waist = radius * 0.22;
  return `M ${fmt(cx)} ${fmt(cy - radius)} Q ${fmt(cx + waist)} ${fmt(cy - waist)} ${fmt(cx + radius)} ${fmt(cy)} Q ${fmt(cx + waist)} ${fmt(cy + waist)} ${fmt(cx)} ${fmt(cy + radius)} Q ${fmt(cx - waist)} ${fmt(cy + waist)} ${fmt(cx - radius)} ${fmt(cy)} Q ${fmt(cx - waist)} ${fmt(cy - waist)} ${fmt(cx)} ${fmt(cy - radius)} Z`;
};

export const dot = (cx: number, cy: number, radius: number): string =>
  `M ${fmt(cx - radius)} ${fmt(cy)} a ${fmt(radius)} ${fmt(radius)} 0 1 0 ${fmt(radius * 2)} 0 a ${fmt(radius)} ${fmt(radius)} 0 1 0 ${fmt(-radius * 2)} 0 Z`;

export const ray = (
  angle: number,
  inner: number,
  outer: number,
  cx = 0,
  cy = 0,
  bend = 0.03,
): Knot[] => {
  const radians = (angle * Math.PI) / 180;
  return [
    [cx + Math.cos(radians) * inner, cy + Math.sin(radians) * inner],
    [
      cx + Math.cos(radians + bend) * ((inner + outer) / 2),
      cy + Math.sin(radians + bend) * ((inner + outer) / 2),
    ],
    [cx + Math.cos(radians) * outer, cy + Math.sin(radians) * outer],
  ];
};
