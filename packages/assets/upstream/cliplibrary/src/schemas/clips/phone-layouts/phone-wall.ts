import { z } from 'zod';
import { zSlider } from '../../control';
import {
  defaultMotion,
  defaultStage,
  lightLook,
  lookSchema,
  motionSchema,
  PHONE_FINISHES,
  PROMO_SCREEN_IDS,
  promoMeta,
  stageSchema,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const phoneWallSchema = z.object({
  screens: z.array(z.enum(PROMO_SCREEN_IDS)).min(1).max(PROMO_SCREEN_IDS.length),
  finish: z.enum(PHONE_FINISHES),
  rows: zSlider({ min: 1, max: 4 }),
  columns: zSlider({ min: 3, max: 11 }),
  columnGap: zSlider({ min: 1.5, max: 4, step: 0.05 }),
  rowGap: zSlider({ min: 3.05, max: 7, step: 0.05 }),
  tilt: zSlider({ min: -1, max: 0.4, step: 0.01 }),
  turn: zSlider({ min: -0.8, max: 0.8, step: 0.01 }),
  twist: zSlider({ min: -0.8, max: 0.8, step: 0.01 }),
  drift: zSlider({ min: 0, max: 4, step: 0.05 }),
  arriveFrames: zSlider({ min: 10, max: 150 }),
  stagger: zSlider({ min: 0, max: 8, step: 0.1 }),
  arriveDrop: zSlider({ min: 0, max: 6, step: 0.1 }),
  look: lookSchema,
  stage: stageSchema,
  motion: motionSchema,
});

export type PhoneWallProps = z.infer<typeof phoneWallSchema>;

export const phoneWallDefaults: PhoneWallProps = {
  screens: PROMO_SCREEN_IDS.filter((id) => !id.startsWith('ride-')),
  finish: 'black',
  rows: 3,
  columns: 9,
  columnGap: 1.95,
  rowGap: 3.7,
  tilt: -0.5,
  turn: 0.18,
  twist: 0.3,
  drift: 1.6,
  arriveFrames: 56,
  stagger: 2.5,
  arriveDrop: 2.2,
  look: lightLook(),
  stage: defaultStage({ cameraDistance: 19, shadow: 0.12 }),
  motion: defaultMotion({ exit: 'fade', exitFrames: 18 }),
};

export const phoneWallMeta: ClipMeta = {
  ...promoMeta,
  id: 'phone-wall',
  title: 'Phone Wall',
  description: 'A tilted wall of phones glides in and drifts in alternating rows.',
  category: 'phone-layouts',
  tags: ['3d', 'phone', 'multi-phone', 'grid'],
  theme: 'light',
  source: 'src/compositions/scenes/phone-layouts/phone-wall.tsx',
  durationInFrames: 210,
  posterFrame: 120,
};
