import { z } from 'zod';
import { zColor, zSlider } from './control';
import type { PromoLook } from './promo-common';

export const HEAVY_WEIGHT = { min: 300, max: 900, step: 100 } as const;

export const TYPE_FACES = ['montserrat', 'onest', 'poppins'] as const;

export type TypeFace = (typeof TYPE_FACES)[number];

export const typeCameraSchema = z.object({
  distance: zSlider({ min: 6, max: 30, step: 0.1 }),
  fieldOfView: zSlider({ min: 15, max: 70, step: 1 }),
  drift: zSlider({ min: 0, max: 2, step: 0.01 }),
});

export type TypeCameraSettings = z.infer<typeof typeCameraSchema>;

export const typeCamera = (overrides: Partial<TypeCameraSettings> = {}): TypeCameraSettings => ({
  distance: 12,
  fieldOfView: 30,
  drift: 0.6,
  ...overrides,
});

export const typeLensSchema = z.object({
  samples: zSlider({ min: 1, max: 32 }),
  shutter: zSlider({ min: 0, max: 2, step: 0.05 }),
  distortion: zSlider({ min: 0, max: 1.5, step: 0.01 }),
  fringe: zSlider({ min: 0, max: 0.03, step: 0.0005 }),
  shift: zSlider({ min: 0, max: 8, step: 0.1 }),
  grain: zSlider({ min: 0, max: 0.08, step: 0.001 }),
  glow: zSlider({ min: 0, max: 1, step: 0.01 }),
  glowX: zSlider({ min: 0, max: 1, step: 0.01 }),
  glowY: zSlider({ min: 0, max: 1, step: 0.01 }),
});

export type TypeLens = z.infer<typeof typeLensSchema>;

export const typeLens = (overrides: Partial<TypeLens> = {}): TypeLens => ({
  samples: 10,
  shutter: 1,
  distortion: 0.35,
  fringe: 0.004,
  shift: 1.5,
  grain: 0.025,
  glow: 0,
  glowX: 0.85,
  glowY: 0.9,
  ...overrides,
});

export const warpGridSchema = z.object({
  show: z.boolean(),
  color: zColor(),
  spacing: zSlider({ min: 0.3, max: 3, step: 0.05 }),
  bulge: zSlider({ min: 0, max: 6, step: 0.05 }),
  lineWidth: zSlider({ min: 0.005, max: 0.08, step: 0.001 }),
  depth: zSlider({ min: -8, max: 0, step: 0.1 }),
});

export type WarpGridSettings = z.infer<typeof warpGridSchema>;

export const warpGrid = (overrides: Partial<WarpGridSettings> = {}): WarpGridSettings => ({
  show: true,
  color: '#d9d8e2',
  spacing: 0.95,
  bulge: 2.6,
  lineWidth: 0.02,
  depth: -2.5,
  ...overrides,
});

export const pastelLook = (accent = '#5b3df5', showBackdrop = true): PromoLook => ({
  showBackdrop,
  background: '#f7f6fb',
  backgroundEdge: '#e4e1f1',
  ink: '#16141f',
  muted: '#8a8798',
  accent,
  grain: 0.09,
  liveGrain: false,
  spotlight: 0.6,
  paper: 'plain',
  paperLine: '#cbd3de',
  paperSpacing: 54,
  paperMargin: false,
});

export const inkLook = (accent = '#8b74ff', showBackdrop = true): PromoLook => ({
  showBackdrop,
  background: '#1b1a24',
  backgroundEdge: '#07070b',
  ink: '#f4f3f8',
  muted: '#8c8a99',
  accent,
  grain: 0.1,
  liveGrain: false,
  spotlight: 0.35,
  paper: 'plain',
  paperLine: '#2a2d36',
  paperSpacing: 54,
  paperMargin: false,
});

export const bonePaperLook = (accent = '#11795f', showBackdrop = true): PromoLook => ({
  ...pastelLook(accent, showBackdrop),
  background: '#f4f0e6',
  backgroundEdge: '#e2dac6',
  ink: '#17221e',
  muted: '#7f857d',
});

export const midnightLook = (accent = '#ffb84d', showBackdrop = true): PromoLook => ({
  ...inkLook(accent, showBackdrop),
  background: '#111b2c',
  backgroundEdge: '#03060c',
  ink: '#f1f5fc',
  muted: '#8a96ab',
});

export const mistLook = (accent = '#2f5bd3', showBackdrop = true): PromoLook => ({
  ...pastelLook(accent, showBackdrop),
  background: '#f2f6fc',
  backgroundEdge: '#c9d8f0',
  ink: '#111a2c',
  muted: '#7d889e',
});

export const forestLook = (accent = '#c4ea5a', showBackdrop = true): PromoLook => ({
  ...inkLook(accent, showBackdrop),
  background: '#0f1916',
  backgroundEdge: '#030706',
  ink: '#f2f6ee',
  muted: '#8c968f',
});
