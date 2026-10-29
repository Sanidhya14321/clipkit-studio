import { Player, type PlayerRef } from "@remotion/player";
import { useEffect, useMemo, useRef, useState } from "react";
import { request } from "./api";
import type { Look } from "./generated/types";
import { ActivityFeed } from "./panels/ActivityFeed";
import { Banner } from "./panels/Banner";
import { BriefPanel } from "./panels/BriefPanel";
import { Inspector } from "./panels/Inspector";
import { Timeline } from "./panels/Timeline";
import { TranscriptPanel } from "./panels/TranscriptPanel";
import { CommentsPanel } from "./panels/CommentsPanel";
import { VoicePanel } from "./panels/VoicePanel";
import { HoverSelect, type Target } from "./preview/HoverSelect";
import type { Pointer } from "./comments";
import type { StudioContext } from "./generated/types";
import { ProjectComposition } from "./preview/ProjectComposition";
import { PROJECT_LAYER, agentLocked, agentStopped, canEdit } from "./state";
import { durationInFrames, mediaUrls } from "./timeline";
import { useNow, useStudio } from "./useStudio";

export function App({ token }: { token: string }) {
  const { connected, project, locks, lastRevision, feed } = useStudio(token);
  const now = useNow();
  const [selected, setSelected] = useState<string | null>(null);
  const [looks, setLooks] = useState<Look[]>([]);
  const [looksFailed, setLooksFailed] = useState(false);
  const player = useRef<PlayerRef>(null);
  const contextQueue = useRef<Promise<unknown>>(Promise.resolve());
  const [playhead, setPlayhead] = useState(0);
  const [hovered, setHovered] = useState<Target | null>(null);
  const [element, setElement] = useState<Target | null>(null);
  const fps = project?.settings.fps ?? 30;

  useEffect(() => {
    const p = player.current;
    if (!p) return;
    const onFrame = (e: { detail: { frame: number } }) =>
      setPlayhead(e.detail.frame / fps);
    p.addEventListener("frameupdate", onFrame);
    p.addEventListener("seeked", onFrame);
    return () => {
      p.removeEventListener("frameupdate", onFrame);
      p.removeEventListener("seeked", onFrame);
    };
  }, [fps, project !== null]);

  // Report what the user points at, for `ck context` (debounced; UI state only).
  useEffect(() => {
    const ctx: StudioContext = {
      ...(selected ? { selection: selected } : {}),
      ...(hovered
        ? {
            hovered: {
              layerId: hovered.layerId,
              ...(hovered.path ? { path: hovered.path } : {}),
            },
          }
        : {}),
      playhead: Math.round(playhead * 1000) / 1000,
    };
    const timer = setTimeout(() => {
      // One post at a time, in order, so an older state never lands last.
      contextQueue.current = contextQueue.current.then(() =>
        request(token, "/api/context", {
          method: "POST",
          body: JSON.stringify(ctx),
        }).catch(() => undefined),
      );
    }, 250);
    return () => clearTimeout(timer);
  }, [token, selected, hovered?.layerId, hovered?.path, playhead]);

  const pointer: Pointer = {
    playhead,
    loopRange: null,
    selection: selected,
    element: element ? { layerId: element.layerId, path: element.path } : null,
  };

  useEffect(() => {
    request<Look[]>(token, "/api/looks").then(setLooks, () =>
      setLooksFailed(true),
    );
  }, [token]);

  const media = useMemo(() => mediaUrls(project, token), [project, token]);
  const stopped = agentStopped(locks, now);
  const agentLayers = useMemo(() => {
    const ids = agentLocked(locks, now);
    if (project && project.agent.status !== "idle" && !stopped) {
      for (const id of project.agent.activeLayers) ids.add(id);
    }
    return ids;
  }, [locks, now, project, stopped]);

  return (
    <div className="studio" data-ck-connected={connected}>
      <header className="studio-header">
        <strong>ClipKit Studio</strong>
        {project && (
          <span data-ck-project={project.id}>
            {project.name} · rev <span data-ck-rev>{project.rev}</span>
          </span>
        )}
        {!connected && <span role="status">Connecting to ck…</span>}
      </header>
      {project && <Banner project={project} stopped={stopped} />}
      {project && (
        <div className="studio-body">
          <main className="studio-preview">
            <HoverSelect
              token={token}
              project={project}
              onHover={setHovered}
              onPick={(t) => {
                setElement(t);
                setSelected(t.layerId);
              }}
            >
              <Player
                ref={player}
                component={ProjectComposition}
                inputProps={{
                  project,
                  looks,
                  agentLayers: [...agentLayers],
                  flash: lastRevision?.touched ?? [],
                  flashRev: lastRevision?.rev ?? 0,
                  media,
                }}
                durationInFrames={durationInFrames(project)}
                fps={project.settings.fps}
                compositionWidth={project.settings.width}
                compositionHeight={project.settings.height}
                style={{ width: "100%", maxHeight: "100%" }}
                controls
              />
            </HoverSelect>
          </main>
          <aside className="studio-side">
            <BriefPanel
              token={token}
              project={project}
              readOnly={!canEdit(locks, PROJECT_LAYER, now)}
              looks={looks}
              looksFailed={looksFailed}
            />
            <Inspector
              key={selected ?? ""}
              token={token}
              project={project}
              layerId={selected}
              readOnly={selected !== null && !canEdit(locks, selected, now)}
            />
            <TranscriptPanel
              token={token}
              project={project}
              readOnly={
                !canEdit(
                  locks,
                  project.tracks.find((t) => t.kind === "video")?.id ??
                    PROJECT_LAYER,
                  now,
                )
              }
              onSeek={(seconds) =>
                player.current?.seekTo(
                  Math.round(seconds * project.settings.fps),
                )
              }
            />
            <VoicePanel
              token={token}
              project={project}
              readOnly={!canEdit(locks, PROJECT_LAYER, now)}
            />
            <CommentsPanel token={token} project={project} pointer={pointer} />
            <ActivityFeed feed={feed} />
          </aside>
          <Timeline
            project={project}
            agentLayers={agentLayers}
            lastRevision={lastRevision}
            selected={selected}
            onSelect={setSelected}
          />
        </div>
      )}
    </div>
  );
}

export function MissingToken() {
  return (
    <div className="studio" role="alert">
      <p>
        No session token. Open the studio from the link that{" "}
        <code>ck studio</code> gives you.
      </p>
    </div>
  );
}
