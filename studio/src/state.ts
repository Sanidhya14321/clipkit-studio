// Pure studio logic: the event reducer, lock liveness and the patches human edits send.
// No React, no DOM, no fetches, so `node --test` runs it directly (see state.test.ts).
// The server is the source of truth: the project comes only from `snapshot` events, locks from
// `locks` events, and `revision` events say who changed what (activity feed and flash).
import type {
  Author,
  Event,
  Item,
  JsonValue,
  Lock,
  PatchRequest,
  Project,
} from "./generated/types";

export type Revision = Extract<Event, { type: "revision" }>;

/** One line of the activity feed per revision. */
export interface FeedEntry {
  rev: number;
  author: Author;
  label: string;
  touched: string[];
}

/** The feed keeps the newest entries only; older steps live in the project history. */
export const FEED_LIMIT = 50;

export interface StudioState {
  connected: boolean;
  project: Project | null;
  locks: Lock[];
  lastRevision: Revision | null;
  feed: FeedEntry[];
}

export const initial: StudioState = {
  connected: false,
  project: null,
  locks: [],
  lastRevision: null,
  feed: [],
};

export function reduce(state: StudioState, event: Event): StudioState {
  switch (event.type) {
    case "snapshot":
      // The snapshot carries the engine's lock list too; it is as fresh as any `locks` event.
      return { ...state, project: event.project, locks: event.project.locks };
    case "locks":
      return { ...state, locks: event.locks };
    case "revision": {
      const entry: FeedEntry = {
        rev: event.rev,
        author: event.author,
        label: event.label,
        touched: event.touched,
      };
      const feed = [entry, ...state.feed].slice(0, FEED_LIMIT);
      return { ...state, lastRevision: event, feed };
    }
  }
}

/** Mirrors `ck_core::model::Lock::is_live`: a lock lapses once its heartbeat is older than its TTL. */
export function isLive(lock: Lock, nowMs: number): boolean {
  return nowMs < lock.heartbeatAtMs + lock.ttlMs;
}

/** Layer ids the agent holds a live lock on. */
export function agentLocked(locks: Lock[], nowMs: number): Set<string> {
  return new Set(
    locks
      .filter((l) => l.holder === "agent" && isLive(l, nowMs))
      .map((l) => l.layerId),
  );
}

/**
 * True when the agent had locks and every one of them has lapsed without a release: the agent
 * process stopped heartbeating (spec 5.3 crash safety).
 */
export function agentStopped(locks: Lock[], nowMs: number): boolean {
  const agent = locks.filter((l) => l.holder === "agent");
  return agent.length > 0 && agent.every((l) => !isLive(l, nowMs));
}

/**
 * Whether a human may edit the layer. Matches the server's lock gate for an edit that touches
 * only this layer (`ck_core::locks::check_patch`): refused while the agent holds it. The server
 * still refuses on its own; this only disables the controls.
 */
export function canEdit(
  locks: Lock[],
  layerId: string,
  nowMs: number,
): boolean {
  return !agentLocked(locks, nowMs).has(layerId);
}

/** Lock id for project-level content such as the brief (`ck_core::patch::PROJECT_LAYER`). */
export const PROJECT_LAYER = "@project";

export interface Located {
  item: Item;
  trackIndex: number;
  itemIndex: number;
}

export function locate(project: Project, layerId: string): Located | null {
  for (const [trackIndex, track] of project.tracks.entries()) {
    const itemIndex = track.items.findIndex((i) => i.id === layerId);
    const item = track.items[itemIndex];
    if (item) return { item, trackIndex, itemIndex };
  }
  return null;
}

/** Item fields the inspector edits in Phase 2. */
export type EditableField = "name" | "purpose" | "at" | "dur";

export function editableFields(item: Item): EditableField[] {
  const fields: EditableField[] = ["name", "purpose"];
  if ("at" in item) fields.push("at");
  if (item.type === "clip") fields.push("dur");
  return fields;
}

/**
 * The patch for one human field edit: a single `replace` built against the revisions the
 * studio last saw, so a change the studio has not seen yet makes it stale (E_PATCH_STALE) instead of
 * overwriting it. Null when the layer is gone or the field does not apply.
 */
export function fieldPatch(
  project: Project,
  layerId: string,
  field: EditableField,
  value: string | number,
): PatchRequest | null {
  const at = locate(project, layerId);
  if (!at || !editableFields(at.item).includes(field)) return null;
  const op: JsonValue = {
    op: "replace",
    path: `/tracks/${at.trackIndex}/items/${at.itemIndex}/${field}`,
    value,
  };
  return {
    baseRev: project.rev,
    layerRevs: { [layerId]: at.item.rev },
    ops: [op],
    author: "human",
    label: `Edit ${field} of ${at.item.name || layerId}`,
  };
}
