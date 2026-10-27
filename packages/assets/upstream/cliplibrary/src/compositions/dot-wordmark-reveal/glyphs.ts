export interface StrokeGlyph {
  width: number;
  d: string;
}

const BOWL = 'M1 0.5 A0.5 0.5 0 0 0 0 0.5 A0.5 0.5 0 0 0 1 0.5';
const BACK_BOWL = 'M0 0.5 A0.5 0.5 0 0 1 1 0.5 A0.5 0.5 0 0 1 0 0.5';

const GLYPHS: Record<string, StrokeGlyph> = {
  a: { width: 1, d: `${BOWL} M1 0 L1 1` },
  b: { width: 1, d: `M0 -0.6 L0 1 ${BACK_BOWL}` },
  c: { width: 0.85, d: 'M0.85 0.15 A0.5 0.5 0 1 0 0.85 0.85' },
  d: { width: 1, d: `${BOWL} M1 -0.6 L1 1` },
  e: { width: 1, d: 'M0.04 0.5 L1 0.5 A0.5 0.5 0 1 0 0.85 0.85' },
  f: { width: 0.7, d: 'M0.7 -0.55 Q0.25 -0.65 0.25 -0.15 L0.25 1 M0 0 L0.65 0' },
  g: { width: 1, d: `${BOWL} M1 0 L1 1.15 Q1 1.55 0.55 1.55 L0.15 1.5` },
  h: { width: 1, d: 'M0 -0.6 L0 1 M0 0.5 A0.5 0.5 0 0 1 1 0.5 L1 1' },
  i: { width: 0.02, d: 'M0.01 0 L0.01 1 M0.01 -0.4 L0.01 -0.39' },
  j: { width: 0.45, d: 'M0.4 0 L0.4 1.2 Q0.4 1.55 0 1.55 M0.4 -0.4 L0.4 -0.39' },
  k: { width: 0.85, d: 'M0 -0.6 L0 1 M0.85 0 L0.05 0.6 M0.3 0.42 L0.85 1' },
  l: { width: 0.02, d: 'M0.01 -0.6 L0.01 1' },
  m: {
    width: 1.6,
    d: 'M0 1 L0 0.4 A0.4 0.4 0 0 1 0.8 0.4 L0.8 1 M0.8 0.4 A0.4 0.4 0 0 1 1.6 0.4 L1.6 1',
  },
  n: { width: 1, d: 'M0 1 L0 0.5 A0.5 0.5 0 0 1 1 0.5 L1 1' },
  o: { width: 1, d: 'M0.5 0 A0.5 0.5 0 0 0 0.5 1 A0.5 0.5 0 0 0 0.5 0' },
  p: { width: 1, d: `M0 0 L0 1.55 ${BACK_BOWL}` },
  q: { width: 1, d: `${BOWL} M1 0 L1 1.55` },
  r: { width: 0.7, d: 'M0 1 L0 0.5 A0.5 0.5 0 0 1 0.5 0 L0.7 0' },
  s: {
    width: 0.85,
    d: 'M0.82 0.1 C0.6 -0.04 0.05 -0.04 0.06 0.26 C0.07 0.52 0.8 0.46 0.8 0.74 C0.8 1.04 0.25 1.06 0.03 0.9',
  },
  t: { width: 0.7, d: 'M0.2 -0.45 L0.2 0.7 A0.3 0.3 0 0 0 0.5 1 L0.7 1 M0 0 L0.65 0' },
  u: { width: 1, d: 'M0 0 L0 0.5 A0.5 0.5 0 0 0 1 0.5 M1 0 L1 1' },
  v: { width: 0.95, d: 'M0 0 L0.475 1 L0.95 0' },
  w: { width: 1.5, d: 'M0 0 L0.375 1 L0.75 0.25 L1.125 1 L1.5 0' },
  x: { width: 1, d: 'M0 0 Q1 0.5 0 1 M1 0 Q0 0.5 1 1' },
  y: { width: 0.95, d: 'M0 0 L0.475 0.95 M0.95 0 L0.35 1.25 Q0.2 1.55 -0.05 1.55' },
  z: { width: 0.9, d: 'M0 0 L0.9 0 L0 1 L0.9 1' },
};

const SPACE: StrokeGlyph = { width: 0.4, d: '' };

export const glyphFor = (char: string): StrokeGlyph => GLYPHS[char.toLowerCase()] ?? SPACE;

const numbers = (d: string): number[] =>
  (d.match(/-?\d*\.?\d+/g) ?? []).map((value) => Number.parseFloat(value));

export const glyphStart = (glyph: StrokeGlyph): { x: number; y: number } => {
  const [x = 0, y = 0] = numbers(glyph.d);
  return { x, y };
};

export const glyphEnd = (glyph: StrokeGlyph): { x: number; y: number } => {
  const values = numbers(glyph.d);
  return { x: values.at(-2) ?? 0, y: values.at(-1) ?? 0 };
};

export interface PlacedGlyph {
  char: string;
  glyph: StrokeGlyph;
  x: number;
  width: number;
}

export interface GlyphMetrics {
  unit: number;
  strokeWidth: number;
}

export const glyphMetrics = (size: number, stroke: number): GlyphMetrics => ({
  unit: size * (1 - stroke),
  strokeWidth: size * stroke,
});

export const layoutWord = (
  text: string,
  size: number,
  stroke: number,
  gap: number,
): { glyphs: PlacedGlyph[]; width: number } => {
  const { unit, strokeWidth } = glyphMetrics(size, stroke);
  let cursor = 0;
  const glyphs = [...text].map((char) => {
    const glyph = glyphFor(char);
    const width = glyph.width * unit + strokeWidth;
    const placed = { char, glyph, x: cursor + strokeWidth / 2, width };
    cursor += width + gap * size;
    return placed;
  });
  return { glyphs, width: Math.max(0, cursor - gap * size) };
};
