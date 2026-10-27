import { z } from 'zod';
import { zImage, zSlider } from '../../control';
import {
  defaultMotion,
  defaultStage,
  lightLook,
  lookSchema,
  motionSchema,
  PROMO_SCREEN_IDS,
  phone,
  phoneSchema,
  position,
  positionSchema,
  promoMeta,
  stageSchema,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const TURNAROUND_ENTRIES = ['rise-back', 'turn-in'] as const;

export const phoneTurnaroundSchema = z.object({
  phone: phoneSchema,
  position: positionSchema,
  scale: zSlider({ min: 0.4, max: 2.6, step: 0.01 }),
  entry: z.enum(TURNAROUND_ENTRIES),
  entryYaw: zSlider({ min: -3.2, max: 3.2, step: 0.01 }),
  entryFrames: zSlider({ min: 4, max: 120 }),
  riseFrames: zSlider({ min: 20, max: 150 }),
  riseYaw: zSlider({ min: -2, max: 2, step: 0.01 }),
  riseTilt: zSlider({ min: 0, max: 1.2, step: 0.01 }),
  turnAt: zSlider({ min: 0.05, max: 0.8, step: 0.01 }),
  rise: zSlider({ min: 0, max: 10, step: 0.1 }),
  turnFrames: zSlider({ min: 20, max: 120 }),
  driftYaw: zSlider({ min: -1, max: 1, step: 0.01 }),
  pushIn: zSlider({ min: 1, max: 1.3, step: 0.01 }),
  secondScreen: z.enum(['none', ...PROMO_SCREEN_IDS]),
  secondCustomScreen: zImage(),
  secondTurnAt: zSlider({ min: 0, max: 600 }),
  secondTurnFrames: zSlider({ min: 20, max: 200 }),
  travelX: zSlider({ min: -6, max: 6, step: 0.05 }),
  endYaw: zSlider({ min: -1, max: 1, step: 0.01 }),
  look: lookSchema,
  stage: stageSchema,
  motion: motionSchema,
});

export type PhoneTurnaroundProps = z.infer<typeof phoneTurnaroundSchema>;

export const phoneTurnaroundDefaults: PhoneTurnaroundProps = {
  phone: phone('analytics', 'blue'),
  position: position(0, 0),
  scale: 1.38,
  entry: 'rise-back',
  entryYaw: -1.5,
  entryFrames: 18,
  riseFrames: 66,
  riseYaw: 1.1,
  riseTilt: 0.6,
  turnAt: 0.5,
  rise: 4.5,
  turnFrames: 60,
  driftYaw: 0.47,
  pushIn: 1.05,
  secondScreen: 'none',
  secondCustomScreen: '',
  secondTurnAt: 140,
  secondTurnFrames: 100,
  travelX: 0,
  endYaw: 0.2,
  look: lightLook(),
  stage: defaultStage({ cameraDistance: 13 }),
  motion: defaultMotion({ enter: 'fade', enterFrames: 16, exit: 'fade', exitFrames: 18 }),
};

export const phoneTurnaroundMeta: ClipMeta = {
  ...promoMeta,
  id: 'phone-turnaround',
  title: 'Phone Turnaround',
  description: 'A phone rises showing its camera module, then turns to reveal the screen.',
  category: 'phone-reveals',
  tags: ['3d', 'phone', 'spin', 'reveal'],
  theme: 'light',
  source: 'src/compositions/scenes/phone-reveals/phone-turnaround.tsx',
  durationInFrames: 180,
  posterFrame: 160,
};
