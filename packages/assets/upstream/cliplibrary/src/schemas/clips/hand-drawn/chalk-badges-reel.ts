import { z } from 'zod';
import {
  CONNECTOR_SET,
  chalkboardLook,
  chalkSchema,
  connectorBadge,
  connectorBadgeSchema,
  connectorChalk,
} from '../../connector-common';
import { zColor, zSlider } from '../../control';
import { defaultMotion, lookSchema, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';
import { chunkyLineSchema, chunkyTypeStackDefaults } from '../kinetic-type/chunky-type-stack';

export const chalkBadgesReelSchema = z.object({
  title: z.object({
    show: z.boolean(),
    left: z.string(),
    right: z.string(),
    size: zSlider({ min: 40, max: 260 }),
    color: zColor(),
    badge: connectorBadgeSchema,
    frames: zSlider({ min: 60, max: 300 }),
  }),
  grid: z.object({
    show: z.boolean(),
    columns: zSlider({ min: 1, max: 6 }),
    scale: zSlider({ min: 0.2, max: 1.5, step: 0.01 }),
    frames: zSlider({ min: 90, max: 400 }),
  }),
  badges: z.array(connectorBadgeSchema).min(1).max(12),
  soloFrames: zSlider({ min: 40, max: 180 }),
  soloScale: zSlider({ min: 0.4, max: 2.2, step: 0.01 }),
  caption: z.string(),
  captionSize: zSlider({ min: 12, max: 60 }),
  captionColor: zColor(),
  outro: z.object({
    show: z.boolean(),
    lines: z.array(chunkyLineSchema).min(1).max(8),
    color: zColor(),
    barColor: zColor(),
    burstText: z.string(),
    burstColor: zColor(),
    endTag: z.string(),
    endTagColor: zColor(),
    frames: zSlider({ min: 120, max: 400 }),
  }),
  overlap: zSlider({ min: 0, max: 30 }),
  chalk: chalkSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type ChalkBadgesReelProps = z.infer<typeof chalkBadgesReelSchema>;

export const chalkBadgesReelDefaults: ChalkBadgesReelProps = {
  title: {
    show: true,
    left: 'Bridges',
    right: 'Segues',
    size: 140,
    color: '#eef2e8',
    badge: connectorBadge('ampersand', '&'),
    frames: 112,
  },
  grid: { show: true, columns: 4, scale: 0.62, frames: 200 },
  badges: CONNECTOR_SET,
  soloFrames: 68,
  soloScale: 1.2,
  caption: 'SWAP THE WORD  /  TINT THE CHALK',
  captionSize: 24,
  captionColor: '#eef2e8',
  outro: {
    show: true,
    lines: chunkyTypeStackDefaults.lines,
    color: chunkyTypeStackDefaults.color,
    barColor: chunkyTypeStackDefaults.bar.color,
    burstText: chunkyTypeStackDefaults.burst.text,
    burstColor: chunkyTypeStackDefaults.burst.color,
    endTag: chunkyTypeStackDefaults.endTag,
    endTagColor: chunkyTypeStackDefaults.endTagColor,
    frames: 232,
  },
  overlap: 6,
  chalk: connectorChalk(),
  look: chalkboardLook(),
  motion: defaultMotion({ exit: 'none', exitFrames: 0, float: 0 }),
};

export const chalkBadgesReelMeta: ClipMeta = {
  ...promoMeta,
  id: 'chalk-badges-reel',
  title: 'Chalk Badges Reel',
  description:
    'A chalkboard showcase: a script title, a full grid of connector badges, each badge solo, then a chunky type outro.',
  category: 'hand-drawn',
  tags: ['badge', 'chalkboard', 'grid', 'handwriting', 'kinetic'],
  theme: 'dark',
  source: 'src/compositions/scenes/hand-drawn/chalk-badges-reel.tsx',
  durationInFrames: 1336,
  posterFrame: 210,
};
