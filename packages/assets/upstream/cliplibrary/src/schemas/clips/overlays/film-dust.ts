import { z } from 'zod';
import { zColor, zSlider } from '../../control';
import {
  defaultMotion,
  kineticLook,
  lookSchema,
  motionSchema,
  promoMeta,
} from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const filmDustSchema = z.object({
  color: zColor(),
  dust: zSlider({ min: 0, max: 1, step: 0.01 }),
  hairs: zSlider({ min: 0, max: 1, step: 0.01 }),
  scratches: zSlider({ min: 0, max: 1, step: 0.01 }),
  vignette: zSlider({ min: 0, max: 1, step: 0.01 }),
  gateFrames: zSlider({ min: 1, max: 6 }),
  seed: zSlider({ min: 0, max: 999 }),
  look: lookSchema,
  motion: motionSchema,
});

export type FilmDustProps = z.infer<typeof filmDustSchema>;

export const filmDustDefaults: FilmDustProps = {
  color: '#b08a3e',
  dust: 0.7,
  hairs: 0.9,
  scratches: 0.1,
  vignette: 0,
  gateFrames: 1,
  seed: 11,
  look: kineticLook(),
  motion: defaultMotion({ exit: 'none', exitFrames: 0 }),
};

export const filmDustMeta: ClipMeta = {
  ...promoMeta,
  id: 'film-dust',
  title: 'Film Dust',
  description:
    'Flickering dust, stray hairs, faint scratches and a vignette like a worn film print.',
  category: 'overlays',
  tags: ['film', 'grain', 'texture', 'overlay'],
  theme: 'dark',
  source: 'src/compositions/scenes/overlays/film-dust.tsx',
  durationInFrames: 150,
  posterFrame: 75,
};
