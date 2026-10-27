import { z } from 'zod';
import { zColor, zSlider } from '../../control';
import { defaultMotion, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';

const frameAt = (max = 600) => zSlider({ min: 0, max, step: 0.5 });
const frames = (max = 60) => zSlider({ min: 0, max, step: 0.5 });
const weight = () => zSlider({ min: 400, max: 900, step: 100 });
const tracking = () => zSlider({ min: -0.12, max: 0.12, step: 0.005 });
const baseline = () => zSlider({ min: 0, max: 1920 });

const calloutPaletteSchema = z.object({
  paper: zColor(),
  ink: zColor(),
  muted: zColor(),
  outline: zColor(),
  night: zColor(),
  glow: zColor(),
  accent: zColor(),
  urlInk: zColor(),
  markFill: zColor(),
  markFace: zColor(),
  markInk: zColor(),
});

const calloutIntroSchema = z.object({
  word: z.string(),
  size: zSlider({ min: 60, max: 500 }),
  weight: weight(),
  tracking: tracking(),
  baseline: baseline(),
  offsetX: zSlider({ min: -400, max: 400 }),
  outlineWidth: zSlider({ min: 0, max: 12, step: 0.5 }),
  cubeSize: zSlider({ min: 20, max: 300 }),
  cubeCorner: zSlider({ min: 0, max: 0.5, step: 0.01 }),
  landOffset: zSlider({ min: -0.6, max: 0.6, step: 0.01 }),
  start: frameAt(),
  dropFrames: frames(),
  hopFrames: frames(),
  arc: zSlider({ min: 0, max: 300 }),
  dwellFrames: frames(),
  press: zSlider({ min: 0, max: 150 }),
  fillDelay: frames(10),
  exitHop: frames(20),
  exitRise: zSlider({ min: 0, max: 400 }),
  exitDistance: zSlider({ min: -800, max: 800 }),
  exitFloor: zSlider({ min: 0, max: 2400 }),
  exitFrames: frames(),
  motionBlur: zSlider({ min: 1, max: 16 }),
  shutterAngle: zSlider({ min: 0, max: 360, step: 5 }),
});

const calloutScrollSchema = z.object({
  word: z.string(),
  size: zSlider({ min: 60, max: 500 }),
  weight: weight(),
  tracking: tracking(),
  baseline: baseline(),
  pitch: zSlider({ min: 40, max: 600 }),
  count: zSlider({ min: 1, max: 12 }),
  start: frameAt(),
  addFrames: frames(30),
  clearAt: frameAt(),
  clearFrames: frames(30),
  endAt: frameAt(),
});

const calloutStackLineSchema = z.object({
  text: z.string(),
  size: zSlider({ min: 20, max: 500 }),
  weight: weight(),
  gap: zSlider({ min: -100, max: 300 }),
  accent: z.boolean(),
  at: frameAt(),
  dimAt: frameAt(),
  leaveAt: frameAt(),
});

type CalloutStackLine = z.infer<typeof calloutStackLineSchema>;

const calloutStackSchema = z.object({
  lines: z.array(calloutStackLineSchema).min(1).max(10),
  tracking: tracking(),
  baseline: baseline(),
  endAt: frameAt(),
});

const calloutPunchSchema = z.object({
  words: z.string(),
  size: zSlider({ min: 60, max: 600 }),
  weight: weight(),
  tracking: tracking(),
  baseline: baseline(),
  start: frameAt(),
  wordFrames: frames(),
  primeFrames: frames(10),
});

const calloutCollapseSchema = z.object({
  at: frameAt(),
  dotSize: zSlider({ min: 10, max: 160 }),
  sweepFrames: frames(),
  drift: zSlider({ min: -300, max: 300 }),
  letterFade: frames(20),
  firstFade: frames(30),
  riseAt: frameAt(),
  riseFrames: frames(),
});

const calloutContactSchema = z.object({
  text: z.string(),
  size: zSlider({ min: 30, max: 300 }),
  weight: weight(),
  tracking: tracking(),
  baseline: baseline(),
  drop: zSlider({ min: -200, max: 200 }),
  at: frameAt(),
  dotHold: frames(20),
  hopAt: frameAt(),
});

const calloutEndSchema = z.object({
  at: frameAt(),
  monogram: z.string(),
  url: z.string(),
  urlSize: zSlider({ min: 12, max: 120 }),
  urlWeight: weight(),
  urlTracking: zSlider({ min: 0, max: 0.4, step: 0.01 }),
  urlBaseline: baseline(),
  urlUppercase: z.boolean(),
  lineWidth: zSlider({ min: 0, max: 1080 }),
  lineThickness: zSlider({ min: 1, max: 40 }),
  lineY: baseline(),
  lineAt: frameAt(),
  lineFrames: frames(),
  overshoot: zSlider({ min: 0, max: 0.6, step: 0.01 }),
  markSize: zSlider({ min: 100, max: 900 }),
  markCorner: zSlider({ min: 0, max: 0.5, step: 0.01 }),
  markY: baseline(),
  markAt: frameAt(),
  markFrames: frames(),
  slices: zSlider({ min: 2, max: 40 }),
  urlAt: frameAt(),
  urlFrames: frames(),
  glow: zSlider({ min: 0, max: 1, step: 0.01 }),
});

export const scrollerCalloutSchema = z.object({
  palette: calloutPaletteSchema,
  intro: calloutIntroSchema,
  scroll: calloutScrollSchema,
  stack: calloutStackSchema,
  punch: calloutPunchSchema,
  collapse: calloutCollapseSchema,
  contact: calloutContactSchema,
  end: calloutEndSchema,
  look: z.object({ showBackdrop: z.boolean() }),
  motion: motionSchema,
});

export type ScrollerCalloutProps = z.infer<typeof scrollerCalloutSchema>;

const line = (
  text: string,
  size: number,
  gap: number,
  at: number,
  dimAt: number,
  leaveAt: number,
  accent = false,
): CalloutStackLine => ({ text, size, weight: 700, gap, accent, at, dimAt, leaveAt });

export const scrollerCalloutDefaults: ScrollerCalloutProps = {
  palette: {
    paper: '#f4efe6',
    ink: '#172340',
    muted: '#8b93a7',
    outline: '#9aa2b5',
    night: '#0e1630',
    glow: '#3d5bd9',
    accent: '#ff5a36',
    urlInk: '#dfe4f2',
    markFill: '#ff5a36',
    markFace: '#f4efe6',
    markInk: '#172340',
  },
  intro: {
    word: 'Psst',
    size: 248,
    weight: 700,
    tracking: 0.06,
    baseline: 1021,
    offsetX: 21,
    outlineWidth: 2.5,
    cubeSize: 123,
    cubeCorner: 0.2,
    landOffset: -0.075,
    start: 0,
    dropFrames: 5,
    hopFrames: 7,
    arc: 40,
    dwellFrames: 4.5,
    press: 62,
    fillDelay: 1.5,
    exitHop: 2.5,
    exitRise: 90,
    exitDistance: 230,
    exitFloor: 2090,
    exitFrames: 12,
    motionBlur: 16,
    shutterAngle: 180,
  },
  scroll: {
    word: 'Swiping',
    size: 261,
    weight: 700,
    tracking: -0.035,
    baseline: 1053,
    pitch: 210,
    count: 5,
    start: 61,
    addFrames: 4,
    clearAt: 79,
    clearFrames: 2,
    endAt: 91,
  },
  stack: {
    lines: [
      line('Swiping', 64, 0, 97, 124, 143),
      line('Won’t', 105, 38, 106, 124, 143),
      line('Make', 170, 29, 109, 124, 151),
      line('Your', 219, 35, 112, 124, 151),
      line('BRAND', 250, 48, 124, 0, 0, true),
      line('Famous', 256, 48, 133, 133, 0),
    ],
    tracking: -0.01,
    baseline: 485,
    endAt: 160,
  },
  punch: {
    words: 'COUNT ON US.',
    size: 262,
    weight: 800,
    tracking: -0.015,
    baseline: 1074,
    start: 160,
    wordFrames: 9,
    primeFrames: 1,
  },
  collapse: {
    at: 203.5,
    dotSize: 55,
    sweepFrames: 9,
    drift: 62,
    letterFade: 2,
    firstFade: 6,
    riseAt: 222,
    riseFrames: 6,
  },
  contact: {
    text: 'say hello today',
    size: 132,
    weight: 700,
    tracking: -0.027,
    baseline: 992,
    drop: 24,
    at: 242,
    dotHold: 2,
    hopAt: 250,
  },
  end: {
    at: 268,
    monogram: 'NF',
    url: 'northfold.studio',
    urlSize: 41,
    urlWeight: 500,
    urlTracking: 0.14,
    urlBaseline: 1282,
    urlUppercase: true,
    lineWidth: 531,
    lineThickness: 9,
    lineY: 1226,
    lineAt: 274,
    lineFrames: 5,
    overshoot: 0.25,
    markSize: 480,
    markCorner: 0.24,
    markY: 803,
    markAt: 280,
    markFrames: 10,
    slices: 14,
    urlAt: 280,
    urlFrames: 8,
    glow: 0.5,
  },
  look: { showBackdrop: true },
  motion: defaultMotion({ exit: 'none', exitFrames: 0, float: 0 }),
};

export const scrollerCalloutMeta: ClipMeta = {
  ...promoMeta,
  id: 'scroller-callout',
  title: 'Scroller Callout',
  description:
    'A tumbling coral tile knocks an outlined word solid, a word stacks and scrolls away, a mixed-weight block builds line by line around one accent word, then a coral full stop collapses the last word into two dots that land in a call to action before a navy end card with a sliding tile badge.',
  category: 'kinetic-type',
  tags: ['kinetic', 'headline', 'stack', 'text', 'drop', 'outline', 'end-card', 'minimal'],
  theme: 'light',
  source: 'src/compositions/scenes/kinetic-type/scroller-callout.tsx',
  width: 1080,
  height: 1920,
  durationInFrames: 339,
  posterFrame: 140,
};
