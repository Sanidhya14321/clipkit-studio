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

export const echoStackSchema = z.object({
  text: z.string(),
  rows: zSlider({ min: 2, max: 12 }),
  accentRow: zSlider({ min: 0, max: 12 }),
  size: zSlider({ min: 24, max: 180 }),
  weight: zSlider(FONT_WEIGHT),
  tracking: zSlider({ min: -0.1, max: 0.3, step: 0.005 }),
  lineHeight: zSlider({ min: 0.8, max: 1.6, step: 0.02 }),
  delay: zSlider({ min: 0, max: 90 }),
  rowStagger: zSlider({ min: 1, max: 12 }),
  rowFrames: zSlider({ min: 2, max: 30 }),
  travel: zSlider({ min: 0, max: 3, step: 0.05 }),
  exitAt: zSlider({ min: 20, max: 600 }),
  placement: placementSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type EchoStackProps = z.infer<typeof echoStackSchema>;

export const echoStackDefaults: EchoStackProps = {
  text: 'HALO SAVES',
  rows: 7,
  accentRow: 0,
  size: 112,
  weight: 700,
  tracking: -0.01,
  lineHeight: 1.24,
  delay: 4,
  rowStagger: 2,
  rowFrames: 8,
  travel: 1,
  exitAt: 76,
  placement: placement('center', 'center', { width: 1200 }),
  look: kineticLook(),
  motion: defaultMotion({ exit: 'fade', exitFrames: 12 }),
};

export const echoStackMeta: ClipMeta = {
  ...promoMeta,
  id: 'echo-stack',
  title: 'Echo Stack',
  description:
    'One phrase repeats down the frame row by row, then the stack rolls away from the top.',
  category: 'kinetic-type',
  tags: ['kinetic', 'text', 'repeat', 'stack'],
  theme: 'dark',
  source: 'src/compositions/scenes/kinetic-type/echo-stack.tsx',
  durationInFrames: 126,
  posterFrame: 60,
};
