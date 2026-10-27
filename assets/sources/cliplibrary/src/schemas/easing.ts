export const EASING_IDS = ['linear', 'easeOut', 'easeInOut', 'easeIn'] as const;

export type EasingId = (typeof EASING_IDS)[number];
