import { z } from 'zod';
import { zMultiline, zSlider } from '../../control';
import { doodleInk, HAND_FONTS, inkSchema, notebookLook } from '../../doodle-common';
import {
  defaultMotion,
  lookSchema,
  motionSchema,
  placement,
  placementSchema,
  promoMeta,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const handwrittenNoteSchema = z.object({
  text: zMultiline(),
  highlight: z.string(),
  font: z.enum(HAND_FONTS),
  size: zSlider({ min: 36, max: 280 }),
  lineHeight: zSlider({ min: 0.7, max: 1.6, step: 0.01 }),
  tilt: zSlider({ min: -15, max: 15, step: 0.5 }),
  placement: placementSchema,
  delay: zSlider({ min: 0, max: 120 }),
  charFrames: zSlider({ min: 2, max: 24 }),
  charStagger: zSlider({ min: 0.5, max: 8, step: 0.5 }),
  wordPause: zSlider({ min: 0, max: 24 }),
  jitter: zSlider({ min: 0, max: 1, step: 0.01 }),
  underline: z.boolean(),
  burst: z.boolean(),
  erase: z.boolean(),
  eraseFrames: zSlider({ min: 4, max: 60 }),
  seed: zSlider({ min: 1, max: 999 }),
  ink: inkSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type HandwrittenNoteProps = z.infer<typeof handwrittenNoteSchema>;

export const handwrittenNoteDefaults: HandwrittenNoteProps = {
  text: 'Plan less.\nFeel more.\nwith Halo',
  highlight: 'Feel',
  font: 'caveat-brush',
  size: 168,
  lineHeight: 0.98,
  tilt: -3,
  placement: placement('center', 'center', { width: 1400 }),
  delay: 0,
  charFrames: 14,
  charStagger: 2,
  wordPause: 2,
  jitter: 0.5,
  underline: true,
  burst: true,
  erase: true,
  eraseFrames: 24,
  seed: 7,
  ink: doodleInk({ width: 7 }),
  look: notebookLook(),
  motion: defaultMotion({ exit: 'none', exitFrames: 0, float: 0.06 }),
};

export const handwrittenNoteMeta: ClipMeta = {
  ...promoMeta,
  id: 'handwritten-note',
  title: 'Handwritten Note',
  description:
    'Marker lettering writes itself on letter by letter, underlines its accent word, then breaks apart.',
  category: 'hand-drawn',
  tags: ['handwriting', 'doodle', 'text', 'headline'],
  theme: 'light',
  source: 'src/compositions/scenes/hand-drawn/handwritten-note.tsx',
  durationInFrames: 180,
  posterFrame: 110,
};
