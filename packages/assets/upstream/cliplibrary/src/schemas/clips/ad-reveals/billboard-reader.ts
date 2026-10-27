import { z } from 'zod';
import { zColor, zMultiline, zSlider } from '../../control';
import { defaultMotion, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';

const unit = (max = 1, step = 0.01) => zSlider({ min: 0, max, step });

const frames = (max = 240) => zSlider({ min: 0, max });

export const readerBoardSchema = z.object({
  headline: zMultiline(),
  tagline: zMultiline(),
  brand: zMultiline(),
  paper: zColor(),
  headlineColor: zColor(),
  highlightLastLine: z.boolean(),
  accentColor: zColor(),
  taglineColor: zColor(),
  brandColor: zColor(),
  markColor: zColor(),
  frame: zColor(),
  pole: zColor(),
  lamps: z.boolean(),
  lampColor: zColor(),
  lampGlow: unit(2, 0.05),
  headlineSize: zSlider({ min: 0.5, max: 1.4, step: 0.01 }),
  textWidth: zSlider({ min: 0.3, max: 1, step: 0.01 }),
  yaw: zSlider({ min: -0.8, max: 0.8, step: 0.01 }),
});

export type ReaderBoard = z.infer<typeof readerBoardSchema>;

export const readerHeroSchema = z.object({
  suit: zColor(),
  trim: zColor(),
  cape: zColor(),
  visor: zColor(),
  emblem: zColor(),
  scale: zSlider({ min: 2, max: 8, step: 0.05 }),
  seatX: zSlider({ min: -6, max: 6, step: 0.05 }),
  capeSway: unit(2),
  newsprint: zColor(),
  newsInk: zColor(),
  masthead: z.string(),
  frontHeadline: zMultiline(),
});

export type ReaderHero = z.infer<typeof readerHeroSchema>;

export const readerTimingSchema = z.object({
  turnStart: frames(),
  turnFrames: zSlider({ min: 10, max: 90 }),
  lowerStart: frames(),
  lowerFrames: zSlider({ min: 10, max: 120 }),
  breathe: unit(2),
  lampsStart: frames(),
  lampsFrames: zSlider({ min: 1, max: 60 }),
  lampsStagger: zSlider({ min: 0, max: 20 }),
});

export type ReaderTiming = z.infer<typeof readerTimingSchema>;

export const readerCameraSchema = z.object({
  fieldOfView: zSlider({ min: 30, max: 75, step: 1 }),
  startDistance: zSlider({ min: 20, max: 90, step: 0.5 }),
  endDistance: zSlider({ min: 10, max: 60, step: 0.5 }),
  startHeight: zSlider({ min: -20, max: 20, step: 0.5 }),
  endHeight: zSlider({ min: -10, max: 30, step: 0.5 }),
  startSide: zSlider({ min: -30, max: 30, step: 0.5 }),
  endSide: zSlider({ min: -30, max: 30, step: 0.5 }),
  startAim: zSlider({ min: -15, max: 15, step: 0.1 }),
  endAim: zSlider({ min: -15, max: 15, step: 0.1 }),
  ease: unit(),
  handheld: unit(2),
});

export type ReaderCamera = z.infer<typeof readerCameraSchema>;

export const readerWorldSchema = z.object({
  skyTop: zColor(),
  skyHorizon: zColor(),
  sunGlow: zColor(),
  haze: zColor(),
  hazeDensity: unit(4, 0.05),
  sunHeight: zSlider({ min: -0.1, max: 0.6, step: 0.01 }),
  sunAzimuth: zSlider({ min: -1.5, max: 1.5, step: 0.01 }),
  sea: zColor(),
  foam: zColor(),
  sand: zColor(),
  road: zColor(),
  land: zColor(),
  walls: zColor(),
  roofs: zColor(),
  foliage: zColor(),
  windows: zColor(),
  coastAngle: zSlider({ min: -1, max: 1, step: 0.01 }),
  waveSpeed: unit(3, 0.05),
  traffic: z.boolean(),
});

export type ReaderWorld = z.infer<typeof readerWorldSchema>;

export const readerLightSchema = z.object({
  sun: zSlider({ min: 0, max: 6, step: 0.05 }),
  sunColor: zColor(),
  fill: unit(3, 0.05),
  fillColor: zColor(),
  exposure: zSlider({ min: 0.2, max: 2, step: 0.01 }),
});

export type ReaderLight = z.infer<typeof readerLightSchema>;

export const readerLookSchema = z.object({
  showBackdrop: z.boolean(),
  vignette: unit(),
  vignetteColor: zColor(),
  grade: unit(),
  gradeTop: zColor(),
  gradeBottom: zColor(),
});

export type ReaderLook = z.infer<typeof readerLookSchema>;

export const billboardReaderSchema = z.object({
  board: readerBoardSchema,
  hero: readerHeroSchema,
  timing: readerTimingSchema,
  camera: readerCameraSchema,
  world: readerWorldSchema,
  light: readerLightSchema,
  look: readerLookSchema,
  motion: motionSchema,
});

export type BillboardReaderProps = z.infer<typeof billboardReaderSchema>;

export const billboardReaderDefaults: BillboardReaderProps = {
  board: {
    headline: 'YOUR BRAND,\nLEGENDARY.',
    tagline: 'Bold stories turn\nscrollers into fans.',
    brand: 'LEGEND\nWORKS',
    paper: '#16204a',
    headlineColor: '#f7efe4',
    highlightLastLine: true,
    accentColor: '#ffb347',
    taglineColor: '#b9c4ee',
    brandColor: '#f7efe4',
    markColor: '#ffb347',
    frame: '#1c1f2c',
    pole: '#1a1c27',
    lamps: true,
    lampColor: '#ffd9a0',
    lampGlow: 1,
    headlineSize: 1,
    textWidth: 0.5,
    yaw: -0.5,
  },
  hero: {
    suit: '#5a3f9e',
    trim: '#2fd3b5',
    cape: '#ff6b5e',
    visor: '#10121d',
    emblem: '#ffb347',
    scale: 5.4,
    seatX: 4.9,
    capeSway: 1,
    newsprint: '#ede8f0',
    newsInk: '#2c2a3a',
    masthead: 'THE EVENING LEGEND',
    frontHeadline: 'CAPED READER\nSPOTTED AT DUSK',
  },
  timing: {
    turnStart: 56,
    turnFrames: 40,
    lowerStart: 84,
    lowerFrames: 40,
    breathe: 1,
    lampsStart: 30,
    lampsFrames: 16,
    lampsStagger: 7,
  },
  camera: {
    fieldOfView: 50,
    startDistance: 36,
    endDistance: 25,
    startHeight: -9.5,
    endHeight: 11,
    startSide: -10,
    endSide: 10,
    startAim: -3.8,
    endAim: 3.7,
    ease: 0.4,
    handheld: 1,
  },
  world: {
    skyTop: '#3a3f7a',
    skyHorizon: '#f6a48c',
    sunGlow: '#ffd0b0',
    haze: '#b48fa8',
    hazeDensity: 0.7,
    sunHeight: 0.03,
    sunAzimuth: -0.3,
    sea: '#2e3560',
    foam: '#efe2f0',
    sand: '#a98a92',
    road: '#2a2a3a',
    land: '#2c3340',
    walls: '#9d93b0',
    roofs: '#4f5f8a',
    foliage: '#24363a',
    windows: '#ffc77a',
    coastAngle: -0.1,
    waveSpeed: 1,
    traffic: true,
  },
  light: {
    sun: 2.8,
    sunColor: '#ffc4ae',
    fill: 1,
    fillColor: '#c9cdf2',
    exposure: 0.95,
  },
  look: {
    showBackdrop: true,
    vignette: 0.4,
    vignetteColor: '#1a1440',
    grade: 0.18,
    gradeTop: '#7a6cff',
    gradeBottom: '#ff8a6b',
  },
  motion: defaultMotion({ exit: 'none', exitFrames: 0, float: 0 }),
};

export const billboardReaderMeta: ClipMeta = {
  ...promoMeta,
  fps: 24,
  width: 1080,
  height: 1920,
  id: 'billboard-reader',
  title: 'Billboard Reader',
  description:
    'A drone glides up to a dark roadside billboard over a dusky coastline as its lamps blink on one by one, while a caped hero sitting on top turns a page of his newspaper.',
  category: 'ad-reveals',
  tags: ['3d', 'billboard', 'camera-move', 'headline', 'paper', 'glow'],
  theme: 'light',
  source: 'src/compositions/scenes/ad-reveals/billboard-reader.tsx',
  durationInFrames: 240,
  posterFrame: 150,
};
