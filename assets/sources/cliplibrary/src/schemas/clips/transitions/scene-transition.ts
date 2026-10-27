import { z } from 'zod';
import { zColor, zSlider } from '../../control';
import { EASING_IDS } from '../../easing';
import { promoMeta } from '../../promo-common';
import type { ClipMeta } from '../../registry';

export const SCENE_TRANSITION_KINDS = [
  'iris',
  'wipe-left',
  'wipe-right',
  'wipe-up',
  'wipe-down',
  'split',
  'split-vertical',
] as const;

export const sceneTransitionSchema = z.object({
  kind: z.enum(SCENE_TRANSITION_KINDS),
  color: zColor(),
  easing: z.enum(EASING_IDS),
  holdFrames: zSlider({ min: 0, max: 30 }),
  centerX: zSlider({ min: 0, max: 100 }),
  centerY: zSlider({ min: 0, max: 100 }),
});

export type SceneTransitionProps = z.infer<typeof sceneTransitionSchema>;

export const sceneTransitionDefaults: SceneTransitionProps = {
  kind: 'iris',
  color: '#2f5bff',
  easing: 'easeInOut',
  holdFrames: 0,
  centerX: 50,
  centerY: 50,
};

export const sceneTransitionMeta: ClipMeta = {
  ...promoMeta,
  id: 'scene-transition',
  title: 'Scene Transition',
  description:
    'A solid colour covers the frame and opens again, to bridge two clips on the timeline.',
  category: 'transitions',
  tags: ['wipe', 'cover', 'minimal'],
  theme: 'dark',
  source: 'src/compositions/scenes/transitions/scene-transition.tsx',
  durationInFrames: 40,
  posterFrame: 20,
};
