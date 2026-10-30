import type { Project, Item } from "../generated/types";
import { isCoreClip } from "../clips/motion.ts";
import { object } from "./json.ts";

/** Mirrors the server's explicit native_adapter registry. No file path is loaded. */
export function registeredClip(project: Project, item: Item) {
  if (item.type !== "clip") return null;
  if (!item.derivedId) return isCoreClip(item.clipId) ? item.clipId : null;
  const entry = project.derived
    .map(object)
    .find((e) => e.id === item.derivedId);
  if (
    entry?.sourceAsset === "rendercomp-free/glitch-text" &&
    entry.nativeAdapter === "glitch-text-v1" &&
    (!item.clipId || item.clipId === "glitch-text")
  )
    return "glitch-text" as const;
  return null;
}
/** Only native clips are registered; derived files on disk are not an executable registry. */
export function unsupportedAssetLayers(project: Project): string[] {
  return project.tracks.flatMap((track) =>
    track.items
      .filter((item) => item.type === "clip" && !registeredClip(project, item))
      .map((item) => item.id),
  );
}
