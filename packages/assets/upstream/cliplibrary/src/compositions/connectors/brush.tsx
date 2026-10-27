import { brushOutline, noise, type Point, slicePath, smoothstep } from '../promo/ink-geometry';

export type Knot = readonly [number, number];

const catmull = (p0: Knot, p1: Knot, p2: Knot, p3: Knot, t: number): Point => {
  const t2 = t * t;
  const t3 = t2 * t;
  const at = (a: number, b: number, c: number, d: number) =>
    0.5 * (2 * b + (-a + c) * t + (2 * a - 5 * b + 4 * c - d) * t2 + (-a + 3 * b - 3 * c + d) * t3);
  return { x: at(p0[0], p1[0], p2[0], p3[0]), y: at(p0[1], p1[1], p2[1], p3[1]) };
};

export const spline = (knots: readonly Knot[], perSpan = 14): Point[] => {
  if (knots.length < 2) {
    return knots.map(([x, y]) => ({ x, y }));
  }
  const points: Point[] = [];
  for (let index = 0; index < knots.length - 1; index++) {
    const p0 = knots[Math.max(0, index - 1)] as Knot;
    const p1 = knots[index] as Knot;
    const p2 = knots[index + 1] as Knot;
    const p3 = knots[Math.min(knots.length - 1, index + 2)] as Knot;
    for (let step = 0; step < perSpan; step++) {
      points.push(catmull(p0, p1, p2, p3, step / perSpan));
    }
  }
  const last = knots.at(-1) as Knot;
  points.push({ x: last[0], y: last[1] });
  return points;
};

export const loop = (knots: readonly Knot[], overlap = 1): Knot[] => [
  ...knots,
  ...knots.slice(0, overlap + 1),
];

export const ellipseKnots = (
  rx: number,
  ry: number,
  count: number,
  seed: number,
  jitter: number,
): Knot[] =>
  Array.from({ length: count }, (_, index) => {
    const angle = -Math.PI / 2 + (index / count) * Math.PI * 2;
    const wobble = 1 + noise(seed, index * 0.8) * jitter;
    return [Math.cos(angle) * rx * wobble, Math.sin(angle) * ry * wobble] as const;
  });

const brushWidth = (width: number, seed: number, t: number): number =>
  width *
  (0.45 + 0.55 * smoothstep(0, 0.08, t)) *
  (0.35 + 0.65 * (1 - smoothstep(0.82, 1, t))) *
  (1 + 0.18 * noise(seed, t * 6));

export const BrushStroke = ({
  points,
  width,
  color,
  head,
  tail,
  seed,
}: {
  points: readonly Point[];
  width: number;
  color: string;
  head: number;
  tail: number;
  seed: number;
}) => {
  if (head - tail <= 0.002 || points.length < 2) {
    return null;
  }
  const part = slicePath(points, tail, head);
  const d = brushOutline(part, (index, count) =>
    brushWidth(width, seed, tail + ((head - tail) * index) / Math.max(1, count - 1)),
  );
  return <path d={d} fill={color} opacity={Math.min(1, (head - tail) / 0.12)} />;
};

export const ChalkFilter = ({
  id,
  roughness,
  dryness,
}: {
  id: string;
  roughness: number;
  dryness: number;
}) => {
  const contrast = dryness * 8;
  return (
    <svg width={0} height={0} style={{ position: 'absolute' }} aria-hidden="true">
      <filter id={id} x="-10%" y="-10%" width="120%" height="120%">
        <feTurbulence
          type="fractalNoise"
          baseFrequency="0.06"
          numOctaves={2}
          seed={4}
          result="warp"
        />
        <feDisplacementMap
          in="SourceGraphic"
          in2="warp"
          scale={roughness}
          xChannelSelector="R"
          yChannelSelector="G"
          result="edge"
        />
        <feTurbulence
          type="fractalNoise"
          baseFrequency="0.7 1.1"
          numOctaves={1}
          seed={9}
          result="grain"
        />
        <feColorMatrix
          in="grain"
          type="matrix"
          values={`0 0 0 0 0  0 0 0 0 0  0 0 0 0 0  ${-contrast} 0 0 0 ${1 + contrast * 0.55}`}
          result="tooth"
        />
        <feComposite in="edge" in2="tooth" operator="in" />
      </filter>
    </svg>
  );
};
