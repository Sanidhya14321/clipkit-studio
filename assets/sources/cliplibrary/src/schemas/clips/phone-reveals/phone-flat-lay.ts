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
  position,
  positionSchema,
  promoMeta,
  stageSchema,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const FLAT_LAY_EASES = ['settle', 'smooth', 'linear'] as const;

export type FlatLayEase = (typeof FLAT_LAY_EASES)[number];

export const phoneFlatLaySchema = z.object({
  phone: phoneSchema,
  position: positionSchema,
  scale: zSlider({ min: 0.4, max: 2.2, step: 0.01 }),
  phoneTurn: zSlider({ min: -3.2, max: 3.2, step: 0.01 }),
  surface: z.object({
    show: z.boolean(),
    color: zColor(),
    count: zSlider({ min: 2, max: 30, step: 1 }),
    width: zSlider({ min: 0.3, max: 4, step: 0.05 }),
    length: zSlider({ min: 10, max: 80, step: 1 }),
    thickness: zSlider({ min: 0.05, max: 1, step: 0.01 }),
    step: zSlider({ min: 0, max: 0.4, step: 0.01 }),
    amplitude: zSlider({ min: 0, max: 1.2, step: 0.01 }),
    waves: zSlider({ min: 0.2, max: 6, step: 0.05 }),
    curl: zSlider({ min: 0, max: 2, step: 0.01 }),
    angle: zSlider({ min: -1.57, max: 1.57, step: 0.01 }),
    roughness: zSlider({ min: 0.2, max: 1, step: 0.01 }),
  }),
  camera: z.object({
    tiltFrom: zSlider({ min: 0, max: 1.2, step: 0.01 }),
    tiltTo: zSlider({ min: 0, max: 1.2, step: 0.01 }),
    turnFrom: zSlider({ min: -1.57, max: 1.57, step: 0.01 }),
    turnTo: zSlider({ min: -1.57, max: 1.57, step: 0.01 }),
    frames: zSlider({ min: 20, max: 400 }),
    delay: zSlider({ min: 0, max: 300 }),
    ease: z.enum(FLAT_LAY_EASES),
    lightX: zSlider({ min: -20, max: 20, step: 0.5 }),
    lightY: zSlider({ min: -20, max: 20, step: 0.5 }),
    lightZ: zSlider({ min: 1, max: 30, step: 0.5 }),
  }),
  look: lookSchema,
  stage: stageSchema,
  motion: motionSchema,
});

export type PhoneFlatLayProps = z.infer<typeof phoneFlatLaySchema>;

export const phoneFlatLayDefaults: PhoneFlatLayProps = {
  phone: phone('ride-nearby', 'natural'),
  position: position(0, 0.1, 0),
  scale: 1.02,
  phoneTurn: 0,
  surface: {
    show: true,
    color: '#8a6c08',
    count: 14,
    width: 2,
    length: 44,
    thickness: 0.18,
    step: 0.16,
    amplitude: 0.8,
    waves: 3.5,
    curl: 0.5,
    angle: -0.2,
    roughness: 0.8,
  },
  camera: {
    tiltFrom: 0.72,
    tiltTo: 0.16,
    turnFrom: 0.5,
    turnTo: 0,
    frames: 200,
    delay: 0,
    ease: 'settle',
    lightX: -6,
    lightY: 7,
    lightZ: 12,
  },
  look: clayLook(),
  stage: defaultStage({ keyLight: 2.2, shadow: 0.2, shadowSoftness: 14, cameraDrift: 0.3 }),
  motion: defaultMotion({ exit: 'fade', exitFrames: 14, float: 0 }),
};

export const phoneFlatLayMeta: ClipMeta = {
  ...promoMeta,
  id: 'phone-flat-lay',
  title: 'Phone Flat Lay',
  description:
    'A phone lies on a field of wavy clay ribbons while the camera turns from oblique to top-down.',
  category: 'phone-reveals',
  tags: ['3d', 'phone', 'top-down', 'camera-move'],
  theme: 'light',
  source: 'src/compositions/scenes/phone-reveals/phone-flat-lay.tsx',
  durationInFrames: 165,
  posterFrame: 120,
};
