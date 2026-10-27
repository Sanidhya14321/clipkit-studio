import { z } from 'zod';
import { zColor, zImage, zSlider } from '../../control';
import {
  clayLook,
  defaultMotion,
  defaultStage,
  lookSchema,
  motionSchema,
  promoMeta,
  stageSchema,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const WORDMARK_WEIGHTS = ['500', '600', '700', '800'] as const;

export const APP_ICON_FLIP_FROM = ['right', 'left', 'top', 'bottom'] as const;

export const appIconRevealSchema = z.object({
  brandName: z.string(),
  icon: z.object({
    color: zColor(),
    faceColor: zColor(),
    art: zImage(),
    artTint: zColor(),
    artScale: zSlider({ min: 0.2, max: 1.6, step: 0.01 }),
    size: zSlider({ min: 0.3, max: 3, step: 0.01 }),
    depth: zSlider({ min: 0.04, max: 0.6, step: 0.01 }),
    roughness: zSlider({ min: 0.05, max: 1, step: 0.01 }),
    y: zSlider({ min: -3, max: 3, step: 0.01 }),
    z: zSlider({ min: 0, max: 3, step: 0.01 }),
  }),
  reveal: z.object({
    flipAt: zSlider({ min: 0, max: 120 }),
    flipFrames: zSlider({ min: 6, max: 120 }),
    flipFrom: z.enum(APP_ICON_FLIP_FROM),
    spins: zSlider({ min: 0, max: 3 }),
    lift: zSlider({ min: 0, max: 2, step: 0.01 }),
    typeAt: zSlider({ min: 0, max: 200 }),
    letterFrames: zSlider({ min: 1, max: 10 }),
    letterFade: zSlider({ min: 1, max: 30 }),
  }),
  wordmark: z.object({
    color: zColor(),
    height: zSlider({ min: 0.1, max: 1.5, step: 0.01 }),
    weight: z.enum(WORDMARK_WEIGHTS),
    tracking: zSlider({ min: -0.08, max: 0.3, step: 0.005 }),
    gap: zSlider({ min: 0, max: 2, step: 0.01 }),
  }),
  camera: z.object({
    pushIn: zSlider({ min: 0, max: 0.5, step: 0.01 }),
    pushFrames: zSlider({ min: 30, max: 600, step: 1 }),
    orbitAt: zSlider({ min: 0, max: 400, step: 1 }),
    orbitFrames: zSlider({ min: 10, max: 160, step: 1 }),
    orbitAngle: zSlider({ min: -1.4, max: 1.4, step: 0.01 }),
    orbitPeak: zSlider({ min: 0.1, max: 0.9, step: 0.01 }),
    orbitPivot: zSlider({ min: -3, max: 3, step: 0.01 }),
    lightX: zSlider({ min: -20, max: 20, step: 0.5 }),
    lightY: zSlider({ min: -20, max: 20, step: 0.5 }),
    lightZ: zSlider({ min: 1, max: 30, step: 0.5 }),
    backdropSweep: zSlider({ min: 0, max: 40 }),
  }),
  look: lookSchema,
  stage: stageSchema,
  motion: motionSchema,
});

export type AppIconRevealProps = z.infer<typeof appIconRevealSchema>;

export const appIconRevealDefaults: AppIconRevealProps = {
  brandName: 'Glide Scooters',
  icon: {
    color: '#b1b1af',
    faceColor: '#b4b4b2',
    art: '',
    artTint: '#ffffff',
    artScale: 0.86,
    size: 1.25,
    depth: 0.26,
    roughness: 0.8,
    y: 0.18,
    z: 0,
  },
  reveal: {
    flipAt: 12,
    flipFrames: 16,
    flipFrom: 'right',
    spins: 0,
    lift: 0.25,
    typeAt: 16,
    letterFrames: 2,
    letterFade: 6,
  },
  wordmark: { color: '#2e3744', height: 0.42, weight: '600', tracking: 0.01, gap: 0.2 },
  camera: {
    pushIn: 0.12,
    pushFrames: 240,
    orbitAt: 400,
    orbitFrames: 66,
    orbitAngle: 0,
    orbitPeak: 0.5,
    orbitPivot: 0,
    lightX: -7,
    lightY: 9,
    lightZ: 9,
    backdropSweep: 14,
  },
  look: clayLook(),
  stage: defaultStage({ keyLight: 1.9, shadow: 0.2, shadowSoftness: 14, cameraDrift: 0.25 }),
  motion: defaultMotion({ exit: 'fade', exitFrames: 12, float: 0 }),
};

export const appIconRevealMeta: ClipMeta = {
  ...promoMeta,
  id: 'app-icon-reveal',
  title: 'App Icon Reveal',
  description:
    'A blank clay tile flips over to reveal the app icon while the name types on beneath it.',
  category: 'brand',
  tags: ['3d', 'logo', 'icon', 'flip', 'reveal'],
  theme: 'light',
  source: 'src/compositions/scenes/brand/app-icon-reveal.tsx',
  durationInFrames: 120,
  posterFrame: 96,
};
