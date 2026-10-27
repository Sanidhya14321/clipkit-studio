import { z } from 'zod';
import { zColor, zImage, zMultiline, zSlider } from '../../control';
import { defaultMotion, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const billboardPosterSchema = z.object({
  image: zImage(),
  eyebrow: z.string(),
  headline: zMultiline(),
  subline: z.string(),
  details: zMultiline(),
  footer: z.string(),
  paper: zColor(),
  ink: zColor(),
  accent: zColor(),
  accentSoft: zColor(),
});

export type BillboardPoster = z.infer<typeof billboardPosterSchema>;

export const unveilClothSchema = z.object({
  color: zColor(),
  sheen: zSlider({ min: 0, max: 1, step: 0.01 }),
  weight: zSlider({ min: 0.05, max: 0.6, step: 0.01 }),
  softness: zSlider({ min: 0, max: 1, step: 0.01 }),
  wind: zSlider({ min: 0, max: 12, step: 0.1 }),
  gusts: zSlider({ min: 0, max: 1.5, step: 0.01 }),
  downwash: zSlider({ min: 0, max: 30, step: 0.5 }),
  flutter: zSlider({ min: 0, max: 2, step: 0.05 }),
});

export type UnveilCloth = z.infer<typeof unveilClothSchema>;

export const unveilHelicopterSchema = z.object({
  body: zColor(),
  trim: zColor(),
  stripe: zColor(),
  glass: zColor(),
  scale: zSlider({ min: 0.5, max: 1.6, step: 0.01 }),
  rotorSpeed: zSlider({ min: 0, max: 2, step: 0.05 }),
  cables: zColor(),
});

export type UnveilHelicopter = z.infer<typeof unveilHelicopterSchema>;

export const unveilSkySchema = z.object({
  zenith: zColor(),
  horizon: zColor(),
  turbidity: zSlider({ min: 1, max: 20, step: 0.1 }),
  rayleigh: zSlider({ min: 0, max: 4, step: 0.05 }),
  exposure: zSlider({ min: 0.2, max: 2, step: 0.01 }),
  cloudShade: zColor(),
  clouds: zSlider({ min: 0, max: 1, step: 0.01 }),
  cloudDrift: zSlider({ min: 0, max: 2, step: 0.05 }),
  sun: zSlider({ min: 0, max: 4, step: 0.05 }),
  sunColor: zColor(),
  sunSide: zSlider({ min: -1, max: 1, step: 0.01 }),
  sunHeight: zSlider({ min: 0.2, max: 1.2, step: 0.01 }),
  haze: zColor(),
  fog: zSlider({ min: 0, max: 0.008, step: 0.0001 }),
});

export type UnveilSky = z.infer<typeof unveilSkySchema>;

export const unveilGroundSchema = z.object({
  showRoad: z.boolean(),
  traffic: z.boolean(),
  road: zColor(),
  verge: zColor(),
  trees: zColor(),
  skyline: zColor(),
  frame: zColor(),
  pole: zColor(),
});

export type UnveilGround = z.infer<typeof unveilGroundSchema>;

export const unveilTimingSchema = z.object({
  liftStart: zSlider({ min: 20, max: 200 }),
  climb: zSlider({ min: 40, max: 160 }),
  cardStart: zSlider({ min: 180, max: 340 }),
  cardFrames: zSlider({ min: 10, max: 60 }),
});

export type UnveilTiming = z.infer<typeof unveilTimingSchema>;

export const unveilOutroSchema = z.object({
  showCard: z.boolean(),
  cardWidth: zSlider({ min: 0.4, max: 0.95, step: 0.01 }),
  blur: zSlider({ min: 0, max: 40 }),
  dim: zSlider({ min: 0, max: 0.9, step: 0.01 }),
  shadow: zSlider({ min: 0, max: 1, step: 0.01 }),
});

export type UnveilOutro = z.infer<typeof unveilOutroSchema>;

export const unveilCameraSchema = z.object({
  fieldOfView: zSlider({ min: 30, max: 80, step: 1 }),
  height: zSlider({ min: 4, max: 30, step: 0.5 }),
  distance: zSlider({ min: 30, max: 90, step: 0.5 }),
  pushIn: zSlider({ min: 0, max: 0.3, step: 0.01 }),
  drift: zSlider({ min: 0, max: 1, step: 0.01 }),
  boardAngle: zSlider({ min: -0.8, max: 0.8, step: 0.01 }),
});

export type UnveilCamera = z.infer<typeof unveilCameraSchema>;

export const helicopterUnveilSchema = z.object({
  poster: billboardPosterSchema,
  cloth: unveilClothSchema,
  helicopter: unveilHelicopterSchema,
  sky: unveilSkySchema,
  ground: unveilGroundSchema,
  camera: unveilCameraSchema,
  timing: unveilTimingSchema,
  outro: unveilOutroSchema,
  motion: motionSchema,
});

export type HelicopterUnveilProps = z.infer<typeof helicopterUnveilSchema>;

export const helicopterUnveilDefaults: HelicopterUnveilProps = {
  poster: {
    image: '',
    eyebrow: 'Late September',
    headline: 'Lantern\nNights',
    subline: 'Harbour Street Food Fair',
    details: 'Sat 26 Sep\nFrom 6 PM\nForty stalls',
    footer: 'lanternnights.example  ·  free entry all weekend',
    paper: '#f5f1ec',
    ink: '#2b1e3a',
    accent: '#e0584a',
    accentSoft: '#f6c8bc',
  },
  cloth: {
    color: '#0f5c5e',
    sheen: 0.35,
    weight: 0.13,
    softness: 0.75,
    wind: 3.5,
    gusts: 0.65,
    downwash: 14,
    flutter: 1,
  },
  helicopter: {
    body: '#f2f3f5',
    trim: '#2b1e3a',
    stripe: '#0f6b6d',
    glass: '#16202c',
    scale: 1,
    rotorSpeed: 1,
    cables: '#2b2d31',
  },
  sky: {
    zenith: '#4c5fa6',
    horizon: '#f0c6a4',
    turbidity: 1.8,
    rayleigh: 2.9,
    exposure: 0.62,
    cloudShade: '#b9a2a6',
    clouds: 0.3,
    cloudDrift: 1,
    sun: 3.2,
    sunColor: '#fff1dc',
    sunSide: 0.42,
    sunHeight: 0.6,
    haze: '#ecd3bf',
    fog: 0.0021,
  },
  ground: {
    showRoad: true,
    traffic: true,
    road: '#4b4f55',
    verge: '#4f7a3a',
    trees: '#ffffff',
    skyline: '#cdbfba',
    frame: '#3a3f47',
    pole: '#9ea4ab',
  },
  camera: {
    fieldOfView: 50,
    height: 19,
    distance: 50,
    pushIn: 0.06,
    drift: 0.4,
    boardAngle: 0.5,
  },
  timing: {
    liftStart: 110,
    climb: 96,
    cardStart: 282,
    cardFrames: 30,
  },
  outro: {
    showCard: true,
    cardWidth: 0.74,
    blur: 22,
    dim: 0.5,
    shadow: 0.55,
  },
  motion: defaultMotion({ exit: 'none', exitFrames: 0, float: 0 }),
};

export const helicopterUnveilMeta: ClipMeta = {
  ...promoMeta,
  width: 1080,
  height: 1920,
  id: 'helicopter-unveil',
  title: 'Helicopter Unveil',
  description:
    'A helicopter lifts a teal cloth off a highway billboard, the cloth streams away into the sky and the poster lands full screen over a blurred backdrop.',
  category: 'ad-reveals',
  tags: ['3d', 'reveal', 'billboard', 'cloth', 'cover', 'camera-move'],
  theme: 'light',
  source: 'src/compositions/scenes/ad-reveals/helicopter-unveil.tsx',
  durationInFrames: 390,
  posterFrame: 60,
};
