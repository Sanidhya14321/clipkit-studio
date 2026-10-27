import { z } from 'zod';
import { zSlider } from '../../control';
import {
  darkLook,
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
  TEXT_SIDES,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const PAIR_ENTRANCES = ['sides', 'from-behind'] as const;

export const phonePairSchema = z.object({
  phones: z.array(phoneSchema).length(2),
  position: positionSchema,
  scale: zSlider({ min: 0.4, max: 2.2, step: 0.01 }),
  gap: zSlider({ min: 0.4, max: 6.5, step: 0.05 }),
  angle: zSlider({ min: 0, max: 0.9, step: 0.01 }),
  stagger: zSlider({ min: -0.6, max: 0.6, step: 0.01 }),
  entrance: z.enum(PAIR_ENTRANCES),
  arriveFrames: zSlider({ min: 10, max: 120 }),
  arriveSwing: zSlider({ min: 0, max: 1.5, step: 0.01 }),
  behindSide: z.enum(TEXT_SIDES),
  slideDelay: zSlider({ min: 0, max: 160 }),
  slideFrames: zSlider({ min: 10, max: 120 }),
  look: lookSchema,
  stage: stageSchema,
  motion: motionSchema,
});

export type PhonePairProps = z.infer<typeof phonePairSchema>;

export const phonePairDefaults: PhonePairProps = {
  phones: [phone('music', 'black'), phone('map', 'blue')],
  position: position(0, 0, 0),
  scale: 1.18,
  gap: 1.1,
  angle: 0.34,
  stagger: 0.3,
  entrance: 'sides',
  arriveFrames: 66,
  arriveSwing: 0.9,
  behindSide: 'left',
  slideDelay: 20,
  slideFrames: 56,
  look: darkLook(),
  stage: defaultStage({ cameraDistance: 13, keyLight: 1.4, shadow: 0.45 }),
  motion: defaultMotion({ exit: 'fade', exitFrames: 16 }),
};

export const phonePairMeta: ClipMeta = {
  ...promoMeta,
  id: 'phone-pair',
  title: 'Phone Pair',
  description: 'Two phones slide in from opposite sides and settle facing each other.',
  category: 'phone-layouts',
  tags: ['3d', 'phone', 'multi-phone', 'slide'],
  theme: 'dark',
  source: 'src/compositions/scenes/phone-layouts/phone-pair.tsx',
  durationInFrames: 135,
  posterFrame: 100,
};
