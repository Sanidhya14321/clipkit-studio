import type { ConnectorBadgeVariant } from '@/schemas';
import { type BadgePart, dot, polygon, ray, shape, sparkle, stroke, word } from './badge-parts';
import { ellipseKnots, type Knot, loop } from './brush';

const scriptSwash = (): BadgePart[] => [
  word('script', 150, [0.04, 0.62], { x: -36, y: -22, rotate: -4, reveal: 'wipe' }),
  stroke(
    [
      [-214, 64],
      [-126, 54],
      [-36, 60],
      [56, 50],
      [124, 38],
      [164, 12],
      [150, -8],
      [130, 12],
      [148, 44],
      [196, 58],
      [230, 54],
    ],
    [0.3, 0.95],
  ),
  shape(sparkle(178, -92, 28), [0.6, 0.94], { origin: [178, -92], spin: 90 }),
  shape(sparkle(216, -52, 14), [0.68, 1], { origin: [216, -52], spin: 90 }),
];

const ruledCaps = (): BadgePart[] => [
  stroke(
    [
      [-152, -100],
      [-40, -106],
      [60, -101],
      [154, -108],
    ],
    [0, 0.38],
    'line',
    0.85,
  ),
  word('thin', 196, [0.12, 0.8], { y: 4, tracking: 0.06 }),
  stroke(
    [
      [-166, 94],
      [-50, 89],
      [70, 94],
      [166, 87],
    ],
    [0.28, 0.66],
    'line',
    0.85,
  ),
  stroke(
    [
      [-118, 116],
      [0, 112],
      [122, 117],
    ],
    [0.42, 0.8],
    'line',
    0.6,
  ),
  shape(
    polygon([
      [-232, 0],
      [-214, -13],
      [-196, 0],
      [-214, 13],
    ]),
    [0.6, 0.86],
    {
      origin: [-214, 0],
    },
  ),
  shape(dot(-252, 0, 6), [0.58, 0.9], { origin: [-252, 0] }),
  shape(
    polygon([
      [232, 0],
      [214, -13],
      [196, 0],
      [214, 13],
    ]),
    [0.68, 0.94],
    { origin: [214, 0] },
  ),
  shape(dot(252, 0, 6), [0.68, 1], { origin: [252, 0] }),
];

const tornEdge = (x: number, direction: number): Knot[] =>
  [-68, -44, -22, 0, 24, 46, 68].map((y, index) => [x + direction * (index % 2 === 0 ? 0 : 12), y]);

const tape = (): BadgePart[] => [
  shape(polygon([...tornEdge(-208, -1), ...tornEdge(208, 1).reverse()]), [0, 0.48], {
    reveal: 'unroll',
    out: [0.45, 1],
    span: [-222, 222],
    rotate: -3,
  }),
  word('bold', 120, [0.28, 0.9], { y: -2, rotate: -3, tracking: 0.03, out: [0, 0.45] }),
  stroke(
    [
      [-70, 104],
      [0, 99],
      [70, 104],
    ],
    [0.62, 1],
    'line',
    0.6,
  ),
];

const boxKnots = (offset: number, seed: number): Knot[] => {
  const jitter = (index: number) => ((seed * 31 + index * 17) % 9) - 4;
  return [
    [-150 + offset, -92 + offset + jitter(0)],
    [0 + offset, -97 + offset],
    [154 + offset, -90 + offset + jitter(1)],
    [158 + offset + jitter(2), 0 + offset],
    [150 + offset, 92 + offset + jitter(3)],
    [0 + offset, 97 + offset],
    [-152 + offset, 90 + offset + jitter(4)],
    [-157 + offset + jitter(5), 0 + offset],
    [-146 + offset, -95 + offset],
    [-110 + offset, -97 + offset],
  ];
};

const frameBox = (): BadgePart[] => [
  stroke(boxKnots(0, 1), [0, 0.6], 'line', 0.95, -4),
  stroke(boxKnots(11, 2), [0.18, 0.78], 'line', 0.6, -4),
  word('bold', 156, [0.3, 0.92], { rotate: -4, tracking: 0.05 }),
];

