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

export const listEntranceSchema = z.object({
  delay: zSlider({ min: 0, max: 90 }),
  stagger: zSlider({ min: 0, max: 20 }),
  frames: zSlider({ min: 4, max: 60 }),
  rise: zSlider({ min: 0, max: 200 }),
});

export const wordPickerSchema = z.object({
  items: z.array(z.string()).min(1).max(24),
  loop: z.boolean(),
  stops: zSlider({ min: 1, max: 40 }),
  timing: stepsSchema,
  entrance: listEntranceSchema,
  type: z.object({
    font: z.enum(PICKER_FONTS),
    activeSize: zSlider({ min: 40, max: 260 }),
    size: zSlider({ min: 20, max: 200 }),
    activeWeight: zSlider(FONT_WEIGHT),
    weight: zSlider(FONT_WEIGHT),
    tracking: zSlider({ min: -0.08, max: 0.1, step: 0.005 }),
    pitch: zSlider({ min: 60, max: 300 }),
    reach: zSlider({ min: 1, max: 5, step: 0.1 }),
    fade: zSlider({ min: 0.2, max: 2, step: 0.05 }),
  }),
  frame: z.object({
    show: z.boolean(),
    width: zSlider({ min: 200, max: 1700 }),
    height: zSlider({ min: 60, max: 400 }),
    fill: zColor(),
    fillOpacity: zSlider({ min: 0, max: 1, step: 0.01 }),
    line: zColor(),
    lineWidth: zSlider({ min: 1, max: 12, step: 0.5 }),
    overhang: zSlider({ min: 0, max: 80 }),
    dot: zSlider({ min: 0, max: 30, step: 0.5 }),
    drawStart: zSlider({ min: 0, max: 120 }),
    drawFrames: zSlider({ min: 4, max: 60 }),
  }),
  offsetX: zSlider({ min: -800, max: 800 }),
  offsetY: zSlider({ min: -450, max: 450 }),
  aura: auraSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type WordPickerProps = z.infer<typeof wordPickerSchema>;

export const wordPickerDefaults: WordPickerProps = {
  items: [
    'Outline',
    'Pacing',
    'Lighting',
    'Contrast',
    'Texture',
    'Rhythm',
    'Storyboard',
    'Spacing',
    'Gesture',
    'Balance',
  ],
  loop: false,
  stops: 4,
  timing: steps({ delay: 36, interval: 36, moveFrames: 20 }),
  entrance: { delay: 14, stagger: 6, frames: 26, rise: 70 },
  type: {
    font: 'inter',
    activeSize: 142,
    size: 82,
    activeWeight: 600,
    weight: 500,
    tracking: -0.01,
    pitch: 166,
    reach: 2.6,
    fade: 0.8,
  },
  frame: {
    show: true,
    width: 920,
    height: 180,
    fill: '#4a5bd8',
    fillOpacity: 0.1,
    line: '#8f9be8',
    lineWidth: 3.5,
    overhang: 22,
    dot: 11,
    drawStart: 0,
    drawFrames: 22,
  },
  offsetX: 0,
  offsetY: 0,
  aura: {
    blobs: [
      blob('#a9b5f4', 100, -16, 46, 0.7),
      blob('#b4a9ee', 8, 122, 58, 0.8),
      blob('#9fc4f2', 88, 126, 56, 0.72),
    ],
    drift: 0.6,
  },
  look: pickerLook({
    background: '#fbfbfe',
    backgroundEdge: '#eef0fa',
    ink: '#12131c',
    muted: '#737688',
    accent: '#4a5bd8',
  }),
  motion: pickerMotion(),
};

export const wordPickerMeta: ClipMeta = {
  ...promoMeta,
  id: 'word-picker',
  title: 'Word Picker',
  description:
    'A vertical word list steps up one word at a time, each new pick growing into an indigo selection frame.',
  category: 'list-pickers',
  tags: ['list', 'picker', 'scroll', 'text', 'minimal'],
  theme: 'light',
  source: 'src/compositions/scenes/list-pickers/word-picker.tsx',
  durationInFrames: 180,
  posterFrame: 110,
};
