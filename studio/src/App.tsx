import { type PlayerRef } from "@remotion/player";
import {
  useCallback,
  useEffect,
  useMemo,
  useRef,
  useState,
  useSyncExternalStore,
} from "react";
import { request } from "./api";
import type {
  JsonValue,
  Look,
  Project,
  SplitRequest,
  StudioContext,
} from "./generated/types";
import { ActivityFeed } from "./panels/ActivityFeed";
import { Banner } from "./panels/Banner";
import { BriefPanel } from "./panels/BriefPanel";
import { Inspector } from "./panels/Inspector";
import { Timeline } from "./panels/Timeline";
import { TranscriptPanel } from "./panels/TranscriptPanel";
import { CommentsPanel } from "./panels/CommentsPanel";
import { VoicePanel } from "./panels/VoicePanel";
import { VoiceSetup } from "./panels/VoiceSetup";
import { PushToTalk } from "./panels/PushToTalk";
import { AskCards } from "./panels/AskCards";
import { StoryboardPanel } from "./panels/StoryboardPanel";
import { HistoryPanel } from "./panels/HistoryPanel";
import { ProgramMonitor } from "./panels/ProgramMonitor";
import { SourcePanel } from "./panels/SourcePanel";
import { LibraryPanel } from "./panels/LibraryPanel";
import { DeliveryPanel } from "./panels/DeliveryPanel";
import type { Target } from "./preview/HoverSelect";
import {
  PROJECT_LAYER,
  agentLocked,
  agentStopped,
  canEdit,
  locate,
} from "./state";
import { durationInFrames, mediaUrls } from "./timeline";
import { useNow, useStudio } from "./useStudio";
import { Workbench } from "./workspace/Workbench";
import { ProjectPanel } from "./workspace/ProjectPanel";
import type { Command } from "./workspace/commands";
import type { PanelId } from "./workspace/layout";
import { createPlaybackClock, type PlaybackClock } from "./editor/clock";
import { useEditor } from "./editor/useEditor";
import {
  removeRequest,
  titleRequest,
  duplicateRequest,
  uniqueId,
} from "./editor/edits";
import { TitleDialog } from "./editor/TitleDialog";

