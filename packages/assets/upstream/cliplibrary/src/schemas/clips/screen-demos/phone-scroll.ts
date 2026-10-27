import { z } from 'zod';
import { zSlider } from '../../control';
import {
  defaultMotion,
  defaultStage,
  lightLook,
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

export const PHONE_SCROLL_EASES = ['smooth', 'linear', 'easeOut', 'easeIn'] as const;

export const phoneScrollSchema = z.object({
  phone: phoneSchema,
  position: positionSchema,
  scale: zSlider({ min: 0.4, max: 2.6, step: 0.01 }),
  lean: zSlider({ min: 0, max: 0.6, step: 0.01 }),
  tilt: zSlider({ min: -0.4, max: 0.6, step: 0.01 }),
  roll: zSlider({ min: -0.3, max: 0.3, step: 0.01 }),
  sway: zSlider({ min: 0, max: 0.2, step: 0.005 }),
  scrollAmount: zSlider({ min: 0, max: 1, step: 0.01 }),
  scrollStart: zSlider({ min: 0, max: 120 }),
  scrollHold: zSlider({ min: 0, max: 120 }),
  scrollEase: z.enum(PHONE_SCROLL_EASES),
  arriveFrames: zSlider({ min: 0, max: 120 }),
  look: lookSchema,
  stage: stageSchema,
  motion: motionSchema,
});

export type PhoneScrollProps = z.infer<typeof phoneScrollSchema>;

export const phoneScrollDefaults: PhoneScrollProps = {
  phone: phone('feed', 'white'),
  position: position(0, -0.05),
  scale: 1.42,
  lean: 0.34,
  tilt: 0.16,
  roll: 0.08,
  sway: 0.03,
  scrollAmount: 1,
  scrollStart: 16,
  scrollHold: 0,
  scrollEase: 'smooth',
  arriveFrames: 60,
  look: lightLook(),
  stage: defaultStage(),
  motion: defaultMotion({ exit: 'fade', exitFrames: 16 }),
};

export const phoneScrollMeta: ClipMeta = {
  ...promoMeta,
  id: 'phone-scroll',
  title: 'Phone Scroll',
  description: 'A leaning phone scrolls smoothly through a long feed screen.',
  category: 'screen-demos',
  tags: ['3d', 'phone', 'screen', 'scroll'],
  theme: 'light',
  source: 'src/compositions/scenes/screen-demos/phone-scroll.tsx',
  durationInFrames: 180,
  posterFrame: 120,
};
