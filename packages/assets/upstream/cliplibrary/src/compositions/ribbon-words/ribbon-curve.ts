const SEGMENTS = 64;
const LENGTH_SAMPLES = 6;
const DERIVATIVE_STEP = 1e-4;

export type Point = { x: number; y: number };

export type CurveFn = (t: number) => Point;

export type Curve = {
  d: string;
  lengths: number[];
  length: number;
};

const cubicPoint = (a: Point, b: Point, c: Point, d: Point, t: number): Point => {
  const u = 1 - t;
  const w0 = u * u * u;
  const w1 = 3 * u * u * t;
  const w2 = 3 * u * t * t;
  const w3 = t * t * t;
  return {
    x: w0 * a.x + w1 * b.x + w2 * c.x + w3 * d.x,
    y: w0 * a.y + w1 * b.y + w2 * c.y + w3 * d.y,
  };
};

const derivative = (fn: CurveFn, t: number): Point => {
  const a = fn(t - DERIVATIVE_STEP);
  const b = fn(t + DERIVATIVE_STEP);
  return {
    x: (b.x - a.x) / (2 * DERIVATIVE_STEP),
    y: (b.y - a.y) / (2 * DERIVATIVE_STEP),
  };
};

export const buildCurve = (fn: CurveFn): Curve => {
  const step = 1 / SEGMENTS;
  const first = fn(0);
  const parts = [`M ${first.x.toFixed(2)} ${first.y.toFixed(2)}`];
  const lengths = [0];
  let length = 0;
  for (let index = 0; index < SEGMENTS; index++) {
    const t0 = index * step;
    const a = fn(t0);
    const d = fn(t0 + step);
    const ta = derivative(fn, t0);
    const td = derivative(fn, t0 + step);
    const b = { x: a.x + (ta.x * step) / 3, y: a.y + (ta.y * step) / 3 };
    const c = { x: d.x - (td.x * step) / 3, y: d.y - (td.y * step) / 3 };
    parts.push(
      `C ${b.x.toFixed(2)} ${b.y.toFixed(2)} ${c.x.toFixed(2)} ${c.y.toFixed(2)} ${d.x.toFixed(2)} ${d.y.toFixed(2)}`,
    );
    let previous = a;
    for (let sample = 1; sample <= LENGTH_SAMPLES; sample++) {
      const point = cubicPoint(a, b, c, d, sample / LENGTH_SAMPLES);
      length += Math.hypot(point.x - previous.x, point.y - previous.y);
      previous = point;
    }
    lengths.push(length);
  }
  return { d: parts.join(' '), lengths, length };
};

export const lengthToT = (curve: Curve, distance: number): number => {
  const { lengths } = curve;
  if (distance <= 0) {
    return 0;
  }
  if (distance >= curve.length) {
    return 1;
  }
  let low = 0;
  let high = lengths.length - 1;
  while (high - low > 1) {
    const middle = (low + high) >> 1;
    if ((lengths[middle] ?? 0) <= distance) {
      low = middle;
    } else {
      high = middle;
    }
  }
  const from = lengths[low] ?? 0;
  const to = lengths[high] ?? from;
  const within = to > from ? (distance - from) / (to - from) : 0;
  return (low + within) / SEGMENTS;
};

export const tToLength = (curve: Curve, t: number): number => {
  const position = Math.min(SEGMENTS, Math.max(0, t * SEGMENTS));
  const index = Math.min(SEGMENTS - 1, Math.floor(position));
  const from = curve.lengths[index] ?? 0;
  const to = curve.lengths[index + 1] ?? from;
  return from + (to - from) * (position - index);
};

export const offsetFn =
  (fn: CurveFn, distance: (t: number, point: Point) => number): CurveFn =>
  (t) => {
    const point = fn(t);
    const tangent = derivative(fn, t);
    const size = Math.hypot(tangent.x, tangent.y) || 1;
    const reach = distance(t, point);
    return {
      x: point.x - (tangent.y / size) * reach,
      y: point.y + (tangent.x / size) * reach,
    };
  };

export const minimumRadius = (fn: CurveFn, samples: number): number => {
  let smallest = Number.POSITIVE_INFINITY;
  let a = fn(0);
  let b = fn(1 / samples);
  for (let index = 2; index <= samples; index++) {
    const c = fn(index / samples);
    const cross = Math.abs((b.x - a.x) * (c.y - a.y) - (b.y - a.y) * (c.x - a.x));
    if (cross > 1e-9) {
      const sides =
        Math.hypot(b.x - a.x, b.y - a.y) *
        Math.hypot(c.x - b.x, c.y - b.y) *
        Math.hypot(c.x - a.x, c.y - a.y);
      smallest = Math.min(smallest, sides / (2 * cross));
    }
    a = b;
    b = c;
  }
  return smallest;
};
