import { z } from 'zod';
import { zColor, zSlider } from '../../control';
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

export const mindNodeSchema = z.object({
  title: z.string(),
  subtitle: z.string(),
});

export type MindNode = z.infer<typeof mindNodeSchema>;

export const mindMapSchema = z.object({
  root: mindNodeSchema,
  nodes: z.array(mindNodeSchema).min(1).max(12),
  font: z.enum(PICKER_FONTS),
  weight: zSlider(FONT_WEIGHT),
  type: z.object({
    rootTitle: zSlider({ min: 20, max: 120 }),
    rootSubtitle: zSlider({ min: 10, max: 60 }),
    title: zSlider({ min: 16, max: 100 }),
    subtitle: zSlider({ min: 8, max: 50 }),
    ink: zColor(),
    subtitleInk: zColor(),
  }),
  layout: z.object({
    rootX: zSlider({ min: 0, max: 1200 }),
    centerY: zSlider({ min: 200, max: 880 }),
    arcX: zSlider({ min: 400, max: 1700 }),
    radius: zSlider({ min: 300, max: 3000 }),
    pitch: zSlider({ min: 120, max: 600 }),
    anchor: zSlider({ min: 0, max: 0.6, step: 0.01 }),
    padding: zSlider({ min: 10, max: 80 }),
  }),
  style: z.object({
    rootFill: zColor(),
    activeFill: zColor(),
    nodeFill: zColor(),
    nodeOpacity: zSlider({ min: 0, max: 1, step: 0.01 }),
    activeOpacity: zSlider({ min: 0, max: 1, step: 0.01 }),
    border: zColor(),
    shadow: zSlider({ min: 0, max: 1, step: 0.01 }),
    shadowColor: zColor(),
    activeScale: zSlider({ min: 1, max: 1.4, step: 0.01 }),
    sideFade: zSlider({ min: 0, max: 1, step: 0.01 }),
    reach: zSlider({ min: 0.5, max: 4, step: 0.1 }),
    showIcon: z.boolean(),
    iconColor: zColor(),
  }),
  lines: z.object({
    color: zColor(),
    width: zSlider({ min: 1, max: 10, step: 0.5 }),
    dash: zSlider({ min: 4, max: 60 }),
    gap: zSlider({ min: 4, max: 60 }),
    opacity: zSlider({ min: 0, max: 1, step: 0.01 }),
  }),
  entrance: z.object({
    frames: zSlider({ min: 6, max: 60 }),
    stagger: zSlider({ min: 0, max: 20 }),
  }),
  timing: stepsSchema,
  aura: auraSchema,
  look: lookSchema,
  motion: motionSchema,
});

export type MindMapProps = z.infer<typeof mindMapSchema>;

const node = (title: string, subtitle: string): MindNode => ({ title, subtitle });

export const mindMapDefaults: MindMapProps = {
  root: node('Product Launch', 'Go-to-Market Roadmap'),
  nodes: [
    node('Market Research', 'Audience Insights'),
    node('Brand Story', 'Voice & Visual Identity'),
    node('Pricing Model', 'Tiers & Bundles'),
    node('Launch Event', 'Live Keynote Stream'),
    node('Press Kit', 'Media & Outreach'),
    node('Social Buzz', 'Creator Campaigns'),
    node('Retail Partners', 'Shelf & Online Stores'),
    node('Support Desk', 'Help Center & Chat'),
    node('Feedback Loop', 'Surveys & Reviews'),
    node('Growth Plan', 'Quarterly Targets'),
  ],
  font: 'sora',
  weight: 500,
  type: {
    rootTitle: 66,
    rootSubtitle: 31,
    title: 52,
    subtitle: 25,
    ink: '#ffffff',
    subtitleInk: '#ffffff',
  },
  layout: {
    rootX: 96,
    centerY: 540,
    arcX: 980,
    radius: 1300,
    pitch: 400,
    anchor: 0.12,
    padding: 34,
  },
  style: {
    rootFill: '#6b4ce6',
    activeFill: '#6a48ee',
    nodeFill: '#b5a8ff',
    nodeOpacity: 0.62,
    activeOpacity: 0.94,
    border: '#e6e0ff',
    shadow: 0.35,
    shadowColor: '#961e28',
    activeScale: 1.08,
    sideFade: 0.25,
    reach: 1.6,
    showIcon: true,
    iconColor: '#ffffff',
  },
  lines: { color: '#ffffff', width: 3, dash: 22, gap: 18, opacity: 0.75 },
  entrance: { frames: 28, stagger: 5 },
  timing: steps({ delay: 30, interval: 28, moveFrames: 22 }),
  aura: {
    blobs: [
      blob('#7d63f2', 76, 34, 58, 0.85),
      blob('#a58dff', 6, 26, 62, 0.85),
      blob('#f1edff', 34, 108, 52, 0.85),
      blob('#cfc4ff', 104, 102, 40, 0.6),
    ],
    drift: 0.7,
  },
  look: pickerLook({
    background: '#a99af5',
    backgroundEdge: '#8f7ded',
    ink: '#ffffff',
    accent: '#6b4ce6',
    grain: 0.035,
    spotlight: 0.2,
  }),
  motion: pickerMotion(),
};

export const mindMapMeta: ClipMeta = {
  ...promoMeta,
  id: 'mind-map',
  title: 'Mind Map',
  description:
    'A root topic on the left links to a dashed arc of child nodes that scrolls past, lighting up each one as it reaches the middle.',
  category: 'list-pickers',
  tags: ['list', 'picker', 'scroll', 'cards'],
  theme: 'light',
  source: 'src/compositions/scenes/list-pickers/mind-map.tsx',
  durationInFrames: 180,
  posterFrame: 98,
};
