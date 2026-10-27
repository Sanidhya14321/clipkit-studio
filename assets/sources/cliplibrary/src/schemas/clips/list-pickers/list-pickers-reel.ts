import { z } from 'zod';
import { zColor, zSlider } from '../../control';
import { pickerMotion } from '../../picker-common';
import { motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const REEL_CLIP_IDS = [
  'word-picker',
  'pill-fan',
  'menu-stack',
  'heading-list',
  'year-dial',
  'mind-map',
  'color-pills',
  'skill-wheel',
  'product-cards',
] as const;

export type ReelClipId = (typeof REEL_CLIP_IDS)[number];

const reelClip = z.enum(REEL_CLIP_IDS);

export const REEL_POP_ORDERS = ['center-first', 'row-by-row'] as const;

export const reelBeatSchema = z.object({
  clip: reelClip,
  frames: zSlider({ min: 45, max: 360 }),
});

export type ReelBeat = z.infer<typeof reelBeatSchema>;

export const listPickersReelSchema = z.object({
  intro: z.object({
    show: z.boolean(),
    frames: zSlider({ min: 60, max: 360 }),
    background: zColor(),
    gap: zSlider({ min: 0, max: 60 }),
    radius: zSlider({ min: 0, max: 40 }),
    popFrames: zSlider({ min: 4, max: 40 }),
    popOrder: z.enum(REEL_POP_ORDERS),
    popScale: zSlider({ min: 0.5, max: 1, step: 0.01 }),
    stagger: zSlider({ min: 0, max: 20 }),
    zoomFrames: zSlider({ min: 12, max: 90 }),
    lead: zSlider({ min: 0, max: 120 }),
  }),
  grid: z.array(reelClip).length(9),
  beats: z.array(reelBeatSchema).min(1).max(12),
  crossfade: zSlider({ min: 0, max: 30 }),
  motion: motionSchema,
});

export type ListPickersReelProps = z.infer<typeof listPickersReelSchema>;

export const listPickersReelDefaults: ListPickersReelProps = {
  intro: {
    show: true,
    frames: 246,
    background: '#14111f',
    gap: 8,
    radius: 12,
    popFrames: 16,
    popOrder: 'center-first',
    popScale: 0.86,
    stagger: 5,
    zoomFrames: 54,
    lead: 48,
  },
  grid: [
    'product-cards',
    'year-dial',
    'color-pills',
    'pill-fan',
    'mind-map',
    'skill-wheel',
    'menu-stack',
    'word-picker',
    'heading-list',
  ],
  beats: [
    { clip: 'word-picker', frames: 150 },
    { clip: 'pill-fan', frames: 162 },
    { clip: 'menu-stack', frames: 132 },
    { clip: 'heading-list', frames: 132 },
    { clip: 'year-dial', frames: 132 },
    { clip: 'mind-map', frames: 174 },
    { clip: 'color-pills', frames: 192 },
    { clip: 'skill-wheel', frames: 132 },
    { clip: 'product-cards', frames: 192 },
  ],
  crossfade: 18,
  motion: pickerMotion({ exitFrames: 18 }),
};

export const listPickersReelMeta: ClipMeta = {
  ...promoMeta,
  id: 'list-pickers-reel',
  title: 'List Pickers Reel',
  description:
    'A grid of every list picker pops in, zooms into one tile, then each picker plays full screen in turn.',
  category: 'list-pickers',
  tags: ['list', 'picker', 'grid', 'zoom', 'crossfade'],
  theme: 'light',
  source: 'src/compositions/scenes/list-pickers/list-pickers-reel.tsx',
  durationInFrames: 1500,
  posterFrame: 150,
};
