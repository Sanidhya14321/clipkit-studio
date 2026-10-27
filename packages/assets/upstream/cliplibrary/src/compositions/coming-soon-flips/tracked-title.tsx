import type { CSSProperties } from 'react';
import { useVideoConfig } from 'remotion';
import type { ComingSoonFlipsProps } from '@/schemas';
import { pixelTween, turn } from '../promo/timing';
import { extendedFont } from './fonts';
import { hash } from './timeline';

const ADVANCE = 0.78;

const FLICKER_FLOOR = 0.3;

type Title = ComingSoonFlipsProps['title'];

const LEAD_LETTERS = 3;

const appearFrames = (title: Title): number[][] => {
  const lines = [Array.from(title.top), Array.from(title.bottom)];
  const slots = lines.flatMap((letters, line) =>
    letters.map((_, index) => ({ line, index, order: hash(title.seed, line, index) })),
  );
  const ranked = [...slots].sort((a, b) => a.order - b.order);
  const late = Math.max(1, ranked.length - LEAD_LETTERS);
  const frames = lines.map((letters) => letters.map(() => 0));
  ranked.forEach((slot, rank) => {
    const row = frames[slot.line];
    if (row !== undefined) {
      row[slot.index] = Math.floor(
        (Math.max(0, rank - LEAD_LETTERS + 1) / late) * title.appearSpan,
      );
    }
  });
  return frames;
};

const letterOpacity = (
  title: Title,
  appear: number,
  line: number,
  index: number,
  local: number,
): number => {
  if (local < appear) {
    return 0;
  }
  if (local >= appear + title.flickerFrames) {
    return 1;
  }
  return FLICKER_FLOOR + (1 - FLICKER_FLOOR) * hash(title.seed, line, index, local);
};

const TitleLine = ({
  text,
  size,
  stretch,
  line,
  title,
  appear,
  local,
  style,
}: {
  text: string;
  size: number;
  stretch: number;
  line: number;
  title: Title;
  appear: number[];
  local: number;
  style: CSSProperties;
}) => {
  const letters = Array.from(text);
  const extra = (title.spread - 1) * size * ADVANCE;
  const middle = (letters.length - 1) / 2;
  const range: [number, number] = [title.settleStart, title.settleStart + title.settleFrames];
  return (
    <div
      style={{
        display: 'flex',
        justifyContent: 'center',
        fontFamily: extendedFont,
        fontWeight: title.weight,
        fontSize: size,
        lineHeight: 0.74,
        letterSpacing: `${title.tracking}em`,
        whiteSpace: 'pre',
        transform: `scaleX(${stretch})`,
      }}
    >
      {letters.map((letter, index) => {
        const from = (index - middle) * extra;
        const offset = pixelTween(local, range, [from, 0], turn);
        return (
          <span
            key={`${line}:${index}:${letter}`}
            style={{
              ...style,
              display: 'inline-block',
              opacity: letterOpacity(title, appear[index] ?? 0, line, index, local),
              transform: `translateX(${offset}px)`,
            }}
          >
            {letter}
          </span>
        );
      })}
    </div>
  );
};

const lineStyle = (title: Title, line: 'top' | 'bottom'): CSSProperties =>
  title.outlined === line
    ? { color: 'transparent', WebkitTextStroke: `${title.outline}px ${title.color}` }
    : { color: line === 'top' ? title.color : title.accent };

export const TrackedTitle = ({ title, local }: { title: Title; local: number }) => {
  const { height } = useVideoConfig();
  const appear = appearFrames(title);
  return (
    <div
      style={{
        position: 'absolute',
        left: 0,
        right: 0,
        top: title.topY * height,
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        gap: title.gap,
      }}
    >
      <TitleLine
        text={title.top}
        size={title.topSize}
        stretch={title.topStretch}
        line={0}
        title={title}
        appear={appear[0] ?? []}
        local={local}
        style={lineStyle(title, 'top')}
      />
      <TitleLine
        text={title.bottom}
        size={title.bottomSize}
        stretch={title.bottomStretch}
        line={1}
        title={title}
        appear={appear[1] ?? []}
        local={local}
        style={lineStyle(title, 'bottom')}
      />
    </div>
  );
};
