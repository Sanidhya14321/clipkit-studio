import { z } from 'zod';

export type ControlMeta =
  | { control: 'color' }
  | { control: 'slider'; step: number }
  | { control: 'textarea' }
  | { control: 'image' }
  | { control: 'video' };

export const HEX_COLOR = /^#([0-9a-fA-F]{6}|[0-9a-fA-F]{8})$/;

export const zColor = () =>
  z
    .string()
    .regex(HEX_COLOR)
    .meta({ control: 'color' } satisfies ControlMeta);

export const zSlider = (options: { min: number; max: number; step?: number }) =>
  z
    .number()
    .min(options.min)
    .max(options.max)
    .meta({ control: 'slider', step: options.step ?? 1 } satisfies ControlMeta);

export const zMultiline = () => z.string().meta({ control: 'textarea' } satisfies ControlMeta);

export const zImage = () => z.string().meta({ control: 'image' } satisfies ControlMeta);

export const zVideo = () => z.string().meta({ control: 'video' } satisfies ControlMeta);

export const readControlMeta = (schema: z.core.$ZodType): ControlMeta | undefined => {
  const meta = z.globalRegistry.get(schema);
  if (meta === undefined) {
    return undefined;
  }
  const candidate = meta as Partial<ControlMeta>;
  if (
    candidate.control === 'color' ||
    candidate.control === 'textarea' ||
    candidate.control === 'image' ||
    candidate.control === 'video'
  ) {
    return { control: candidate.control };
  }
  if (candidate.control === 'slider') {
    return { control: 'slider', step: typeof candidate.step === 'number' ? candidate.step : 1 };
  }
  return undefined;
};