function LiveComments({
  token,
  project,
  clock,
  selection,
  element,
}: {
  token: string;
  project: Project;
  clock: PlaybackClock;
  selection: string | null;
  element: Target | null;
}) {
  const playhead = useSyncExternalStore(
    clock.subscribe,
    () => Math.floor(clock.get() * 4) / 4,
  );
  return (
    <CommentsPanel
      token={token}
      project={project}
      pointer={{
        playhead,
        loopRange: null,
        selection,
        element: element
          ? { layerId: element.layerId, path: element.path }
          : null,
      }}
      getPlayhead={clock.get}
    />
  );
}
function AgentCommands({
  token,
  clock,
  selected,
  onPlay,
  onPause,
  onSeek,
}: {
  token: string;
  clock: PlaybackClock;
  selected: string | null;
  onPlay: () => void;
  onPause: () => void;
  onSeek: (s: number) => void;
}) {
  const playhead = useSyncExternalStore(
    clock.subscribe,
    () => Math.floor(clock.get() * 4) / 4,
  );
  return (
    <PushToTalk
      token={token}
      selection={selected}
      playhead={playhead}
      getPlayhead={clock.get}
      onPlay={onPlay}
      onPause={onPause}
      onSeek={onSeek}
    />
  );
}
export function App({ token }: { token: string }) {
  const state = useStudio(token);
  const { connected, locks, lastRevision, feed, asks, refresh } = state;
  const editor = useEditor(state.project, token, refresh);
  const project = editor.project;
  const now = useNow();
  const [selected, setSelected] = useState<string | null>(null),
    [sourceId, setSourceId] = useState<string | null>(null);
  const [looks, setLooks] = useState<Look[]>([]),
    [looksFailed, setLooksFailed] = useState(false);
  const [hovered, setHovered] = useState<Target | null>(null),
    [element, setElement] = useState<Target | null>(null);
  const [openRequest, setOpenRequest] = useState<{
      panel: PanelId;
      nonce: number;
    } | null>(null),
    [titleDialog, setTitleDialog] = useState(false),
    [showImport, setShowImport] = useState(false),
    [actionError, setActionError] = useState("");
  const clock = useMemo(createPlaybackClock, []);
  const player = useRef<PlayerRef | null>(null),
    [playerInstance, setPlayerInstance] = useState<PlayerRef | null>(null);
  const attachPlayer = useCallback((instance: PlayerRef | null) => {
    player.current = instance;
    setPlayerInstance(instance);
  }, []);
  const getPlayer = useCallback(() => player.current, []);
  const fps = project?.settings.fps ?? 30;
  const projectRef = useRef(project);
  projectRef.current = project;
  const seek = useCallback(
    (seconds: number) => {
      const p = projectRef.current;
      if (!p || !Number.isFinite(seconds)) return;
      const frame = Math.max(
        0,
        Math.min(durationInFrames(p) - 1, Math.round(seconds * p.settings.fps)),
      );
      player.current?.seekTo(frame);
      clock.set(frame / p.settings.fps);
    },
    [clock],
  );
  const openPanel = useCallback(
    (panel: PanelId) =>
      setOpenRequest({ panel, nonce: Date.now() + Math.random() }),
    [],
  );
  const select = useCallback(
    (id: string) => {
      setSelected(id);
      openPanel("inspector");
    },
    [openPanel],
  );
  useEffect(() => {
    if (!playerInstance) return;
    const onFrame = (e: { detail: { frame: number } }) =>
      clock.set(e.detail.frame / fps);
    playerInstance.addEventListener("frameupdate", onFrame);
    playerInstance.addEventListener("seeked", onFrame);
    return () => {
      playerInstance.removeEventListener("frameupdate", onFrame);
      playerInstance.removeEventListener("seeked", onFrame);
    };
  }, [playerInstance, fps, clock]);
  useEffect(() => {
    request<Look[]>(token, "/api/looks").then(setLooks, () =>
      setLooksFailed(true),
    );
  }, [token]);
  useEffect(() => {
    let last = "",
      busy = false,
      cancelled = false;
    const update = () => {
      if (busy || !connected) return;
      const context: StudioContext = {
        ...(selected ? { selection: selected } : {}),
        ...(hovered
          ? {
              hovered: {
                layerId: hovered.layerId,
                ...(hovered.path ? { path: hovered.path } : {}),
              },
            }
          : {}),
        playhead: Math.round(clock.get() * 1000) / 1000,
      };
      const body = JSON.stringify(context);
      if (body === last) return;
      busy = true;
      void request(token, "/api/context", { method: "POST", body })
        .then(() => {
          if (!cancelled) last = body;
        })
        .catch(() => undefined)
        .finally(() => {
          busy = false;
        });
    };
    const timer = setInterval(update, 250);
    update();
    return () => {
      cancelled = true;
      clearInterval(timer);
    };
  }, [token, connected, selected, hovered?.layerId, hovered?.path, clock]);
  useEffect(() => {
    if (project && selected && !locate(project, selected)) setSelected(null);
  }, [project, selected]);
  const stopped = agentStopped(locks, now);
  const key = [
    ...agentLocked(locks, now),
    ...(project && project.agent.status !== "idle" && !stopped
      ? project.agent.activeLayers
      : []),
  ]
    .sort()
    .join("\0");
  const agentLayers = useMemo(() => new Set(key ? key.split("\0") : []), [key]);
  const media = useMemo(() => mediaUrls(project, token), [project, token]);
  if (!project)
    return (
      <div
        className="studio connecting-screen"
        data-ck-connected={connected}
        role="status"
      >
        <strong>ClipKit Studio</strong>
        <p>Connecting to your local project…</p>
        <small>
          Start ck studio and open its session link if this continues.
        </small>
      </div>
    );
  const globalReadOnly =
    !connected || !canEdit(locks, PROJECT_LAYER, now) || editor.pending;
  const selectedReadOnly =
    globalReadOnly || Boolean(selected && !canEdit(locks, selected, now));
  const run = async (path: string, body?: JsonValue) => {
    setActionError("");
    try {
      await request(token, path, {
        method: "POST",
        ...(body ? { body: JSON.stringify(body) } : {}),
      });
      await refresh();
    } catch (e) {
      setActionError(
        e instanceof Error ? e.message : "The action did not reach ck.",
      );
    }
  };
  const split = () => {
    if (!selected || selectedReadOnly) return;
    const req: SplitRequest = {
      layerId: selected,
      at: clock.get(),
      author: "human",
    };
    void run("/api/split", req as unknown as JsonValue);
  };
  const remove = (ripple: boolean) => {
    if (selected && !selectedReadOnly)
      void editor.commit(removeRequest(project, selected, ripple));
  };
  const duplicate = () => {
    if (selected && !selectedReadOnly)
      void editor.commit(duplicateRequest(project, selected));
  };
  const commands: Command[] = [
    {
      id: "undo",
      label: "Undo",
      group: "Edit",
      shortcut: "⌘ Z",
      reason: globalReadOnly
        ? "Reconnect or wait for the current edit/lock"
        : undefined,
      run: () => run("/api/undo", { author: "human" }),
    },
    {
      id: "redo",
      label: "Redo",
      group: "Edit",
      shortcut: "⇧ ⌘ Z",
      reason: globalReadOnly
        ? "Reconnect or wait for the current edit/lock"
        : undefined,
      run: () => run("/api/redo", { author: "human" }),
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
      id: "add-title",
      label: "Add title at playhead",
      group: "Sequence",
      reason: globalReadOnly ? "Editing is temporarily unavailable" : undefined,
      run: () => setTitleDialog(true),
    },
    {
      id: "split",
      label: "Split selected layer",
      group: "Sequence",
      shortcut: "⌘ B",
      reason:
        !selected || selectedReadOnly
          ? "Select an editable clip or footage layer"
          : undefined,
      run: split,
    },
    {
      id: "delete",
      label: "Delete selected layer",
      group: "Edit",
      shortcut: "Delete",
      reason:
        !selected || selectedReadOnly ? "Select an editable layer" : undefined,
      run: () => remove(false),
    },
    {
      id: "duplicate",
      label: "Duplicate selected layer",
      group: "Edit",
      reason:
        !selected || selectedReadOnly
          ? "Select an editable timed layer"
          : undefined,
      run: duplicate,
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
      run: () => run("/api/stop"),
    },
    {
      id: "ask",
      label: "Ask agent about selection",
      group: "Agent",
      run: () => openPanel("agent"),
    },
    {
      id: "import",
      label: "Import media",
      group: "File",
      run: () => {
        setShowImport(true);
        openPanel("project");
      },
    },
    {
      id: "export",
      label: "Review and export",
      group: "File",
      run: () => openPanel("delivery"),
    },
    {
      id: "save-version",
      label: "Save a named version",
      group: "File",
      run: () => openPanel("history"),
    },
  ];
  const panels = {
    project: (
      <ProjectPanel
        project={project}
        token={token}
        readOnly={globalReadOnly}
        onApplied={refresh}
        selected={selected}
        onSelect={select}
        showImport={showImport}
        onStartBrief={() => openPanel("brief")}
        onSource={(id) => {
          setSourceId(id);
          openPanel("source");
        }}
      />
    ),
    program: (
      <ProgramMonitor
        token={token}
        project={project}
        looks={looks}
        agentLayers={agentLayers}
        lastRevision={lastRevision}
        media={media}
        attachPlayer={attachPlayer}
        getPlayer={getPlayer}
        clock={clock}
        onHover={setHovered}
        onPick={(t) => {
          setElement(t);
          select(t.layerId);
        }}
        onSeek={seek}
      />
    ),
    timeline: (
      <Timeline
        project={project}
        agentLayers={agentLayers}
        lastRevision={lastRevision}
        selected={selected}
        onSelect={select}
        clock={clock}
        onSeek={seek}
        onCommit={editor.commit}
        onSplit={split}
        onDelete={remove}
        readOnly={globalReadOnly}
        pending={editor.pending}
      />
    ),
    inspector: (
      <Inspector
        key={selected ?? ""}
        token={token}
        project={project}
        layerId={selected}
        readOnly={selectedReadOnly}
        onCommit={editor.commit}
        onDuplicate={duplicate}
        onDelete={() => remove(false)}
        onSeek={seek}
        looks={looks}
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
    library: (
      <LibraryPanel
        token={token}
        project={project}
        readOnly={globalReadOnly}
        getPlayhead={clock.get}
        onApplied={refresh}
      />
    ),
    source: (
      <SourcePanel
        key={sourceId ?? ""}
        token={token}
        project={project}
        sourceId={sourceId}
        getPlayhead={clock.get}
        onCommit={editor.commit}
        readOnly={globalReadOnly}
      />
    ),
    delivery: (
      <DeliveryPanel
        token={token}
        project={project}
        connected={connected}
        onSelect={select}
        onSeek={seek}
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
          <TranscriptSetup
            token={token}
            project={project}
            readOnly={globalReadOnly}
            onApplied={refresh}
          />
        )}
      </>
    ),
    storyboard: (
      <>
        <StoryboardPanel token={token} project={project} />
        {!project.brief.checkpoints?.includes("storyboard") && (
          <div className="empty-state">
            <strong>Plan before you polish</strong>
            <p>
              Request a storyboard checkpoint through your agent. The rendered
              scene sheet and confirmation appear here when ready.
            </p>
            <button onClick={() => openPanel("agent")}>Ask your agent</button>
          </div>
        )}
      </>
    ),
    voice: (
      <>
        <VoiceSetup
          token={token}
          project={project}
          readOnly={globalReadOnly}
          onApplied={refresh}
        />
        <VoicePanel token={token} project={project} readOnly={globalReadOnly} />
      </>
    ),
    comments: (
      <LiveComments
        token={token}
        project={project}
        clock={clock}
        selection={selected}
        element={element}
      />
    ),
    history: (
      <HistoryPanel token={token} project={project} readOnly={globalReadOnly} />
    ),
    agent: (
      <>
        <section className="panel agent-summary">
          <span className="eyebrow">YOUR COLLABORATOR</span>
          <h2>Direct the next edit</h2>
          <p>
            {project.agent.label ||
              "Ask for a change to the selection, or describe what should happen next."}
          </p>
          <small>
            {connected ? "Event bridge connected" : "Waiting for ck"} · Stage:{" "}
            {project.pipeline.stage} ·{" "}
            {project.pipeline.pass ?? "no active pass"}
          </small>
        </section>
        <AskCards token={token} asks={asks} />
        <AgentCommands
          token={token}
          clock={clock}
          selected={selected}
          onPlay={() => player.current?.play()}
          onPause={() => player.current?.pause()}
          onSeek={seek}
        />
        <ActivityFeed feed={feed} />
      </>
    ),
  };
  return (
    <>
      <Workbench
        project={project}
        connected={connected}
        agentStatus={stopped ? "stopped" : project.agent.status}
        panels={panels}
        banner={<Banner token={token} project={project} stopped={stopped} />}
        commands={commands}
        selection={selected}
        playhead={0}
        clock={clock}
        openRequest={openRequest}
        pending={editor.pending}
        editError={editor.error || actionError}
        onDismissError={() => {
          editor.dismiss();
          setActionError("");
        }}
      />
      {titleDialog && (
        <TitleDialog
          onClose={() => setTitleDialog(false)}
          onAdd={async (value) => {
            const patch = titleRequest(project, clock.get(), value);
            const id = uniqueId(project, "title");
            const ok = await editor.commit(patch);
            if (ok) {
              setSelected(id || null);
              openPanel("inspector");
            }
            return ok;
          }}
        />
      )}
    </>
  );
}
function TranscriptSetup({
  token,
  project,
  readOnly,
  onApplied,
}: {
  token: string;
  project: Project;
  readOnly: boolean;
  onApplied: () => Promise<void>;
}) {
  const [source, setSource] = useState(""),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  return (
    <section className="panel empty-state">
      <strong>Turn speech into an editable transcript</strong>
      <p>
        Choose an imported source. Transcription requires a configured
        transcription provider.
      </p>
      <select
        aria-label="Transcribe source"
        value={source}
        onChange={(e) => setSource(e.currentTarget.value)}
      >
        <option value="">Choose source</option>
        {project.sources.map((s) => (
          <option key={s.id} value={s.id}>
            {s.id}
          </option>
        ))}
      </select>
      <button
        disabled={readOnly || busy || !source}
        onClick={() => {
          setBusy(true);
          setError("");
          void request(token, "/api/transcribe", {
            method: "POST",
            body: JSON.stringify({ source }),
          })
            .then(onApplied)
            .catch((e) =>
              setError(
                e instanceof Error ? e.message : "Transcription unavailable.",
              ),
            )
            .finally(() => setBusy(false));
        }}
      >
        {busy ? "Transcribing…" : "Transcribe source"}
      </button>
      {error && (
        <p className="error" role="alert">
          {error}
        </p>
      )}
    </section>
  );
}
export function MissingToken() {
  return (
    <div className="studio connecting-screen" role="alert">
      <strong>Open your project session</strong>
      <p>This window needs the session link created by ck studio.</p>
      <small>
        Project data stays on your computer. Open the full session URL to
        connect.
      </small>
    </div>
  );
}
