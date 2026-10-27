import { z } from 'zod';
import { zColor, zSlider } from '../../control';
import {
  auraSchema,
  blob,
  PICKER_FONTS,
  pickerLook,
  pickerMotion,
  steps,
  stepsSchema,
} from '../../picker-common';
import { FONT_WEIGHT, lookSchema, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';
import { listEntranceSchema } from './word-picker';

export const menuStackSchema = z.object({
  label: z.string(),
  items: z.array(z.string()).min(1).max(24),
  loop: z.boolean(),
  stops: zSlider({ min: 1, max: 40 }),
  timing: stepsSchema,
  entrance: listEntranceSchema,
  type: z.object({
    font: z.enum(PICKER_FONTS),
    labelSize: zSlider({ min: 24, max: 200 }),
    labelWeight: zSlider(FONT_WEIGHT),
    size: zSlider({ min: 16, max: 120 }),
    weight: zSlider(FONT_WEIGHT),
  }),
  layout: z.object({
    labelX: zSlider({ min: 0, max: 1200 }),
    pointerX: zSlider({ min: 0, max: 1600 }),
    cardX: zSlider({ min: 200, max: 1700 }),
    centerY: zSlider({ min: 200, max: 880 }),
  }),
  card: z.object({
    width: zSlider({ min: 240, max: 1400 }),
    height: zSlider({ min: 50, max: 300 }),
    radius: zSlider({ min: 0, max: 80 }),
    fill: zColor(),
    veil: zColor(),
    pitch: zSlider({ min: 30, max: 300 }),
    shrink: zSlider({ min: 0, max: 0.3, step: 0.005 }),
    reach: zSlider({ min: 1, max: 6, step: 0.1 }),
    fade: zSlider({ min: 0.2, max: 4, step: 0.05 }),
    shadow: zSlider({ min: 0, max: 0.5, step: 0.01 }),
    shadowBlur: zSlider({ min: 0, max: 120 }),
    shadowY: zSlider({ min: 0, max: 80 }),
    padding: zSlider({ min: 0, max: 120 }),
    gap: zSlider({ min: 0, max: 120 }),
  }),
  icon: z.object({
    show: z.boolean(),
    size: zSlider({ min: 20, max: 200 }),
    radius: zSlider({ min: 0, max: 60 }),
    color: zColor(),
    glyph: zColor(),
    glyphStroke: zSlider({ min: 1, max: 16, step: 0.5 }),
  }),
  pointer: z.object({
    show: z.boolean(),
    width: zSlider({ min: 10, max: 200 }),
    height: zSlider({ min: 10, max: 200 }),
    notch: zSlider({ min: 0, max: 0.9, step: 0.01 }),
    color: zColor(),
  }),
  aura: auraSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type MenuStackProps = z.infer<typeof menuStackSchema>;

export const menuStackDefaults: MenuStackProps = {
  label: 'Checklist',
  items: [
    'Choose a plan',
    'Workspace name',
    'Invite teammates',
    'Connect calendar',
    'Notifications',
    'Theme',
    'Import contacts',
    'Time zone',
    'Profile photo',
    'Shared folders',
    'Final review',
  ],
  loop: false,
  stops: 5,
  timing: steps({ delay: 40, interval: 24, moveFrames: 15, start: 1 }),
  entrance: { delay: 6, stagger: 3, frames: 30, rise: 60 },
  type: { font: 'inter', labelSize: 76, labelWeight: 500, size: 44, weight: 400 },
  layout: { labelX: 96, pointerX: 500, cardX: 958, centerY: 540 },
  card: {
    width: 644,
    height: 148,
    radius: 26,
    fill: '#fdfeff',
    veil: '#e8eef1',
    pitch: 114,
    shrink: 0.095,
    reach: 4.2,
    fade: 2.6,
    shadow: 0.11,
    shadowBlur: 54,
    shadowY: 26,
    padding: 34,
    gap: 44,
  },
  icon: { show: true, size: 80, radius: 40, color: '#3fa79c', glyph: '#ffffff', glyphStroke: 5 },
  pointer: { show: true, width: 80, height: 88, notch: 0.32, color: '#3fa79c' },
  aura: { blobs: [blob('#ffffff', 52, 46, 70, 0.35)], drift: 0.4 },
  look: pickerLook({
    background: '#e8edf0',
    backgroundEdge: '#dde4e8',
    ink: '#17212a',
    muted: '#8d98a2',
    accent: '#3fa79c',
    grain: 0.03,
    spotlight: 0.3,
  }),
  motion: pickerMotion(),
};

export const menuStackMeta: ClipMeta = {
  ...promoMeta,
  id: 'menu-stack',
  title: 'Menu Stack',
  description:
    'A drum of soft white menu cards rolls one row at a time, the picked card swelling to the front beside a pointer.',
  category: 'list-pickers',
  tags: ['list', 'picker', 'cards', 'scroll', 'stack'],
  theme: 'light',
  source: 'src/compositions/scenes/list-pickers/menu-stack.tsx',
  durationInFrames: 165,
  posterFrame: 100,
};
