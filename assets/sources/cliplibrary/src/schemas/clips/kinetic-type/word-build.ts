import { z } from 'zod';
import { zColor, zMultiline, zSlider } from '../../control';
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

export const WORD_BUILD_ENTRIES = [
  'scroll',
  'slide',
  'zoom',
  'shrink',
  'step',
  'tilt',
  'pop',
  'echo',
] as const;

export const WORD_BUILD_EXITS = ['cut', 'fall', 'squeeze'] as const;

export const SENTENCE_ENTRIES = ['rise', 'wave', 'type'] as const;

export const TRAVEL_DIRECTIONS = ['left', 'right'] as const;

export const wordBuildSchema = z.object({
  text: zMultiline(),
  highlight: z.string(),
  entry: z.enum(WORD_BUILD_ENTRIES),
  direction: z.enum(TRAVEL_DIRECTIONS),
  bigWords: zSlider({ min: 0, max: 8 }),
  bigSize: zSlider({ min: 80, max: 800 }),
  bigWeight: zSlider(FONT_WEIGHT),
  bigColor: zColor(),
  sizeRamp: zSlider({ min: 0.2, max: 1, step: 0.01 }),
  zoomFrom: zSlider({ min: 1, max: 4, step: 0.05 }),
  bigExit: z.enum(WORD_BUILD_EXITS),
  size: zSlider({ min: 28, max: 200 }),
  weight: zSlider(FONT_WEIGHT),
  lineHeight: zSlider({ min: 0.8, max: 1.6, step: 0.02 }),
  sentenceEntry: z.enum(SENTENCE_ENTRIES),
  buildFrom: z.enum(TRAVEL_DIRECTIONS),
  lineDelay: zSlider({ min: 0, max: 60 }),
  buildFrames: zSlider({ min: 1, max: 12 }),
  walkHighlight: z.boolean(),
  delay: zSlider({ min: 0, max: 90 }),
  wordFrames: zSlider({ min: 4, max: 40 }),
  echoes: zSlider({ min: 0, max: 8 }),
  exitByWord: z.boolean(),
  exitStagger: zSlider({ min: 0, max: 12 }),
  placement: placementSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type WordBuildProps = z.infer<typeof wordBuildSchema>;

export const wordBuildDefaults: WordBuildProps = {
  text: 'Halo Money For Modern Savers',
  highlight: '',
  entry: 'scroll',
  direction: 'left',
  bigWords: 2,
  bigSize: 380,
  bigWeight: 700,
  bigColor: '#f1f1f1',
  sizeRamp: 0.6,
  zoomFrom: 2.4,
  bigExit: 'cut',
  size: 72,
  weight: 700,
  lineHeight: 1.1,
  sentenceEntry: 'rise',
  buildFrom: 'left',
  lineDelay: 0,
  buildFrames: 3,
  walkHighlight: true,
  delay: 2,
  wordFrames: 11,
  echoes: 4,
  exitByWord: true,
  exitStagger: 3,
  placement: placement('center', 'center', { width: 1700 }),
  look: kineticLook(),
  motion: defaultMotion({ exit: 'fade', exitFrames: 6 }),
};

export const wordBuildMeta: ClipMeta = {
  ...promoMeta,
  id: 'word-build',
  title: 'Word Build',
  description:
    'Huge words scroll, sweep, zoom or step through the frame, then a line builds word by word with a walking accent.',
  category: 'kinetic-type',
  tags: ['kinetic', 'headline', 'text', 'zoom'],
  theme: 'dark',
  source: 'src/compositions/scenes/kinetic-type/word-build.tsx',
  durationInFrames: 90,
  posterFrame: 60,
};
