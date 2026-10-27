import { z } from 'zod';
import { zColor, zSlider } from '../../control';
import {
  clayLook,
  defaultMotion,
  defaultStage,
  lookSchema,
  motionSchema,
  phone,
  phoneSchema,
  promoMeta,
  stageSchema,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const phonePedestalSchema = z.object({
  phone: phoneSchema,
  scale: zSlider({ min: 0.4, max: 2.2, step: 0.01 }),
  lean: zSlider({ min: -1, max: 1, step: 0.01 }),
  yaw: zSlider({ min: -1, max: 1, step: 0.01 }),
  pitch: zSlider({ min: -0.6, max: 0.6, step: 0.01 }),
  rock: zSlider({ min: 0, max: 0.2, step: 0.005 }),
  rockSpeed: zSlider({ min: 0.25, max: 3, step: 0.05 }),
  pedestal: z.object({
    show: z.boolean(),
    color: zColor(),
    size: zSlider({ min: 1, max: 60, step: 0.1 }),
    turn: zSlider({ min: 0, max: 1.57, step: 0.01 }),
    top: zSlider({ min: -4, max: 2, step: 0.05 }),
    bevel: zSlider({ min: 0, max: 0.3, step: 0.005 }),
    roughness: zSlider({ min: 0.2, max: 1, step: 0.01 }),
  }),
  entry: z.object({
    drop: zSlider({ min: -10, max: 10, step: 0.1 }),
    frames: zSlider({ min: 10, max: 120 }),
    cameraTilt: zSlider({ min: 0, max: 3, step: 0.05 }),
  }),
  camera: z.object({
    height: zSlider({ min: -6, max: 2, step: 0.05 }),
    rise: zSlider({ min: 0, max: 2, step: 0.01 }),
    orbit: zSlider({ min: -1, max: 1, step: 0.01 }),
    aimY: zSlider({ min: -3, max: 3, step: 0.05 }),
    lightX: zSlider({ min: -20, max: 20, step: 0.5 }),
    lightY: zSlider({ min: -20, max: 20, step: 0.5 }),
    lightZ: zSlider({ min: 1, max: 30, step: 0.5 }),
  }),
  look: lookSchema,
  stage: stageSchema,
  motion: motionSchema,
});

export type PhonePedestalProps = z.infer<typeof phonePedestalSchema>;

export const phonePedestalDefaults: PhonePedestalProps = {
  phone: phone('ride-unlock', 'natural'),
  scale: 1.05,
  lean: 0.5,
  yaw: -0.22,
  pitch: -0.06,
  rock: 0.035,
  rockSpeed: 1,
  pedestal: {
    show: true,
    color: '#9a7a0c',
    size: 34,
    turn: 0.785,
    top: -1.7,
    bevel: 0.06,
    roughness: 0.78,
  },
  entry: { drop: 4.5, frames: 54, cameraTilt: 1.2 },
  camera: {
    height: -4.3,
    rise: 0.8,
    orbit: 0.45,
    aimY: 0.1,
    lightX: -5,
    lightY: 8,
    lightZ: 11,
  },
  look: { ...clayLook(), background: '#aeaeab', backgroundEdge: '#929290' },
  stage: defaultStage({ keyLight: 2.1, shadow: 0.22, shadowSoftness: 16, cameraDrift: 0.3 }),
  motion: defaultMotion({ exit: 'fade', exitFrames: 14 }),
};

export const phonePedestalMeta: ClipMeta = {
  ...promoMeta,
  id: 'phone-pedestal',
  title: 'Phone Pedestal',
  description:
    'A phone drops in and balances, tilted, on the peak of a big clay block while the camera rises.',
  category: 'phone-reveals',
  tags: ['3d', 'phone', 'drop', 'camera-move'],
  theme: 'light',
  source: 'src/compositions/scenes/phone-reveals/phone-pedestal.tsx',
  durationInFrames: 165,
  posterFrame: 110,
};
