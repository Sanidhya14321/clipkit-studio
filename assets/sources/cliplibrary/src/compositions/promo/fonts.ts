import { loadFont as loadBodoniModa } from '@remotion/google-fonts/BodoniModa';
import { loadFont as loadCaveatBrush } from '@remotion/google-fonts/CaveatBrush';
import { loadFont as loadInter } from '@remotion/google-fonts/Inter';
import { loadFont as loadInterTight } from '@remotion/google-fonts/InterTight';
import { loadFont as loadKalam } from '@remotion/google-fonts/Kalam';
import { loadFont as loadLexend } from '@remotion/google-fonts/Lexend';
import { loadFont as loadOnest } from '@remotion/google-fonts/Onest';
import { loadFont as loadOswald } from '@remotion/google-fonts/Oswald';
import { loadFont as loadPermanentMarker } from '@remotion/google-fonts/PermanentMarker';
import { loadFont as loadPinyonScript } from '@remotion/google-fonts/PinyonScript';
import { loadFont as loadPlayfairDisplay } from '@remotion/google-fonts/PlayfairDisplay';
import { loadFont as loadPoppins } from '@remotion/google-fonts/Poppins';
import { loadFont as loadQuicksand } from '@remotion/google-fonts/Quicksand';
import { loadFont as loadSora } from '@remotion/google-fonts/Sora';
import type { HandFont, PageFont, PickerFont } from '@/schemas';

const interTight = loadInterTight('normal', {
  weights: ['300', '500', '600', '700'],
  subsets: ['latin'],
});

export const displayFont = interTight.fontFamily;

export const displayFontReady = interTight.waitUntilDone;

const inter = loadInter('normal', {
  weights: ['400', '500', '600', '700'],
  subsets: ['latin'],
});

export const bodyFont = inter.fontFamily;

export const bodyFontReady = inter.waitUntilDone;

const handFontLoads = {
  'caveat-brush': loadCaveatBrush('normal', { weights: ['400'], subsets: ['latin'] }),
  'permanent-marker': loadPermanentMarker('normal', { weights: ['400'], subsets: ['latin'] }),
  kalam: loadKalam('normal', { weights: ['700'], subsets: ['latin'] }),
} satisfies Record<HandFont, { fontFamily: string; waitUntilDone: () => Promise<unknown> }>;

export const handFonts: Record<HandFont, string> = {
  'caveat-brush': handFontLoads['caveat-brush'].fontFamily,
  'permanent-marker': handFontLoads['permanent-marker'].fontFamily,
  kalam: handFontLoads.kalam.fontFamily,
};

export const handFontReady = (font: HandFont): Promise<unknown> =>
  handFontLoads[font].waitUntilDone();

const poppins = loadPoppins('normal', {
  weights: ['500', '600', '700', '800'],
  subsets: ['latin'],
});

export const roundedFont = poppins.fontFamily;

export const roundedFontReady = poppins.waitUntilDone;

export const kineticFont = loadOnest('normal', {
  weights: ['300', '500', '600', '700', '800'],
  subsets: ['latin'],
}).fontFamily;

const playfair = loadPlayfairDisplay('normal', {
  weights: ['400', '500', '600', '700'],
  subsets: ['latin'],
});

const bodoni = loadBodoniModa('normal', { weights: ['400', '500'], subsets: ['latin'] });

export const didoneFont = bodoni.fontFamily;

export const didoneFontReady = bodoni.waitUntilDone;

export const pageFonts: Record<PageFont, string> = {
  condensed: loadOswald('normal', {
    weights: ['300', '400', '500', '600', '700'],
    subsets: ['latin'],
  }).fontFamily,
  script: loadPinyonScript('normal', { weights: ['400'], subsets: ['latin'] }).fontFamily,
  serif: playfair.fontFamily,
  sans: displayFont,
};

const sora = loadSora('normal', { weights: ['400', '500', '600', '700'], subsets: ['latin'] });

const lexend = loadLexend('normal', {
  weights: ['300', '400', '500', '600', '700'],
  subsets: ['latin'],
});

const quicksand = loadQuicksand('normal', {
  weights: ['300', '400', '500', '600', '700'],
  subsets: ['latin'],
});

export const pickerFonts: Record<PickerFont, string> = {
  inter: bodyFont,
  sora: sora.fontFamily,
  lexend: lexend.fontFamily,
  quicksand: quicksand.fontFamily,
};
