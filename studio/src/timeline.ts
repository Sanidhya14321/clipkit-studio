// Item timing, mirroring `ck_core::timeline::span` and `duration` (crates/ck-core/src/timeline.rs).
// Seconds everywhere; frames only at the Player boundary.
import type { Item, Project, TimeRange } from "./generated/types";

export function span(project: Project, item: Item): TimeRange | null {
  let start: number;
  let len: number;
  switch (item.type) {
    case "footage":
      start = item.at;
      len = (item.srcOut - item.srcIn) / item.speed;
      break;
    case "clip":
      start = item.at;
      len = item.dur;
      break;
    case "voice": {
      const slot = project.voiceSlots.find((s) => s.id === item.slotId);
      const take = slot?.takes.find((t) => t.id === slot.activeTake);
      if (!take) return null;
      start = item.at;
      len = take.durationSec;
      break;
    }
    case "audio":
      start = item.at;
      len = item.ranges.reduce((sum, r) => sum + (r.end - r.start), 0);
      break;
    case "caption": {
      if (item.from === "voice") {
        const voice = project.tracks
          .flatMap((t) => t.items)
          .find((i) => i.type === "voice" && i.slotId === item.ref);
        if (voice) return span(project, voice);
      }
      return null;
    }
  }
  if (!Number.isFinite(start) || !Number.isFinite(len) || len < 0) return null;
  return { start, end: start + len };
}

export function duration(project: Project): number {
  let end = 0;
  for (const track of project.tracks) {
    for (const item of track.items) {
      const s = span(project, item);
      if (s && s.end > end) end = s.end;
    }
  }
  return end;
}

/** Frames for the Player: at least one, so an empty project still mounts. */
export function durationInFrames(project: Project): number {
  return Math.max(1, Math.ceil(duration(project) * project.settings.fps));
}

/** Proxy URL of every video source that has one. A `<video>` cannot send headers, so the
 * session token rides in the query (the server accepts it there only for `/media/`). */
export function mediaUrls(
  project: Project | null,
  token: string,
): Record<string, string> {
  const out: Record<string, string> = {};
  for (const s of project?.sources ?? []) {
    if (s.kind === "video" && s.proxyPath) {
      out[s.id] = `/media/${s.hash}.mp4?token=${encodeURIComponent(token)}`;
    } else if (s.kind === "image") {
      out[s.id] = `/media/source/${s.hash}?token=${encodeURIComponent(token)}`;
    }
  }
  return out;
}
