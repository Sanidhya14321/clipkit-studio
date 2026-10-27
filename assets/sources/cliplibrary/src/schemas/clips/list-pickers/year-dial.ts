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

export const YEAR_DIAL_EASINGS = ['smooth', 'settle', 'linear'] as const;

export const yearDialSchema = z.object({
  label: z.string(),
  labelColor: zColor(),
  firstYear: zSlider({ min: 1900, max: 2100 }),
  yearStep: zSlider({ min: 1, max: 100 }),
  countDown: z.boolean(),
  yearType: z.object({
    font: z.enum(PICKER_FONTS),
    size: zSlider({ min: 20, max: 240 }),
    weight: zSlider(FONT_WEIGHT),
    labelSize: zSlider({ min: 16, max: 200 }),
    labelWeight: zSlider(FONT_WEIGHT),
  }),
  highlight: z.object({
    color: zColor(),
    text: zColor(),
    width: zSlider({ min: 60, max: 800 }),
    height: zSlider({ min: 30, max: 400 }),
    radius: zSlider({ min: 0, max: 200 }),
  }),
  pointer: z.object({
    color: zColor(),
    size: zSlider({ min: 10, max: 200 }),
    gap: zSlider({ min: 0, max: 300 }),
  }),
  dial: z.object({
    radius: zSlider({ min: 100, max: 1400 }),
    slot: zSlider({ min: 8, max: 90, step: 0.5 }),
    fadeStart: zSlider({ min: 0, max: 12, step: 0.1 }),
    range: zSlider({ min: 1, max: 12, step: 0.1 }),
    textFollow: zSlider({ min: 0, max: 1, step: 0.01 }),
  }),
  ticks: z.object({
    radius: zSlider({ min: 100, max: 1600 }),
    length: zSlider({ min: 4, max: 160 }),
    spacing: zSlider({ min: 1, max: 20, step: 0.5 }),
    majorEvery: zSlider({ min: 1, max: 12 }),
    stroke: zSlider({ min: 0.5, max: 10, step: 0.1 }),
    color: zColor(),
    minorColor: zColor(),
  }),
  camera: z.object({
    perspective: zSlider({ min: 400, max: 6000 }),
    tilt: zSlider({ min: 0, max: 80, step: 0.5 }),
    spin: zSlider({ min: -180, max: 180, step: 0.5 }),
    x: zSlider({ min: -1200, max: 1200 }),
    y: zSlider({ min: -800, max: 800 }),
    scale: zSlider({ min: 0.3, max: 3, step: 0.01 }),
    drift: zSlider({ min: 0, max: 1, step: 0.01 }),
  }),
  entryFrames: zSlider({ min: 6, max: 60 }),
  entryTurn: zSlider({ min: 0, max: 6, step: 0.1 }),
  steps: stepsSchema,
  turnEasing: z.enum(YEAR_DIAL_EASINGS),
  aura: auraSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type YearDialProps = z.infer<typeof yearDialSchema>;

export const yearDialDefaults: YearDialProps = {
  label: 'Journey',
  labelColor: '#111a2e',
  firstYear: 2027,
  yearStep: 1,
  countDown: false,
  yearType: { font: 'lexend', size: 92, weight: 600, labelSize: 78, labelWeight: 600 },
  highlight: { color: '#2f66e0', text: '#ffffff', width: 350, height: 176, radius: 0 },
  pointer: { color: '#2f66e0', size: 64, gap: 22 },
  dial: { radius: 580, slot: 23.5, fadeStart: 4.5, range: 7, textFollow: 0 },
  ticks: {
    radius: 760,
    length: 46,
    spacing: 5,
    majorEvery: 3,
    stroke: 3,
    color: '#111a2e',
    minorColor: '#98a2b6',
  },
  camera: { perspective: 1700, tilt: 56, spin: -16, x: 620, y: -200, scale: 1.15, drift: 0.6 },
  entryFrames: 24,
  entryTurn: 1.5,
  steps: steps({ delay: 26, interval: 27, moveFrames: 21 }),
  turnEasing: 'smooth',
  aura: { blobs: [], drift: 0.5 },
  look: pickerLook({
    background: '#f2f4f8',
    backgroundEdge: '#e2e7ef',
    ink: '#111a2e',
    accent: '#2f66e0',
    grain: 0.03,
    spotlight: 0.35,
  }),
  motion: pickerMotion(),
};

export const yearDialMeta: ClipMeta = {
  ...promoMeta,
  id: 'year-dial',
  title: 'Year Dial',
  description:
    'A tilted timeline dial turns year by year under a fixed highlight, seen in deep perspective.',
  category: 'list-pickers',
  tags: ['wheel', 'picker', '3d', 'spin', 'text'],
  theme: 'light',
  source: 'src/compositions/scenes/list-pickers/year-dial.tsx',
  durationInFrames: 150,
  posterFrame: 70,
};
