import { z } from 'zod';
import { zColor, zSlider } from '../../control';
import {
  auraSchema,
  PICKER_FONTS,
  pickerLook,
  pickerMotion,
  steps,
  stepsSchema,
} from '../../picker-common';
import { FONT_WEIGHT, lookSchema, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';
import { listEntranceSchema } from './word-picker';

export const headingListSchema = z.object({
  heading: z.string(),
  headingColor: zColor(),
  items: z.array(z.string()).min(1).max(24),
  loop: z.boolean(),
  stops: zSlider({ min: 1, max: 40 }),
  timing: stepsSchema,
  entrance: listEntranceSchema,
  type: z.object({
    font: z.enum(PICKER_FONTS),
    headingSize: zSlider({ min: 24, max: 200 }),
    headingWeight: zSlider(FONT_WEIGHT),
    activeSize: zSlider({ min: 24, max: 220 }),
    size: zSlider({ min: 16, max: 200 }),
    weight: zSlider(FONT_WEIGHT),
    activeWeight: zSlider(FONT_WEIGHT),
    pitch: zSlider({ min: 40, max: 260 }),
    reach: zSlider({ min: 1, max: 6, step: 0.1 }),
    fade: zSlider({ min: 0.2, max: 3, step: 0.05 }),
    activeShift: zSlider({ min: 0, max: 300 }),
  }),
  layout: z.object({
    headingX: zSlider({ min: 0, max: 1200 }),
    listX: zSlider({ min: 200, max: 1700 }),
    centerY: zSlider({ min: 200, max: 880 }),
  }),
  arrow: z.object({
    x: zSlider({ min: 200, max: 1600 }),
    length: zSlider({ min: 40, max: 400 }),
    stroke: zSlider({ min: 2, max: 24, step: 0.5 }),
    headWidth: zSlider({ min: 10, max: 120 }),
    headHeight: zSlider({ min: 20, max: 200 }),
    steps: zSlider({ min: 1, max: 8 }),
    retreat: zSlider({ min: 0, max: 200 }),
    color: zColor(),
    drawStart: zSlider({ min: 0, max: 90 }),
    drawFrames: zSlider({ min: 4, max: 60 }),
  }),
  aura: auraSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type HeadingListProps = z.infer<typeof headingListSchema>;

export const headingListDefaults: HeadingListProps = {
  heading: 'Explore',
  headingColor: '#1e3529',
  items: [
    'Overview',
    'Library',
    'Calendar',
    'Collections',
    'Workspace',
    'Billing',
    'Shortcuts',
    'Archive',
    'Inbox',
    'Team Spaces',
    'Downloads',
    'Bookmarks',
  ],
  loop: false,
  stops: 4,
  timing: steps({ delay: 44, interval: 26, moveFrames: 18 }),
  entrance: { delay: 8, stagger: 3, frames: 30, rise: 160 },
  type: {
    font: 'inter',
    headingSize: 82,
    headingWeight: 500,
    activeSize: 100,
    size: 82,
    weight: 500,
    activeWeight: 500,
    pitch: 148,
    reach: 4.2,
    fade: 1.6,
    activeShift: 96,
  },
  layout: { headingX: 140, listX: 840, centerY: 540 },
  arrow: {
    x: 700,
    length: 176,
    stroke: 9,
    headWidth: 36,
    headHeight: 78,
    steps: 3,
    retreat: 70,
    color: '#c2502b',
    drawStart: 14,
    drawFrames: 22,
  },
  aura: { blobs: [], drift: 0 },
  look: pickerLook({
    background: '#ece6da',
    backgroundEdge: '#e3dccd',
    ink: '#1e3529',
    muted: '#c3baa8',
    accent: '#c2502b',
    grain: 0.03,
    spotlight: 0.2,
  }),
  motion: pickerMotion(),
};

export const headingListMeta: ClipMeta = {
  ...promoMeta,
  id: 'heading-list',
  title: 'Heading List',
  description:
    'A pixel arrow points from a heading into a scrolling menu, nudging each new item forward as it lands.',
  category: 'list-pickers',
  tags: ['list', 'picker', 'scroll', 'text', 'minimal'],
  theme: 'light',
  source: 'src/compositions/scenes/list-pickers/heading-list.tsx',
  durationInFrames: 165,
  posterFrame: 100,
};
