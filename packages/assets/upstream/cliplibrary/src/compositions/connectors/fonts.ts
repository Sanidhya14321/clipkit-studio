import { loadFont as loadAmaticSC } from '@remotion/google-fonts/AmaticSC';
import { loadFont as loadAnton } from '@remotion/google-fonts/Anton';
import { loadFont as loadKaushanScript } from '@remotion/google-fonts/KaushanScript';
import { loadFont as loadLuckiestGuy } from '@remotion/google-fonts/LuckiestGuy';
import { loadFont as loadPlayfairDisplay } from '@remotion/google-fonts/PlayfairDisplay';

export const CONNECTOR_FONTS = {
  script: loadKaushanScript('normal', { weights: ['400'], subsets: ['latin'] }).fontFamily,
  thin: loadAmaticSC('normal', { weights: ['700'], subsets: ['latin'] }).fontFamily,
  bold: loadAnton('normal', { weights: ['400'], subsets: ['latin'] }).fontFamily,
  chunky: loadLuckiestGuy('normal', { weights: ['400'], subsets: ['latin'] }).fontFamily,
  serif: loadPlayfairDisplay('normal', { weights: ['800'], subsets: ['latin'] }).fontFamily,
} as const;

export type ConnectorFont = keyof typeof CONNECTOR_FONTS;
