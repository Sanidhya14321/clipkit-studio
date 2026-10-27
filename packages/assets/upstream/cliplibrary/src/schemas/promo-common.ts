import { z } from 'zod';
import { zColor, zImage, zSlider } from './control';

export const PROMO_SCREEN_IDS = [
  'home',
  'onboarding',
  'analytics',
  'tasks',
  'calendar',
  'messages',
  'wallet',
  'chat',
  'activity',
  'feed',
  'music',
  'map',
  'profile',
  'settings',
  'ride-welcome',
  'ride-unlock',
  'ride-nearby',
  'ride-receipt',
] as const;

export type PromoScreenId = (typeof PROMO_SCREEN_IDS)[number];

export const PROMO_CARD_IDS = ['income', 'growth', 'goal'] as const;

export type PromoCardId = (typeof PROMO_CARD_IDS)[number];

export const PHONE_FINISHES = ['natural', 'black', 'white', 'blue', 'desert'] as const;

export type PhoneFinish = (typeof PHONE_FINISHES)[number];

export const PROMO_TRANSITIONS = [
  'none',
  'fade',
  'blur',
  'rise',
  'iris',
  'push-left',
  'push-up',
  'zoom',
  'grow',
  'blast',
] as const;

export type PromoTransition = (typeof PROMO_TRANSITIONS)[number];

export const TEXT_SIDES = ['left', 'right'] as const;

export const TYPEFACES = ['inter-tight', 'onest'] as const;

export const SCRIBBLE_SHAPES = ['swash', 'loop', 'underline', 'circle'] as const;

export type ScribbleShape = (typeof SCRIBBLE_SHAPES)[number];

export const FONT_WEIGHT = { min: 300, max: 700, step: 100 } as const;

export const HORIZONTAL_ALIGNMENTS = ['left', 'center', 'right'] as const;

export const VERTICAL_ALIGNMENTS = ['top', 'center', 'bottom'] as const;

export const placementSchema = z.object({
  horizontal: z.enum(HORIZONTAL_ALIGNMENTS),
  vertical: z.enum(VERTICAL_ALIGNMENTS),
  insetX: zSlider({ min: 0, max: 700 }),
  insetY: zSlider({ min: 0, max: 400 }),
  width: zSlider({ min: 300, max: 1700 }),
});

export type PromoPlacement = z.infer<typeof placementSchema>;

export const placement = (
  horizontal: PromoPlacement['horizontal'],
  vertical: PromoPlacement['vertical'],
  overrides: Partial<PromoPlacement> = {},
): PromoPlacement => ({ horizontal, vertical, insetX: 160, insetY: 90, width: 760, ...overrides });

export const positionSchema = z.object({
  x: zSlider({ min: -7, max: 7, step: 0.05 }),
  y: zSlider({ min: -4, max: 4, step: 0.05 }),
  z: zSlider({ min: -4, max: 3, step: 0.05 }),
});

export type PromoPosition = z.infer<typeof positionSchema>;

export const position = (x = 0, y = 0, depth = 0): PromoPosition => ({ x, y, z: depth });

export const BACKDROP_TONES = ['light', 'dark'] as const;

export const PAPER_PATTERNS = ['plain', 'ruled', 'grid'] as const;

export type PaperPattern = (typeof PAPER_PATTERNS)[number];

export const phoneSchema = z.object({
  screen: z.enum(PROMO_SCREEN_IDS),
  customScreen: zImage(),
  finish: z.enum(PHONE_FINISHES),
});

export type PromoPhone = z.infer<typeof phoneSchema>;

export const lookSchema = z.object({
  showBackdrop: z.boolean(),
  background: zColor(),
  backgroundEdge: zColor(),
  ink: zColor(),
  muted: zColor(),
  accent: zColor(),
  grain: zSlider({ min: 0, max: 0.3, step: 0.01 }),
  liveGrain: z.boolean(),
  spotlight: zSlider({ min: 0, max: 1, step: 0.01 }),
  paper: z.enum(PAPER_PATTERNS),
  paperLine: zColor(),
  paperSpacing: zSlider({ min: 20, max: 200 }),
  paperMargin: z.boolean(),
});

