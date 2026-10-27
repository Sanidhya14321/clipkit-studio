import { z } from 'zod';
import { zSlider } from '../../control';
import {
  defaultMotion,
  FONT_WEIGHT,
  lightLook,
  lookSchema,
  motionSchema,
  promoMeta,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const STREAM_DIRECTIONS = ['up', 'down'] as const;

export const wordStreamsSchema = z.object({
  word: z.object({
    text: z.string(),
    size: zSlider({ min: 30, max: 400 }),
    weight: zSlider(FONT_WEIGHT),
    tracking: zSlider({ min: -0.1, max: 0.2, step: 0.005 }),
    spacing: zSlider({ min: -0.3, max: 1, step: 0.01 }),
  }),
  streams: z.object({
    count: zSlider({ min: 1, max: 5 }),
    margin: zSlider({ min: 0, max: 300 }),
    phaseStep: zSlider({ min: 0, max: 360, step: 5 }),
  }),
  wave: z.object({
    amplitude: zSlider({ min: 0, max: 300 }),
    length: zSlider({ min: 400, max: 4000, step: 10 }),
    sway: zSlider({ min: -8, max: 8, step: 0.25 }),
  }),
  swell: z.object({
    peak: zSlider({ min: 30, max: 500 }),
    length: zSlider({ min: 3, max: 40, step: 0.5 }),
    sharpness: zSlider({ min: 1, max: 8, step: 0.5 }),
    offset: zSlider({ min: 0, max: 1, step: 0.01 }),
    drift: zSlider({ min: 0, max: 8, step: 0.25 }),
    outline: z.boolean(),
    outlineFrom: zSlider({ min: 0, max: 1, step: 0.01 }),
    outlineWidth: zSlider({ min: 0.5, max: 10, step: 0.25 }),
  }),
  flow: z.object({
    speed: zSlider({ min: 0, max: 30 }),
    direction: z.enum(STREAM_DIRECTIONS),
    alternate: z.boolean(),
    riseDistance: zSlider({ min: 0, max: 2400 }),
    riseFrames: zSlider({ min: 0, max: 120 }),
    riseStagger: zSlider({ min: 0, max: 40 }),
    fadeFrames: zSlider({ min: 0, max: 60 }),
  }),
  tone: z.object({
    dark: z.boolean(),
  }),
  look: lookSchema,
  motion: motionSchema,
});

export type WordStreamsProps = z.infer<typeof wordStreamsSchema>;

export const wordStreamsDefaults: WordStreamsProps = {
  word: {
    text: 'up',
    size: 104,
    weight: 600,
    tracking: -0.03,
    spacing: 0.08,
  },
  streams: {
    count: 3,
    margin: 48,
    phaseStep: 90,
  },
  wave: {
    amplitude: 60,
    length: 1400,
    sway: 2,
  },
  swell: {
    peak: 210,
    length: 12,
    sharpness: 2,
    offset: 0.5,
    drift: 0,
    outline: false,
    outlineFrom: 0.5,
    outlineWidth: 3,
  },
  flow: {
    speed: 6,
    direction: 'up',
    alternate: false,
    riseDistance: 420,
    riseFrames: 54,
    riseStagger: 6,
    fadeFrames: 20,
  },
  tone: { dark: false },
  look: { ...lightLook('#0b0b0c'), grain: 0.05, spotlight: 0 },
  motion: defaultMotion({ float: 0 }),
};

export const wordStreamsMeta: ClipMeta = {
  ...promoMeta,
  id: 'word-streams',
  title: 'Word Streams',
  description:
    'Serpentine streams of one repeated word flow up the frame like conveyors, each copy tilted along its curve, with a swell of bigger copies riding up every stream.',
  category: 'kinetic-type',
  tags: ['kinetic', 'text', 'repeat', 'scroll'],
  theme: 'light',
  source: 'src/compositions/scenes/kinetic-type/word-streams.tsx',
  width: 1080,
  height: 1920,
  durationInFrames: 150,
  posterFrame: 90,
};
