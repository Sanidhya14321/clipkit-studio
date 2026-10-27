import { z } from 'zod';
import { zColor, zSlider } from '../../control';
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

export const outlineMarqueeSchema = z.object({
  text: z.string(),
  label: z.string(),
  rows: zSlider({ min: 1, max: 6 }),
  size: zSlider({ min: 100, max: 480 }),
  rowSpacing: zSlider({ min: 0.8, max: 2, step: 0.05 }),
  weight: zSlider(FONT_WEIGHT),
  tracking: zSlider({ min: -0.05, max: 0.4, step: 0.005 }),
  stroke: zSlider({ min: 0.5, max: 6, step: 0.25 }),
  outlineOpacity: zSlider({ min: 0.05, max: 1, step: 0.01 }),
  outlineColor: zColor(),
  speed: zSlider({ min: 0, max: 8 }),
  reverse: z.boolean(),
  repeats: zSlider({ min: 2, max: 24 }),
  repeatGap: zSlider({ min: 0, max: 2, step: 0.05 }),
  labelSize: zSlider({ min: 24, max: 200 }),
  labelTracking: zSlider({ min: -0.05, max: 0.4, step: 0.005 }),
  labelWeight: zSlider(FONT_WEIGHT),
  delay: zSlider({ min: 0, max: 90 }),
  rowStagger: zSlider({ min: 0, max: 20 }),
  labelDelay: zSlider({ min: 0, max: 90 }),
  placement: placementSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type OutlineMarqueeProps = z.infer<typeof outlineMarqueeSchema>;

export const outlineMarqueeDefaults: OutlineMarqueeProps = {
  text: 'CALM MONEY',
  label: 'Calm Money',
  rows: 2,
  size: 420,
  rowSpacing: 1.3,
  weight: 700,
  tracking: -0.02,
  stroke: 2,
  outlineOpacity: 0.8,
  outlineColor: '#f1f1f1',
  speed: 2,
  reverse: false,
  repeats: 8,
  repeatGap: 0.5,
  labelSize: 60,
  labelTracking: 0.12,
  labelWeight: 700,
  delay: 4,
  rowStagger: 4,
  labelDelay: 8,
  placement: placement('center', 'center', { width: 1400 }),
  look: kineticLook(),
  motion: defaultMotion({ exit: 'fade', exitFrames: 14 }),
};

export const outlineMarqueeMeta: ClipMeta = {
  ...promoMeta,
  id: 'outline-marquee',
  title: 'Outline Marquee',
  description: 'Rows of huge outlined type glide in opposite directions behind a solid bold label.',
  category: 'kinetic-type',
  tags: ['kinetic', 'headline', 'marquee', 'outline', 'text'],
  theme: 'dark',
  source: 'src/compositions/scenes/kinetic-type/outline-marquee.tsx',
  durationInFrames: 120,
  posterFrame: 70,
};
