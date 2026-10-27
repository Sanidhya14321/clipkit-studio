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

export const TYPEWRITER_INTROS = ['none', 'block', 'typed', 'cut'] as const;

export const TYPEWRITER_OUTROS = ['none', 'select', 'erase'] as const;

export const typewriterSchema = z.object({
  text: z.string(),
  size: zSlider({ min: 24, max: 240 }),
  weight: zSlider(FONT_WEIGHT),
  tracking: zSlider({ min: -0.06, max: 0.4, step: 0.005 }),
  intro: z.enum(TYPEWRITER_INTROS),
  outro: z.enum(TYPEWRITER_OUTROS),
  delay: zSlider({ min: 0, max: 90 }),
  charFrames: zSlider({ min: 0.25, max: 6, step: 0.25 }),
  zoom: zSlider({ min: 1, max: 4, step: 0.05 }),
  caretHold: zSlider({ min: 0, max: 60 }),
  outroAt: zSlider({ min: 10, max: 600 }),
  showCaret: z.boolean(),
  caretPulse: zSlider({ min: 8, max: 60 }),
  caretColor: zColor(),
  selectionColor: zColor(),
  placement: placementSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type TypewriterProps = z.infer<typeof typewriterSchema>;

export const typewriterDefaults: TypewriterProps = {
  text: 'Smarter Budgets, Calmer Days',
  size: 72,
  weight: 600,
  tracking: -0.02,
  intro: 'block',
  outro: 'select',
  delay: 4,
  charFrames: 0.5,
  caretHold: 16,
  zoom: 1,
  outroAt: 92,
  showCaret: true,
  caretPulse: 24,
  caretColor: '#f1f1f1',
  selectionColor: '#f1f1f1',
  placement: placement('center', 'center', { width: 1600 }),
  look: kineticLook(),
  motion: defaultMotion({ exit: 'fade', exitFrames: 12 }),
};

export const typewriterMeta: ClipMeta = {
  ...promoMeta,
  id: 'typewriter',
  title: 'Typewriter',
  description:
    'A block shrinks into a caret, the line types itself out, then a selection sweeps it away.',
  category: 'text-reveals',
  tags: ['typing', 'text', 'caret'],
  theme: 'dark',
  source: 'src/compositions/scenes/text-reveals/typewriter.tsx',
  durationInFrames: 150,
  posterFrame: 80,
};