export type PromoLook = z.infer<typeof lookSchema>;

export const stageSchema = z.object({
  cameraDistance: zSlider({ min: 8, max: 30, step: 0.1 }),
  fieldOfView: zSlider({ min: 15, max: 60, step: 1 }),
  keyLight: zSlider({ min: 0, max: 4, step: 0.05 }),
  reflections: zSlider({ min: 0, max: 3, step: 0.05 }),
  shadow: zSlider({ min: 0, max: 0.6, step: 0.01 }),
  shadowSoftness: zSlider({ min: 1, max: 40, step: 1 }),
  cameraDrift: zSlider({ min: 0, max: 1, step: 0.01 }),
});

export type PromoStage = z.infer<typeof stageSchema>;

export const motionSchema = z.object({
  enter: z.enum(PROMO_TRANSITIONS),
  enterFrames: zSlider({ min: 0, max: 60 }),
  exit: z.enum(PROMO_TRANSITIONS),
  exitFrames: zSlider({ min: 0, max: 60 }),
  float: zSlider({ min: 0, max: 0.3, step: 0.005 }),
  speed: zSlider({ min: 0.25, max: 2, step: 0.05 }),
});

export type PromoMotion = z.infer<typeof motionSchema>;

export const lightLook = (accent = '#2f5bff', showBackdrop = true): PromoLook => ({
  showBackdrop,
  background: '#f4f4f2',
  backgroundEdge: '#e6e6e2',
  ink: '#0b0b0c',
  muted: '#7c7c80',
  accent,
  grain: 0.06,
  liveGrain: false,
  spotlight: 0.55,
  paper: 'plain',
  paperLine: '#cbd3de',
  paperSpacing: 54,
  paperMargin: false,
});

export const clayLook = (accent = '#9a7a0c', showBackdrop = true): PromoLook => ({
  showBackdrop,
  background: '#d4d4d1',
  backgroundEdge: '#b9b9b5',
  ink: '#1b1f3b',
  muted: '#6f7184',
  accent,
  grain: 0.05,
  liveGrain: false,
  spotlight: 0.45,
  paper: 'plain',
  paperLine: '#cbd3de',
  paperSpacing: 54,
  paperMargin: false,
});

export const darkLook = (accent = '#7d9bff', showBackdrop = true): PromoLook => ({
  showBackdrop,
  background: '#16161a',
  backgroundEdge: '#050506',
  ink: '#f5f5f4',
  muted: '#8d8d93',
  accent,
  grain: 0.08,
  liveGrain: false,
  spotlight: 0.7,
  paper: 'plain',
  paperLine: '#2a2d36',
  paperSpacing: 54,
  paperMargin: false,
});

export const kineticLook = (accent = '#e5262b', showBackdrop = true): PromoLook => ({
  showBackdrop,
  background: '#020202',
  backgroundEdge: '#000000',
  ink: '#f1f1f1',
  muted: '#7d7d7d',
  accent,
  grain: 0.14,
  liveGrain: true,
  spotlight: 0,
  paper: 'plain',
  paperLine: '#2a2d36',
  paperSpacing: 54,
  paperMargin: false,
});

export const defaultStage = (overrides: Partial<PromoStage> = {}): PromoStage => ({
  cameraDistance: 14,
  fieldOfView: 30,
  keyLight: 2,
  reflections: 1.1,
  shadow: 0.16,
  shadowSoftness: 18,
  cameraDrift: 0.35,
  ...overrides,
});

export const defaultMotion = (overrides: Partial<PromoMotion> = {}): PromoMotion => ({
  enter: 'none',
  enterFrames: 18,
  exit: 'fade',
  exitFrames: 16,
  float: 0.06,
  speed: 1,
  ...overrides,
});

export const phone = (
  screen: PromoScreenId,
  finish: PhoneFinish = 'natural',
  customScreen = '',
): PromoPhone => ({ screen, customScreen, finish });

export const promoMeta = {
  fps: 30,
  width: 1920,
  height: 1080,
} as const;
