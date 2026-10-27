import { useEffect, useMemo, useState } from 'react';
import { useDelayRender } from 'remotion';
import type { DotWordmarkRevealProps } from '@/schemas';
import { displayFont, displayFontReady } from '../promo/fonts';
import { pixelTween, settle, tween } from '../promo/timing';

type TypeProps = DotWordmarkRevealProps['type'];

export interface TaglineLayout {
  words: string[];
  lefts: number[];
  widths: number[];
}

const taglineWords = (tagline: string): string[] =>
  tagline
    .split('\n')
    .map((line) => line.trim())
    .filter((line) => line.length > 0);

const fontSpec = (type: TypeProps): string =>
  `${type.taglineWeight} ${type.taglineSize}px "${displayFont}"`;

const useTaglineFont = (type: TypeProps): boolean => {
  const [loaded, setLoaded] = useState(false);
  const { delayRender, continueRender } = useDelayRender();
  const spec = fontSpec(type);
  useEffect(() => {
    const handle = delayRender('Loading the tagline font');
    let active = true;
    displayFontReady()
      .then(() => document.fonts.load(spec))
      .then(() => {
        if (active) {
          setLoaded(true);
        }
        continueRender(handle);
      })
      .catch(() => {
        continueRender(handle);
      });
    return () => {
      active = false;
      continueRender(handle);
    };
  }, [spec, delayRender, continueRender]);
  return loaded;
};

export const useTaglineLayout = (
  tagline: string,
  type: TypeProps,
  canvasWidth: number,
): TaglineLayout => {
  const loaded = useTaglineFont(type);
  return useMemo(() => {
    const words = taglineWords(tagline);
    const context = document.createElement('canvas').getContext('2d');
    const tracking = type.taglineTracking * type.taglineSize;
    const widths = words.map((word) => {
      if (context === null || !loaded) {
        return word.length * type.taglineSize * 0.68;
      }
      context.font = fontSpec(type);
      return context.measureText(word).width + tracking * word.length;
    });
    const total =
      widths.reduce((sum, width) => sum + width, 0) + type.wordGap * Math.max(0, words.length - 1);
    let cursor = (canvasWidth - total) / 2;
    const lefts = widths.map((width) => {
      const left = cursor;
      cursor += width + type.wordGap;
      return left;
    });
    return { words, lefts, widths };
  }, [tagline, type, canvasWidth, loaded]);
};

export const Tagline = ({
  layout,
  frame,
  appearAt,
  fadeStart,
  fadeEnd,
  centerY,
  color,
  type,
}: {
  layout: TaglineLayout;
  frame: number;
  appearAt: number[];
  fadeStart: number;
  fadeEnd: number;
  centerY: number;
  color: string;
  type: TypeProps;
}) => {
  const fade = tween(frame, [fadeStart, fadeEnd], [1, 0], settle);
  if (fade <= 0) {
    return null;
  }
  return (
    <div style={{ position: 'absolute', inset: 0, opacity: fade }}>
      {layout.words.map((word, index) => {
        const at = appearAt[index] ?? Number.POSITIVE_INFINITY;
        if (frame < at) {
          return null;
        }
        const drop = pixelTween(frame, [at, at + 6], [type.taglineSize * 0.22, 0], settle);
        return (
          <span
            key={`${word}-${index.toString()}`}
            style={{
              position: 'absolute',
              left: layout.lefts[index] ?? 0,
              top: centerY - type.taglineSize * 0.6 + drop,
              fontFamily: displayFont,
              fontWeight: type.taglineWeight,
              fontSize: type.taglineSize,
              lineHeight: `${type.taglineSize * 1.2}px`,
              letterSpacing: `${type.taglineTracking}em`,
              color,
              whiteSpace: 'pre',
            }}
          >
            {word}
          </span>
        );
      })}
    </div>
  );
};
