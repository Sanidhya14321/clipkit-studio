// Pure Brief page logic (spec 8.1): the essentials, the pacing scale and the confirm patch.
// No React, no DOM, no fetches, so `node --test` runs it directly (see brief.test.ts).
// The agent proposes inferred values; the studio renders them, the user adjusts, and the confirm
// is one human patch on `/brief` that marks edited fields as `user` and sets the status.
import type {
  Brief,
  BriefField,
  JsonValue,
  PatchRequest,
  Project,
} from "./generated/types";

/** Essentials above the fold, in display order (spec 8.1). */
export const ESSENTIALS = [
  "goal",
  "cta",
  "audience",
  "platform",
  "aspect",
  "length",
  "pacing",
  "look",
  "voice",
] as const;
export type Essential = (typeof ESSENTIALS)[number];

export const ESSENTIAL_LABEL: Record<Essential, string> = {
  goal: "Goal",
  cta: "Call to action",
  audience: "Audience",
  platform: "Platform",
  aspect: "Aspect",
  length: "Length (s)",
  pacing: "Pacing",
  look: "Look",
  voice: "Voice",
};

/** Voice choices of spec 8.1. */
export const VOICE_OPTIONS = ["ai", "recording", "cloned", "none"] as const;

/** One pacing level and the numbers it maps to (assets/taste/pacing.md). */
export interface PacingLevel {
  level: number;
  name: string;
  avgShot: number;
  minShot: number;
  maxShot: number;
  cutsPer15s: string;
}

/** Kept in step with the table in assets/taste/pacing.md; a test checks the file. */
export const PACING: PacingLevel[] = [
  {
    level: 1,
    name: "calm",
    avgShot: 4.5,
    minShot: 2.5,
    maxShot: 8.0,
    cutsPer15s: "3 to 4",
  },
  {
    level: 2,
    name: "steady",
    avgShot: 3.2,
    minShot: 1.8,
    maxShot: 6.0,
    cutsPer15s: "4 to 5",
  },
  {
    level: 3,
    name: "lively",
    avgShot: 2.2,
    minShot: 1.2,
    maxShot: 4.0,
    cutsPer15s: "6 to 7",
  },
  {
    level: 4,
    name: "punchy",
    avgShot: 1.5,
    minShot: 0.8,
    maxShot: 3.0,
    cutsPer15s: "9 to 10",
  },
  {
    level: 5,
    name: "rapid",
    avgShot: 1.0,
    minShot: 0.5,
    maxShot: 2.0,
    cutsPer15s: "14 to 15",
  },
];

/** The middle level when the proposed value is missing or not on the scale. */
export const DEFAULT_PACING = 3;

/** Reads a proposed pacing value: a level number or a level name; anything else is the default. */
export function pacingLevel(value: JsonValue | undefined): number {
  if (typeof value === "number" && PACING.some((p) => p.level === value))
    return value;
  if (typeof value === "string") {
    const hit = PACING.find((p) => p.name === value.toLowerCase());
    if (hit) return hit.level;
  }
  return DEFAULT_PACING;
}

/** The text a field shows in an input; objects and arrays show as JSON. */
export function display(value: JsonValue | undefined): string {
  if (value === undefined || value === null) return "";
  if (typeof value === "string") return value;
  if (typeof value === "number" || typeof value === "boolean")
    return String(value);
  return JSON.stringify(value);
}

/** User edits held in the form until confirm: field name to new value. */
export type Edits = Record<string, JsonValue>;

/**
 * The confirm patch: every edited field becomes `{value, source: "user", confidence: 1}`,
 * answered questions get their answer, and the status becomes `confirmed`. Unedited fields keep
 * their source. Built against the revision the studio last saw, so a change it has not seen
 * makes the confirm stale (E_PATCH_STALE) instead of overwriting the agent's proposal.
 */
export function confirmPatch(
  project: Project,
  edits: Edits,
  answers: Record<string, string> = {},
): PatchRequest {
  const brief: Brief = project.brief;
  const ops: JsonValue[] = [];
  for (const [name, value] of Object.entries(edits)) {
    if (sameValue(brief.fields[name]?.value, value)) continue;
    const field: BriefField = { value, source: "user", confidence: 1 };
    ops.push({
      op: "add",
      path: `/brief/fields/${pointer(name)}`,
      value: field,
    });
  }
  for (const [i, q] of (brief.questions ?? []).entries()) {
    const answer = answers[q.id]?.trim();
    if (answer)
      ops.push({
        op: "add",
        path: `/brief/questions/${i}/answer`,
        value: answer,
      });
  }
  ops.push({ op: "replace", path: "/brief/status", value: "confirmed" });
  return {
    baseRev: project.rev,
    ops,
    author: "human",
    label: "Confirm brief",
  };
}

/** Escapes one JSON Pointer segment (RFC 6901). */
function pointer(segment: string): string {
  return segment.replaceAll("~", "~0").replaceAll("/", "~1");
}

function sameValue(a: JsonValue | undefined, b: JsonValue): boolean {
  return JSON.stringify(a) === JSON.stringify(b);
}
