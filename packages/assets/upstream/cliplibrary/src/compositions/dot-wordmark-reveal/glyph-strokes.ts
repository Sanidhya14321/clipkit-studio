import type { StrokeGlyph } from './glyphs';

export interface GlyphStrokePart {
  d: string;
  from: number;
  to: number;
}

const MIN_SHARE = 0.12;

const arcLength = (radius: number, chord: number, largeArc: boolean): number => {
  const half = Math.asin(Math.min(1, chord / (2 * radius)));
  const angle = largeArc ? 2 * Math.PI - 2 * half : 2 * half;
  return radius * angle;
};

const curveLength = (points: number[], x: number, y: number): number => {
  let polygon = 0;
  let px = x;
  let py = y;
  for (let index = 0; index < points.length; index += 2) {
    const nx = points[index] ?? px;
    const ny = points[index + 1] ?? py;
    polygon += Math.hypot(nx - px, ny - py);
    px = nx;
    py = ny;
  }
  return (polygon + Math.hypot(px - x, py - y)) / 2;
};

const subpathLength = (d: string): number => {
  const commands = d.match(/[MLAQC][^MLAQC]*/g) ?? [];
  let length = 0;
  let x = 0;
  let y = 0;
  for (const command of commands) {
    const type = command[0];
    const values = (command.slice(1).match(/-?\d*\.?\d+/g) ?? []).map(Number);
    const endX = values.at(-2) ?? x;
    const endY = values.at(-1) ?? y;
    if (type === 'L') {
      length += Math.hypot(endX - x, endY - y);
    } else if (type === 'A') {
      length += arcLength(values[0] ?? 1, Math.hypot(endX - x, endY - y), values[3] === 1);
    } else if (type === 'Q' || type === 'C') {
      length += curveLength(values, x, y);
    }
    x = endX;
    y = endY;
  }
  return length;
};

const cache = new Map<string, GlyphStrokePart[]>();

export const glyphStrokeParts = (glyph: StrokeGlyph): GlyphStrokePart[] => {
  const cached = cache.get(glyph.d);
  if (cached !== undefined) {
    return cached;
  }
  const subpaths = (glyph.d.match(/M[^M]*/g) ?? []).map((part) => part.trim());
  const weights = subpaths.map((part) => Math.max(MIN_SHARE, subpathLength(part)));
  const total = weights.reduce((sum, weight) => sum + weight, 0) || 1;
  let cursor = 0;
  const parts = subpaths.map((d, index) => {
    const from = cursor;
    cursor += (weights[index] ?? 0) / total;
    return { d, from, to: cursor };
  });
  cache.set(glyph.d, parts);
  return parts;
};
