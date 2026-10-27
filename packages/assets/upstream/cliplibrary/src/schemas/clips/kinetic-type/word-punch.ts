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

export const WORD_PUNCH_STYLES = ['punch', 'type'] as const;

export const wordPunchSchema = z.object({
  text: z.string(),
  highlight: z.string(),
  style: z.enum(WORD_PUNCH_STYLES),
  size: zSlider({ min: 120, max: 800 }),
  shrink: zSlider({ min: 0, max: 0.6, step: 0.01 }),
  weight: zSlider(FONT_WEIGHT),
  tracking: zSlider({ min: -0.06, max: 0.4, step: 0.005 }),
  accentPunctuation: z.boolean(),
  delay: zSlider({ min: 0, max: 90 }),
  wordFrames: zSlider({ min: 8, max: 60 }),
  swapFrames: zSlider({ min: 1, max: 16 }),
  typeFrames: zSlider({ min: 1, max: 6, step: 0.25 }),
  punch: zSlider({ min: 0, max: 0.4, step: 0.01 }),
  rise: zSlider({ min: 0, max: 1, step: 0.01 }),
  placement: placementSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type WordPunchProps = z.infer<typeof wordPunchSchema>;

export const wordPunchDefaults: WordPunchProps = {
  text: 'Turn. Plans. Into.',
  highlight: '',
  style: 'type',
  size: 760,
  shrink: 0.3,
  weight: 700,
  tracking: -0.02,
  accentPunctuation: false,
  delay: 4,
  wordFrames: 16,
  swapFrames: 10,
  typeFrames: 2.25,
  punch: 0.14,
  rise: 0.32,
  placement: placement('center', 'center', { width: 1700 }),
  look: kineticLook(),
  motion: defaultMotion({ exit: 'fade', exitFrames: 12 }),
};

export const wordPunchMeta: ClipMeta = {
  ...promoMeta,
  id: 'word-punch',
  title: 'Word Punch',
  description:
    'One huge word at a time types on behind its full stop and deletes from the front, or punches in.',
  category: 'kinetic-type',
  tags: ['kinetic', 'headline', 'typing', 'text'],
  theme: 'dark',
  source: 'src/compositions/scenes/kinetic-type/word-punch.tsx',
  durationInFrames: 96,
  posterFrame: 60,
};
