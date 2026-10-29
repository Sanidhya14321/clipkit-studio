// Brief tuning chips (spec 5.4, 8.1): one click turns the built video punchier, shorter, calmer
// or more premium by editing brief fields as the user. Each chip shows its impact first: the
// layers whose briefDeps name a changed field, which are the only ones the agent re-plans.
// "More premium" also switches the stored look, so it re-themes through tokens at once.
import type {
  BriefField,
  JsonValue,
  PatchRequest,
  Project,
} from "./generated/types";
import { PACING, pacingLevel } from "./brief.ts";

export const CHIPS = ["punchier", "shorter", "calmer", "premium"] as const;
export type Chip = (typeof CHIPS)[number];

export const CHIP_LABEL: Record<Chip, string> = {
  punchier: "Punchier",
  shorter: "Shorter",
  calmer: "Calmer",
  premium: "More premium",
};

/** The look "more premium" moves to. */
export const PREMIUM_LOOK = "cinematic";
/** "Shorter" keeps this share of the length. */
export const SHORTER = 0.8;

/** The brief field edits a chip makes, or null when it changes nothing (already at the end). */
export function chipEdits(
  project: Project,
  chip: Chip,
): Record<string, JsonValue> | null {
  const f = project.brief.fields;
  const pace = pacingLevel(f.pacing?.value);
  const max = PACING.length;
  switch (chip) {
    case "punchier":
      return pace < max ? { pacing: pace + 1 } : null;
    case "calmer":
      return pace > 1 ? { pacing: pace - 1 } : null;
    case "shorter": {
      const len = f.length?.value;
      if (typeof len !== "number" || len <= 1) return null;
      return { length: Math.max(1, Math.round(len * SHORTER)) };
    }
    case "premium":
      return f.look?.value === PREMIUM_LOOK ? null : { look: PREMIUM_LOOK };
  }
}

/** Layers whose briefDeps name any of `fields`, in track order. */
export function impacted(project: Project, fields: string[]): string[] {
  return project.tracks
    .flatMap((t) => t.items)
    .filter((i) => i.briefDeps.some((d) => fields.includes(d)))
    .map((i) => i.id);
}

/** The human patch for a chip: user-sourced fields, plus the stored look for "premium". */
export function chipPatch(
  project: Project,
  chip: Chip,
  edits: Record<string, JsonValue>,
): PatchRequest {
  const ops: JsonValue[] = Object.entries(edits).map(([name, value]) => {
    const field: BriefField = { value, source: "user", confidence: 1 };
    return { op: "add", path: `/brief/fields/${name}`, value: field };
  });
  if (typeof edits.look === "string") {
    ops.push({ op: "add", path: "/tokens/look", value: edits.look });
  }
  return {
    baseRev: project.rev,
    ops,
    author: "human",
    label: `Tune: ${CHIP_LABEL[chip].toLowerCase()}`,
  };
}
