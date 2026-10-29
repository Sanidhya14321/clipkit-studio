// Pure transcript-panel logic: which words the footage segments cut, the delete-to-cut set a
// click sends, and where a source time lands on the timeline. No React, no fetches.
import type { Item, Project, Word } from "./generated/types";

type Footage = Extract<Item, { type: "footage" }>;

/** The footage segments of `sourceId` on every video track, in timeline order. */
export function segments(project: Project, sourceId: string): Footage[] {
  const out: Footage[] = [];
  for (const track of project.tracks) {
    if (track.kind !== "video") continue;
    for (const item of track.items) {
      if (item.type === "footage" && item.src === sourceId) out.push(item);
    }
  }
  return out.sort((a, b) => a.at - b.at);
}

/** Words whose midpoint no segment keeps. With no segments yet, nothing is cut. */
export function deletedWords(
  project: Project,
  sourceId: string,
  words: Word[],
): Set<number> {
  const segs = segments(project, sourceId);
  const out = new Set<number>();
  if (segs.length === 0) return out;
  for (const w of words) {
    const mid = (w.start + w.end) / 2;
    if (!segs.some((s) => mid >= s.srcIn && mid < s.srcOut)) out.add(w.i);
  }
  return out;
}

/** The whole deleted set after toggling word `i` (delete it, or restore it). */
export function toggled(deleted: Set<number>, i: number): number[] {
  const next = new Set(deleted);
  if (next.has(i)) next.delete(i);
  else next.add(i);
  return [...next].sort((a, b) => a - b);
}

/** Timeline time of source time `t`, or null when `t` is cut. Without segments the source
 * plays from 0, so the time is unchanged. */
export function timelineTime(
  project: Project,
  sourceId: string,
  t: number,
): number | null {
  const segs = segments(project, sourceId);
  if (segs.length === 0) return t;
  for (const s of segs) {
    if (t >= s.srcIn && t < s.srcOut) return s.at + (t - s.srcIn) / s.speed;
  }
  return null;
}
