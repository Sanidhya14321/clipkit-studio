type LeafValue = string | number | boolean;

export type ControlPath<T> = {
  [Key in keyof T & string]: T[Key] extends readonly unknown[]
    ? Key
    : T[Key] extends LeafValue
      ? Key
      : `${Key}.${ControlPath<T[Key]>}`;
}[keyof T & string];

export type ControlField<Path extends string = string> =
  | Path
  | { path: Path; label?: string; hint?: string; items?: Record<string, string> }
  | { link: readonly [Path, Path, ...Path[]]; label: string; hint?: string };

export type ControlSection<Path extends string = string> = {
  title: string;
  fields: readonly ControlField<Path>[];
  more?: readonly ControlField<Path>[];
};

export type ClipControls<Path extends string = string> = {
  sections: readonly ControlSection<Path>[];
  hidden?: readonly Path[];
};

export const defineControls = <Props>(
  controls: ClipControls<ControlPath<Props>>,
): ClipControls<ControlPath<Props>> => controls;
