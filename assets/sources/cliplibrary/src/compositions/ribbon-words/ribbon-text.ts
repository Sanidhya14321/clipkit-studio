import type { RibbonWordsProps } from '@/schemas';
import { displayFont } from '../promo/fonts';

export type TextPart = { text: string; highlighted: boolean };

let context: CanvasRenderingContext2D | null | undefined;

const getContext = (): CanvasRenderingContext2D | null => {
  if (context === undefined) {
    context =
      typeof document === 'undefined' ? null : document.createElement('canvas').getContext('2d');
  }
  return context;
};

export const measureLine = (text: string, words: RibbonWordsProps['words']): number => {
  const spacing = words.tracking * words.size * [...text].length;
  const ctx = getContext();
  if (ctx === null) {
    return words.size * 0.52 * [...text].length + spacing;
  }
  ctx.font = `${words.weight} ${words.size}px "${displayFont}"`;
  return ctx.measureText(text).width + spacing;
};

export const splitHighlight = (text: string, highlight: string): TextPart[] => {
  const word = highlight.trim();
  if (word.length === 0 || !text.includes(word)) {
    return [{ text, highlighted: false }];
  }
  const parts: TextPart[] = [];
  text.split(word).forEach((piece, index) => {
    if (index > 0) {
      parts.push({ text: word, highlighted: true });
    }
    if (piece.length > 0) {
      parts.push({ text: piece, highlighted: false });
    }
  });
  return parts;
};
