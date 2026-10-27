import { z } from 'zod';
import { zColor, zImage, zSlider } from '../../control';
import {
  auraSchema,
  blob,
  PICKER_FONTS,
  pickerLook,
  pickerMotion,
  steps,
  stepsSchema,
} from '../../picker-common';
import { FONT_WEIGHT, lookSchema, motionSchema, promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const PRODUCT_ICONS = [
  'watch',
  'earbuds',
  'camera',
  'headset',
  'power-bank',
  'drone',
  'lamp',
  'mouse',
  'microphone',
  'ring',
] as const;

export type ProductIcon = (typeof PRODUCT_ICONS)[number];

export const productCardSchema = z.object({
  title: z.string(),
  description: z.string(),
  icon: z.enum(PRODUCT_ICONS),
  tint: zColor(),
  image: zImage(),
});

export type ProductCard = z.infer<typeof productCardSchema>;

export const productCardsSchema = z.object({
  cards: z.array(productCardSchema).min(1).max(12),
  font: z.enum(PICKER_FONTS),
  type: z.object({
    title: zSlider({ min: 20, max: 120 }),
    titleWeight: zSlider(FONT_WEIGHT),
    description: zSlider({ min: 10, max: 60 }),
    descriptionWeight: zSlider(FONT_WEIGHT),
    ink: zColor(),
    descriptionInk: zColor(),
  }),
  card: z.object({
    width: zSlider({ min: 400, max: 1600 }),
    height: zSlider({ min: 100, max: 400 }),
    radius: zSlider({ min: 0, max: 100 }),
    fill: zColor(),
    fillOpacity: zSlider({ min: 0, max: 1, step: 0.01 }),
    border: zColor(),
    borderWidth: zSlider({ min: 0, max: 12 }),
    glow: zSlider({ min: 0, max: 2, step: 0.05 }),
    blur: zSlider({ min: 0, max: 40 }),
    shadow: zSlider({ min: 0, max: 1, step: 0.01 }),
    shadowColor: zColor(),
  }),
  tile: z.object({
    size: zSlider({ min: 40, max: 300 }),
    radius: zSlider({ min: 0, max: 80 }),
    iconScale: zSlider({ min: 0.3, max: 1, step: 0.01 }),
    color: zColor(),
    ink: zColor(),
  }),
  layout: z.object({
    x: zSlider({ min: -800, max: 800 }),
    y: zSlider({ min: -400, max: 400 }),
    pitch: zSlider({ min: 100, max: 600 }),
    sideScale: zSlider({ min: 0.3, max: 1, step: 0.01 }),
    sideOpacity: zSlider({ min: 0, max: 1, step: 0.01 }),
    loop: z.boolean(),
  }),
  entrance: z.object({
    frames: zSlider({ min: 6, max: 60 }),
    stagger: zSlider({ min: 0, max: 20 }),
    rise: zSlider({ min: 0, max: 300 }),
  }),
  timing: stepsSchema,
  aura: auraSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type ProductCardsProps = z.infer<typeof productCardsSchema>;

const product = (
  title: string,
  description: string,
  icon: ProductIcon,
  tint: string,
): ProductCard => ({ title, description, icon, tint, image: '' });

export const productCardsDefaults: ProductCardsProps = {
  cards: [
    product('Pulse Band', 'Sleep, stride and heart rhythm.', 'watch', '#2f4a52'),
    product('Hush Buds', 'Pocket calm for loud commutes.', 'earbuds', '#f2f7f7'),
    product('Trail Cam', 'Rugged clips from every summit.', 'camera', '#4d6a70'),
    product('Orbit Visor', 'Step into worlds beyond the room.', 'headset', '#e8f1f1'),
    product('Charge Puck', 'Slim snap-on battery for the road.', 'power-bank', '#ff7a45'),
    product('Sky Scout', 'Pocket flyer for aerial stories.', 'drone', '#8fa5a8'),
    product('Glow Arc', 'Warm reading light that dims itself.', 'lamp', '#d6e2e2'),
    product('Glide Mouse', 'Featherweight clicks for long days.', 'mouse', '#7f9599'),
    product('Studio Mic', 'Rich, clear voice capture over USB.', 'microphone', '#c3d1d2'),
    product('Loop Ring', 'A quiet ring that tracks recovery.', 'ring', '#adbdbf'),
  ],
  font: 'lexend',
  type: {
    title: 64,
    titleWeight: 600,
    description: 26,
    descriptionWeight: 300,
    ink: '#ffffff',
    descriptionInk: '#ffffff',
  },
  card: {
    width: 900,
    height: 228,
    radius: 28,
    fill: '#ffffff',
    fillOpacity: 0.16,
    border: '#ffffff',
    borderWidth: 3,
    glow: 1,
    blur: 18,
    shadow: 0.4,
    shadowColor: '#781e5a',
  },
  tile: { size: 156, radius: 30, iconScale: 0.74, color: '#ffffff', ink: '#1f2a2c' },
  layout: { x: 0, y: 0, pitch: 312, sideScale: 0.7, sideOpacity: 0.72, loop: false },
  entrance: { frames: 30, stagger: 5, rise: 120 },
  timing: steps({ delay: 32, interval: 32, moveFrames: 15 }),
  aura: {
    blobs: [
      blob('#0f9e90', 50, -8, 70, 0.9),
      blob('#3fd1bd', 8, 30, 48, 0.55),
      blob('#a5b4fc', 50, 112, 70, 0.85),
      blob('#d5fbf3', 100, 60, 40, 0.5),
    ],
    drift: 0.8,
  },
  look: pickerLook({
    background: '#6ccabd',
    backgroundEdge: '#3fb0a6',
    ink: '#ffffff',
    accent: '#0d9488',
    grain: 0.035,
    spotlight: 0.25,
  }),
  motion: pickerMotion(),
};

export const productCardsMeta: ClipMeta = {
  ...promoMeta,
  id: 'product-cards',
  title: 'Product Cards',
  description:
    'A vertical carousel of frosted glass product cards that steps one card at a time, with the centered card enlarged.',
  category: 'list-pickers',
  tags: ['list', 'cards', 'scroll', 'picker'],
  theme: 'light',
  source: 'src/compositions/scenes/list-pickers/product-cards.tsx',
  durationInFrames: 180,
  posterFrame: 100,
};
