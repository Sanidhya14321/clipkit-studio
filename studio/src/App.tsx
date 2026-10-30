import { Player, type PlayerRef } from "@remotion/player";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  type CSSProperties,
} from "react";
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
import { PushToTalk } from "./panels/PushToTalk";
import { AskCards } from "./panels/AskCards";
import { StoryboardPanel } from "./panels/StoryboardPanel";
import { HistoryPanel } from "./panels/HistoryPanel";
import { HoverSelect, type Target } from "./preview/HoverSelect";
import type { Pointer } from "./comments";
import type { StudioContext } from "./generated/types";
import { ProjectComposition } from "./preview/ProjectComposition";
import { PROJECT_LAYER, agentLocked, agentStopped, canEdit } from "./state";
import { durationInFrames, mediaUrls } from "./timeline";
import { useNow, useStudio } from "./useStudio";
import { Workbench } from "./workspace/Workbench";
import { ProjectPanel } from "./workspace/ProjectPanel";
import type { Command } from "./workspace/commands";

export function App({ token }: { token: string }) {
  const { connected, project, locks, lastRevision, feed, asks } =
    useStudio(token);
  const now = useNow();
  const [selected, setSelected] = useState<string | null>(null);
  const [looks, setLooks] = useState<Look[]>([]);
  const [looksFailed, setLooksFailed] = useState(false);
  const player = useRef<PlayerRef>(null);
  const [playerInstance, setPlayerInstance] = useState<PlayerRef | null>(null);
  const attachPlayer = useCallback((instance: PlayerRef | null) => {
    player.current = instance;
    setPlayerInstance(instance);
  }, []);
  const contextQueue = useRef<Promise<unknown>>(Promise.resolve());
  const [playhead, setPlayhead] = useState(0);
  const [hovered, setHovered] = useState<Target | null>(null);
  const [element, setElement] = useState<Target | null>(null);
  const fps = project?.settings.fps ?? 30;

  useEffect(() => {
    const p = playerInstance;
    if (!p) return;
    const onFrame = (e: { detail: { frame: number } }) =>
      setPlayhead(e.detail.frame / fps);
    p.addEventListener("frameupdate", onFrame);
    p.addEventListener("seeked", onFrame);
    return () => {
      p.removeEventListener("frameupdate", onFrame);
      p.removeEventListener("seeked", onFrame);
    };
  }, [fps, playerInstance]);

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

  if (!project)
    return (
      <div
        className="studio connecting-screen"
        data-ck-connected={connected}
        role="status"
      >
        <strong>ClipKit Studio</strong>
        <p>Connecting to your local ck project…</p>
        <small>
          If this continues, start ck studio and open its session link.
        </small>
      </div>
    );

  const globalReadOnly = !connected || !canEdit(locks, PROJECT_LAYER, now);
  const post = (path: string) =>
    request(token, path, {
      method: "POST",
      body: JSON.stringify({ author: "human" }),
    }).then(() => undefined);
  const commands: Command[] = [
    {
      id: "undo",
      label: "Undo",
      group: "Edit",
      shortcut: "⌘ Z",
      reason: globalReadOnly
        ? "Reconnect or wait for the project lock to release"
        : undefined,
      run: () => post("/api/undo"),
    },
    {
      id: "redo",
      label: "Redo",
      group: "Edit",
      shortcut: "⇧ ⌘ Z",
      reason: globalReadOnly
        ? "Reconnect or wait for the project lock to release"
        : undefined,
      run: () => post("/api/redo"),
    },
    {
      id: "play",
      label: "Play / pause",
      group: "Sequence",
      shortcut: "Space",
      run: () => {
        if (player.current?.isPlaying()) player.current.pause();
        else player.current?.play();
      },
    },
    {
      id: "stop",
      label: "Stop after current step",
      group: "Agent",
      reason: !connected
        ? "Agent is disconnected"
        : project.agent.status === "idle" || project.agent.stopRequested
          ? "No running agent step"
          : undefined,
      run: () =>
        request(token, "/api/stop", { method: "POST" }).then(() => undefined),
    },
    {
      id: "import",
      label: "Import media",
      group: "File",
      reason: "Media import panel arrives in UI-5; use ck import",
      run: () => undefined,
    },
    {
      id: "export",
      label: "Export video",
      group: "File",
      reason: "Export panel arrives in UI-6; use ck export",
      run: () => undefined,
    },
  ];
  const seek = (seconds: number) =>
    player.current?.seekTo(Math.round(seconds * fps));
  const panels = {
    project: (
      <ProjectPanel
        project={project}
        selected={selected}
        onSelect={setSelected}
      />
    ),
    program: (
      <main className="studio-preview">
        <div className="monitor-meta">
          <span>SEQUENCE / {project.name}</span>
          <span>
            {project.settings.width} × {project.settings.height}
          </span>
        </div>
        <div className="monitor-stage">
          <div
            className="monitor-frame"
            style={
              {
                "--monitor-ratio":
                  project.settings.width / project.settings.height,
              } as CSSProperties
            }
          >
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
                ref={attachPlayer}
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
                fps={fps}
                compositionWidth={project.settings.width}
                compositionHeight={project.settings.height}
                style={{ width: "100%", maxHeight: "100%" }}
                controls
              />
            </HoverSelect>
          </div>
        </div>
        <div className="monitor-footer">
          <span>PROGRAM</span>
          <span>Preview · {fps} fps</span>
        </div>
      </main>
    ),
    timeline: (
      <Timeline
        project={project}
        agentLayers={agentLayers}
        lastRevision={lastRevision}
        selected={selected}
        onSelect={setSelected}
      />
    ),
    inspector: (
      <Inspector
        key={selected ?? ""}
        token={token}
        project={project}
        layerId={selected}
        readOnly={
          globalReadOnly ||
          (selected !== null && !canEdit(locks, selected, now))
        }
      />
    ),
    brief: (
      <BriefPanel
        token={token}
        project={project}
        readOnly={globalReadOnly}
        looks={looks}
        looksFailed={looksFailed}
      />
    ),
    transcript: (
      <>
        <TranscriptPanel
          token={token}
          project={project}
          readOnly={
            globalReadOnly ||
            !canEdit(
              locks,
              project.tracks.find((t) => t.kind === "video")?.id ??
                PROJECT_LAYER,
              now,
            )
          }
          onSeek={seek}
        />
        {!project.sources.some((s) => s.transcriptPath) && (
          <div className="empty-state">
            <strong>No transcript yet</strong>
            <p>
              Import and transcribe footage through your agent. Words will
              appear here for seeking and editing.
            </p>
          </div>
        )}
      </>
    ),
    storyboard: (
      <>
        <StoryboardPanel token={token} project={project} />
        {!project.brief.checkpoints?.includes("storyboard") && (
          <div className="empty-state">
            <strong>No storyboard checkpoint</strong>
            <p>
              Ask your agent for a storyboard checkpoint to review the scene
              plan here.
            </p>
          </div>
        )}
      </>
    ),
    voice: (
      <>
        <VoicePanel token={token} project={project} readOnly={globalReadOnly} />
        {!project.voiceSlots.length && (
          <div className="empty-state">
            <strong>Give your video a voice</strong>
            <p>
              Ask your agent to create a voice script. Record and manage takes
              here once slots are ready.
            </p>
          </div>
        )}
      </>
    ),
    comments: (
      <CommentsPanel token={token} project={project} pointer={pointer} />
    ),
    history: (
      <HistoryPanel token={token} project={project} readOnly={globalReadOnly} />
    ),
    agent: (
      <>
        <section className="panel agent-summary">
          <h2>Agent connection</h2>
          <p>{connected ? "ck event bridge connected" : "Waiting for ck"}</p>
          <p className="muted">
            {project.agent.label ||
              "Ask a question or give your agent a direction."}
          </p>
          <small>
            Stage: {project.pipeline.stage} ·{" "}
            {project.pipeline.pass ?? "no active pass"}
          </small>
        </section>
        <AskCards token={token} asks={asks} />
        <PushToTalk
          token={token}
          selection={selected}
          playhead={playhead}
          onPlay={() => player.current?.play()}
          onPause={() => player.current?.pause()}
          onSeek={seek}
        />
        <ActivityFeed feed={feed} />
      </>
    ),
  };
  return (
    <Workbench
      project={project}
      connected={connected}
      agentStatus={stopped ? "stopped" : project.agent.status}
      panels={panels}
      banner={<Banner token={token} project={project} stopped={stopped} />}
      commands={commands}
      selection={selected}
      playhead={playhead}
    />
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
