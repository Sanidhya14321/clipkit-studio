import { z } from 'zod';
import { zColor, zSlider } from './control';
import { darkLook, type PromoLook } from './promo-common';

export const CONNECTOR_BADGES = [
  'script-swash',
  'ruled-caps',
  'tape',
  'frame-box',
  'arc-script',
  'ribbon',
  'starburst',
  'stamp',
  'bubble',
  'arrow-sign',
  'pop-lines',
  'ampersand',
] as const;

export type ConnectorBadgeVariant = (typeof CONNECTOR_BADGES)[number];

export const connectorBadgeSchema = z.object({
  variant: z.enum(CONNECTOR_BADGES),
  word: z.string(),
  line: zColor(),
  fill: zColor(),
  text: zColor(),
});

export type ConnectorBadge = z.infer<typeof connectorBadgeSchema>;

const CHALK_INK = '#eef2e8';
const CHALK_DEEP = '#16261f';

const BADGE_COLORS: Record<ConnectorBadgeVariant, Omit<ConnectorBadge, 'variant' | 'word'>> = {
  'script-swash': { line: CHALK_INK, fill: '#f4a9c0', text: CHALK_INK },
  'ruled-caps': { line: '#f2dc8c', fill: '#c5b4ee', text: CHALK_INK },
  tape: { line: CHALK_INK, fill: '#c5b4ee', text: CHALK_DEEP },
  'frame-box': { line: '#a6d4ec', fill: '#a6d4ec', text: CHALK_INK },
  'arc-script': { line: CHALK_INK, fill: '#f6bd8e', text: '#f6bd8e' },
  ribbon: { line: CHALK_DEEP, fill: '#a6d4ec', text: CHALK_DEEP },
  starburst: { line: CHALK_INK, fill: '#f08a76', text: CHALK_DEEP },
  stamp: { line: CHALK_INK, fill: '#f4a9c0', text: CHALK_INK },
  bubble: { line: '#f2dc8c', fill: '#f2dc8c', text: CHALK_INK },
  'arrow-sign': { line: CHALK_INK, fill: '#c5b4ee', text: CHALK_DEEP },
  'pop-lines': { line: CHALK_INK, fill: '#a6d4ec', text: '#a6d4ec' },
  ampersand: { line: CHALK_INK, fill: '#f2dc8c', text: CHALK_INK },
};

export const connectorBadge = (
  variant: ConnectorBadgeVariant,
  word: string,
  overrides: Partial<ConnectorBadge> = {},
): ConnectorBadge => ({ variant, word, ...BADGE_COLORS[variant], ...overrides });

export const CONNECTOR_SET: ConnectorBadge[] = [
  connectorBadge('script-swash', 'also'),
  connectorBadge('ruled-caps', 'NOR'),
  connectorBadge('tape', 'NEXT'),
  connectorBadge('frame-box', 'IF'),
  connectorBadge('arc-script', 'into'),
  connectorBadge('ribbon', 'ONTO'),
  connectorBadge('starburst', 'AT'),
  connectorBadge('stamp', 'AS'),
  connectorBadge('bubble', 'per'),
  connectorBadge('arrow-sign', 'NOW'),
  connectorBadge('pop-lines', 'too'),
  connectorBadge('ampersand', '&'),
];

export const chalkSchema = z.object({
  width: zSlider({ min: 2, max: 30, step: 0.5 }),
  roughness: zSlider({ min: 0, max: 12, step: 0.25 }),
  dryness: zSlider({ min: 0, max: 1, step: 0.01 }),
  shadow: zSlider({ min: 0, max: 1, step: 0.01 }),
});

export type ConnectorChalk = z.infer<typeof chalkSchema>;

export const connectorChalk = (overrides: Partial<ConnectorChalk> = {}): ConnectorChalk => ({
  width: 9,
  roughness: 3.5,
  dryness: 0.35,
  shadow: 0.45,
  ...overrides,
});

export const badgeTimingSchema = z.object({
  drawFrames: zSlider({ min: 6, max: 90 }),
  eraseAt: zSlider({ min: 10, max: 600 }),
  eraseFrames: zSlider({ min: 4, max: 60 }),
});

export type BadgeTiming = z.infer<typeof badgeTimingSchema>;

export const chalkboardLook = (showBackdrop = true): PromoLook => ({
  ...darkLook('#f2dc8c', showBackdrop),
  background: '#1f3a33',
  backgroundEdge: '#0f1f1b',
  ink: CHALK_INK,
  muted: '#86a39a',
  grain: 0.22,
  spotlight: 0.12,
  paper: 'grid',
  paperLine: '#2b4840',
  paperSpacing: 144,
});
