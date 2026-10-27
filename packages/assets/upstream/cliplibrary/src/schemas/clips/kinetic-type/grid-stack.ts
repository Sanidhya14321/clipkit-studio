import { z } from 'zod';
import { zMultiline, zSlider } from '../../control';
import { defaultMotion, lookSchema, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';
import {
  bonePaperLook,
  HEAVY_WEIGHT,
  TYPE_FACES,
  typeCamera,
  typeCameraSchema,
  typeLens,
  typeLensSchema,
  warpGrid,
  warpGridSchema,
} from '../../type3d-common';

export const gridStackSchema = z.object({
  text: zMultiline(),
  highlight: z.string(),
  face: z.enum(TYPE_FACES),
  weight: zSlider(HEAVY_WEIGHT),
  bigSize: zSlider({ min: 0.5, max: 5, step: 0.05 }),
  smallSize: zSlider({ min: 0.3, max: 3, step: 0.05 }),
  maxWidth: zSlider({ min: 3, max: 16, step: 0.1 }),
  gap: zSlider({ min: 0, max: 0.6, step: 0.01 }),
  offset: zSlider({ min: 0, max: 0.5, step: 0.01 }),
  bend: zSlider({ min: -1.5, max: 1.5, step: 0.01 }),
  delay: zSlider({ min: 0, max: 60 }),
  wordGap: zSlider({ min: 3, max: 30 }),
  arriveFrames: zSlider({ min: 3, max: 30 }),
  settleFrames: zSlider({ min: 3, max: 30 }),
  holdFrames: zSlider({ min: 0, max: 90 }),
  exitFrames: zSlider({ min: 3, max: 30 }),
  swing: zSlider({ min: 0, max: 45, step: 0.5 }),
  tilt: zSlider({ min: 0, max: 20, step: 0.5 }),
  gridOpacity: zSlider({ min: 0, max: 1, step: 0.01 }),
  grid: warpGridSchema,
  camera: typeCameraSchema,
  lens: typeLensSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type GridStackProps = z.infer<typeof gridStackSchema>;

export const gridStackDefaults: GridStackProps = {
  text: 'shape the story\nwords with weight\nkeep it sharp\nturn heads fast\nnever stand still',
  highlight: 'story weight sharp heads still',
  face: 'montserrat',
  weight: 800,
  bigSize: 2.15,
  smallSize: 0.95,
  maxWidth: 10,
  gap: 0,
  offset: 0.09,
  bend: 0.35,
  delay: 2,
  wordGap: 9,
  arriveFrames: 10,
  settleFrames: 9,
  holdFrames: 16,
  exitFrames: 8,
  swing: 16,
  tilt: 6,
  gridOpacity: 1,
  grid: warpGrid({ color: '#d9d1bd', spacing: 0.75, bulge: 1.2, lineWidth: 0.018, depth: -3 }),
  camera: typeCamera({ drift: 0.8 }),
  lens: typeLens({ distortion: 0.45, fringe: 0.005, shift: 2, shutter: 1.5, samples: 12 }),
  look: bonePaperLook(),
  motion: defaultMotion({ exit: 'none', exitFrames: 8 }),
};

export const gridStackMeta: ClipMeta = {
  ...promoMeta,
  id: 'grid-stack',
  title: 'Grid Stack',
  description:
    'Phrases build word by word into a punchy stack over a fisheye grid, with heavy motion blur and a swinging camera.',
  category: 'kinetic-type',
  tags: ['kinetic', 'headline', 'stack', '3d', 'grid'],
  theme: 'light',
  source: 'src/compositions/scenes/kinetic-type/grid-stack.tsx',
  durationInFrames: 250,
  posterFrame: 175,
};
