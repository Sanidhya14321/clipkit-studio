import { Easing } from 'remotion';
import { EASING_IDS, type EasingId } from '@/schemas';

export type { EasingId };
export { EASING_IDS };

const EASINGS: Record<EasingId, (t: number) => number> = {
  linear: Easing.linear,
  easeOut: Easing.out(Easing.cubic),
  easeInOut: Easing.inOut(Easing.cubic),
  easeIn: Easing.in(Easing.cubic),
};

export const resolveEasing = (id: EasingId): ((t: number) => number) => EASINGS[id];
