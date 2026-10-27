import { z } from 'zod';
import { zColor, zSlider } from './control';

export const ANNIVERSARY_EFFECTS = [
  'hold',
  'wave-stack',
  'gradient',
  'fill-sweep',
  'flash-cut',
  'drop',
  'slant-repeat',
  'outline-flicker',
  'type-on',
  'triple-slide',
  'triple-stack',
  'track-in',
  'tracked-outline',
] as const;

export type AnniversaryEffect = (typeof ANNIVERSARY_EFFECTS)[number];

export const ANNIVERSARY_BACKDROPS = ['ink', 'paper', 'night', 'dusk', 'texture', 'band'] as const;

export type AnniversaryBackdrop = (typeof ANNIVERSARY_BACKDROPS)[number];

export const ANNIVERSARY_DECORS = ['none', 'outline'] as const;

export type AnniversaryDecor = (typeof ANNIVERSARY_DECORS)[number];

export const anniversaryBeatSchema = z.object({
  text: z.string(),
  start: zSlider({ min: 0, max: 900 }),
  effect: z.enum(ANNIVERSARY_EFFECTS),
  backdrop: z.enum(ANNIVERSARY_BACKDROPS),
  decor: z.enum(ANNIVERSARY_DECORS),
  decorText: z.string(),
  decorSize: zSlider({ min: 200, max: 4000 }),
  decorX: zSlider({ min: -1, max: 1, step: 0.01 }),
  decorY: zSlider({ min: -1, max: 1, step: 0.01 }),
  decorFillAt: zSlider({ min: 0, max: 120 }),
  decorFill: zSlider({ min: 0, max: 30 }),
  color: zColor(),
  size: zSlider({ min: 40, max: 400 }),
  weight: zSlider({ min: 300, max: 700, step: 100 }),
  tracking: zSlider({ min: -0.1, max: 0.6, step: 0.005 }),
  spread: zSlider({ min: 0, max: 1, step: 0.01 }),
  zoom: zSlider({ min: -0.5, max: 0.8, step: 0.01 }),
  rate: zSlider({ min: 0.25, max: 30, step: 0.05 }),
  ghost: zSlider({ min: 0, max: 1, step: 0.01 }),
  glitch: zSlider({ min: 0, max: 30 }),
  glitchAt: zSlider({ min: 0, max: 120 }),
  pulseAt: zSlider({ min: 0, max: 120 }),
  pulseBlur: zSlider({ min: 0, max: 40 }),
});

export type AnniversaryBeat = z.infer<typeof anniversaryBeatSchema>;

export const anniversaryLookSchema = z.object({
  showBackdrop: z.boolean(),
  ink: zColor(),
  paper: zColor(),
  night: zColor(),
  dusk: zColor(),
  flash: zColor(),
  dim: zColor(),
  muted: zColor(),
  gradientFrom: zColor(),
  gradientMid: zColor(),
  gradientTo: zColor(),
  texture: zColor(),
  textureShade: zColor(),
  textureContrast: zSlider({ min: 0, max: 3, step: 0.05 }),
  textureDrift: zSlider({ min: 0, max: 12, step: 0.1 }),
  bandHeight: zSlider({ min: 0.1, max: 1, step: 0.01 }),
  stroke: zColor(),
  strokeWidth: zSlider({ min: 0.5, max: 8, step: 0.5 }),
  shape: zColor(),
  slant: zColor(),
  dust: zColor(),
  dustCount: zSlider({ min: 0, max: 40 }),
  glitchAmount: zSlider({ min: 0, max: 2, step: 0.05 }),
  glitchSeed: zSlider({ min: 1, max: 99 }),
  drift: zSlider({ min: 0, max: 0.2, step: 0.005 }),
});

export type AnniversaryLook = z.infer<typeof anniversaryLookSchema>;

export const anniversaryLogoSchema = z.object({
  start: zSlider({ min: 0, max: 900 }),
  number: z.string(),
  ordinal: z.string(),
  caption: z.string(),
  name: z.string(),
  since: z.string(),
  tagline: z.string(),
  background: zColor(),
  numberColor: zColor(),
  ringColor: zColor(),
  nameColor: zColor(),
  initialColor: zColor(),
  taglineColor: zColor(),
  size: zSlider({ min: 0.4, max: 2, step: 0.01 }),
  blur: zSlider({ min: 0, max: 40 }),
  blurFrames: zSlider({ min: 1, max: 30 }),
});

export type AnniversaryLogo = z.infer<typeof anniversaryLogoSchema>;

const BEAT_BASE: Omit<AnniversaryBeat, 'text' | 'start' | 'effect' | 'backdrop'> = {
  decor: 'none',
  decorText: '',
  decorSize: 1600,
  decorX: 0,
  decorY: 0,
  decorFillAt: 0,
  decorFill: 0,
  color: '#fbf3e6',
  size: 150,
  weight: 700,
  tracking: -0.01,
  spread: 0,
  zoom: 0,
  rate: 1,
  ghost: 0,
  glitch: 0,
  glitchAt: 0,
  pulseAt: 0,
  pulseBlur: 0,
};

export const anniversaryBeat = (
  text: string,
  start: number,
  effect: AnniversaryEffect,
  backdrop: AnniversaryBackdrop,
  overrides: Partial<AnniversaryBeat> = {},
): AnniversaryBeat => ({ ...BEAT_BASE, text, start, effect, backdrop, ...overrides });
