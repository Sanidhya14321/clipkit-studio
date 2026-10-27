export const withAlpha = (color: string, alpha: number): string =>
  `color-mix(in srgb, ${color} ${(Math.min(1, Math.max(0, alpha)) * 100).toFixed(1)}%, transparent)`;

export const glowShadow = (color: string, strength: number, size: number): string => {
  if (strength <= 0) {
    return 'none';
  }
  return [0.04, 0.12, 0.32]
    .map(
      (spread, index) =>
        `0 0 ${(size * spread).toFixed(1)}px ${withAlpha(color, strength * (0.9 - index * 0.25))}`,
    )
    .join(', ');
};