const arcScript = (): BadgePart[] => [
  stroke(
    [
      [-162, -48],
      [-92, -96],
      [8, -114],
      [110, -106],
      [176, -80],
      [198, -58],
    ],
    [0, 0.48],
  ),
  word('script', 142, [0.2, 0.82], { y: 2, rotate: -6, reveal: 'wipe' }),
  stroke(
    [
      [-112, 86],
      [-20, 72],
      [80, 74],
      [150, 62],
    ],
    [0.52, 0.94],
    'line',
    0.8,
  ),
  shape(dot(224, -38, 6), [0.5, 0.82], { origin: [224, -38] }),
  shape(dot(242, -18, 5), [0.58, 0.9], { origin: [242, -18] }),
  shape(dot(256, 4, 4), [0.66, 0.98], { origin: [256, 4] }),
];

const ribbonTail = (side: number): string =>
  polygon([
    [side * 132, -16],
    [side * 252, -16],
    [side * 222, 28],
    [side * 252, 72],
    [side * 132, 72],
  ]);

const ribbon = (): BadgePart[] => [
  shape(ribbonTail(-1), [0.3, 0.62], {
    tone: 'shade',
    origin: [-150, 40],
    spin: -10,
    out: [0.15, 0.55],
  }),
  shape(ribbonTail(1), [0.36, 0.68], {
    tone: 'shade',
    origin: [150, 40],
    spin: 10,
    out: [0.15, 0.55],
  }),
  shape(
    polygon([
      [-170, 48],
      [-132, 72],
      [-132, 46],
    ]),
    [0.3, 0.62],
    { tone: 'deep', origin: [-150, 40], spin: -10, out: [0.15, 0.55] },
  ),
  shape(
    polygon([
      [170, 48],
      [132, 72],
      [132, 46],
    ]),
    [0.36, 0.68],
    { tone: 'deep', origin: [150, 40], spin: 10, out: [0.15, 0.55] },
  ),
  shape('M -172 -54 Q 0 -70 172 -54 L 172 48 Q 0 32 -172 48 Z', [0, 0.46], {
    reveal: 'grow',
    span: [-180, 180],
    out: [0.4, 1],
  }),
  {
    ...stroke(
      [
        [-160, -46],
        [0, -58],
        [160, -46],
      ],
      [0.42, 0.86],
      'line',
      0.42,
    ),
    out: [0, 0.4],
  },
  {
    ...stroke(
      [
        [-160, 38],
        [0, 26],
        [160, 38],
      ],
      [0.5, 0.94],
      'line',
      0.42,
    ),
    out: [0, 0.4],
  },
  word('serif', 88, [0.36, 0.96], { y: -8, tracking: 0.06, out: [0, 0.45] }),
];

const burstPath = (): string =>
  polygon(
    Array.from({ length: 36 }, (_, index) => {
      const angle = (index / 36) * Math.PI * 2;
      const radius = index % 2 === 0 ? 152 + ((index * 7) % 5) * 2 : 126;
      return [Math.cos(angle) * radius, Math.sin(angle) * radius] as const;
    }),
  );

const starburst = (): BadgePart[] => [
  shape(burstPath(), [0, 0.5], { spin: -30, out: [0.38, 1] }),
  {
    ...stroke(loop(ellipseKnots(106, 106, 10, 5, 0.03)), [0.3, 0.8], 'text', 0.38),
    out: [0, 0.42],
  },
  word('chunky', 132, [0.34, 0.92], { y: 8, rotate: -6, tracking: 0.02, out: [0, 0.42] }),
  stroke(
    [
      [-188, -118],
      [-158, -150],
    ],
    [0.66, 0.86],
    'line',
    0.7,
  ),
  stroke(
    [
      [-214, -84],
      [-176, -106],
    ],
    [0.74, 0.96],
    'line',
    0.7,
  ),
  stroke(
    [
      [188, 118],
      [158, 150],
    ],
    [0.7, 0.9],
    'line',
    0.7,
  ),
  stroke(
    [
      [214, 84],
      [176, 106],
    ],
    [0.78, 1],
    'line',
    0.7,
  ),
];

