// The preview composition: a pure function of the project (no fetches, no clocks). Core clip
// items render through their native clip (src/clips); other layers (media from proxies arrives
// with its phase) draw as a labelled box. Every layer is addressable by `data-ck-layer`.
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import type { Item, Look, Project, TrackKind } from "../generated/types";
import { span } from "../timeline";
import { CLIPS } from "../clips/clips";
import { type Props, isCoreClip, lookId, resolveTokens } from "../clips/motion";

export interface CompositionProps extends Record<string, unknown> {
  project: Project;
  /** Built-in looks; clip props resolve their token refs against the project's look. */
  looks: Look[];
  /** Layers the agent is working on: drawn with the selection outline. */
  agentLayers: string[];
  /** Layers the last patch touched, and that patch's rev (a new rev restarts the flash). */
  flash: string[];
  flashRev: number;
}

const KIND_COLOR: Record<TrackKind, string> = {
  video: "#2d4a6b",
  overlay: "#5a3d6b",
  audio: "#2f5d46",
  caption: "#6b5a2d",
};

export function ProjectComposition({
  project,
  looks,
  agentLayers,
  flash,
  flashRev,
}: CompositionProps) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = frame / fps;
  const id = lookId(project.brief);
  const look = looks.find((l) => l.id === id);

  return (
    <AbsoluteFill style={{ backgroundColor: "#111" }}>
      {project.tracks.map((track, ti) =>
        track.items
          .filter((item) => isVisibleAt(project, item, t))
          .map((item) =>
            item.type === "clip" && isCoreClip(item.clipId) ? (
              <div
                key={
                  flash.includes(item.id) ? `${item.id}@${flashRev}` : item.id
                }
                data-ck-layer={item.id}
                data-ck-clip={item.clipId}
                data-ck-agent={agentLayers.includes(item.id)}
                className={flash.includes(item.id) ? "flash" : undefined}
                style={{
                  position: "absolute",
                  inset: 0,
                  outline: agentLayers.includes(item.id)
                    ? "4px dashed #facc15"
                    : undefined,
                  outlineOffset: -4,
                }}
              >
                {CLIPS[item.clipId]({
                  props: resolveTokens(propsOf(item.props), look),
                  t: t - item.at,
                  dur: item.dur,
                  width: project.settings.width,
                  height: project.settings.height,
                })}
              </div>
            ) : (
              <div
                key={
                  flash.includes(item.id) ? `${item.id}@${flashRev}` : item.id
                }
                data-ck-layer={item.id}
                data-ck-agent={agentLayers.includes(item.id)}
                className={flash.includes(item.id) ? "flash" : undefined}
                style={{
                  outline: agentLayers.includes(item.id)
                    ? "4px dashed #facc15"
                    : undefined,
                  outlineOffset: 4,
                  position: "absolute",
                  left: "5%",
                  right: "5%",
                  top: `${8 + ti * 14}%`,
                  height: "11%",
                  background: KIND_COLOR[track.kind],
                  color: "#fff",
                  fontFamily: "system-ui, sans-serif",
                  fontSize: project.settings.height / 30,
                  display: "flex",
                  alignItems: "center",
                  padding: "0 2%",
                  borderRadius: 8,
                  overflow: "hidden",
                  whiteSpace: "nowrap",
                }}
              >
                {item.name || item.id}
              </div>
            ),
          ),
      )}
    </AbsoluteFill>
  );
}

function propsOf(v: unknown): Props {
  return typeof v === "object" && v !== null && !Array.isArray(v)
    ? (v as Props)
    : {};
}

function isVisibleAt(project: Project, item: Item, t: number): boolean {
  const s = span(project, item);
  return s !== null && t >= s.start && t < s.end;
}
