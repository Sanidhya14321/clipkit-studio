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

export const colorPillSchema = z.object({
  label: z.string(),
  color: zColor(),
  offset: zSlider({ min: -200, max: 200 }),
});

export type ColorPill = z.infer<typeof colorPillSchema>;

export const colorPillsSchema = z.object({
  pills: z.array(colorPillSchema).min(1).max(10),
  font: z.enum(PICKER_FONTS),
  size: zSlider({ min: 20, max: 120 }),
  weight: zSlider(FONT_WEIGHT),
  textColor: zColor(),
  dotColor: zColor(),
  layout: z.object({
    x: zSlider({ min: -800, max: 800 }),
    y: zSlider({ min: -400, max: 400 }),
    pitch: zSlider({ min: 60, max: 260 }),
    height: zSlider({ min: 40, max: 220 }),
    paddingX: zSlider({ min: 10, max: 100 }),
    dot: zSlider({ min: 0, max: 40 }),
    glow: zSlider({ min: 0, max: 30, step: 0.5 }),
    glowStrength: zSlider({ min: 0, max: 1, step: 0.01 }),
    shine: zSlider({ min: 0, max: 1, step: 0.01 }),
    roundness: zSlider({ min: 0, max: 1, step: 0.01 }),
  }),
  active: z.object({
    nudge: zSlider({ min: 0, max: 300 }),
    spread: zSlider({ min: 0, max: 1, step: 0.01 }),
    grow: zSlider({ min: 0, max: 0.4, step: 0.01 }),
  }),
  cursor: z.object({
    color: zColor(),
    halo: zColor(),
    size: zSlider({ min: 8, max: 80 }),
    gap: zSlider({ min: 0, max: 120 }),
    haloSize: zSlider({ min: 0, max: 3, step: 0.05 }),
  }),
  entrance: z.object({
    stagger: zSlider({ min: 0, max: 20 }),
    frames: zSlider({ min: 6, max: 60 }),
    rise: zSlider({ min: 0, max: 200 }),
  }),
  timing: stepsSchema,
  aura: auraSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type ColorPillsProps = z.infer<typeof colorPillsSchema>;

const pill = (label: string, color: string, offset: number): ColorPill => ({
  label,
  color,
  offset,
});

export const colorPillsDefaults: ColorPillsProps = {
  pills: [
    pill('Plum', '#8e2fd0', 8),
    pill('Coral', '#ff5a4e', -24),
    pill('Jade', '#0fae7c', 30),
    pill('Indigo', '#3434c8', 6),
    pill('Azure', '#0b8fd6', 40),
    pill('Ruby', '#d81b4a', -16),
  ],
  font: 'quicksand',
  size: 52,
  weight: 600,
  textColor: '#ffffff',
  dotColor: '#ffffff',
  layout: {
    x: -170,
    y: 0,
    pitch: 150,
    height: 104,
    paddingX: 34,
    dot: 16,
    glow: 7,
    glowStrength: 0.8,
    shine: 0.45,
    roundness: 1,
  },
  active: { nudge: 110, spread: 0.3, grow: 0.06 },
  cursor: { color: '#1b2140', halo: '#8b93b5', size: 30, gap: 34, haloSize: 1 },
  entrance: { stagger: 4, frames: 26, rise: 60 },
  timing: steps({ delay: 34, interval: 20, moveFrames: 17 }),
  aura: {
    blobs: [
      blob('#c9d6ff', 4, 6, 58, 0.75),
      blob('#d9cfff', 98, 96, 60, 0.7),
      blob('#f5f7ff', 52, 48, 46, 0.9),
      blob('#cfe6ff', 96, 4, 40, 0.45),
    ],
    drift: 1,
  },
  look: pickerLook({ background: '#f3f5fc', backgroundEdge: '#dfe4f5', grain: 0.035 }),
  motion: pickerMotion(),
};

export const colorPillsMeta: ClipMeta = {
  ...promoMeta,
  id: 'color-pills',
  title: 'Color Pills',
  description:
    'A column of bright color pills where a radio dot slides down one option at a time and nudges the chosen pill aside.',
  category: 'list-pickers',
  tags: ['list', 'picker', 'glow', 'minimal'],
  theme: 'light',
  source: 'src/compositions/scenes/list-pickers/color-pills.tsx',
  durationInFrames: 180,
  posterFrame: 96,
};