const stamp = (): BadgePart[] => [
  stroke(loop(ellipseKnots(134, 134, 12, 3, 0.03)), [0, 0.52], 'line', 0.85),
  stroke(loop(ellipseKnots(112, 112, 12, 8, 0.03)), [0.14, 0.66], 'line', 0.5),
  ...Array.from({ length: 16 }, (_, index) =>
    stroke(
      ray(index * 22.5 - 90, 150, 180, 0, 0, 0),
      [0.34 + index * 0.025, 0.6 + index * 0.025],
      'fill',
      0.6,
    ),
  ),
  word('thin', 168, [0.3, 0.88], { y: 4, tracking: 0.04 }),
];

const bubble = (): BadgePart[] => [
  stroke(
    [
      [-150, 122],
      [-118, 74],
      [-176, 30],
      [-192, -38],
      [-128, -100],
      [0, -120],
      [132, -104],
      [196, -40],
      [184, 40],
      [110, 92],
      [0, 106],
      [-84, 98],
      [-150, 122],
    ],
    [0, 0.62],
    'line',
    0.9,
  ),
  word('script', 140, [0.24, 0.9], { y: -10, rotate: -5, reveal: 'wipe' }),
];

const arrowSign = (): BadgePart[] => [
  shape(
    polygon([
      [-204, -64],
      [128, -68],
      [206, 0],
      [132, 70],
      [-200, 64],
    ]),
    [0, 0.5],
    {
      reveal: 'unroll',
      out: [0.45, 1],
      span: [-214, 214],
      rotate: -3,
    },
  ),
  stroke(
    [
      [-218, -80],
      [130, -84],
      [226, 0],
      [134, 86],
      [-214, 80],
      [-220, -64],
    ],
    [0.32, 0.86],
    'line',
    0.5,
    -3,
  ),
  word('bold', 112, [0.3, 0.94], { x: -22, y: -2, rotate: -3, tracking: 0.05, out: [0, 0.45] }),
];

const popLines = (): BadgePart[] => [
  word('script', 204, [0, 0.6], { y: -10, rotate: -4, reveal: 'wipe' }),
  ...[150, 180, 210].map((angle, index) =>
    stroke(ray(angle, 140, 210, 0, -6), [0.4 + index * 0.08, 0.7 + index * 0.08], 'line', 0.75),
  ),
  ...[30, 0, -30].map((angle, index) =>
    stroke(ray(angle, 140, 210, 0, -6), [0.44 + index * 0.08, 0.74 + index * 0.08], 'line', 0.75),
  ),
];

const ampersand = (): BadgePart[] => [
  word('serif', 270, [0.26, 0.92], {
    x: 12,
    y: 30,
    reveal: 'wipe',
    angle: 120,
    outline: 4,
    tone: 'line',
  }),
  word('serif', 270, [0, 0.74], { y: 20, reveal: 'wipe', angle: 120 }),
  shape(sparkle(-44, -150, 22), [0.52, 0.84], { origin: [-44, -150], spin: 90 }),
  shape(sparkle(4, -176, 30), [0.6, 0.92], { origin: [4, -176], spin: 90 }),
  shape(sparkle(52, -148, 20), [0.68, 1], { origin: [52, -148], spin: 90 }),
];

export const BADGE_DESIGNS: Record<ConnectorBadgeVariant, () => BadgePart[]> = {
  'script-swash': scriptSwash,
  'ruled-caps': ruledCaps,
  tape,
  'frame-box': frameBox,
  'arc-script': arcScript,
  ribbon,
  starburst,
  stamp,
  bubble,
  'arrow-sign': arrowSign,
  'pop-lines': popLines,
  ampersand,
};
