import { glyphStrokeParts } from './glyph-strokes';
import type { StrokeGlyph } from './glyphs';

const clamp01 = (value: number): number => Math.min(1, Math.max(0, value));

export const GlyphStroke = ({
  glyph,
  x,
  top,
  unit,
  strokeWidth,
  progress,
  color,
  scale = 1,
  opacity = 1,
}: {
  glyph: StrokeGlyph;
  x: number;
  top: number;
  unit: number;
  strokeWidth: number;
  progress: number;
  color: string;
  scale?: number;
  opacity?: number;
}) => {
  const drawn = clamp01(progress);
  if (glyph.d.length === 0 || drawn <= 0.002 || scale <= 0.001 || opacity <= 0.001) {
    return null;
  }
  const centerX = x + (glyph.width * unit) / 2;
  const centerY = top + unit / 2;
  const transform = `translate(${centerX} ${centerY}) scale(${scale}) translate(${-centerX} ${-centerY}) translate(${x} ${top}) scale(${unit})`;
  return (
    <g transform={transform} opacity={opacity}>
      {glyphStrokeParts(glyph).map((part) => {
        const local = part.to > part.from ? (drawn - part.from) / (part.to - part.from) : 1;
        if (local <= 0.002) {
          return null;
        }
        return (
          <path
            key={part.d}
            d={part.d}
            fill="none"
            stroke={color}
            strokeWidth={strokeWidth / unit}
            strokeLinecap="round"
            strokeLinejoin="round"
            pathLength={1}
            strokeDasharray={local >= 1 ? undefined : `${local} 2`}
          />
        );
      })}
    </g>
  );
};

export const Comet = ({
  d,
  head,
  tail,
  width,
  color,
  opacity = 1,
}: {
  d: string;
  head: number;
  tail: number;
  width: number;
  color: string;
  opacity?: number;
}) => {
  const end = clamp01(head);
  const start = clamp01(head - tail);
  const length = end - start;
  if (d.length === 0 || length <= 0.002 || opacity <= 0.01) {
    return null;
  }
  return (
    <path
      d={d}
      fill="none"
      stroke={color}
      strokeWidth={width}
      strokeLinecap="round"
      pathLength={1}
      strokeDasharray={`${length} 2`}
      strokeDashoffset={-start}
      opacity={opacity}
    />
  );
};
