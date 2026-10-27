import { z } from 'zod';
import { zSlider } from '../../control';
import {
  defaultMotion,
  FONT_WEIGHT,
  kineticLook,
  lookSchema,
  motionSchema,
  placement,
  placementSchema,
  promoMeta,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const letterFlipSchema = z.object({
  text: z.string(),
  size: zSlider({ min: 80, max: 480 }),
  weight: zSlider(FONT_WEIGHT),
  tracking: zSlider({ min: -0.08, max: 0.3, step: 0.005 }),
  startAngle: zSlider({ min: 30, max: 180 }),
  slide: zSlider({ min: 0, max: 2, step: 0.05 }),
  delay: zSlider({ min: 0, max: 90 }),
  stagger: zSlider({ min: 0, max: 12 }),
  flipFrames: zSlider({ min: 10, max: 60 }),
  depth: zSlider({ min: 1, max: 8, step: 0.1 }),
  highlight: zSlider({ min: 0, max: 20 }),
  glowOut: zSlider({ min: 0, max: 1, step: 0.01 }),
  placement: placementSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type LetterFlipProps = z.infer<typeof letterFlipSchema>;

export const letterFlipDefaults: LetterFlipProps = {
  text: 'SAVINGS',
  size: 360,
  weight: 700,
  tracking: -0.02,
  startAngle: 180,
  slide: 0.15,
  delay: 4,
  stagger: 3,
  flipFrames: 12,
  depth: 3,
  highlight: 0,
  glowOut: 1,
  placement: placement('center', 'center', { width: 1700 }),
  look: kineticLook(),
  motion: defaultMotion({ exit: 'blur', exitFrames: 14 }),
};

export const letterFlipMeta: ClipMeta = {
  ...promoMeta,
  id: 'letter-flip',
  title: 'Letter Flip',
  description: 'Each letter swings in from its mirror image and lands in turn.',
  category: 'text-reveals',
  tags: ['flip', 'text', 'logo', 'reveal'],
  theme: 'dark',
  source: 'src/compositions/scenes/text-reveals/letter-flip.tsx',
  durationInFrames: 90,
  posterFrame: 70,
};
