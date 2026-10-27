import { z } from 'zod';
import { zColor, zSlider } from '../../control';
import {
  defaultMotion,
  FONT_WEIGHT,
  lightLook,
  lookSchema,
  motionSchema,
  promoMeta,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const brandRevealSchema = z.object({
  eyebrow: z.string(),
  brandName: z.string(),
  tagline: z.string(),
  showMark: z.boolean(),
  markColor: zColor(),
  wordmarkSize: zSlider({ min: 80, max: 320 }),
  wordmarkWeight: zSlider(FONT_WEIGHT),
  wordmarkTracking: zSlider({ min: -0.12, max: 0.3, step: 0.005 }),
  markScale: zSlider({ min: 0.3, max: 1.5, step: 0.01 }),
  markGap: zSlider({ min: 0, max: 0.6, step: 0.01 }),
  markStart: zSlider({ min: 0, max: 60 }),
  eyebrowSize: zSlider({ min: 12, max: 60 }),
  taglineSize: zSlider({ min: 16, max: 80 }),
  eyebrowGap: zSlider({ min: 0, max: 120 }),
  taglineGap: zSlider({ min: 0, max: 120 }),
  wordmarkStart: zSlider({ min: 0, max: 60 }),
  letterStagger: zSlider({ min: 0, max: 12 }),
  letterRise: zSlider({ min: 4, max: 60 }),
  taglineStart: zSlider({ min: 0, max: 110 }),
  look: lookSchema,
  motion: motionSchema,
});

export type BrandRevealProps = z.infer<typeof brandRevealSchema>;

export const brandRevealDefaults: BrandRevealProps = {
  eyebrow: 'Introducing',
  brandName: 'Halo',
  tagline: 'Everything you need, in one calm app.',
  showMark: true,
  markColor: '#2f5bff',
  wordmarkSize: 200,
  wordmarkWeight: 600,
  wordmarkTracking: -0.055,
  markScale: 0.78,
  markGap: 0.16,
  markStart: 4,
  eyebrowSize: 22,
  taglineSize: 34,
  eyebrowGap: 34,
  taglineGap: 34,
  wordmarkStart: 14,
  letterStagger: 3,
  letterRise: 26,
  taglineStart: 44,
  look: lightLook(),
  motion: defaultMotion({ exit: 'blur', exitFrames: 18 }),
};

export const brandRevealMeta: ClipMeta = {
  ...promoMeta,
  id: 'brand-reveal',
  title: 'Brand Reveal',
  description:
    'A ring mark draws itself, the wordmark rises letter by letter and a tagline follows.',
  category: 'brand',
  tags: ['logo', 'reveal', 'draw', 'minimal'],
  theme: 'light',
  source: 'src/compositions/scenes/brand/brand-reveal.tsx',
  durationInFrames: 120,
  posterFrame: 80,
};
