import { z } from 'zod';
import { zColor, zSlider } from '../../control';
import { defaultMotion, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const FLIP_TONES = ['dark', 'light'] as const;

export type FlipTone = (typeof FLIP_TONES)[number];

export const FLIP_FONTS = [
  'condensed',
  'serif',
  'didone',
  'marker',
  'brush',
  'hand',
  'thin',
  'light',
  'script',
  'poster',
] as const;

export type FlipFont = (typeof FLIP_FONTS)[number];

export const TITLE_LINES = ['top', 'bottom'] as const;

const frames = (max = 120) => zSlider({ min: 0, max });

const unit = (max = 1, step = 0.01) => zSlider({ min: 0, max, step });

export const flipBeatSchema = z.object({
  text: z.string(),
  tone: z.enum(FLIP_TONES),
  frames: zSlider({ min: 1, max: 120 }),
  blank: frames(12),
  cycleFonts: z.boolean(),
});

export type FlipBeat = z.infer<typeof flipBeatSchema>;

export const comingSoonFlipsSchema = z.object({
  beats: z.array(flipBeatSchema).min(1).max(16),
  words: z.object({
    size: zSlider({ min: 60, max: 400 }),
    weight: zSlider({ min: 400, max: 700, step: 100 }),
    tracking: zSlider({ min: -0.1, max: 0.3, step: 0.005 }),
    centerY: unit(),
    cycle: z.array(z.enum(FLIP_FONTS)).min(1).max(24),
    cycleHold: zSlider({ min: 1, max: 12 }),
    bar: z.object({
      show: z.boolean(),
      length: zSlider({ min: 0, max: 900 }),
      thickness: zSlider({ min: 1, max: 40 }),
      gap: zSlider({ min: -40, max: 200 }),
      startWidth: unit(),
      onDark: zColor(),
      onLight: zColor(),
    }),
  }),
  title: z.object({
    top: z.string(),
    bottom: z.string(),
    topSize: zSlider({ min: 40, max: 300 }),
    bottomSize: zSlider({ min: 40, max: 300 }),
    topStretch: zSlider({ min: 0.6, max: 1.6, step: 0.01 }),
    bottomStretch: zSlider({ min: 0.6, max: 1.6, step: 0.01 }),
    weight: zSlider({ min: 700, max: 900, step: 100 }),
    tracking: zSlider({ min: -0.1, max: 0.3, step: 0.005 }),
    gap: zSlider({ min: -60, max: 200 }),
    topY: unit(),
    outline: zSlider({ min: 1, max: 16, step: 0.5 }),
    outlined: z.enum(TITLE_LINES),
    color: zColor(),
    accent: zColor(),
    spread: zSlider({ min: 1, max: 3, step: 0.01 }),
    settleStart: frames(),
    settleFrames: zSlider({ min: 1, max: 120 }),
    appearSpan: frames(60),
    flickerFrames: frames(30),
    seed: zSlider({ min: 1, max: 999 }),
  }),
  logo: z.object({
    mark: z.string(),
    caption: z.string(),
    size: zSlider({ min: 80, max: 800 }),
    overlap: zSlider({ min: -100, max: 300 }),
    gap: zSlider({ min: 0, max: 40 }),
    captionSize: zSlider({ min: 16, max: 140 }),
    captionGap: zSlider({ min: -40, max: 200 }),
    centerY: unit(),
    color: zColor(),
    captionColor: zColor(),
  }),
  wall: z.object({
    base: zColor(),
    blotch: zColor(),
    speck: zColor(),
    speckAmount: unit(),
    blotchScale: zSlider({ min: 0.001, max: 0.03, step: 0.0005 }),
    blotchAmount: unit(),
    grit: unit(),
    seed: zSlider({ min: 1, max: 99 }),
  }),
  beam: z.object({
    angle: zSlider({ min: 0, max: 180 }),
    softness: zSlider({ min: 20, max: 600 }),
    slats: zSlider({ min: 1, max: 5 }),
    slatGap: zSlider({ min: 0, max: 800 }),
    strength: unit(1.5),
    tint: zColor(),
    start: zSlider({ min: -1200, max: 3000 }),
    peak: zSlider({ min: -1200, max: 3000 }),
    end: zSlider({ min: -1200, max: 3000 }),
    turnAt: frames(200),
    turnFrames: zSlider({ min: 2, max: 60 }),
  }),
  flashFrames: frames(12),
  look: z.object({
    showBackdrop: z.boolean(),
    dark: zColor(),
    light: zColor(),
    onDark: zColor(),
    onLight: zColor(),
    grain: unit(0.3),
    liveGrain: z.boolean(),
  }),
  motion: motionSchema,
});

export type ComingSoonFlipsProps = z.infer<typeof comingSoonFlipsSchema>;

const beat = (
  text: string,
  tone: FlipTone,
  length: number,
  blank = 0,
  cycleFonts = false,
): FlipBeat => ({ text, tone, frames: length, blank, cycleFonts });

export const comingSoonFlipsDefaults: ComingSoonFlipsProps = {
  beats: [
    beat('', 'dark', 18),
    beat('HOLD', 'light', 26),
    beat('YOUR', 'dark', 26, 1),
    beat('BREATH', 'light', 22, 1),
    beat('BREATH', 'dark', 32, 1, true),
    beat("WHAT'S", 'light', 20, 1),
    beat('NEXT', 'dark', 26),
    beat('IS', 'light', 27),
    beat('LANDING', 'dark', 23, 1),
  ],
  words: {
    size: 130,
    weight: 700,
    tracking: 0,
    centerY: 0.488,
    cycle: [
      'serif',
      'marker',
      'didone',
      'hand',
      'condensed',
      'thin',
      'brush',
      'light',
      'poster',
      'script',
      'serif',
      'condensed',
      'marker',
      'thin',
      'brush',
      'hand',
    ],
    cycleHold: 2,
    bar: {
      show: true,
      length: 150,
      thickness: 9,
      gap: 30,
      startWidth: 0.3,
      onDark: '#ff6b35',
      onLight: '#0d1530',
    },
  },
  title: {
    top: 'ALMOST',
    bottom: 'HERE',
    topSize: 112,
    bottomSize: 214,
    topStretch: 1.12,
    bottomStretch: 0.9,
    weight: 800,
    tracking: 0.035,
    gap: 34,
    topY: 0.132,
    outline: 4,
    outlined: 'top',
    color: '#fff3e6',
    accent: '#ff6b35',
    spread: 1.5,
    settleStart: 2,
    settleFrames: 32,
    appearSpan: 9,
    flickerFrames: 8,
    seed: 34,
  },
  logo: {
    mark: 'NV',
    caption: 'Nova Studio',
    size: 440,
    overlap: 70,
    gap: 12,
    captionSize: 66,
    captionGap: 40,
    centerY: 0.486,
    color: '#0d1530',
    captionColor: '#0d1530',
  },
  wall: {
    base: '#c4d2e6',
    blotch: '#b3c4dc',
    speck: '#8a9fbe',
    speckAmount: 0.45,
    blotchScale: 0.012,
    blotchAmount: 0.5,
    grit: 0.35,
    seed: 11,
  },
  beam: {
    angle: 65,
    softness: 72,
    slats: 2,
    slatGap: 250,
    strength: 1.1,
    tint: '#fff0e2',
    start: -100,
    peak: 1950,
    end: -230,
    turnAt: 59,
    turnFrames: 40,
  },
  flashFrames: 2,
  look: {
    showBackdrop: true,
    dark: '#0d1530',
    light: '#ff6b35',
    onDark: '#fff3e6',
    onLight: '#0d1530',
    grain: 0.05,
    liveGrain: true,
  },
  motion: defaultMotion({ exit: 'none', exitFrames: 0, float: 0 }),
};

export const comingSoonFlipsMeta: ClipMeta = {
  ...promoMeta,
  width: 1080,
  height: 1920,
  id: 'coming-soon-flips',
  title: 'Coming Soon Flips',
  description:
    'Condensed words with a growing underline hard cut between ink and tangerine plates, one flickering through typefaces, then a tracked-in title while slatted light sweeps a plaster wall to reveal a logo.',
  category: 'kinetic-type',
  tags: ['kinetic', 'text', 'headline', 'flip', 'outline', 'reveal', 'logo', 'texture'],
  theme: 'dark',
  source: 'src/compositions/scenes/kinetic-type/coming-soon-flips.tsx',
  durationInFrames: 355,
  posterFrame: 318,
};
