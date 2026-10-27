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

const frames = (max: number) => zSlider({ min: 1, max });
const delay = (max: number) => zSlider({ min: 0, max });

export const BLOB_SHAPES = ['wobble', 'starburst'] as const;

export type BlobShape = (typeof BLOB_SHAPES)[number];

export const circlePopSchema = z.object({
  word: z.object({
    text: z.string(),
    startSize: zSlider({ min: 40, max: 400 }),
    endSize: zSlider({ min: 80, max: 900 }),
    weight: zSlider(FONT_WEIGHT),
    tracking: zSlider({ min: -0.1, max: 0.2, step: 0.005 }),
    rotation: zSlider({ min: -30, max: 30, step: 0.5 }),
    color: zColor(),
    floodColor: zColor(),
  }),
  disc: z.object({
    size: zSlider({ min: 120, max: 1000 }),
    offsetX: zSlider({ min: -400, max: 400 }),
    offsetY: zSlider({ min: -800, max: 800 }),
  }),
  pop: z.object({
    delay: delay(60),
    frames: zSlider({ min: 2, max: 60 }),
    overshoot: zSlider({ min: 0, max: 0.3, step: 0.01 }),
    fadeFrames: frames(20),
  }),
  wobble: z.object({
    shape: z.enum(BLOB_SHAPES),
    lobes: zSlider({ min: 2, max: 24 }),
    amount: zSlider({ min: 0, max: 0.2, step: 0.005 }),
    speed: zSlider({ min: 0, max: 40, step: 0.5 }),
    buildFrames: frames(90),
  }),
  flood: z.object({
    frames: frames(90),
    reach: zSlider({ min: 1.01, max: 1.6, step: 0.01 }),
    wordDelay: delay(60),
    wordFrames: frames(90),
  }),
  tilt: z.object({
    angle: zSlider({ min: -30, max: 30, step: 0.5 }),
    delay: delay(90),
    frames: frames(90),
  }),
  subline: z.object({
    text: z.string(),
    size: zSlider({ min: 16, max: 160 }),
    weight: zSlider(FONT_WEIGHT),
    tracking: zSlider({ min: -0.05, max: 0.4, step: 0.005 }),
    color: zColor(),
    gap: zSlider({ min: -200, max: 400 }),
    rise: zSlider({ min: 0, max: 120 }),
    delay: delay(90),
    frames: frames(60),
  }),
  inverted: z.boolean(),
  look: lookSchema,
  motion: motionSchema,
});

export type CirclePopProps = z.infer<typeof circlePopSchema>;

export const circlePopDefaults: CirclePopProps = {
  word: {
    text: 'Pop',
    startSize: 150,
    endSize: 520,
    weight: 700,
    tracking: -0.04,
    rotation: -6,
    color: '#020202',
    floodColor: '#020202',
  },
  disc: {
    size: 420,
    offsetX: 0,
    offsetY: 0,
  },
  pop: {
    delay: 2,
    frames: 18,
    overshoot: 0.08,
    fadeFrames: 8,
  },
  wobble: {
    shape: 'wobble',
    lobes: 7,
    amount: 0.06,
    speed: 12,
    buildFrames: 22,
  },
  flood: {
    frames: 22,
    reach: 1.04,
    wordDelay: 2,
    wordFrames: 26,
  },
  tilt: {
    angle: 3,
    delay: 10,
    frames: 18,
  },
  subline: {
    text: '',
    size: 48,
    weight: 500,
    tracking: 0.02,
    color: '#020202',
    gap: 36,
    rise: 28,
    delay: 18,
    frames: 16,
  },
  inverted: false,
  look: { ...kineticLook(), backgroundEdge: '#020202', grain: 0 },
  motion: defaultMotion(),
};

export const circlePopMeta: ClipMeta = {
  ...promoMeta,
  id: 'circle-pop',
  title: 'Circle Pop',
  description:
    'A white disc springs open around a bold word, its edge turns into a wobbling blob, then it bursts past every corner to flood the frame white while the word swells to fill it and tilts into place.',
  category: 'kinetic-type',
  tags: ['kinetic', 'headline', 'zoom', 'reveal', 'minimal'],
  theme: 'dark',
  source: 'src/compositions/scenes/kinetic-type/circle-pop.tsx',
  width: 1080,
  height: 1920,
  durationInFrames: 96,
  posterFrame: 78,
};
