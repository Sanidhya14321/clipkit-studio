import { z } from 'zod';
import { zSlider } from './control';
import { lightLook, type PromoLook } from './promo-common';

export const ARROW_SHAPES = [
  'straight',
  'arc',
  'wave',
  'zigzag',
  'loop',
  'spiral',
  'heart',
  'squiggle',
] as const;

export type ArrowShape = (typeof ARROW_SHAPES)[number];

export const HAND_FONTS = ['caveat-brush', 'permanent-marker', 'kalam'] as const;

export type HandFont = (typeof HAND_FONTS)[number];

export const inkSchema = z.object({
  width: zSlider({ min: 2, max: 30, step: 0.5 }),
  roughness: zSlider({ min: 0, max: 1, step: 0.01 }),
  boil: zSlider({ min: 0, max: 1, step: 0.01 }),
  boilFrames: zSlider({ min: 1, max: 8 }),
  lead: zSlider({ min: 0, max: 1, step: 0.01 }),
  leadOffset: zSlider({ min: 0, max: 20, step: 0.5 }),
  flecks: zSlider({ min: 0, max: 1, step: 0.01 }),
});

export type DoodleInk = z.infer<typeof inkSchema>;

export const doodleInk = (overrides: Partial<DoodleInk> = {}): DoodleInk => ({
  width: 11,
  roughness: 0.5,
  boil: 0.45,
  boilFrames: 3,
  lead: 0.85,
  leadOffset: 6,
  flecks: 0.6,
  ...overrides,
});

export const notebookLook = (showBackdrop = true): PromoLook => ({
  ...lightLook('#e0263a', showBackdrop),
  background: '#f7f8f9',
  backgroundEdge: '#e3e6ea',
  ink: '#141416',
  muted: '#6d7079',
  grain: 0.07,
  spotlight: 0.4,
  paper: 'ruled',
  paperLine: '#c9d3e1',
  paperSpacing: 54,
});
