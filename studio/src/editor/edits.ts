import type {
  Item,
  JsonValue,
  PatchRequest,
  Project,
  TrackKind,
} from "../generated/types";
import { locate } from "../state.ts";
import { span } from "../timeline.ts";

export function editRequest(
  project: Project,
  label: string,
  ops: JsonValue[],
  ids: string[] = [],
): PatchRequest {
  return {
    baseRev: project.rev,
    layerRevs: Object.fromEntries(
      ids.flatMap((id) => {
        const found = locate(project, id);
        return found ? [[id, found.item.rev]] : [];
      }),
    ),
    author: "human",
    label,
    ops,
  };
}
export function itemRequest(
  project: Project,
  id: string,
  values: Record<string, JsonValue>,
  label = "Edit layer",
): PatchRequest | null {
  const found = locate(project, id);
  if (!found) return null;
  return editRequest(
    project,
    label,
    Object.entries(values).map(([key, value]) => ({
      op: key in found.item ? "replace" : "add",
      path: `/tracks/${found.trackIndex}/items/${found.itemIndex}/${key}`,
      value,
    })),
    [id],
  );
}
export function moveRequest(
  project: Project,
  id: string,
  at: number,
): PatchRequest | null {
  if (!Number.isFinite(at) || at < 0) return null;
  const item = locate(project, id)?.item;
  if (!item || !("at" in item)) return null;
  return itemRequest(project, id, { at }, "Move layer");
}
export function trimRequest(
  project: Project,
  id: string,
  edge: "start" | "end",
  time: number,
): PatchRequest | null {
  const item = locate(project, id)?.item;
  if (!item) return null;
  const range = span(project, item);
  if (!range || !Number.isFinite(time) || time < 0) return null;
  const minimum = 1 / project.settings.fps;
  if (item.type === "clip") {
    if (edge === "start" && time <= range.end - minimum)
      return itemRequest(
        project,
        id,
        { at: time, dur: range.end - time },
        "Trim layer start",
      );
    if (edge === "end" && time >= range.start + minimum)
      return itemRequest(
        project,
        id,
        { dur: time - range.start },
        "Trim layer end",
      );
  }
  if (item.type === "footage") {
    const source = project.sources.find((s) => s.id === item.src);
    if (edge === "start" && time <= range.end - minimum) {
      const srcIn = item.srcIn + (time - item.at) * item.speed;
      if (srcIn >= 0)
        return itemRequest(
          project,
          id,
          { at: time, srcIn },
          "Trim footage start",
        );
    }
    if (edge === "end" && time >= range.start + minimum) {
      const srcOut = item.srcIn + (time - item.at) * item.speed;
      if (source?.durationSec != null && srcOut > source.durationSec)
        return null;
      return itemRequest(project, id, { srcOut }, "Trim footage end");
    }
  }
  return null;
}
export function removeRequest(
  project: Project,
  id: string,
  ripple = false,
): PatchRequest | null {
  const found = locate(project, id);
  if (!found) return null;
  const range = span(project, found.item);
  const ids = [id];
  const ops: JsonValue[] = [];
  if (ripple && range)
    for (const item of project.tracks[found.trackIndex]!.items) {
      if (item.id !== id && "at" in item && item.at >= range.end) {
        const index = project.tracks[found.trackIndex]!.items.indexOf(item);
        ids.push(item.id);
        ops.push({
          op: "replace",
          path: `/tracks/${found.trackIndex}/items/${index}/at`,
          value: Math.max(0, item.at - (range.end - range.start)),
        });
      }
    }
  ops.push({
    op: "remove",
    path: `/tracks/${found.trackIndex}/items/${found.itemIndex}`,
  });
  return editRequest(
    project,
    ripple ? "Ripple delete layer" : "Delete layer",
    ops,
    ids,
  );
}
export function uniqueId(project: Project, prefix: string): string {
  const ids = new Set(project.tracks.flatMap((t) => t.items.map((i) => i.id)));
  let index = 1;
  while (ids.has(`${prefix}-${index}`)) index++;
  return `${prefix}-${index}`;
}
export function addItemRequest(
  project: Project,
  kind: TrackKind,
  item: Item,
  label: string,
): PatchRequest {
  const index = project.tracks.findIndex((t) => t.kind === kind);
  if (index >= 0)
    return editRequest(project, label, [
      {
        op: "add",
        path: `/tracks/${index}/items/-`,
        value: item as unknown as JsonValue,
      },
    ]);
  let trackId = `${kind}-1`,
    n = 1;
  while (project.tracks.some((t) => t.id === trackId))
    trackId = `${kind}-${++n}`;
  return editRequest(project, label, [
    {
      op: "add",
      path: "/tracks/-",
      value: { id: trackId, kind, items: [item as unknown as JsonValue] },
    },
  ]);
}
export function titleRequest(
  project: Project,
  at: number,
  text: string,
): PatchRequest {
  return addItemRequest(
    project,
    "overlay",
    {
      id: uniqueId(project, "title"),
      name: text,
      purpose: "Title",
      briefDeps: [],
      rev: 0,
      type: "clip",
      clipId: "kinetic-text",
      at,
      dur: 3,
      props: { text, color: "$colors.text", font: "$fonts.display" },
    },
    "Add title",
  );
}
export function duplicateRequest(
  project: Project,
  id: string,
): PatchRequest | null {
  const found = locate(project, id);
  if (!found || !("at" in found.item)) return null;
  const range = span(project, found.item);
  if (!range) return null;
  return addItemRequest(
    project,
    project.tracks[found.trackIndex]!.kind,
    {
      ...found.item,
      id: uniqueId(project, `${id}-copy`),
      name: `${found.item.name || id} copy`,
      at: range.end,
      rev: 0,
    },
    "Duplicate layer",
  );
}
export function snapTime(
  value: number,
  points: number[],
  fps: number,
  threshold: number,
): number {
  const nearest = points.reduce(
    (best, point) =>
      Math.abs(point - value) < Math.abs(best - value) ? point : best,
    Infinity,
  );
  return Math.max(
    0,
    Math.round(
      (Math.abs(nearest - value) <= threshold ? nearest : value) * fps,
    ) / fps,
  );
}
// Only the patch operations our command builders emit are applied optimistically.
export function optimisticProject(
  project: Project,
  req: PatchRequest,
): Project {
  const next = structuredClone(project);
  for (const raw of req.ops) {
    if (
      !raw ||
      typeof raw !== "object" ||
      Array.isArray(raw) ||
      typeof raw.path !== "string"
    )
      throw new Error("Invalid editor patch");
    const parts = raw.path
      .split("/")
      .slice(1)
      .map((s) => s.replace(/~1/g, "/").replace(/~0/g, "~"));
    let parent: unknown = next;
    for (const key of parts.slice(0, -1))
      parent = (parent as Record<string, unknown>)[key];
    const key = parts.at(-1)!;
    if (Array.isArray(parent)) {
      const index = key === "-" ? parent.length : Number(key);
      if (raw.op === "add") parent.splice(index, 0, raw.value);
      else if (raw.op === "remove") parent.splice(index, 1);
      else if (raw.op === "replace") parent[index] = raw.value;
      else throw new Error("Unsupported editor patch");
    } else if (parent && typeof parent === "object") {
      if (raw.op === "remove") delete (parent as Record<string, unknown>)[key];
      else if (raw.op === "replace" || raw.op === "add")
        (parent as Record<string, unknown>)[key] = raw.value;
      else throw new Error("Unsupported editor patch");
    } else throw new Error("The layer changed. Select it again.");
  }
  return next;
}
