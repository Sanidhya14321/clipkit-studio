import { z } from 'zod';
import { zColor, zSlider } from '../../control';
import {
  defaultMotion,
  FONT_WEIGHT,
  kineticLook,
  lookSchema,
  motionSchema,
  promoMeta,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const TEXT_RING_MARKS = ['scribble', 'word', 'circle', 'none'] as const;

export type TextRingMark = (typeof TEXT_RING_MARKS)[number];

export const textRingsSchema = z.object({
  words: z.object({
    text: z.string(),
    separator: z.string(),
    innerSize: zSlider({ min: 8, max: 80 }),
    outerSize: zSlider({ min: 40, max: 400 }),
    weight: zSlider(FONT_WEIGHT),
    oddWeight: zSlider(FONT_WEIGHT),
    tracking: zSlider({ min: -0.05, max: 0.4, step: 0.005 }),
    gap: zSlider({ min: 0, max: 3, step: 0.05 }),
  }),
  rings: z.object({
    count: zSlider({ min: 1, max: 14 }),
    firstAt: zSlider({ min: 0, max: 140 }),
    lastAt: zSlider({ min: 0, max: 149 }),
    spacing: zSlider({ min: 1.2, max: 2.2, step: 0.01 }),
    startGap: zSlider({ min: 0, max: 300 }),
    fadeFrames: zSlider({ min: 1, max: 40 }),
    oddOpacity: zSlider({ min: 0.1, max: 1, step: 0.01 }),
  }),
  spin: z.object({
    speed: zSlider({ min: 0, max: 12, step: 0.25 }),
    alternate: z.boolean(),
    clockwise: z.boolean(),
    offset: zSlider({ min: 0, max: 180 }),
  }),
  mark: z.object({
    style: z.enum(TEXT_RING_MARKS),
    word: z.string(),
    wordSize: zSlider({ min: 12, max: 200 }),
    wordWeight: zSlider(FONT_WEIGHT),
    size: zSlider({ min: 40, max: 600 }),
    stroke: zSlider({ min: 0.5, max: 16, step: 0.25 }),
    scribbleStroke: zSlider({ min: 0.5, max: 16, step: 0.25 }),
    loops: zSlider({ min: 2, max: 12 }),
    scribbleSize: zSlider({ min: 0.2, max: 1, step: 0.01 }),
    drawAt: zSlider({ min: 0, max: 90 }),
    drawFrames: zSlider({ min: 6, max: 90 }),
    scribbleDelay: zSlider({ min: 0, max: 60 }),
    shrinkTo: zSlider({ min: 0.4, max: 1.2, step: 0.01 }),
    shrinkFrames: zSlider({ min: 10, max: 240 }),
  }),
  inverted: z.boolean(),
  light: z.object({
    page: zColor(),
    ink: zColor(),
  }),
  look: lookSchema,
  motion: motionSchema,
});

export type TextRingsProps = z.infer<typeof textRingsSchema>;

export const textRingsDefaults: TextRingsProps = {
  words: {
    text: 'all night',
    separator: '·',
    innerSize: 22,
    outerSize: 186,
    weight: 500,
    oddWeight: 500,
    tracking: 0.02,
    gap: 0.55,
  },
  rings: {
    count: 8,
    firstAt: 20,
    lastAt: 118,
    spacing: 1.38,
    startGap: 19,
    fadeFrames: 16,
    oddOpacity: 1,
  },
  spin: {
    speed: 4,
    alternate: true,
    clockwise: true,
    offset: 17,
  },
  mark: {
    style: 'scribble',
    word: 'night',
    wordSize: 44,
    wordWeight: 600,
    size: 200,
    stroke: 3,
    scribbleStroke: 3,
    loops: 5,
    scribbleSize: 0.66,
    drawAt: 0,
    drawFrames: 26,
    scribbleDelay: 6,
    shrinkTo: 0.86,
    shrinkFrames: 110,
  },
  inverted: false,
  light: {
    page: '#f1f1f1',
    ink: '#020202',
  },
  look: { ...kineticLook('#f1f1f1'), grain: 0, liveGrain: false },
  motion: defaultMotion({ exit: 'fade', exitFrames: 16 }),
};

export const textRingsMeta: ClipMeta = {
  ...promoMeta,
  id: 'text-rings',
  title: 'Text Rings',
  description:
    'A small circle and looping scribble draw on, then rings of repeated words grow out from it, spinning in alternate directions until the frame fills into a vortex of type.',
  category: 'kinetic-type',
  tags: ['kinetic', 'text', 'repeat', 'spin', 'orbit', 'draw'],
  theme: 'dark',
  source: 'src/compositions/scenes/kinetic-type/text-rings.tsx',
  width: 1080,
  height: 1920,
  durationInFrames: 150,
  posterFrame: 118,
};
