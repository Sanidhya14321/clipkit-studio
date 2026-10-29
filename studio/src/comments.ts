// Pure comment logic for the studio: the anchor a new comment gets from what the user points at,
// and a short label for an anchor. No React, no fetches.
import type { Anchor, Project } from "./generated/types";

export type AnchorKind = "moment" | "range" | "layer" | "element";

export interface Pointer {
  playhead: number;
  loopRange: [number, number] | null;
  selection: string | null;
  /** Hovered or clicked element: its layer and the part path inside it. */
  element: { layerId: string; path: string | null } | null;
}

/** The anchor kinds available right now (range needs a loop, layer a selection, ...). */
export function kinds(p: Pointer): AnchorKind[] {
  const out: AnchorKind[] = ["moment"];
  if (p.loopRange) out.push("range");
  if (p.selection) out.push("layer");
  if (p.element?.path) out.push("element");
  return out;
}

const round = (t: number) => Math.round(t * 1000) / 1000;

export function anchorFor(kind: AnchorKind, p: Pointer): Anchor {
  switch (kind) {
    case "moment":
      return { tStart: round(p.playhead) };
    case "range":
      return p.loopRange
        ? { tStart: round(p.loopRange[0]), tEnd: round(p.loopRange[1]) }
        : { tStart: round(p.playhead) };
    case "layer":
      return p.selection ? { layerId: p.selection } : {};
    case "element":
      return p.element
        ? {
            layerId: p.element.layerId,
            ...(p.element.path ? { propPath: p.element.path } : {}),
          }
        : {};
  }
}

export function anchorLabel(project: Project, a: Anchor): string {
  const parts: string[] = [];
  if (a.layerId) {
    const name = project.tracks
      .flatMap((t) => t.items)
      .find((i) => i.id === a.layerId)?.name;
    parts.push(name || a.layerId);
  }
  if (a.propPath) parts.push(a.propPath);
  if (a.tStart != null) {
    parts.push(
      a.tEnd != null
        ? `${a.tStart.toFixed(1)}–${a.tEnd.toFixed(1)}s`
        : `${a.tStart.toFixed(1)}s`,
    );
  }
  return parts.join(" · ") || "project";
}
