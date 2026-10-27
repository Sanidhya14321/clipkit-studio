import { z } from 'zod';
import { zSlider } from '../../control';
import { ARROW_SHAPES, doodleInk, inkSchema, notebookLook } from '../../doodle-common';
import { defaultMotion, lookSchema, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const doodleArrowSchema = z.object({
  shape: z.enum(ARROW_SHAPES),
  x: zSlider({ min: 0, max: 100, step: 0.5 }),
  y: zSlider({ min: 0, max: 100, step: 0.5 }),
  size: zSlider({ min: 40, max: 600 }),
  rotation: zSlider({ min: -180, max: 180 }),
  flipX: z.boolean(),
  flipY: z.boolean(),
  delay: zSlider({ min: 0, max: 150 }),
});

export type DoodleArrow = z.infer<typeof doodleArrowSchema>;

export const doodleArrow = (
  shape: DoodleArrow['shape'],
  x: number,
  y: number,
  overrides: Partial<DoodleArrow> = {},
): DoodleArrow => ({
  shape,
  x,
  y,
  size: 220,
  rotation: 0,
  flipX: false,
  flipY: false,
  delay: 0,
  ...overrides,
});

export const doodleArrowsSchema = z.object({
  arrows: z.array(doodleArrowSchema).min(1).max(14),
  delay: zSlider({ min: 0, max: 120 }),
  drawFrames: zSlider({ min: 6, max: 90 }),
  headSize: zSlider({ min: 0.05, max: 0.5, step: 0.01 }),
  eraseFrames: zSlider({ min: 4, max: 60 }),
  eraseStagger: zSlider({ min: 0, max: 20 }),
  breakApart: z.boolean(),
  seed: zSlider({ min: 1, max: 999 }),
  ink: inkSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type DoodleArrowsProps = z.infer<typeof doodleArrowsSchema>;

export const doodleArrowsDefaults: DoodleArrowsProps = {
  arrows: [
    doodleArrow('wave', 38, 40, { size: 250, rotation: -4 }),
    doodleArrow('straight', 60, 64, { size: 280, rotation: -6, delay: 12 }),
  ],
  delay: 0,
  drawFrames: 38,
  headSize: 0.2,
  eraseFrames: 22,
  eraseStagger: 4,
  breakApart: true,
  seed: 11,
  ink: doodleInk(),
  look: notebookLook(),
  motion: defaultMotion({ exit: 'none', exitFrames: 0, float: 0 }),
};

export const doodleArrowsMeta: ClipMeta = {
  ...promoMeta,
  id: 'doodle-arrows',
  title: 'Doodle Arrows',
  description:
    'Hand-drawn ink arrows sketch themselves on with a red lead stroke, wobble while they hold, then break apart.',
  category: 'hand-drawn',
  tags: ['doodle', 'sketch', 'arrows', 'overlay'],
  theme: 'light',
  source: 'src/compositions/scenes/hand-drawn/doodle-arrows.tsx',
  durationInFrames: 180,
  posterFrame: 90,
};
