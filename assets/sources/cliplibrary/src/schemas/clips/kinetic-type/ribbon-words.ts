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

const frameAt = (max = 600) => zSlider({ min: 0, max });

export const RIBBON_STYLES = ['solid', 'outline'] as const;

export const RIBBON_SHAPES = ['wave', 'swoop', 'swell'] as const;
const span = (max = 120) => zSlider({ min: 2, max });

export const ribbonWordsSchema = z.object({
  text: z.string(),
  highlight: z.string(),
  words: z.object({
    size: zSlider({ min: 20, max: 200 }),
    weight: zSlider(FONT_WEIGHT),
    tracking: zSlider({ min: -0.1, max: 0.3, step: 0.005 }),
    centering: zSlider({ min: -0.2, max: 0.6, step: 0.01 }),
  }),
  ribbon: z.object({
    color: zColor(),
    width: zSlider({ min: 30, max: 320 }),
    style: z.enum(RIBBON_STYLES),
    outline: zSlider({ min: 1, max: 30 }),
  }),
  second: z.object({
    enabled: z.boolean(),
    text: z.string(),
    offsetY: zSlider({ min: -1200, max: 1200 }),
    shift: zSlider({ min: -180, max: 180 }),
    delay: zSlider({ min: 0, max: 60 }),
  }),
  inverted: z.boolean(),
  path: z.object({
    shape: z.enum(RIBBON_SHAPES),
    centerY: zSlider({ min: 0, max: 1920 }),
    drop: zSlider({ min: -2000, max: 2000 }),
    amplitude: zSlider({ min: 0, max: 800 }),
    waves: zSlider({ min: 0, max: 3, step: 0.05 }),
    shift: zSlider({ min: 0, max: 360 }),
    breathe: zSlider({ min: 0, max: 60 }),
    breatheFrames: zSlider({ min: 30, max: 600 }),
  }),
  timing: z.object({
    drawAt: frameAt(),
    drawFrames: span(),
    lag: zSlider({ min: 0, max: 400 }),
    settleAt: zSlider({ min: 0.1, max: 0.9, step: 0.01 }),
    settleFrames: span(),
    drift: zSlider({ min: 1, max: 6, step: 0.1 }),
    retractAt: frameAt(),
    retractFrames: span(),
  }),
  look: lookSchema,
  motion: motionSchema,
});

export type RibbonWordsProps = z.infer<typeof ribbonWordsSchema>;

export const ribbonWordsDefaults: RibbonWordsProps = {
  text: 'keep it moving',
  highlight: '',
  words: {
    size: 112,
    weight: 600,
    tracking: -0.02,
    centering: 0.28,
  },
  ribbon: {
    color: '#f1f1f1',
    width: 180,
    style: 'solid',
    outline: 5,
  },
  second: {
    enabled: false,
    text: 'never stand still',
    offsetY: 420,
    shift: 0,
    delay: 8,
  },
  inverted: false,
  path: {
    shape: 'wave',
    centerY: 980,
    drop: 1300,
    amplitude: 330,
    waves: 0.85,
    shift: 20,
    breathe: 18,
    breatheFrames: 160,
  },
  timing: {
    drawAt: 0,
    drawFrames: 46,
    lag: 36,
    settleAt: 0.5,
    settleFrames: 30,
    drift: 1.2,
    retractAt: 68,
    retractFrames: 58,
  },
  look: { ...kineticLook('#8a8a8a'), backgroundEdge: '#020202', grain: 0, liveGrain: false },
  motion: defaultMotion({ exit: 'fade', exitFrames: 16, float: 0 }),
};

export const ribbonWordsMeta: ClipMeta = {
  ...promoMeta,
  id: 'ribbon-words',
  title: 'Ribbon Words',
  description:
    'A bold white ribbon draws a big S down a black frame, carrying a short phrase that settles mid-ribbon and drifts, then rides off as the ribbon retracts.',
  category: 'kinetic-type',
  tags: ['kinetic', 'text', 'draw', 'slide'],
  theme: 'dark',
  source: 'src/compositions/scenes/kinetic-type/ribbon-words.tsx',
  width: 1080,
  height: 1920,
  durationInFrames: 120,
  posterFrame: 56,
};
