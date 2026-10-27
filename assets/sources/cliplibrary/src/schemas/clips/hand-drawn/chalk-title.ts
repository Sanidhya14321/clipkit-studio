import { z } from 'zod';
import {
  chalkboardLook,
  chalkSchema,
  connectorBadge,
  connectorBadgeSchema,
  connectorChalk,
} from '../../connector-common';
import { zColor, zSlider } from '../../control';
import { defaultMotion, lookSchema, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';

const CHALK_TITLE_FONTS = ['script', 'thin', 'bold', 'chunky', 'serif'] as const;

export const chalkTitleSchema = z.object({
  left: z.string(),
  right: z.string(),
  wordSize: zSlider({ min: 40, max: 260 }),
  wordColor: zColor(),
  wordFont: z.enum(CHALK_TITLE_FONTS),
  wordTilt: zSlider({ min: -20, max: 20, step: 0.5 }),
  gap: zSlider({ min: 40, max: 400 }),
  centerScale: zSlider({ min: 0.3, max: 2, step: 0.01 }),
  center: connectorBadgeSchema,
  delay: zSlider({ min: 0, max: 120 }),
  drawFrames: zSlider({ min: 12, max: 180 }),
  eraseAt: zSlider({ min: 20, max: 600 }),
  eraseFrames: zSlider({ min: 6, max: 90 }),
  chalk: chalkSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type ChalkTitleProps = z.infer<typeof chalkTitleSchema>;

export const chalkTitleDefaults: ChalkTitleProps = {
  left: 'Bridges',
  right: 'Segues',
  wordSize: 140,
  wordColor: '#eef2e8',
  wordFont: 'script',
  wordTilt: -3,
  gap: 150,
  centerScale: 1.06,
  center: connectorBadge('ampersand', '&'),
  delay: 4,
  drawFrames: 66,
  eraseAt: 88,
  eraseFrames: 22,
  chalk: connectorChalk(),
  look: chalkboardLook(),
  motion: defaultMotion({ exit: 'none', exitFrames: 0, float: 0 }),
};

export const chalkTitleMeta: ClipMeta = {
  ...promoMeta,
  id: 'chalk-title',
  title: 'Chalk Title',
  description:
    'Two script words write themselves on either side of a hand-drawn ampersand, then unwrite in reverse.',
  category: 'hand-drawn',
  tags: ['chalkboard', 'handwriting', 'headline', 'draw', 'reveal'],
  theme: 'dark',
  source: 'src/compositions/scenes/hand-drawn/chalk-title.tsx',
  durationInFrames: 120,
  posterFrame: 76,
};
