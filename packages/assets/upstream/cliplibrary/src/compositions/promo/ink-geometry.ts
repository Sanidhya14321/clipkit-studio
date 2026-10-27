export interface Point {
  x: number;
  y: number;
}

export const hash = (seed: number, index: number): number => {
  let value = Math.imul(seed ^ 0x9e3779b9, 0x85ebca6b) ^ Math.imul(index + 1, 0xc2b2ae35);
  value = Math.imul(value ^ (value >>> 15), 0x2c1b3c6d);
  value = Math.imul(value ^ (value >>> 12), 0x297a2d39);
  return ((value ^ (value >>> 15)) >>> 0) / 4294967296;
};

export const signedHash = (seed: number, index: number): number => hash(seed, index) * 2 - 1;

const fade = (t: number): number => t * t * (3 - 2 * t);

export const noise = (seed: number, t: number): number => {
  const cell = Math.floor(t);
  const local = fade(t - cell);
  return signedHash(seed, cell) * (1 - local) + signedHash(seed, cell + 1) * local;
};

export const smoothstep = (edge0: number, edge1: number, value: number): number => {
  const t = Math.min(1, Math.max(0, (value - edge0) / (edge1 - edge0)));
  return t * t * (3 - 2 * t);
};

const distance = (a: Point, b: Point): number => Math.hypot(b.x - a.x, b.y - a.y);

const lerpPoint = (a: Point, b: Point, t: number): Point => ({
  x: a.x + (b.x - a.x) * t,
  y: a.y + (b.y - a.y) * t,
});

const cumulativeLengths = (points: readonly Point[]): number[] => {
  const lengths = [0];
  for (let index = 1; index < points.length; index++) {
    const previous = points[index - 1];
    const current = points[index];
    const last = lengths[index - 1] ?? 0;
    lengths.push(previous && current ? last + distance(previous, current) : last);
  }
  return lengths;
};

export const pathLength = (points: readonly Point[]): number =>
  cumulativeLengths(points).at(-1) ?? 0;

const pointAtLength = (points: readonly Point[], lengths: readonly number[], target: number) => {
  for (let index = 1; index < points.length; index++) {
    const end = lengths[index] ?? 0;
    if (end >= target) {
      const start = lengths[index - 1] ?? 0;
      const a = points[index - 1] as Point;
      const b = points[index] as Point;
      return { point: lerpPoint(a, b, end > start ? (target - start) / (end - start) : 0), index };
    }
  }
  return { point: points.at(-1) ?? { x: 0, y: 0 }, index: points.length };
};

export const resample = (points: readonly Point[], count: number): Point[] => {
  const lengths = cumulativeLengths(points);
  const total = lengths.at(-1) ?? 0;
  return Array.from(
    { length: count },
    (_, index) => pointAtLength(points, lengths, (total * index) / (count - 1)).point,
  );
};

export const slicePath = (points: readonly Point[], from: number, to: number): Point[] => {
  const lengths = cumulativeLengths(points);
  const total = lengths.at(-1) ?? 0;
  const start = pointAtLength(points, lengths, total * Math.max(0, from));
  const end = pointAtLength(points, lengths, total * Math.min(1, to));
  if (end.index < start.index || to <= from) {
    return [];
  }
  return [start.point, ...points.slice(start.index, end.index), end.point];
};

const normalAt = (points: readonly Point[], index: number): Point => {
  const before = points[Math.max(0, index - 1)] as Point;
  const after = points[Math.min(points.length - 1, index + 1)] as Point;
  const dx = after.x - before.x;
  const dy = after.y - before.y;
  const length = Math.hypot(dx, dy) || 1;
  return { x: -dy / length, y: dx / length };
};

export const wobble = (
  points: readonly Point[],
  amount: number,
  seed: number,
  frequency: number,
): Point[] => {
  if (amount === 0) {
    return [...points];
  }
  const lengths = cumulativeLengths(points);
  return points.map((point, index) => {
    const normal = normalAt(points, index);
    const along = (lengths[index] ?? 0) * frequency;
    const offset = noise(seed, along) * amount;
    return { x: point.x + normal.x * offset, y: point.y + normal.y * offset };
  });
};

const format = (value: number): string => value.toFixed(2);

export const brushOutline = (
  points: readonly Point[],
  widthAt: (index: number, count: number) => number,
): string => {
  if (points.length < 2) {
    return '';
  }
  const left: Point[] = [];
  const right: Point[] = [];
  points.forEach((point, index) => {
    const normal = normalAt(points, index);
    const half = Math.max(0.05, widthAt(index, points.length) / 2);
    left.push({ x: point.x + normal.x * half, y: point.y + normal.y * half });
    right.push({ x: point.x - normal.x * half, y: point.y - normal.y * half });
  });
  const endHalf = Math.max(0.05, widthAt(points.length - 1, points.length) / 2);
  const startHalf = Math.max(0.05, widthAt(0, points.length) / 2);
  const forward = left.map((point) => `${format(point.x)} ${format(point.y)}`).join(' L ');
  const backward = right
    .reverse()
    .map((point) => `${format(point.x)} ${format(point.y)}`)
    .join(' L ');
  const tail = right[0] as Point;
  const head = left[0] as Point;
  return `M ${forward} A ${format(endHalf)} ${format(endHalf)} 0 0 0 ${format(tail.x)} ${format(tail.y)} L ${backward} A ${format(startHalf)} ${format(startHalf)} 0 0 0 ${format(head.x)} ${format(head.y)} Z`;
};
