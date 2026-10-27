import { z } from 'zod';
import { chalkboardLook, chalkSchema, connectorChalk } from '../../connector-common';
import { zColor, zSlider } from '../../control';
import { defaultMotion, lookSchema, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';

const CHUNKY_STACK_FONTS = ['chunky', 'bold', 'thin', 'script', 'serif'] as const;

export const chunkyLineSchema = z.object({
  text: z.string(),
  size: zSlider({ min: 40, max: 320 }),
  offset: zSlider({ min: -600, max: 600 }),
});

export type ChunkyLine = z.infer<typeof chunkyLineSchema>;

export const chunkyTypeStackSchema = z.object({
  lines: z.array(chunkyLineSchema).min(1).max(8),
  color: zColor(),
  font: z.enum(CHUNKY_STACK_FONTS),
  lineGap: zSlider({ min: -0.3, max: 0.6, step: 0.01 }),
  jitter: zSlider({ min: 0, max: 1, step: 0.01 }),
  seed: zSlider({ min: 1, max: 999 }),
  delay: zSlider({ min: 0, max: 120 }),
  letterFrames: zSlider({ min: 0.25, max: 6, step: 0.05 }),
  popFrames: zSlider({ min: 2, max: 30 }),
  popBounce: zSlider({ min: 0, max: 4, step: 0.1 }),
  bar: z.object({
    show: z.boolean(),
    line: zSlider({ min: 0, max: 7 }),
    width: zSlider({ min: 5, max: 100 }),
    height: zSlider({ min: 4, max: 60 }),
    color: zColor(),
    frames: zSlider({ min: 4, max: 60 }),
  }),
  exitAt: zSlider({ min: 20, max: 600 }),
  exitFrames: zSlider({ min: 4, max: 40 }),
  lineStagger: zSlider({ min: 0, max: 20 }),
  exitLetterStagger: zSlider({ min: 0, max: 3, step: 0.05 }),
  burst: z.object({
    text: z.string(),
    size: zSlider({ min: 200, max: 2000 }),
    color: zColor(),
    frames: zSlider({ min: 6, max: 60 }),
  }),
  endTag: z.string(),
  endTagSize: zSlider({ min: 16, max: 160 }),
  endTagColor: zColor(),
  endTagSpacing: zSlider({ min: 0, max: 0.5, step: 0.01 }),
  chalk: chalkSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type ChunkyTypeStackProps = z.infer<typeof chunkyTypeStackSchema>;

export const chunkyTypeStackDefaults: ChunkyTypeStackProps = {
  lines: [
    { text: 'SMALL WORDS', size: 178, offset: -46 },
    { text: 'BIG MOVES', size: 250, offset: 28 },
    { text: 'SKETCHED LIVE', size: 146, offset: -20 },
    { text: 'MADE TO STACK UP.', size: 112, offset: 80 },
  ],
  color: '#f2dc8c',
  font: 'chunky',
  lineGap: 0.04,
  jitter: 0.6,
  seed: 12,
  delay: 6,
  letterFrames: 1.6,
  popFrames: 8,
  popBounce: 1.9,
  bar: { show: true, line: 3, width: 36, height: 22, color: '#f08a76', frames: 16 },
  exitAt: 132,
  exitFrames: 10,
  lineStagger: 4,
  exitLetterStagger: 0.5,
  burst: { text: '&', size: 1100, color: '#f4a9c0', frames: 20 },
  endTag: 'CLIPLIBRARY',
  endTagSize: 54,
  endTagColor: '#eef2e8',
  endTagSpacing: 0.06,
  chalk: connectorChalk({ roughness: 2.5, dryness: 0.04, shadow: 0.4 }),
  look: chalkboardLook(),
  motion: defaultMotion({ exit: 'none', exitFrames: 0, float: 0 }),
};

export const chunkyTypeStackMeta: ClipMeta = {
  ...promoMeta,
  id: 'chunky-type-stack',
  title: 'Chunky Type Stack',
  description:
    'Chunky hand-cut letters pop in one by one into a staggered headline stack, then a giant glyph punches through to an end tag.',
  category: 'kinetic-type',
  tags: ['kinetic', 'headline', 'stack', 'typing', 'zoom'],
  theme: 'dark',
  source: 'src/compositions/scenes/kinetic-type/chunky-type-stack.tsx',
  durationInFrames: 232,
  posterFrame: 130,
};
