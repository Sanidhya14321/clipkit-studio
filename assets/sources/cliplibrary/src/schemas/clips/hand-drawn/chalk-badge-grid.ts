import { z } from 'zod';
import {
  badgeTimingSchema,
  CONNECTOR_SET,
  chalkboardLook,
  chalkSchema,
  connectorBadgeSchema,
  connectorChalk,
} from '../../connector-common';
import { zSlider } from '../../control';
import { defaultMotion, lookSchema, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const BADGE_GRID_ORDERS = ['scattered', 'in-order', 'reverse'] as const;

export type BadgeGridOrder = (typeof BADGE_GRID_ORDERS)[number];

export const chalkBadgeGridSchema = z.object({
  badges: z.array(connectorBadgeSchema).min(1).max(16),
  columns: zSlider({ min: 1, max: 6 }),
  scale: zSlider({ min: 0.2, max: 1.5, step: 0.01 }),
  insetX: zSlider({ min: 0, max: 400 }),
  insetY: zSlider({ min: 0, max: 300 }),
  delay: zSlider({ min: 0, max: 120 }),
  stagger: zSlider({ min: 0, max: 20, step: 0.5 }),
  order: z.enum(BADGE_GRID_ORDERS),
  eraseStagger: zSlider({ min: 0, max: 20, step: 0.5 }),
  shuffle: zSlider({ min: 0, max: 999 }),
  timing: badgeTimingSchema,
  chalk: chalkSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type ChalkBadgeGridProps = z.infer<typeof chalkBadgeGridSchema>;

export const chalkBadgeGridDefaults: ChalkBadgeGridProps = {
  badges: CONNECTOR_SET,
  columns: 4,
  scale: 0.62,
  insetX: 40,
  insetY: 70,
  delay: 2,
  stagger: 3.5,
  order: 'scattered',
  eraseStagger: 1.5,
  shuffle: 7,
  timing: { drawFrames: 34, eraseAt: 150, eraseFrames: 26 },
  chalk: connectorChalk({ width: 10, shadow: 0.35 }),
  look: chalkboardLook(),
  motion: defaultMotion({ exit: 'none', exitFrames: 0, float: 0 }),
};

export const chalkBadgeGridMeta: ClipMeta = {
  ...promoMeta,
  id: 'chalk-badge-grid',
  title: 'Chalk Badge Grid',
  description:
    'A full grid of hand-drawn connector badges sketches on in a scattered order, holds, then wipes away.',
  category: 'hand-drawn',
  tags: ['badge', 'chalkboard', 'grid', 'doodle', 'draw'],
  theme: 'dark',
  source: 'src/compositions/scenes/hand-drawn/chalk-badge-grid.tsx',
  durationInFrames: 210,
  posterFrame: 100,
};
