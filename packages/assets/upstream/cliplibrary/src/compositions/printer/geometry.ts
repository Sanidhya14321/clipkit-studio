import type { PrinterPagesProps } from '@/schemas';
import { PAGE_RATIO, SLOT_DEPTH } from './page-motion';

export type PrinterColors = PrinterPagesProps['printer'];

export type PrinterGeometry = {
  pageWidth: number;
  pageHeight: number;
  across: number;
  along: number;
  bodyWidth: number;
  bodyDepth: number;
  front: number;
  slotWidth: number;
  slotTop: number;
  deck: number;
  trayWidth: number;
  trayEnd: number;
};

const SIDE = 250;

export const printerGeometry = (pageWidth: number): PrinterGeometry => {
  const pageHeight = Math.round(pageWidth * PAGE_RATIO);
  const across = pageHeight;
  const along = pageWidth;
  return {
    pageWidth,
    pageHeight,
    across,
    along,
    bodyWidth: across + SIDE * 2,
    bodyDepth: 2600,
    front: 8,
    slotWidth: across + 64,
    slotTop: -SLOT_DEPTH - 4,
    deck: -120,
    trayWidth: across + 150,
    trayEnd: along + 190,
  };
};

export const svgId = (raw: string): string => raw.replace(/[^a-zA-Z0-9_-]/g, '');
