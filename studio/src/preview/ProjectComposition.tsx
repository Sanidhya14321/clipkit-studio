// The preview composition: a pure function of the project (no fetches, no clocks). Media
// rendering from proxies and component clips arrive with their phases; this shell draws each
// visible layer as a labelled box so every layer is present and addressable by `data-ck-layer`.
import { AbsoluteFill, useCurrentFrame, useVideoConfig } from "remotion";
import type { Item, Project, TrackKind } from "../generated/types";
import { span } from "../timeline";

export interface CompositionProps extends Record<string, unknown> {
  project: Project;
}

const KIND_COLOR: Record<TrackKind, string> = {
  video: "#2d4a6b",
  overlay: "#5a3d6b",
  audio: "#2f5d46",
  caption: "#6b5a2d",
};

export function ProjectComposition({ project }: CompositionProps) {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const t = frame / fps;

  return (
    <AbsoluteFill style={{ backgroundColor: "#111" }}>
      {project.tracks.map((track, ti) =>
        track.items
          .filter((item) => isVisibleAt(project, item, t))
          .map((item) => (
            <div
              key={item.id}
              data-ck-layer={item.id}
              style={{
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
          )),
      )}
    </AbsoluteFill>
  );
}

function isVisibleAt(project: Project, item: Item, t: number): boolean {
  const s = span(project, item);
  return s !== null && t >= s.start && t < s.end;
}
