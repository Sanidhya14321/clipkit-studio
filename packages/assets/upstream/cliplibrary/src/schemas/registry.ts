import type { z } from 'zod';
import type { ClipId } from './clip-props';
import type { ClipControls } from './control-layout';
import type { ClipCategory, ClipTag, ClipTheme } from './taxonomy';

export type ClipMeta = {
  id: ClipId;
  title: string;
  description: string;
  category: ClipCategory;
  tags: readonly ClipTag[];
  theme: ClipTheme;
  source: string;
  durationInFrames: number;
  posterFrame: number;
  fps: number;
  width: number;
  height: number;
};

export type ClipModule = {
  id: ClipId;
  schema: z.ZodType;
  defaults: unknown;
  meta: ClipMeta;
  controls: ClipControls;
};
