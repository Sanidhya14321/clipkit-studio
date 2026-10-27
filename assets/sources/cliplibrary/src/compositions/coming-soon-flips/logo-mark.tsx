import { useId } from 'react';
import { useVideoConfig } from 'remotion';
import type { ComingSoonFlipsProps } from '@/schemas';
import { svgId } from '../printer/geometry';
import { logoFont } from './fonts';

const CAP_HEIGHT = 0.7;

export const LogoMark = ({ logo }: { logo: ComingSoonFlipsProps['logo'] }) => {
  const id = svgId(useId());
  const { width, height } = useVideoConfig();
  const glyphs = Array.from(logo.mark);
  const front = glyphs[0] ?? '';
  const behind = glyphs.slice(1).join('');
  const centerX = width / 2;
  const baseline = logo.centerY * height + (logo.size * CAP_HEIGHT) / 2;
  const glyphStyle = {
    fontFamily: logoFont,
    fontWeight: 800,
    fontSize: logo.size,
    letterSpacing: '-0.02em',
  };
  return (
    <svg
      width={width}
      height={height}
      style={{ position: 'absolute', inset: 0 }}
      aria-hidden="true"
    >
      <defs>
        <mask id={`${id}-cut`} maskUnits="userSpaceOnUse" x={0} y={0} width={width} height={height}>
          <rect width={width} height={height} fill="#ffffff" />
          <text
            x={centerX + logo.overlap / 2}
            y={baseline}
            textAnchor="end"
            fill="#000000"
            stroke="#000000"
            strokeWidth={logo.gap * 2}
            strokeLinejoin="round"
            style={glyphStyle}
          >
            {front}
          </text>
        </mask>
      </defs>
      <text
        x={centerX - logo.overlap / 2}
        y={baseline}
        textAnchor="start"
        fill={logo.color}
        mask={`url(#${id}-cut)`}
        style={glyphStyle}
      >
        {behind}
      </text>
      <text
        x={centerX + logo.overlap / 2}
        y={baseline}
        textAnchor="end"
        fill={logo.color}
        style={glyphStyle}
      >
        {front}
      </text>
      <text
        x={centerX}
        y={baseline + logo.captionGap + logo.captionSize * CAP_HEIGHT}
        textAnchor="middle"
        fill={logo.captionColor}
        style={{ fontFamily: logoFont, fontWeight: 600, fontSize: logo.captionSize }}
      >
        {logo.caption}
      </text>
    </svg>
  );
};
