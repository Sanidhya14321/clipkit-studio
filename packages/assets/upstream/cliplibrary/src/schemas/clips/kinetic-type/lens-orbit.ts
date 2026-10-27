import { z } from 'zod';
import { zColor, zMultiline, zSlider } from '../../control';
import { defaultMotion, lookSchema, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';
import {
  HEAVY_WEIGHT,
  midnightLook,
  TYPE_FACES,
  typeCamera,
  typeCameraSchema,
  typeLens,
  typeLensSchema,
} from '../../type3d-common';

export const lensOrbitSchema = z.object({
  words: zMultiline(),
  satellites: z.string(),
  highlight: z.string(),
  face: z.enum(TYPE_FACES),
  weight: zSlider(HEAVY_WEIGHT),
  radius: zSlider({ min: 1, max: 4, step: 0.05 }),
  wordHeight: zSlider({ min: 10, max: 60, step: 1 }),
  maxArc: zSlider({ min: 40, max: 150, step: 1 }),
  satelliteScale: zSlider({ min: 0.1, max: 0.8, step: 0.01 }),
  satelliteColor: zColor(),
  satelliteAccent: zColor(),
  edgeFade: zSlider({ min: 0, max: 1, step: 0.01 }),
  delay: zSlider({ min: 0, max: 60 }),
  introFrames: zSlider({ min: 6, max: 60 }),
  introSpin: zSlider({ min: 0, max: 6.3, step: 0.05 }),
  introScale: zSlider({ min: 0.05, max: 1, step: 0.01 }),
  holdFrames: zSlider({ min: 4, max: 90 }),
  turnFrames: zSlider({ min: 4, max: 40 }),
  wobble: zSlider({ min: 0, max: 0.4, step: 0.01 }),
  camera: typeCameraSchema,
  lens: typeLensSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type LensOrbitProps = z.infer<typeof lensOrbitSchema>;

export const lensOrbitDefaults: LensOrbitProps = {
  words: ['curve', 'glide', 'flow', 'sweep', 'drift'].join('\n'),
  satellites: 'halo ring tide hum coil blur ease veer wisp lift',
  highlight: 'glide sweep',
  face: 'montserrat',
  weight: 800,
  radius: 2.1,
  wordHeight: 34,
  maxArc: 92,
  satelliteScale: 0.34,
  satelliteColor: '#f1f5fc',
  satelliteAccent: '#ffb84d',
  edgeFade: 1,
  delay: 0,
  introFrames: 16,
  introSpin: 2.4,
  introScale: 0.35,
  holdFrames: 22,
  turnFrames: 10,
  wobble: 0.14,
  camera: typeCamera({ drift: 0.4 }),
  lens: typeLens({
    distortion: 0.2,
    fringe: 0.003,
    shift: 1,
    shutter: 1.4,
    samples: 24,
    glow: 0.6,
    glowX: 0.92,
    glowY: 0.95,
  }),
  look: midnightLook(),
  motion: defaultMotion({ exit: 'fade', exitFrames: 10 }),
};

export const lensOrbitMeta: ClipMeta = {
  ...promoMeta,
  id: 'lens-orbit',
  title: 'Lens Orbit',
  description:
    'Big words ride an invisible sphere that tumbles from word to word with heavy motion blur, small words orbiting around each one.',
  category: 'kinetic-type',
  tags: ['kinetic', 'headline', '3d', 'spin', 'orbit'],
  theme: 'dark',
  source: 'src/compositions/scenes/kinetic-type/lens-orbit.tsx',
  durationInFrames: 184,
  posterFrame: 30,
};
