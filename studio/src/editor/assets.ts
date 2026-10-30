import type { Project } from "../generated/types";
import { isCoreClip } from "../clips/motion.ts";
/** Only native clips are registered; derived files on disk are not an executable registry. */
export function unsupportedAssetLayers(project: Project): string[] {
  return project.tracks.flatMap((track) =>
    track.items
      .filter(
        (item) =>
          item.type === "clip" &&
          (Boolean(item.derivedId) || !isCoreClip(item.clipId)),
      )
      .map((item) => item.id),
  );
}
