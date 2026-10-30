// Pure helpers the core clips share: prop readers, look easings, enter/exit progress, stagger,
// counters and the solver's box. Times are seconds into the clip; no clocks, no fetches, so a
// frame always renders the same.

export type Props = Record<string, unknown>;

/** Core clip ids (mirrors `CORE_CLIPS` in crates/ck-assets/src/recipe.rs). */
export const CORE_CLIPS = [
  "backdrop",
  "bounce-in-headline",
  "bullet-list",
  "crt-scanlines",
  "device-frame",
  "glitch-text",
  "gradient-text-sweep",
  "kinetic-slam",
  "kinetic-text",
  "lower-third",
  "particle-snow",
  "quote-card",
  "scramble-text",
  "slide-wipe",
  "split-compare",
  "starfield",
  "stat-counter",
  "transition-circle-wipe",
  "typewriter",
  "wave-text",
] as const;
export type CoreClip = (typeof CORE_CLIPS)[number];

export function isCoreClip(id: unknown): id is CoreClip {
  return (
    typeof id === "string" && (CORE_CLIPS as readonly string[]).includes(id)
  );
}

/** Enter time used when a look gives no duration (seconds). */
export const ENTER_SEC = 0.4;
/** Exit time at the end of a clip (seconds). */
export const EXIT_SEC = 0.25;

export function str(props: Props, key: string, fallback = ""): string {
  const v = props[key];
  return typeof v === "string"
    ? v
    : typeof v === "number"
      ? String(v)
      : fallback;
}

export function num(props: Props, key: string, fallback: number): number {
  const v = props[key];
  if (typeof v === "number" && Number.isFinite(v)) return v;
  if (typeof v === "string" && v.trim() !== "") {
    const n = Number(v);
    if (Number.isFinite(n)) return n;
  }
  return fallback;
}

export function list(props: Props, key: string): string[] {
  const v = props[key];
  if (Array.isArray(v))
    return v.filter((x): x is string => typeof x === "string");
  return typeof v === "string" && v !== "" ? [v] : [];
}

/** The look recipe layers fall back to (mirrors `DEFAULT_LOOK` in ck-server recipe apply). */
export const DEFAULT_LOOK = "clean-product";

/**
 * The project's look id (the rule of `project_look_id` in crates/ck-server/src/session/look.rs):
 * the stored `tokens.look`, else the brief's `look` field, else the default.
 */
export function lookId(project: {
  tokens?: Record<string, unknown>;
  brief: { fields: Record<string, { value: unknown } | undefined> };
}): string {
  for (const v of [project.tokens?.look, project.brief.fields.look?.value]) {
    if (typeof v === "string" && v !== "") return v;
  }
  return DEFAULT_LOOK;
}

/**
 * Resolves `"$role.name"` token refs in layer props against a look (the rule of
 * `token_value` in crates/ck-assets/src/recipe.rs: `size` reads the look's top level). Props
 * keep the refs, so a re-theme swaps the look without touching layers. Unresolved refs are
 * dropped, so a clip uses its own default instead of an invalid value.
 */
export function resolveTokens(props: Props, look: object | undefined): Props {
  const out: Props = {};
  for (const [k, v] of Object.entries(props)) {
    if (typeof v !== "string" || !v.startsWith("$")) {
      out[k] = v;
      continue;
    }
    const [role, name] = v.slice(1).split(".", 2);
    const src = (
      role === "size"
        ? look
        : (look as Record<string, unknown> | undefined)?.[role ?? ""]
    ) as Record<string, unknown> | undefined;
    const r = name !== undefined ? src?.[name] : undefined;
    if (typeof r === "string" || typeof r === "number") out[k] = r;
  }
  return out;
}

type Ease = (x: number) => number;

/** The easing names looks use (`motion.enterEasing`, `motion.exitEasing`). */
export const EASINGS: Record<string, Ease> = {
  linear: (x) => x,
  "quad-in": (x) => x * x,
  "quad-out": (x) => 1 - (1 - x) * (1 - x),
  "cubic-in": (x) => x * x * x,
  "cubic-out": (x) => 1 - (1 - x) ** 3,
  "expo-out": (x) => (x >= 1 ? 1 : 1 - 2 ** (-10 * x)),
  "sine-in-out": (x) => -(Math.cos(Math.PI * x) - 1) / 2,
};

export function easing(name: unknown): Ease {
  return (typeof name === "string" && EASINGS[name]) || EASINGS["cubic-out"]!;
}

const clamp01 = (x: number) => Math.min(1, Math.max(0, x));

/**
 * Visibility of a clip part at `t` seconds into a clip of `dur` seconds: eases in over
 * ENTER_SEC after `delay`, holds at 1, eases out over the last EXIT_SEC. Always in [0, 1].
 */
export function presence(
  t: number,
  dur: number,
  props: Props,
  delay = 0,
): number {
  const enter = easing(props.easing ?? props.enter)(
    clamp01((t - delay) / ENTER_SEC),
  );
  const exitStart = Math.max(0, dur - EXIT_SEC);
  const exit =
    t <= exitStart
      ? 1
      : 1 -
        easing(props.exit ?? "quad-in")(clamp01((t - exitStart) / EXIT_SEC));
  return clamp01(Math.min(enter, exit));
}

/** Delay of the `i`th staggered part (`stagger` is seconds per step, from the look). */
export function staggerDelay(props: Props, i: number): number {
  return Math.max(0, num(props, "stagger", 0.08)) * i;
}

/**
 * The counted value at `t`: rises from 0 to the target over the first 60% of the clip with the
 * enter easing, keeping the target's decimals. Non-numbers are shown as written.
 */
export function counterText(
  value: string,
  t: number,
  dur: number,
  props: Props,
): string {
  const m = /^(-?)(\d+(?:\.(\d+))?)$/.exec(value.trim());
  if (!m) return value;
  const target = Number(m[2]);
  const decimals = m[3]?.length ?? 0;
  const p = easing(props.easing)(clamp01(t / Math.max(0.001, dur * 0.6)));
  return `${m[1]}${(target * p).toFixed(decimals)}`;
}

export type Box = { x: number; y: number; w: number; h: number };

/** The solver's `box` prop (fractions of the frame), when well formed. */
export function box(props: Props): Box | null {
  const b = props.box;
  if (typeof b !== "object" || b === null) return null;
  const r = b as Record<string, unknown>;
  const vals = [r.x, r.y, r.w, r.h];
  if (!vals.every((v) => typeof v === "number" && Number.isFinite(v)))
    return null;
  return {
    x: r.x as number,
    y: r.y as number,
    w: r.w as number,
    h: r.h as number,
  };
}

/** Absolute CSS for a box, or the fallback region (fractions) when the props have none. */
export function boxStyle(
  props: Props,
  fallback: Box,
): { left: string; top: string; width: string; height: string } {
  const b = box(props) ?? fallback;
  const pct = (v: number) => `${(v * 100).toFixed(3)}%`;
  return { left: pct(b.x), top: pct(b.y), width: pct(b.w), height: pct(b.h) };
}

/** Font size in pixels: the solver's `fontSize` (fraction of height) or a fallback fraction. */
export function fontPx(props: Props, height: number, fallback: number): number {
  return num(props, "fontSize", fallback) * height;
}

/** Lines to draw: the solver's `lines` when present, else the raw text. */
export function textLines(props: Props, key = "text"): string[] {
  const lines = list(props, "lines");
  return lines.length > 0 ? lines : [str(props, key)].filter((s) => s !== "");
}
