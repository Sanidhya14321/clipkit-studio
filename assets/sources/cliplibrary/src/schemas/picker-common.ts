import { z } from 'zod';
import { zColor, zSlider } from './control';
import { defaultMotion, type PromoLook, type PromoMotion } from './promo-common';

export const PICKER_FONTS = ['inter', 'sora', 'lexend', 'quicksand'] as const;

export type PickerFont = (typeof PICKER_FONTS)[number];

export const auraBlobSchema = z.object({
  color: zColor(),
  x: zSlider({ min: -40, max: 140 }),
  y: zSlider({ min: -40, max: 140 }),
  size: zSlider({ min: 10, max: 160 }),
  strength: zSlider({ min: 0, max: 1, step: 0.01 }),
});

export type AuraBlob = z.infer<typeof auraBlobSchema>;

export const auraSchema = z.object({
  blobs: z.array(auraBlobSchema).max(6),
  drift: zSlider({ min: 0, max: 1, step: 0.01 }),
});

export type PickerAura = z.infer<typeof auraSchema>;

export const blob = (color: string, x: number, y: number, size = 70, strength = 0.8): AuraBlob => ({
  color,
  x,
  y,
  size,
  strength,
});

export const stepsSchema = z.object({
  delay: zSlider({ min: 0, max: 120 }),
  interval: zSlider({ min: 12, max: 120 }),
  moveFrames: zSlider({ min: 6, max: 60 }),
  start: zSlider({ min: 0, max: 20 }),
});

export type PickerSteps = z.infer<typeof stepsSchema>;

export const steps = (overrides: Partial<PickerSteps> = {}): PickerSteps => ({
  delay: 36,
  interval: 30,
  moveFrames: 20,
  start: 0,
  ...overrides,
});

export const pickerLook = (overrides: Partial<PromoLook>): PromoLook => ({
  showBackdrop: true,
  background: '#fbf8f9',
  backgroundEdge: '#f1e9ec',
  ink: '#111113',
  muted: '#8a8a8e',
  accent: '#f0287a',
  grain: 0.04,
  liveGrain: false,
  spotlight: 0.4,
  paper: 'plain',
  paperLine: '#cbd3de',
  paperSpacing: 54,
  paperMargin: false,
  ...overrides,
});

export const pickerMotion = (overrides: Partial<PromoMotion> = {}): PromoMotion =>
  defaultMotion({ exit: 'fade', exitFrames: 14, float: 0, ...overrides });
