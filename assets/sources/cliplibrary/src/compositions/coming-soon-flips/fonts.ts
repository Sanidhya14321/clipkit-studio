import { loadFont as loadAntonio } from '@remotion/google-fonts/Antonio';
import { loadFont as loadArchivo } from '@remotion/google-fonts/Archivo';
import type { FlipFont } from '@/schemas';
import { CONNECTOR_FONTS } from '../connectors/fonts';
import { didoneFont, handFonts, pageFonts, roundedFont } from '../promo/fonts';

const condensedFont = loadAntonio('normal', {
  weights: ['400', '500', '600', '700'],
  subsets: ['latin'],
}).fontFamily;

export const extendedFont = loadArchivo('normal', {
  weights: ['700', '800', '900'],
  subsets: ['latin'],
}).fontFamily;

export const logoFont = roundedFont;

type FaceStyle = { family: string; weight: number; scale: number };

export const FLIP_FACES: Record<FlipFont, FaceStyle> = {
  condensed: { family: condensedFont, weight: 700, scale: 1 },
  serif: { family: pageFonts.serif, weight: 500, scale: 0.9 },
  didone: { family: didoneFont, weight: 400, scale: 0.92 },
  marker: { family: handFonts['permanent-marker'], weight: 400, scale: 0.85 },
  brush: { family: CONNECTOR_FONTS.script, weight: 400, scale: 0.88 },
  hand: { family: handFonts.kalam, weight: 700, scale: 0.88 },
  thin: { family: CONNECTOR_FONTS.thin, weight: 700, scale: 1.2 },
  light: { family: pageFonts.condensed, weight: 300, scale: 1.05 },
  script: { family: handFonts['caveat-brush'], weight: 400, scale: 1.04 },
  poster: { family: CONNECTOR_FONTS.bold, weight: 400, scale: 1 },
};
