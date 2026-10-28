import {BriefStudioPanel} from './BriefStudio';
import './brief.css';
import {getProperty,setProperty} from '../../assets/src/property-path';
import {MotionPanel} from './MotionPanel';
import {SceneSourcePanel} from './SceneSourcePanel';
import {FrameSelection,FrameHandles} from './FrameSelection';
import {formatTimecode,affectedItems} from '../../core/src/frames';
import {editableFields,defaultFontFamily,supportsFont} from '../../assets/src/controls';
import {TokenUsage} from './TokenUsage';
import {LibraryPanel} from './LibraryPanel';
import {visualFields,audioFields} from '../../assets/src/visual';
import {useWorkspace} from './workspace';
import {ActivityPanel} from './ActivityPanel';
import React, { useCallback, useEffect, useRef, useState } from "react";
import { createRoot } from "react-dom/client";
import { Player, type PlayerRef } from "@remotion/player";
import { Composition } from "@clipkit/render/Composition";
import { catalog } from "@clipkit/assets";
import {
  allItems,
  duration,
  formatTime,
  routeVoice,
  timelineWords,
  type Anchor,
  type Item,
  type Project,
  type StudioState,
  type VoiceSlot,
} from "@clipkit/core";
import {
  ArrowDownToLine,
  ArrowLeft,
  ArrowRight,
  AudioLines,
  Check,
  ChevronDown,
  ChevronRight,
  Clapperboard,
  Copy,
  Film,
  FolderOpen,
  Layers,
  LoaderCircle,
  MessageSquare,
  Mic,
  MousePointer2,
  Pause,
  Play,
  Plus,
  Redo2,
  Scissors,
  Search,
  Settings2,
  Sparkles,
  Square,
  Trash2,
  Undo2,
  Upload,
  Volume2,
  WandSparkles,
  X,
  ZoomIn,
  ZoomOut,
} from "lucide-react";
import "./style.css";
import "./workspace.css";
import { captureAnchor, type AnchorTarget } from "./anchor";
type Op = (name: string, args?: Record<string, unknown>) => Promise<any>;
type Descriptor = {
  id: string;
  previewPath?: string;
  name: string;
  category: string;
  durationSec: number;
  description: string;
  tags: string[];
  fields: {
    key: string;
    label: string;
    type: string;
    min?: number;
    max?: number;
    step?: number;
    options?: any[];
  }[];
};
const nativeAssets = catalog as unknown as Descriptor[];
function IconButton({
  title,
  onClick,
  disabled = false,
  children,
  active = false,
}: {
  title: string;
  onClick: () => void;
  disabled?: boolean;
  children: React.ReactNode;
  active?: boolean;
}) {
  return (
    <button
      className={`icon-button ${active ? "active" : ""}`}
      title={title}
      aria-label={title}
      onClick={onClick}
      disabled={disabled}
    >
      {children}
    </button>
  );
}
function Field({
  label,
  value,
  type = "text",
  onCommit,
  onPreview,
  min,
  max,
  step,
}: {
  label: string;
  value: any;
  type?: string;
  onCommit: (v: any) => void;
  onPreview?: (v: any) => void;
  min?: number;
  max?: number;
  step?: number;
}) {
  const [draft, setDraft] = useState(String(value ?? ""));
  const lastCommit = useRef(String(value ?? ""));
  useEffect(() => {
    setDraft(String(value ?? ""));
    lastCommit.current = String(value ?? "");
  }, [value]);
  const numeric = type === "number" || type === "slider";
  const change = (text: string) => {
    setDraft(text);
    if (numeric) {
      if (text !== "" && Number.isFinite(Number(text)))
        onPreview?.(Number(text));
    } else onPreview?.(text);
  };
  const commit = () => {
    if (draft === lastCommit.current) return;
    const next = numeric ? Number(draft) : draft;
    if (numeric && (draft === "" || !Number.isFinite(next))) return;
    lastCommit.current = draft;
    onCommit(next);
  };
  return (
    <label className="field">
      <span>{label}</span>
      {type === "textarea" ? (
        <textarea
          aria-label={label}
          value={draft}
          onChange={(e) => change(e.target.value)}
          onBlur={commit}
        />
      ) : (
        <input
          aria-label={label}
          type={numeric ? "number" : type}
          value={draft}
          min={min}
          max={max}
          step={step}
          onChange={(e) => change(e.target.value)}
          onBlur={commit}
          onKeyDown={(e) => e.key === "Enter" && e.currentTarget.blur()}
        />
      )}
      {numeric && min !== undefined && max !== undefined && (
        <input
          className="property-slider"
          aria-label={`${label} slider`}
          type="range"
          min={min}
          max={max}
          step={step ?? 0.01}
          value={Number(draft) || 0}
          onChange={(e) => change(e.target.value)}
          onPointerUp={commit}
          onKeyUp={(e) => {
            if (
              e.key.startsWith("Arrow") ||
              e.key === "Home" ||
              e.key === "End"
            )
              commit();
          }}
          onBlur={commit}
        />
      )}
    </label>
  );
}
function trackLanes(items: Item[]) {
  const ends: number[] = [];
  const positions: Record<string, number> = {};
  for (const item of [...items].sort((a, b) => a.at - b.at)) {
    let lane = ends.findIndex((end) => end <= item.at + 0.001);
    if (lane < 0) lane = ends.length;
    ends[lane] = item.at + item.dur;
    positions[item.id] = lane;
  }
  return { positions, count: Math.max(1, ends.length) };
}
function App() {
  const [briefOpen,setBriefOpen]=useState(false);
  const workspace = useWorkspace();
  const [state, setState] = useState<StudioState | null>(null),
    [token, setToken] = useState(""),
    [connected, setConnected] = useState(false),
    [fatal, setFatal] = useState(""),
    [toast, setToast] = useState(""),
    [left, setLeft] = useState("assets"),
    [right, setRight] = useState("inspector"),
    [search, setSearch] = useState(""),
    [category, setCategory] = useState("All"),
    [candidates, setCandidates] = useState<
      Record<
        string,
        {
          silences: { start: number; end: number }[];
          fillers: { start: number; end: number; text: string }[];
        }
      >
    >({}),
    [selected, setSelectedState] = useState<string>(),
    [time, setTime] = useState(0),
    [playing, setPlaying] = useState(false),
    [loop, setLoop] = useState(false),
    [zoom, setZoom] = useState(55),
    [comment, setComment] = useState(""),
    [rangeEnd, setRangeEnd] = useState(""),
    [rangeTool,setRangeTool]=useState(false),
    [script, setScript] = useState(""),
    [command, setCommand] = useState(""),
    [recording, setRecording] = useState(""),
    [recordSecs, setRecordSecs] = useState(0),
    [countdown, setCountdown] = useState(0),
    [hover, setHover] = useState<{
      left: number;
      top: number;
      width: number;
      height: number;
      layerId?: string;
      propPath?: string;
      elementId?: string;
      point?: { x: number; y: number };
    }>(),
    [draftEdits, setDraftEdits] = useState<Record<string, Partial<Item>>>({}),
    [historyHighlight, setHistoryHighlight] = useState<string[]>([]),
    [range, setRange] = useState<{ start: number; end: number } | null>(null),
    [heard, setHeard] = useState<{ text: string; action: string } | null>(null),
    [exportDialog, setExportDialog] = useState(false),
    [dismissedJobs, setDismissedJobs] = useState<string[]>([]),
    [exportPreset, setExportPreset] = useState("vertical-1080"),
    [exportQuality, setExportQuality] = useState("standard"),
    [licenses, setLicenses] = useState<any>(null),
    [compare, setCompare] = useState(false),
    [suggest, setSuggest] = useState(false),
    [askMode, setAskMode] = useState(false),
    [cloneConsent, setCloneConsent] = useState(false),
    [selectionBox, setSelectionBox] = useState<{
      left: number;
      top: number;
      width: number;
      height: number;
    }>(),
    [gesture, setGesture] = useState<{
      id: string;
      at: number;
      dur: number;
      srcIn: number;
    } | null>(null);
  const selectedDetail = useRef<AnchorTarget>({});
  const committedProps = useRef<Record<string, Record<string, unknown>>>({});
  const leaseObserved = useRef<{ turnId: string; at: string } | null>(null);
  if (state?.lease && leaseObserved.current?.turnId !== state.lease.turnId)
    leaseObserved.current = {
      turnId: state.lease.turnId,
      at: new Date().toISOString(),
    };
  const setSelected = useCallback((id?: string) => {
    selectedDetail.current = {};
    setSelectedState(id);
    if(id){workspace.show("inspector");setRight("inspector");}
  }, [workspace.show]);
  const ghostPlayer = useRef<PlayerRef>(null),
    player = useRef<PlayerRef>(null),
    fileInput = useRef<HTMLInputElement>(null),
    stage = useRef<HTMLDivElement>(null),
    recorder = useRef<MediaRecorder | null>(null),
    frozenAnchor = useRef<Anchor>({ tStart: 0 }),
    recordStart = useRef(0),
    recognition = useRef<any>(null),
    commandHeld = useRef(false),
    gestureRef = useRef<any>(null),
    stateRef = useRef(state),
    timeRef = useRef(time),
    selectedRef = useRef(selected),
    tokenRef = useRef(token);
  const assets = Array.from(
    new Map(
      [
        ...nativeAssets,
        ...(((state as any)?.assetRegistry?.clips ?? []) as Descriptor[]),
      ].map((asset) => [asset.id, asset]),
    ).values(),
  );
  stateRef.current = state;
  timeRef.current = time;
  selectedRef.current = selected;
  tokenRef.current = token;
  const tell = useCallback((s: string) => {
    setToast(s);
    window.setTimeout(() => setToast(""), 5000);
  }, []);
  const op = useCallback<Op>(
    async (name, args = {}) => {
      try {
        const response = await fetch(`/api/op/${name}`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
            "X-ClipKit-Token": tokenRef.current,
          },
          body: JSON.stringify(args),
        });
        const data = await response.json();
        if (!response.ok || data.error)
          throw new Error(
            typeof data.error === "string"
              ? data.error
              : JSON.stringify(data.error ?? data),
          );
        if (data.state) setState(data.state);
        return data;
      } catch (e) {
        tell((e as Error).message);
        throw e;
      }
    },
    [tell],
  );
  useEffect(()=>{if(state?.project.brief?.status==='proposed')setBriefOpen(true)},[state?.project.brief?.rev,state?.project.brief?.status]);
  useEffect(()=>{if(!state||!token)return;const timer=setTimeout(()=>void op('context_set',{playhead:time,loop:range,selection:selected?{layerId:selected,propertyPath:selectedDetail.current.propPath,label:state.project.tracks.flatMap(t=>t.items).find(i=>i.id===selected)?.name??selected}:null,hovered:hover?.layerId?{layerId:hover.layerId,propertyPath:hover.propPath,elementId:hover.elementId}:null}).catch(()=>{}),250);return()=>clearTimeout(timer)},[time,range,selected,hover?.layerId,hover?.propPath,token]);
  const run = useCallback(
    (name: string, args: Record<string, unknown> = {}) => {
      if (stateRef.current?.lease && name !== "agent_cancel") {
        tell("The agent owns the edit lease. Cancel or wait for it to finish.");
        return;
      }
      void op(name, args).catch(() => {});
    },
    [op, tell],
  );
  useEffect(() => {
    let disposed = false,
      ws: WebSocket | undefined,
      retry: number;
    async function connect() {
      try {
        const boot = await fetch("/api/bootstrap").then((r) => r.json());
        if (disposed) return;
        setToken(boot.token);
        tokenRef.current = boot.token;
        const response = await fetch("/api/state", {
          headers: { "X-ClipKit-Token": boot.token },
        });
        if (!response.ok) throw new Error("Studio server is unavailable");
        const data = await response.json();
        if (disposed) return;
        setState(data.state ?? data);
        setFatal("");
        ws = new WebSocket(
          `${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/events?token=${encodeURIComponent(boot.token)}`,
        );
        ws.onopen = () => setConnected(true);
        ws.onmessage = (e) => {
          const msg = JSON.parse(e.data);
          if (msg.type === "state") setState(msg.state);
          if (msg.type === "error") tell(msg.error);
        };
        ws.onclose = () => {
          setConnected(false);
          if (!disposed) retry = window.setTimeout(connect, 2500);
        };
      } catch (e) {
        if (!disposed) {
          setFatal((e as Error).message);
          retry = window.setTimeout(connect, 3000);
        }
      }
    }
    void connect();
    return () => {
      disposed = true;
      clearTimeout(retry);
      ws?.close();
    };
  }, [tell]);
  useEffect(() => {
    const p = player.current;
    if (!p || !state) return;
    const update = (e: any) => {
      setTime(e.detail.frame / state.project.settings.fps);
      ghostPlayer.current?.seekTo(e.detail.frame);
    };
    const play = () => setPlaying(true),
      pause = () => setPlaying(false);
    p.addEventListener("frameupdate", update);
    p.addEventListener("play", play);
    p.addEventListener("pause", pause);
    return () => {
      p.removeEventListener("frameupdate", update);
      p.removeEventListener("play", play);
      p.removeEventListener("pause", pause);
    };
  }, [!!state, state?.project.settings.fps]);
  const seek = useCallback((t: number) => {
    const p = stateRef.current?.project;
    if (p) {
      const next = Math.max(0, Math.min(duration(p), t));
      setTime(next);
      player.current?.seekTo(Math.round(next * p.settings.fps));
    }
  }, []);
  const toggle = useCallback(() => player.current?.toggle(), []);
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if (
        e.target instanceof HTMLElement &&
        (e.target.matches("input,textarea,select") ||
          e.target.isContentEditable ||
          e.target.closest("[role=dialog]"))
      )
        return;
      if (
        e.code === "Space" &&
        e.target instanceof HTMLElement &&
        e.target.closest("button,a,summary")
      )
        return;
      if (e.code === "Space") {
        e.preventDefault();
        toggle();
      }
      if(e.key.toLowerCase()==='i'){const start=Math.round(timeRef.current*stateRef.current!.project.settings.fps)/stateRef.current!.project.settings.fps;setRange(r=>({start,end:Math.max(start+1/stateRef.current!.project.settings.fps,r?.end??start+1/stateRef.current!.project.settings.fps)}))}
      if(e.key.toLowerCase()==='o'){const end=Math.round(timeRef.current*stateRef.current!.project.settings.fps)/stateRef.current!.project.settings.fps;setRange(r=>({start:Math.min(r?.start??0,Math.max(0,end-1/stateRef.current!.project.settings.fps)),end:Math.max(1/stateRef.current!.project.settings.fps,end)}))}
      if (e.key === "Escape") {
        setSelected(undefined);
        setHover(undefined);
      }
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "z") {
        e.preventDefault();
        run(e.shiftKey ? "history_redo" : "history_undo");
      }
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "y") {
        e.preventDefault();
        run("history_redo");
      }
      if (stateRef.current?.lease) return;
      const itemId = selectedRef.current;
      if ((e.key === "Delete" || e.key === "Backspace") && itemId) {
        e.preventDefault();
        run("item_delete", { itemId });
        setSelected(undefined);
      }
      if (e.key.toLowerCase() === "s" && itemId)
        run("item_split", { itemId, at: timeRef.current });
      if (!e.altKey && e.key.toLowerCase() === "d" && itemId) {
        e.preventDefault();
        run("item_duplicate", { itemId });
      }
      if (e.key === "ArrowLeft")
        seek(
          timeRef.current -
            (e.shiftKey
              ? 1
              : 1 / (stateRef.current?.project.settings.fps ?? 30)),
        );
      if (e.key === "ArrowRight")
        seek(
          timeRef.current +
            (e.shiftKey
              ? 1
              : 1 / (stateRef.current?.project.settings.fps ?? 30)),
        );
    };
    window.addEventListener("keydown", key);
    return () => window.removeEventListener("keydown", key);
  }, [run, seek, toggle]);
  useEffect(() => {
    if (!recording) return;
    const t = window.setInterval(
      () => setRecordSecs((Date.now() - recordStart.current) / 1000),
      200,
    );
    return () => clearInterval(t);
  }, [recording]);
  useEffect(
    () => () => {
      recognition.current?.abort();
      window.speechSynthesis?.cancel();
      recorder.current?.stream.getTracks().forEach((t) => t.stop());
      if (recorder.current?.state === "recording") recorder.current.stop();
    },
    [],
  );
  useEffect(() => {
    setDraftEdits({});
    for (const [id, props] of Object.entries(committedProps.current)) {
      const actual = state?.project.tracks
        .flatMap((t) => t.items)
        .find((item) => item.id === id)?.props;
      if (
        !actual ||
        Object.entries(props).every(
          ([key, value]) =>
            JSON.stringify(actual[key]) === JSON.stringify(value),
        )
      )
        delete committedProps.current[id];
    }
  }, [state?.project.rev]);
  useEffect(() => {
    if (!exportDialog && !compare) return;
    const previous = document.activeElement as HTMLElement | null;
    const frame = requestAnimationFrame(() =>
      document
        .querySelector<HTMLElement>("[role=dialog] button,[role=dialog] select")
        ?.focus(),
    );
    const key = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.preventDefault();
        setExportDialog(false);
        setCompare(false);
      }
      if (event.key === "Tab") {
        const elements = Array.from(
          document.querySelectorAll<HTMLElement>(
            "[role=dialog] button:not(:disabled),[role=dialog] summary,[role=dialog] select,[role=dialog] a[href],[role=dialog] video",
          ),
        ).filter((element) => element.getClientRects().length > 0);
        const first = elements[0],
          last = elements[elements.length - 1];
        if (event.shiftKey && document.activeElement === first) {
          event.preventDefault();
          last?.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first?.focus();
        }
      }
    };
    window.addEventListener("keydown", key);
    return () => {
      cancelAnimationFrame(frame);
      window.removeEventListener("keydown", key);
      previous?.focus();
    };
  }, [exportDialog, compare]);
  const project = state?.project,
    items = project ? allItems(project) : [],
    item = items.find((i) => i.id === selected),
    locked = !!state?.lease,
    total = project ? duration(project) : 5;
  useEffect(() => {
    if (!stage.current || !selected) {
      setSelectionBox(undefined);
      return;
    }
    const measure = () => {
      const container = stage.current;
      if (!container) return;
      const nodes = Array.from(
        container.querySelectorAll<HTMLElement>("[data-ck-layer]"),
      ).filter((el) => el.dataset.ckLayer === selected);
      const detail = selectedDetail.current;
      const exact = detail.propPath
        ? nodes.filter((el) => el.dataset.ckPath === detail.propPath)
        : [];
      const visible = (exact.length ? exact : nodes)
        .map((el) => el.getBoundingClientRect())
        .filter((r) => r.width > 0 && r.height > 0);
      if (!visible.length) {
        setSelectionBox(undefined);
        return;
      }
      const parent = container.getBoundingClientRect(),
        left = Math.max(parent.left, Math.min(...visible.map((r) => r.left))),
        top = Math.max(parent.top, Math.min(...visible.map((r) => r.top))),
        right = Math.min(
          parent.right,
          Math.max(...visible.map((r) => r.right)),
        ),
        bottom = Math.min(
          parent.bottom,
          Math.max(...visible.map((r) => r.bottom)),
        );
      setSelectionBox({
        left: left - parent.left,
        top: top - parent.top,
        width: Math.max(0, right - left),
        height: Math.max(0, bottom - top),
      });
    };
    const raf = requestAnimationFrame(measure);
    const observer = new ResizeObserver(measure);
    observer.observe(stage.current);
    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
    };
  }, [selected, time, draftEdits, state?.project.rev]);
  useEffect(() => {
    if (loop && range && playing && (time >= range.end || time < range.start))
      seek(range.start);
  }, [loop, range, playing, time, seek]);
  const anchor = (): Anchor =>
    captureAnchor(
      range?.start ?? timeRef.current,
      selectedRef.current,
      hover,
      selectedDetail.current,
      range?.end ?? (Number(rangeEnd) || undefined),
    );
  async function upload(
    file: Blob,
    name: string,
    kind = "footage",
    slotId?: string,
  ) {
    const form = new FormData();
    form.append("file", file, name);
    const query = new URLSearchParams({ kind, ...(kind==="footage"?{place:"false"}:{}) });
    if (slotId) {
      query.set("slotId", slotId);
      query.set("mode", "recorded");
    }
    const r = await fetch(kind==="footage"&&/\.(json|html|jsx|tsx|js|css)$/i.test(name)?"/api/library-upload":`/api/upload?${query}`, {
      method: "POST",
      headers: { "X-ClipKit-Token": tokenRef.current },
      body: form,
    });
    const data = await r.json();
    if (!r.ok || data.error) throw new Error(data.error ?? "Upload failed");
    return data;
  }
  async function uploadFiles(files: FileList | null) {
    if (!files || locked) return;
    for (const file of Array.from(files))
      try {
        tell(`Importing ${file.name}…`);
        await upload(file, file.name);
        tell(`${file.name} imported`);
      } catch (e) {
        tell((e as Error).message);
      }
  }
  function executeCommand(
    text: string,
    a = anchor(),
    origin: "human" | "voice" = "human",
    confidence = 1,
  ) {
    if(origin==='voice'&&(stateRef.current?.briefStudio?.question&&!stateRef.current.briefStudio.question.answer||stateRef.current?.project.brief?.status==='proposed')){void op('brief_voice',{text,apply:true}).then(result=>{if(!result?.mapped&&result?.text)tell('Please choose a brief field or type your answer.');else tell('Voice answer received')}).catch(()=>{});return;}
    const intent = routeVoice(text, confidence);
    const voiceArgs = { author: origin };
    setHeard({
      text,
      action:
        intent.kind === "local"
          ? intent.action
          : intent.kind === "comment"
            ? "Ask agent"
            : intent.kind === "repeat"
              ? "Please repeat"
              : "Needs confirmation",
    });
    if (intent.kind === "repeat") {
      tell("Please repeat your editing request, or type it below.");
      return;
    }
    if (intent.kind === "comment") {
      if (stateRef.current?.lease) {
        tell("The agent owns the edit lease. Wait for this edit to finish.");
        return;
      }
      setRight("comments");
      tell(`Heard “${text}” · pinning your request…`);
      void op("comments_add", { text, anchor: a, author: origin })
        .then(async (result) => {
          if (!result.commentId) {
            tell(
              "Your anchored request was saved. Use Apply open to send it to your agent.",
            );
            return;
          }
          if(origin==='voice'){tell('Voice comment queued; Apply open or the waiting agent can pick it up.');return;}
          await op("agent_run", {
            client: stateRef.current?.agentClient ?? "codex",
            commentIds: [result.commentId],
            suggest,
          });
          tell(`Sent “${text}” to ${stateRef.current?.agentClient ?? "codex"}`);
        })
        .catch(() => {});
      return;
    }
    if (intent.kind === "confirm") {
      if (intent.action === "delete-track") {
        const track = stateRef.current?.project.tracks.find((t) =>
          t.items.some((i) => i.id === a.layerId),
        );
        if (
          track &&
          confirm(
            `Delete all ${track.items.length} elements in ${track.name}? You can undo this change.`,
          )
        )
          run("track_delete", {
            trackId: track.id,
            confirmed: true,
            ...voiceArgs,
          });
        else if (!track) tell("Select a track element first.");
        return;
      }
      if (confirm("Delete every timeline item? You can undo this change."))
        run("project_clear", { confirmed: true, ...voiceArgs });
      return;
    }
    switch (intent.action) {
      case "play":
        player.current?.play();
        break;
      case "pause":
        player.current?.pause();
        break;
      case "undo":
        run("history_undo", voiceArgs);
        break;
      case "redo":
        run("history_redo", voiceArgs);
        break;
      case "seek":
        seek(intent.value ?? 0);
        break;
      case "split":
        if (a.layerId)
          run("item_split", { itemId: a.layerId, at: a.tStart, ...voiceArgs });
        else tell("Select a clip to split");
        break;
      case "delete":
        if (a.layerId) run("item_delete", { itemId: a.layerId, ...voiceArgs });
        break;
      case "zoom-in":
        setZoom((z) => Math.min(180, z + 15));
        break;
      case "zoom-out":
        setZoom((z) => Math.max(20, z - 15));
        break;
      case "loop":
        if (!range) {
          const current = stateRef.current?.project.tracks
            .flatMap((t) => t.items)
            .find((i) => i.id === a.layerId);
          if (current)
            setRange({ start: current.at, end: current.at + current.dur });
          else if (a.tEnd && a.tEnd > a.tStart)
            setRange({ start: a.tStart, end: a.tEnd });
          else {
            tell("Drag a range on the timeline first.");
            return;
          }
        }
        setLoop(true);
        break;
    }
    tell(`Heard “${text}” · ${intent.action.replaceAll("-", " ")}`);
  }
  async function startRecord(
    kind: string,
    slotId?: string,
    preserveAnchor = false,
  ) {
    if (locked) return;
    if (kind === "clone-sample" && !cloneConsent) {
      tell(
        "Confirm your rights to this voice before recording a clone sample.",
      );
      return;
    }
    window.speechSynthesis?.cancel();
    if (!navigator.mediaDevices?.getUserMedia) {
      tell("Microphone recording needs localhost or HTTPS");
      return;
    }
    try {
      if (!preserveAnchor) frozenAnchor.current = anchor();
      const stream = await navigator.mediaDevices.getUserMedia({ audio: true });
      if (kind === "command" && !commandHeld.current) {
        stream.getTracks().forEach((t) => t.stop());
        return;
      }
      if (slotId || kind === "script-recording") {
        for (let n = 3; n > 0; n--) {
          setCountdown(n);
          await new Promise((r) => setTimeout(r, 1000));
        }
        setCountdown(0);
      }
      const rec = new MediaRecorder(stream);
      recorder.current = rec;
      const chunks: Blob[] = [];
      let previousMuted: boolean | undefined;
      rec.ondataavailable = (e) => {
        if (e.data.size) chunks.push(e.data);
      };
      rec.onstop = async () => {
        if (slotId || kind === "script-recording") {
          player.current?.pause();
          if (previousMuted !== undefined) {
            if (previousMuted) player.current?.mute();
            else player.current?.unmute();
          }
        }
        stream.getTracks().forEach((t) => t.stop());
        setRecording("");
        if (
          kind === "clone-sample" &&
          Date.now() - recordStart.current < 10000
        ) {
          tell("Clone samples need at least 10 seconds. Please record again.");
          return;
        }
        try {
          tell("Uploading recording…");
          const data = await upload(
            new Blob(chunks, { type: rec.mimeType }),
            `recording.${rec.mimeType.includes("mp4") ? "m4a" : "webm"}`,
            kind,
            slotId,
          );
          if (kind === "command") {
            if (data.transcript)
              executeCommand(data.transcript, frozenAnchor.current, "voice");
            else
              tell(
                "No speech recognized. Use the command field to type your request.",
              );
          } else if (kind === "clone-sample")
            run("voice_clone", {
              sampleSourceId: data.sourceId,
              consent: cloneConsent,
            });
          else
            tell(
              data.warning ??
                (kind === "script-recording"
                  ? "Whole-script recording uploaded for alignment"
                  : "Recording saved as a new take"),
            );
        } catch (e) {
          tell((e as Error).message);
        }
      };
      recordStart.current = Date.now();
      setRecordSecs(0);
      setRecording(slotId ?? kind);
      rec.start();
      if (slotId || kind === "script-recording") {
        const slot = stateRef.current?.project.voiceSlots.find(
          (s) => s.id === slotId,
        );
        if (player.current) {
          previousMuted = player.current.isMuted();
          player.current.mute();
          seek(slot?.targetWindow.start ?? 0);
          player.current.play();
        }
      }
    } catch (e) {
      setCountdown(0);
      tell((e as Error).message);
    }
  }
  function startCommand() {
    if (locked) return;
    if(stateRef.current?.provider.fish){commandHeld.current=true;frozenAnchor.current=anchor();void startRecord('command',undefined,true);return;}
    commandHeld.current = true;
    frozenAnchor.current = anchor();
    const Constructor =
      (window as any).SpeechRecognition ??
      (window as any).webkitSpeechRecognition;
    if (!Constructor) {
      void startRecord("command", undefined, true);
      return;
    }
    let transcript = "",
      confidence = 1,
      failed = false;
    const engine = new Constructor();
    recognition.current = engine;
    engine.lang = navigator.language || "en-US";
    engine.continuous = true;
    engine.interimResults = true;
    engine.onresult = (event: any) => {
      const scored = Array.from(event.results as any[]).filter(
        (result: any) =>
          result.isFinal && typeof result[0]?.confidence === "number",
      );
      if (scored.length)
        confidence = Math.min(
          ...scored.map((result: any) => result[0].confidence),
        );
      transcript = Array.from(event.results as any[])
        .map((result: any) => result[0]?.transcript ?? "")
        .join(" ")
        .trim();
      setHeard({
        text: transcript || "Listening…",
        action: "Browser speech recognition · hold to speak",
      });
    };
    engine.onerror = (event: any) => {
      failed = true;
      recognition.current = null;
      engine.abort();
      setRecording("");
      if (commandHeld.current && event.error !== "not-allowed") {
        tell(
          "Browser recognition unavailable. Using recorded speech fallback.",
        );
        void startRecord("command", undefined, true);
      } else {
        setHeard({
          text: transcript,
          action: "Please repeat or type your request",
        });
        tell(
          event.error === "not-allowed"
            ? "Microphone permission was denied. Type your request below."
            : "No clear speech heard. Please repeat.",
        );
      }
    };
    engine.onend = () => {
      if (recognition.current === engine) recognition.current = null;
      if (failed) return;
      setRecording("");
      if (transcript && !commandHeld.current)
        executeCommand(transcript, frozenAnchor.current, "voice", confidence);
      else if (!commandHeld.current)
        setHeard({ text: "", action: "No clear speech heard. Please repeat." });
    };
    recordStart.current = Date.now();
    setRecordSecs(0);
    setRecording("command");
    try {
      engine.start();
    } catch {
      failed = true;
      recognition.current = null;
      setRecording("");
      void startRecord("command", undefined, true);
    }
  }
  function stopCommand() {
    commandHeld.current = false;
    if (recognition.current) recognition.current.stop();
    if (recorder.current?.state === "recording") recorder.current.stop();
  }
  function scrubTimeline(e: React.PointerEvent<HTMLDivElement>) {
    if (e.button !== 0) return;
    const rect = e.currentTarget.getBoundingClientRect(),
      toTime = (x: number) =>
        Math.round(Math.max(0, Math.min(total, (x - rect.left) / zoom))*project!.settings.fps)/project!.settings.fps,
      start = toTime(e.clientX),
      selectRange = e.shiftKey||rangeTool;
    seek(start);
    if (selectRange)
      setRange({ start, end: start + 1 / project!.settings.fps });
    const move = (event: PointerEvent) => {
      const value = toTime(event.clientX);
      seek(value);
      if (selectRange) {
        setRange({
          start: Math.min(start, value),
          end: Math.max(Math.min(start,value)+1/project!.settings.fps,Math.max(start,value)),
        });
        setRangeEnd(String(Math.max(start, value)));
      }
    };
    const end = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", end);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", end, { once: true });
  }
  function askSelected() {
    setAskMode(true);
    setRight("comments");
    setComment("");
    window.setTimeout(
      () =>
        document
          .querySelector<HTMLTextAreaElement>('[aria-label="New comment"]')
          ?.focus(),
      0,
    );
  }
  function beginGesture(
    e: React.PointerEvent,
    item: Item,
    mode: "move" | "left" | "right",
  ) {
    if (locked || e.button !== 0) return;
    e.stopPropagation();
    setSelected(item.id);
    const origin = e.clientX,
      original = { ...item };
    gestureRef.current = {
      id: item.id,
      at: item.at,
      dur: item.dur,
      srcIn: item.srcIn,
    };
    const move = (event: PointerEvent) => {
      let delta = (event.clientX - origin) / zoom;
      const points = [
        0,
        timeRef.current,
        ...items
          .filter((i) => i.id !== item.id)
          .flatMap((i) => [i.at, i.at + i.dur]),
      ];
      const snap = (v: number) => {
        const point = points.find((p) => Math.abs(p - v) < 7 / zoom);
        return (
          Math.round((point ?? v) * project!.settings.fps) /
          project!.settings.fps
        );
      };
      let at = original.at,
        dur = original.dur,
        srcIn = original.srcIn;
      if (mode === "move") at = Math.max(0, snap(at + delta));
      if (mode === "right")
        dur = Math.max(1 / project!.settings.fps, snap(at + dur + delta) - at);
      if (mode === "left") {
        const next = Math.max(
          0,
          Math.min(
            original.at + original.dur - 1 / project!.settings.fps,
            snap(original.at + delta),
          ),
        );
        delta = next - original.at;
        if (original.src && original.srcIn + delta < 0) return;
        at = next;
        dur = original.dur - delta;
        srcIn = Math.max(0, original.srcIn + delta);
      }
      const value = { id: item.id, at, dur, srcIn };
      gestureRef.current = value;
      setGesture(value);
    };
    const end = (event: PointerEvent) => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", end);
      const value = gestureRef.current;
      setGesture(null);
      gestureRef.current = null;
      const targetTrack = (
        document
          .elementFromPoint(event.clientX, event.clientY)
          ?.closest("[data-track-id]") as HTMLElement | null
      )?.dataset.trackId;
      const fromTrack = project!.tracks.findIndex((t) =>
          t.items.some((i) => i.id === item.id),
        ),
        toTrack = project!.tracks.findIndex((t) => t.id === targetTrack);
      if (value && mode === "move" && toTrack >= 0 && fromTrack !== toTrack) {
        if (project!.tracks[fromTrack].kind === project!.tracks[toTrack].kind)
          run("item_move", {
            itemId: item.id,
            at: value.at,
            trackId: targetTrack,
          });
        else tell("Move elements between tracks of the same type.");
      } else if (
        value &&
        (value.at !== original.at || value.dur !== original.dur)
      )
        run("item_update", {
          itemId: item.id,
          changes: { at: value.at, dur: value.dur, srcIn: value.srcIn },
        });
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", end, { once: true });
  }
  function selectLayer(e: React.MouseEvent) {
    const element = (e.target as HTMLElement).closest(
      "[data-ck-layer]",
    ) as HTMLElement | null;
    if (element) {
      const id = element.dataset.ckLayer;
      if (id) {
        setSelected(id);
        const rect = stage.current?.getBoundingClientRect();
        selectedDetail.current = {
          layerId: id,
          propPath: element.dataset.ckPath,
          elementId: element.dataset.ckElement,
          ...(rect
            ? {
                point: {
                  x: (e.clientX - rect.left) / rect.width,
                  y: (e.clientY - rect.top) / rect.height,
                },
              }
            : {}),
        };
      }
      setRight("inspector");
    }
  }
  function hoverLayer(e: React.MouseEvent) {
    const element = (e.target as HTMLElement).closest(
      "[data-ck-layer]",
    ) as HTMLElement | null;
    if (!element || !stage.current) {
      setHover(undefined);
      return;
    }
    const rect = element.getBoundingClientRect(),
      container = stage.current.getBoundingClientRect();
    setHover({
      left: rect.left - container.left,
      top: rect.top - container.top,
      width: rect.width,
      height: rect.height,
      layerId: element.dataset.ckLayer,
      propPath: element.dataset.ckPath,
      elementId: element.dataset.ckElement,
      point: {
        x: (e.clientX - container.left) / container.width,
        y: (e.clientY - container.top) / container.height,
      },
    });
  }
  if (!state || !project)
    return (
      <div className="loading">
        <div className="brand-mark">
          <Clapperboard />
        </div>
        <h1>ClipKit</h1>
        <LoaderCircle className="spin" />
        <p>{fatal || "Connecting to your local studio…"}</p>
        <small>
          Start the studio server with <code>pnpm dev</code>
        </small>
      </div>
    );
  const matching = assets.filter(
    (a) =>
      (category === "All" || a.category === category) &&
      `${a.name} ${a.description} ${a.tags.join(" ")}`
        .toLowerCase()
        .includes(search.toLowerCase()),
  );
  const previewUpdate = (changes: Partial<Item>) =>
    item &&
    !locked &&
    setDraftEdits((previous) => ({
      ...previous,
      [item.id]: { ...previous[item.id], ...changes },
    }));
  const pendingProposal = ((state as any).suggestions ?? []).find(
    (p: any) => p.status === "pending",
  );
  const latestExport = [...state.jobs]
    .reverse()
    .find(
      (j) =>
        j.kind === "export" &&
        j.status === "done" &&
        typeof j.result?.url === "string",
    );
  const originalVideo = Object.values(project.sources).find(
    (source) => source.kind === "video",
  );
  const runtimeAssets = (state as any).assetRegistry;
  const turnEntries = state.lease
    ? state.history.filter((h) => h.turnId === state.lease?.turnId)
    : [];
  const turnStarted =
    turnEntries[0]?.timestamp ??
    [...state.activity]
      .reverse()
      .find((activity) =>
        activity.text.includes("is connecting to ClipKit tools"),
      )?.at ??
    leaseObserved.current?.at;
  const activeTouched = [
    ...new Set([
      ...turnEntries.flatMap((entry) => entry.changedItemIds),
      ...state.activity
        .filter((activity) => !!turnStarted && activity.at >= turnStarted)
        .flatMap((activity) => activity.itemIds ?? []),
    ]),
  ].slice(-8);
  const miniKey =
    selectedDetail.current.propPath &&
    item?.props[selectedDetail.current.propPath] !== undefined
      ? selectedDetail.current.propPath
      : "text";
  const miniField = assets
    .find((asset) => asset.id === item?.clipId)
    ?.fields.find((field) => field.key === miniKey);
  const previewProject = {
    ...project,
    tracks: project.tracks.map((track) => ({
      ...track,
      items: track.items.map((i) =>
        draftEdits[i.id] ? { ...i, ...draftEdits[i.id] } : i,
      ),
    })),
  };
  const update = (changes: Partial<Item>) =>
    item && run("item_update", { itemId: item.id, changes });
  const updateProps = (delta: Record<string, unknown>) => {
    if (!item || locked) return;
    const current = stateRef.current?.project.tracks
      .flatMap((t) => t.items)
      .find((candidate) => candidate.id === item.id);
    if (!current) return;
    const props = {
      ...current.props,
      ...committedProps.current[item.id],
    };
    for(const [key,value]of Object.entries(delta))Object.assign(props,setProperty(props,key,value));
    committedProps.current[item.id] = props;
    setDraftEdits((drafts) => ({
      ...drafts,
      [item.id]: { ...drafts[item.id], props },
    }));
    void op("item_update", { itemId: item.id, changes: { props } }).catch(
      () => {
        delete committedProps.current[item.id];
        setDraftEdits((drafts) => {
          const next = { ...drafts };
          delete next[item.id];
          return next;
        });
      },
    );
  };
  return (
    <div
      className={`app panel-${workspace.effective} library-${workspace.layout.view}`}
      style={workspace.style}
      onDragOver={(e) => e.preventDefault()}
      onDrop={(e) => {
        e.preventDefault();
        const assetId=e.dataTransfer.getData('application/x-clipkit-asset');if(assetId){const lane=(e.target as HTMLElement).closest<HTMLElement>('[data-track-id]');const at=lane?Math.max(0,(e.clientX-lane.getBoundingClientRect().left)/zoom):time;void op('library_place',{assetId,at,...lane?{trackId:lane.dataset.trackId}:{}}).then(result=>{if(result.itemId)setSelected(result.itemId)}).catch(error=>setToast(error instanceof Error?error.message:String(error)));}else void uploadFiles(e.dataTransfer.files);
      }}
    >
      <BriefStudioPanel project={project} op={op} open={briefOpen} onClose={()=>setBriefOpen(false)} waiting={state.briefStudio?.waiting} question={state.briefStudio?.question} onMic={startCommand} onMicEnd={stopCommand} onQuestionAnswer={answer=>void op('ask_answer',{id:state.briefStudio?.question?.id,answer})}/>
      <header className="topbar"><button aria-label="Open creative Brief" onClick={()=>setBriefOpen(true)}>Brief</button>
        <button className="panel-toggle" aria-label="Toggle Library" aria-pressed={['library','both'].includes(workspace.effective)} onClick={()=>workspace.toggle('library')}><FolderOpen size={16}/><span>Library</span></button>
        <button className="panel-toggle" aria-label="Toggle Properties" aria-pressed={['inspector','both'].includes(workspace.effective)} onClick={()=>workspace.toggle('inspector')}><Settings2 size={16}/><span>Properties</span></button>
        <button className="workspace-reset" aria-label="Reset Workspace" onClick={workspace.reset}>Reset Workspace</button>
        <a className="brand" href="/" aria-label="ClipKit studio">
          <span className="brand-mark">
            <Clapperboard size={19} />
          </span>
          clipkit<span className="beta">STUDIO</span>
        </a>
        <div className="divider" />
        <div className="project-title">
          <input
            aria-label="Project name"
            key={`${project.id}-${project.name}`}
            defaultValue={project.name}
            disabled={locked}
            onBlur={(e) =>
              e.target.value.trim() !== project.name &&
              run("project_rename", {
                name: e.target.value.trim() || project.name,
              })
            }
          />
          <ChevronDown size={13} />
          <span className="saved">
            <span className={`dot ${connected ? "" : "offline"}`} />
            {connected ? "Saved locally" : "Reconnecting"}
          </span>
        </div>
        <div className="top-actions">
          <IconButton
            title="Undo (Ctrl Z)"
            disabled={locked}
            onClick={() => run("history_undo")}
          >
            <Undo2 size={17} />
          </IconButton>
          <IconButton
            title="Redo (Ctrl Shift Z)"
            disabled={locked}
            onClick={() => run("history_redo")}
          >
            <Redo2 size={17} />
          </IconButton>
          <TokenUsage usage={state.telemetry}/>
          <button
            className="export"
            disabled={locked || !items.length}
            onClick={() => {
              setExportDialog(true);
              void op("export_licenses")
                .then(setLicenses)
                .catch(() => {});
            }}
          >
            <ArrowDownToLine size={15} />
            Export video
          </button>

        </div>
      </header>
      <div className="workspace">
        <aside className="library">
          <div className="panel-heading">
            <h2>Components & direction</h2>
            <button aria-label="Toggle library list view" onClick={()=>workspace.setView(workspace.layout.view==='grid'?'list':'grid')}>{workspace.layout.view==='grid'?'List':'Grid'}</button>
          </div>
          <div className="tabbar">
            {[
              ["assets", "Components"],
              ["skills", "Storyboard"],
              ["footage", "Media"],
              ["transcript", "Transcript"],
            ].map(([id, title]) => (
              <button
                key={id}
                className={left === id ? "selected" : ""}
                onClick={() => setLeft(id)}
              >
                {title}
              </button>
            ))}
          </div>
          {left === "assets" ? (<LibraryPanel op={op} state={state} token={token} time={time} componentOnly onSelect={id=>{setSelected(id);setRight("inspector");workspace.show("inspector")}}/>) : left === "skills" ? (<MotionPanel op={op} state={state} token={token} selectedId={selected} range={range} onSeek={seek}/>
          ) : left === "footage" ? (
            <div className="panel-scroll">
              <button
                className="upload-zone"
                disabled={locked}
                onClick={() => fileInput.current?.click()}
              >
                <Upload size={25} />
                <b>Bring your story in</b>
                <span>Drop video, audio, or images here</span>
                <small>or click to browse files</small>
              </button>
              {Object.values(project.sources).map((source) => (
                <div className="media-card" key={source.id}>
                  <div className="media-thumb">
                    {source.kind === "image" ? (
                      <img
                        alt=""
                        src={`/media/${encodeURIComponent(source.id)}?token=${encodeURIComponent(token)}`}
                      />
                    ) : source.kind === "video" ? (
                      <Film />
                    ) : (
                      <AudioLines />
                    )}
                  </div>
                  <div>
                    <b>{source.name}</b><button className="primary" disabled={locked} onClick={()=>void op('library_place',{assetId:'media-'+source.id,at:time}).then(result=>setSelected(result.itemId)).catch(()=>{})}>Add again</button>
                    <small>
                      {source.kind} · {formatTime(source.durationSec)}
                    </small>
                    <button
                      disabled={locked}
                      onClick={() =>
                        run("footage_transcribe", { sourceId: source.id })
                      }
                    >
                      Transcribe
                    </button>
                    <button
                      disabled={locked}
                      onClick={() => {
                        void op("footage_candidates", { sourceId: source.id })
                          .then((result) =>
                            setCandidates((current) => ({
                              ...current,
                              [source.id]: result,
                            })),
                          )
                          .catch(() => {});
                      }}
                    >
                      Find moments
                    </button>
                  </div>
                  {candidates[source.id] && (
                    <div className="candidate-list">
                      <div className="section-label">MOMENTS TO REVIEW</div>
                      {candidates[source.id].fillers.map((range, i) => {
                        const linked = items.find(
                          (item) =>
                            item.src === source.id &&
                            item.type === "footage" &&
                            item.srcIn <= range.start &&
                            item.srcIn + item.dur >= range.end,
                        );
                        return (
                          <div key={`f-${i}`}>
                            <button
                              title="Review filler in timeline"
                              onClick={() =>
                                linked
                                  ? seek(linked.at + range.start - linked.srcIn)
                                  : tell(
                                      "This source range is outside the current timeline.",
                                    )
                              }
                            >
                              <Play size={10} />“{range.text}” ·{" "}
                              {formatTime(range.start)}
                            </button>
                            <button
                              className="candidate-cut"
                              disabled={locked || !linked}
                              onClick={() =>
                                linked &&
                                run("footage_delete_words", {
                                  sourceId: source.id,
                                  itemId: linked.id,
                                  start: range.start,
                                  end: range.end,
                                })
                              }
                            >
                              Cut filler
                            </button>
                          </div>
                        );
                      })}
                      {candidates[source.id].silences.map((range, i) => {
                        const linked = items.find(
                          (item) =>
                            item.src === source.id &&
                            item.type === "footage" &&
                            item.srcIn <= range.start &&
                            item.srcIn + item.dur >= range.start,
                        );
                        return (
                          <div key={`s-${i}`}>
                            <button
                              title="Review detected silence; no automatic cut"
                              onClick={() =>
                                linked
                                  ? seek(linked.at + range.start - linked.srcIn)
                                  : tell(
                                      "This source range is outside the current timeline.",
                                    )
                              }
                            >
                              <Play size={10} />
                              Silence · {formatTime(range.start)}–
                              {formatTime(range.end)}
                            </button>
                            <span>Review</span>
                          </div>
                        );
                      })}
                      {!candidates[source.id].silences.length &&
                        !candidates[source.id].fillers.length && (
                          <p>
                            No silence or timestamped fillers found. Transcribe
                            this source to detect filler words.
                          </p>
                        )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <div className="panel-scroll">
              <div className="section-label">THE STORY, WORD BY WORD</div>
              <p className="muted">
                Click a word to find it in your timeline. Select a clip, then
                use × to cut a word.
              </p>
              <div className="transcript">
                {timelineWords(project, state.transcripts).map((w, i) => (
                  <span
                    key={`${w.itemId}-${i}`}
                    className={
                      time >= w.start && time < w.end ? "word active" : "word"
                    }
                  >
                    <button
                      onClick={() => {
                        seek(w.start);
                        setSelected(w.itemId);
                      }}
                    >
                      {w.text}
                    </button>
                    <button
                      className="word-delete"
                      title="Delete this word and ripple footage"
                      disabled={locked}
                      onClick={() =>
                        run("footage_delete_words", {
                          sourceId: w.sourceId,
                          itemId: w.itemId,
                          start: w.sourceStart,
                          end: w.sourceEnd,
                        })
                      }
                    >
                      ×
                    </button>
                  </span>
                ))}
              </div>
              {!timelineWords(project, state.transcripts).length && (
                <div className="empty-small">
                  <AudioLines />
                  <b>Your words will live here</b>
                  <p>
                    Import footage and transcribe it from Media to edit your
                    story through text.
                  </p>
                </div>
              )}
            </div>
          )}
          <input
            ref={fileInput}
            type="file"
            multiple
            accept="video/*,audio/*,image/png,image/jpeg,image/webp,image/svg+xml,.json,.html,.jsx,.tsx,.js,.css"
            hidden
            onChange={(e) => {
              void uploadFiles(e.target.files);
              e.target.value = "";
            }}
          />
        </aside>
        <div className="panel-resizer left-resizer" {...workspace.separator("left")}/><main className="main">
          <div className="preview-header">
            <div>
              <span className="section-label">PREVIEW</span>
              <span className="preview-ratio">
                {project.settings.width === project.settings.height
                  ? "1:1"
                  : project.settings.width > project.settings.height
                    ? "16:9"
                    : "9:16"}
                <ChevronDown size={12} />
              </span>
            </div>
            <span className="preview-quality">
              {project.settings.width} × {project.settings.height}
              <span> · </span>
              {project.settings.fps} fps
            </span>
            <IconButton
              title="Add anchored comment"
              onClick={() => {
                setRight("comments");
                setComment("");
              }}
            >
              <MessageSquare size={15} />
            </IconButton>
          </div>
          <div className="preview-area">
            <div
              className={`stage ${items.length ? "" : "stage-empty"}`}
              ref={stage}
              onClick={selectLayer}
              onMouseMove={hoverLayer}
              onMouseLeave={() => setHover(undefined)}
              style={{
                aspectRatio: `${project.settings.width}/${project.settings.height}`,
              }}
            >
              <Player
                ref={player}
                component={Composition}
                inputProps={{
                  project: previewProject,
                  mediaBaseUrl: `${location.origin}/media`,
                  assetBaseUrl: location.origin,
                  preview: true,
                  accessToken: token,
                  assetRegistry: runtimeAssets,
                }}
                durationInFrames={Math.max(
                  1,
                  Math.ceil(total * project.settings.fps),
                )}
                fps={project.settings.fps}
                compositionWidth={project.settings.width}
                compositionHeight={project.settings.height}
                style={{ width: "100%", height: "100%" }}
                controls={false}
                clickToPlay={false}
                loop={loop}
              />
              {!items.length && (
                <div className="canvas-empty">
                  <div className="empty-grid-icon">
                    <Clapperboard size={31} />
                  </div>
                  <span>EVERY STORY STARTS SOMEWHERE</span>
                  <h1>
                    Make something
                    <br />
                    <em>worth watching.</em>
                  </h1>
                  <p>
                    Add an element, import footage,
                    <br />
                    or direct an original video from Storyboard.
                  </p>
                  <button disabled={locked} onClick={()=>{setLeft("assets");workspace.show("library");requestAnimationFrame(()=>document.querySelector<HTMLTextAreaElement>('[aria-label="Creative brief"]')?.focus())}}>
                    <Sparkles size={15} />
                    Create a reel
                  </button>
                  <small>Original scenes · shared frame score</small>
                </div>
              )}
              {selectionBox && selected && (
                <div className="selection-outline" style={selectionBox}>
                  <span>{selected}</span>
                </div>
              )}
              {pendingProposal?.previewProject && (
                <div className="proposal-ghost">
                  <Player
                    ref={ghostPlayer}
                    component={Composition}
                    inputProps={{
                      project: pendingProposal.previewProject,
                      mediaBaseUrl: `${location.origin}/media`,
                      assetBaseUrl: location.origin,
                      preview: true,
                      accessToken: token,
                      assetRegistry: runtimeAssets,
                    }}
                    durationInFrames={Math.max(
                      1,
                      Math.ceil(
                        duration(pendingProposal.previewProject) *
                          project.settings.fps,
                      ),
                    )}
                    fps={project.settings.fps}
                    compositionWidth={project.settings.width}
                    compositionHeight={project.settings.height}
                    initialFrame={Math.floor(time * project.settings.fps)}
                    style={{ width: "100%", height: "100%" }}
                    controls={false}
                  />
                  <span>PROPOSED CHANGES</span>
                </div>
              )}
              {hover && items.length > 0 && (
                <div
                  className="layer-outline"
                  style={{
                    left: hover.left,
                    top: hover.top,
                    width: hover.width,
                    height: hover.height,
                  }}
                >
                  <span>
                    {hover.layerId}
                    {hover.propPath ? ` · ${hover.propPath}` : ""}
                  </span>
                </div>
              )}
            </div>
            {item && (
              <details className="floating-inspector"><summary>Quick edit</summary>
                <span>{selectedDetail.current.propPath ?? item.type}</span>
                {item.props[miniKey] !== undefined &&
                  miniField?.type !== "select" &&
                  miniField?.type !== "image" && (
                    <Field
                      label={`Selected ${miniKey}`}
                      type={
                        miniField?.type ??
                        (typeof item.props[miniKey] === "number"
                          ? "number"
                          : "text")
                      }
                      value={item.props[miniKey]}
                      onPreview={(value) =>
                        previewUpdate({
                          props: { ...item.props, [miniKey]: value },
                        })
                      }
                      onCommit={(value) => updateProps({ [miniKey]: value })}
                    />
                  )}
                <button disabled={locked} onClick={askSelected}>
                  <Sparkles size={12} />
                  Ask agent
                </button>
              </details>
            )}
            <span className="canvas-note">
              <MousePointer2 size={11} /> Click a layer to make it yours
            </span>
          </div>
          <div className="transport">
            <div className="timecode">
              {formatTime(time)}
              <span> / {formatTime(total)}</span>
            </div>
            <div className="play-controls">
              <IconButton
                title="Previous frame"
                onClick={() => seek(time - 1 / project.settings.fps)}
              >
                <ArrowLeft size={15} />
              </IconButton>
              <button
                className="play"
                aria-label={playing ? "Pause" : "Play"}
                onClick={toggle}
              >
                {playing ? (
                  <Pause size={18} fill="currentColor" />
                ) : (
                  <Play size={18} fill="currentColor" />
                )}
              </button>
              <IconButton
                title="Next frame"
                onClick={() => seek(time + 1 / project.settings.fps)}
              >
                <ArrowRight size={15} />
              </IconButton>
            </div>
            <div className="transport-right">
              <button
                className="text-button"
                disabled={!latestExport || !originalVideo}
                title="Compare raw footage with latest export"
                onClick={() => setCompare(true)}
              >
                Compare
              </button>
              <Volume2 size={15} />
              <span>Fit</span>
              <ChevronDown size={12} />
            </div>
          </div>
          <div className="command-bar">
            <div className="command-icon">
              <WandSparkles size={16} />
            </div>
            <input
              aria-label="Voice or typed editing command"
              placeholder="A thought? Say it. An edit? Ask for it."
              value={command}
              onChange={(e) => setCommand(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter" && command.trim()) {
                  executeCommand(command);
                  setCommand("");
                }
              }}
            />
            {command && (
              <button
                className="text-button"
                onClick={() => {
                  executeCommand(command);
                  setCommand("");
                }}
              >
                Send <ArrowRight size={13} />
              </button>
            )}
            <span className="command-hint">Try “split here”</span>
            <button
              className={`mic-button ${recording === "command" ? "recording" : ""}`}
              disabled={locked || (!!recording && recording !== "command")}
              aria-label={
                recording === "command"
                  ? "Release to send voice command"
                  : "Hold to record voice command"
              }
              title="Hold to speak. Release to send."
              onPointerDown={(e) => {
                if (e.button !== 0) return;
                e.currentTarget.setPointerCapture(e.pointerId);
                startCommand();
              }}
              onPointerUp={() => {
                stopCommand();
              }}
              onPointerCancel={() => {
                stopCommand();
              }}
              onKeyDown={(e) => {
                if ((e.key === "Enter" || e.code === "Space") && !e.repeat) {
                  e.preventDefault();
                  startCommand();
                }
              }}
              onKeyUp={(e) => {
                if (e.key === "Enter" || e.code === "Space") {
                  e.preventDefault();
                  stopCommand();
                }
              }}
            >
              <Mic size={15} />
              {recording === "command"
                ? `${recordSecs.toFixed(0)}s · release`
                : "Hold to talk"}
            </button>
          </div>
          {heard && (
            <div className="heard-chip" role="status">
              <Mic size={12} />
              <span>Heard: {heard.text || "—"}</span>
              <b>{heard.action}</b>
              <button
                aria-label="Dismiss heard command"
                onClick={() => setHeard(null)}
              >
                <X size={12} />
              </button>
            </div>
          )}
        </main>
        <div className="panel-resizer right-resizer" {...workspace.separator("right")}/><aside className="inspector">
          <div className="tabbar">
            {[
              ["inspector", "Properties"],
              ["comments", "Comments"],
              ["voice", "Voice"],
              ["history", "History"],
              ["activity", "Activity"],
            ].map(([id, title]) => (
              <button
                key={id}
                className={right === id ? "selected" : ""}
                onClick={() => setRight(id)}
              >
                {title}
                {id === "comments" &&
                  project.comments.some((c) => c.status === "open") && (
                    <i className="notification-dot" />
                  )}
              </button>
            ))}
          </div>
          <div className="panel-scroll">
            {right === "activity" ? <><div className="scene-phases">{Object.entries((state as any).sceneActivity??{}).map(([id,value]:[string,any])=><p key={id}><b>{id}</b> · {value.phase}{value.error&&<span role="alert"> · {value.error}</span>}</p>)}</div><ActivityPanel jobs={state.jobs} token={token} retry={id=>run("job_retry",{jobId:id})}/></> : right === "inspector" ? (
              item ? (
                <>
                  <div className="selected-heading">
                    <span className="property-icon">
                      <Settings2 size={18} />
                    </span>
                    <div>
                      <h3>
                        {item.clipId
                          ? (assets.find((a) => a.id === item.clipId)?.name ??
                            item.clipId)
                          : item.type}
                      </h3>
                      <small>
                        {item.type.toUpperCase()} · {item.id.slice(-8)}
                      </small>
                    </div>
                  </div>
                  <fieldset disabled={locked}>
                    {item.sceneVersion&&<SceneSourcePanel item={item} op={op} locked={locked}/>}
                    <div className="section-label">TIMING</div>
                    <div className="field-row">
                      <Field
                        label="Starts at (s)"
                        type="number"
                        min={0}
                        step={1 / project.settings.fps}
                        value={item.at}
                        onPreview={(at) => previewUpdate({ at })}
                        onCommit={(at) => update({ at })}
                      />
                      <Field
                        label="Duration (s)"
                        type="number"
                        min={1 / project.settings.fps}
                        step={1 / project.settings.fps}
                        value={item.dur}
                        onPreview={(dur) => previewUpdate({ dur })}
                        onCommit={(dur) => update({ dur })}
                      />
                    </div>
                    {item.src&&<label className="field"><span>Replace source</span><select aria-label="Replace source" disabled={locked} value={item.src} onChange={e=>update({src:e.target.value})}>{Object.values(project.sources).filter(source=>item.type==='audio'?source.kind==='audio':['video','image'].includes(source.kind)).map(source=><option key={source.id} value={source.id}>{source.name}</option>)}</select></label>}
                    {item.src && (
                      <Field
                        label="Source in (s)"
                        type="number"
                        min={0}
                        value={item.srcIn}
                        onCommit={(srcIn) => update({ srcIn })}
                      />
                    )}
                    <div className="section-label property-section">APPEARANCE</div>
                    {item.type==='clip'&&nativeAssets.some(a=>a.id===item.clipId)&&supportsFont(item.clipId!)&&Boolean(item.props.text)&&<label className="field"><span>Font</span><select aria-label="Font" value={String(item.props.fontFamily??defaultFontFamily(item.clipId!))} onChange={e=>updateProps({fontFamily:e.target.value})}><option>Space Grotesk</option><option>Inter</option></select></label>}
                    {(
                      (item.clipId?editableFields(item.clipId,assets.find(a=>a.id===item.clipId)?.fields as any[]??[],item.props,(state.assetRegistry?.clips.find(a=>a.id===item.clipId) as any)?.propsSchema):undefined) ?? (item.type === "footage" ? visualFields : item.type === "audio" ? audioFields : undefined) ??
                      Object.keys(item.props).map((key) => ({
                        key,
                        label: key,
                        type:
                          typeof item.props[key] === "number"
                            ? "number"
                            : "text",
                      }))
                    ).map((f: any) =>
                      f.type === "image" ? (
                        <label className="field" key={f.key}>
                          <span>{f.label}</span>
                          <select
                            aria-label={f.label}
                            value={String(getProperty(item.props,f.key) ?? f.default ?? "")}
                            onChange={(e) =>
                              updateProps({ [f.key]: e.target.value })
                            }
                          >
                            <option value="">Native placeholder</option>
                            {Object.values(project.sources)
                              .filter((source) => source.kind === "image")
                              .map((source) => (
                                <option key={source.id} value={source.id}>
                                  {source.name}
                                </option>
                              ))}
                          </select>
                          <input
                            type="file"
                            aria-label={`Upload ${f.label}`}
                            accept="image/png,image/jpeg,image/webp"
                            onChange={(e) => {
                              const file = e.target.files?.[0];
                              if (file)
                                void upload(file, file.name)
                                  .then((result) => {
                                    updateProps({ [f.key]: result.sourceId });
                                    tell("Image imported and applied");
                                  })
                                  .catch((error) => tell(error.message));
                            }}
                          />
                        </label>
                      ) : f.type === "select" ? (
                        <label className="field" key={f.key}>
                          <span>{f.label}</span>
                          <select
                            aria-label={f.label}
                            value={String(getProperty(item.props,f.key) ?? f.default ?? "")}
                            onChange={(e) =>
                              updateProps({ [f.key]: e.target.value })
                            }
                          >
                            {f.options?.map((o: any) => (
                              <option
                                key={typeof o === "string" ? o : o.value}
                                value={typeof o === "string" ? o : o.value}
                              >
                                {typeof o === "string" ? o : o.label}
                              </option>
                            ))}
                          </select>
                        </label>
                      ) : (
                        <Field
                          key={item.id + f.key}
                          label={f.label}
                          type={f.type}
                          value={getProperty(item.props,f.key) ?? f.default}
                          min={f.min}
                          max={f.max}
                          step={f.step}
                          onPreview={(v) =>
                            previewUpdate({
                              props: setProperty(item.props,f.key,v),
                            })
                          }
                          onCommit={(v) => updateProps({ [f.key]: (f as any).valueType==='boolean'?String(v)==='true':v })}
                        />
                      ),
                    )}
                    {["footage", "audio", "voice"].includes(item.type) && (
                      <Field
                        label="Volume"
                        type="number"
                        value={item.volume}
                        min={0}
                        max={2}
                        step={0.05}
                        onPreview={(volume) => previewUpdate({ volume })}
                        onCommit={(volume) => update({ volume })}
                      />
                    )}{" "}
                    {item.type === "audio" && (
                      <label className="duck-control">
                        <input
                          type="checkbox"
                          checked={item.props.duck === true}
                          onChange={(e) =>
                            updateProps({ duck: e.target.checked })
                          }
                        />
                        <span>
                          Duck under narration
                          <small>
                            Lower this audio while a narration take plays.
                          </small>
                        </span>
                      </label>
                    )}
                    <div className="item-actions">
                      <button
                        onClick={() =>
                          run("item_split", { itemId: item.id, at: time })
                        }
                      >
                        <Scissors size={14} />
                        Split at playhead
                      </button>
                      <button
                        onClick={() =>
                          run("item_duplicate", { itemId: item.id })
                        }
                      >
                        <Copy size={14} />
                        Duplicate
                      </button>
                      <button
                        className="danger"
                        onClick={() => {
                          run("item_delete", { itemId: item.id });
                          setSelected(undefined);
                        }}
                      >
                        <Trash2 size={14} />
                        Delete element
                      </button>
                    </div>
                  </fieldset>
                </>
              ) : (
                <>
                  <div className="empty-properties">
                    <div className="property-orbit">
                      <MousePointer2 size={24} />
                    </div>
                    <h3>
                      Properties
                    </h3>
                    <p>
                      Select an element on the canvas
                      <br />
                      or timeline to fine-tune it.
                    </p>
                  </div>
                  <div className="shortcut-card">
                    <span className="section-label">KEYBOARD SHORTCUTS</span>
                    {[
                      ["Play / pause", "Space"],
                      ["Split at playhead", "S"],
                      ["Duplicate", "D / Ctrl D"],
                      ["Undo", "Ctrl Z"],
                      ["Step one frame", "← →"],
                    ].map(([label, key]) => (
                      <div key={label}>
                        <span>{label}</span>
                        <kbd>{key}</kbd>
                      </div>
                    ))}
                  </div>
                  <div className="local-card">
                    <span className="dot" />
                    <div>
                      <b></b>
                      <p></p>
                    </div>
                  </div>
                </>
              )
            ) : right === "comments" ? (
              <>
                <div className="section-label">
                  MAKE YOUR FEEDBACK ACTIONABLE
                </div>
                <p className="muted">
                  Comments capture the playhead and selected layer. An agent can
                  turn them into edits.
                </p>
                <div className="comment-compose">
                  <div>
                    <span className="anchor-chip">{(()=>{const a=comment?frozenAnchor.current:anchor();return `${formatTimecode(Math.round(a.tStart*project.settings.fps),project.settings.fps)}${a.tEnd!==undefined?` → ${formatTimecode(Math.round(a.tEnd*project.settings.fps),project.settings.fps)}`:""}`})()}</span>
                    {selected && (
                      <span className="anchor-chip">{selected.slice(-12)}</span>
                    )}
                  </div>

                  <textarea
                    placeholder="Make the headline a little bolder…"
                    aria-label="New comment"
                    onFocus={()=>{if(!comment)frozenAnchor.current=anchor()}}
                    value={comment}
                    onChange={(e) => setComment(e.target.value)}
                  />
                  <button
                    className="primary"
                    disabled={locked || !comment.trim()}
                    onClick={() => {
                      const captured = frozenAnchor.current;
                      if (askMode) {
                        void op("comments_add", {
                          text: comment,
                          anchor: captured,
                        })
                          .then((result) =>
                            op("agent_run", {
                              client: state.agentClient,
                              commentIds: [result.commentId],
                              suggest,
                            }),
                          )
                          .catch(() => {});
                        setAskMode(false);
                      } else
                        run("comments_add", {
                          text: comment,
                          anchor: captured,
                        });
                      setComment("");
                    }}
                  >
                    <Plus size={13} />
                    {askMode ? "Send to agent" : "Add comment"}
                  </button>
                </div>
                <label className="suggest-toggle">
                  <input
                    type="checkbox"
                    checked={suggest}
                    onChange={(e) => setSuggest(e.target.checked)}
                  />
                  Suggest changes for review
                </label>
                <div className="agent-control">
                  <select
                    aria-label="Agent client"
                    value={state.agentClient}
                    disabled={locked}
                    onChange={(e) =>
                      run("agent_client", { client: e.target.value })
                    }
                  >
                    <option value="codex">Codex CLI</option>
                    <option value="claude">Claude CLI</option>
                  </select>
                  <button
                    className="primary"
                    disabled={
                      locked ||
                      !project.comments.some((c) => c.status === "open")
                    }
                    onClick={() =>
                      run("agent_run", {
                        client: state.agentClient,
                        suggest,
                        commentIds: project.comments
                          .filter((c) => c.status === "open")
                          .map((c) => c.id),
                      })
                    }
                  >
                    <Sparkles size={13} />
                    Apply open
                  </button>
                </div>
                {project.comments.map((c) => (
                  <article key={c.id} className={`comment-card ${c.status}`}>
                    <div className="comment-meta">
                      <span className="comment-avatar">
                        {c.author === "human" ? (
                          "Y"
                        ) : c.author === "voice" ? (
                          <Mic size={12} />
                        ) : (
                          <Sparkles size={12} />
                        )}
                      </span>
                      <b>
                        {c.author === "human"
                          ? "You"
                          : c.author === "voice"
                            ? "Voice note"
                            : "Agent"}
                      </b>
                      <button
                        className="anchor-chip"
                        onClick={() => {
                          seek(c.anchor.tStart);
                          setSelected(affectedItems(project,[c.anchor])[0]?.id??c.anchor.layerId);
                          setRange(c.anchor.tEnd!==undefined?{start:c.anchor.tStart,end:c.anchor.tEnd}:null);
                        }}
                      >
                        {c.anchor.tEnd!==undefined?`${formatTime(c.anchor.tStart)}–${formatTime(c.anchor.tEnd)}`:formatTime(c.anchor.tStart)}
                      </button>
                      <span className="comment-status">
                        {c.status}
                        {c.resolvedRev !== undefined
                          ? ` · r${c.resolvedRev}`
                          : ""}
                      </span>
                    </div>
                    <p>{c.text}</p>
                    {c.anchor.layerId && (
                      <small className="muted">
                        ↳ {c.anchor.layerId}
                        {c.anchor.propPath ? ` · ${c.anchor.propPath}` : ""}
                      </small>
                    )}
                    {c.replies.map((r, i) => (
                      <blockquote key={i}>{r.text}</blockquote>
                    ))}
                    {!!c.diff?.length && (
                      <details className="comment-diff">
                        <summary>View {c.diff.length} changes</summary>
                        <div className="diff-values">
                          {c.diff.map((change: any, index: number) => (
                            <div key={index}>
                              <code>{change.path}</code>
                              <div>
                                <span>Before</span>
                                <pre>
                                  {change.beforeExists === false
                                    ? "(absent)"
                                    : JSON.stringify(
                                        Object.hasOwn(change, "before")
                                          ? change.before
                                          : "(not recorded)",
                                        null,
                                        2,
                                      )}
                                </pre>
                              </div>
                              <div>
                                <span>After</span>
                                <pre>
                                  {change.afterExists === false
                                    ? "(removed)"
                                    : JSON.stringify(
                                        Object.hasOwn(change, "after")
                                          ? change.after
                                          : Object.hasOwn(change, "value")
                                            ? change.value
                                            : "(not recorded)",
                                        null,
                                        2,
                                      )}
                                </pre>
                              </div>
                            </div>
                          ))}
                        </div>
                      </details>
                    )}
                    {c.status !== "resolved" && (
                      <button
                        className="text-button"
                        disabled={locked}
                        onClick={() =>
                          run("comments_resolve", { commentId: c.id })
                        }
                      >
                        <Check size={12} />
                        Resolve
                      </button>
                    )}
                  </article>
                ))}
                {!project.comments.length && (
                  <div className="empty-small">
                    <MessageSquare />
                    <b>Keep the conversation close.</b>
                    <p>Leave a note at any moment in your story.</p>
                  </div>
                )}
                <div className="section-label property-section">ACTIVITY</div>
                {state.activity
                  .slice(-8)
                  .reverse()
                  .map((a, i) => (
                    <div className="activity" key={i}>
                      <span className="dot" />
                      {a.text}
                    </div>
                  ))}
                <button
                  className="text-button"
                  disabled={locked}
                  onClick={() => run("history_revert_turn")}
                >
                  Revert last agent turn
                </button>
              </>
            ) : right === "history" ? (
              <>
                <div className="section-label">EVERY EDIT, ACCOUNTED FOR</div>
                <h3 className="voice-title">The story so far.</h3>
                <p className="muted">
                  Review actual project revisions and their changes.
                </p>
                <div className="history-actions">
                  <button
                    className="primary"
                    disabled={locked}
                    onClick={() => run("history_undo")}
                  >
                    <Undo2 size={12} />
                    Undo
                  </button>
                  <button
                    className="primary"
                    disabled={locked}
                    onClick={() => run("history_redo")}
                  >
                    <Redo2 size={12} />
                    Redo
                  </button>
                </div>
                {[...state.history].reverse().map((h) => (
                  <article className="history-entry" key={h.id}>
                    <div>
                      <span>REV {h.rev}</span>
                      <small>
                        {h.author} · {h.kind}
                      </small>
                    </div>
                    <button
                      className="history-select"
                      onClick={() => {
                        setHistoryHighlight(h.changedItemIds);
                        const changed = items.find((i) =>
                          h.changedItemIds.includes(i.id),
                        );
                        if (changed) {
                          setSelected(changed.id);
                          seek(changed.at);
                        }
                      }}
                    >
                      {h.label}
                    </button>
                    <time>{new Date(h.timestamp).toLocaleTimeString()}</time>
                    <p>
                      {h.changedItemIds.length} element
                      {h.changedItemIds.length === 1 ? "" : "s"} changed
                    </p>
                    <button
                      className="text-button"
                      disabled={locked}
                      onClick={() => run("history_revert", { rev: h.rev })}
                    >
                      <Undo2 size={11} />
                      Revert this edit
                    </button>
                    <details className="comment-diff">
                      <summary>View {h.ops.length} operations</summary>
                      <pre>{JSON.stringify(h.ops, null, 2)}</pre>
                    </details>
                  </article>
                ))}
                {!state.history.length && (
                  <div className="empty-small">
                    Your edits will appear here.
                  </div>
                )}
              </>
            ) : (
              <>
                <div className="section-label">A VOICE FOR YOUR STORY</div>
                <h3 className="voice-title">Make it sound like you.</h3>
                <p className="muted">
                  Write your narration, then choose a voice or record each line.
                </p>
                <textarea
                  className="script-input"
                  aria-label="Voice script"
                  placeholder={"One line. One idea.\nYour story starts here."}
                  value={script}
                  onChange={(e) => setScript(e.target.value)}
                />
                <button
                  className="primary full"
                  disabled={locked || !script.trim()}
                  onClick={() => {
                    run("voice_script", { text: script });
                    setScript("");
                  }}
                >
                  <Plus size={14} />
                  Create voice slots
                </button>
                <div className="voice-bulk">
                  <button
                    disabled={
                      locked ||
                      !project.voiceSlots.length ||
                      (!!recording && recording !== "script-recording")
                    }
                    onClick={() =>
                      recording === "script-recording"
                        ? recorder.current?.stop()
                        : void startRecord("script-recording")
                    }
                  >
                    <Mic size={12} />
                    {recording === "script-recording"
                      ? `Stop script · ${recordSecs.toFixed(0)}s`
                      : "Record whole script"}
                  </button>
                  <button
                    disabled={
                      locked || !state.provider.fish || !state.clone?.id
                    }
                    onClick={() => run("voice_convert_all")}
                  >
                    <Sparkles size={12} />
                    Convert all to my clone
                  </button>
                </div>
                {recording === "script-recording" && (
                  <div className="script-teleprompter">
                    {project.voiceSlots.map((slot) => (
                      <p key={slot.id}>{slot.text}</p>
                    ))}
                  </div>
                )}
                <div className="provider-status">
                  <span
                    className={`dot ${state.provider.fish ? "" : "offline"}`}
                  />
                  {state.provider.fish
                    ? "Fish Audio connected"
                    : "AI voice requires FISH_API_KEY"}
                </div>
                {project.voiceSlots.map((slot, i) => (
                  <VoiceCard
                    key={slot.id}
                    slot={slot}
                    index={i}
                    locked={locked}
                    provider={state.provider.fish}
                    clone={state.clone?.id}
                    op={run}
                    recording={recording === slot.id}
                    onRecord={() =>
                      recording === slot.id
                        ? recorder.current?.stop()
                        : void startRecord("recording", slot.id)
                    }
                    recordSecs={recordSecs}
                  />
                ))}
                <div className="clone-card">
                  <Mic size={18} />
                  <h4>Your voice, on demand.</h4>
                  <p>
                    Record a clean sample of at least 10 seconds to create a
                    Fish Audio voice clone. Only record your own voice or a
                    voice you have permission to clone. By recording, you
                    confirm you have those rights.
                  </p>
                  <label className="clone-consent">
                    <input
                      type="checkbox"
                      checked={cloneConsent}
                      disabled={recording === "clone-sample"}
                      onChange={(e) => setCloneConsent(e.target.checked)}
                    />
                    I own this voice or have explicit permission to clone it.
                  </label>
                  <button
                    disabled={
                      locked ||
                      !cloneConsent ||
                      !state.provider.fish ||
                      (!!recording && recording !== "clone-sample")
                    }
                    onClick={() =>
                      recording === "clone-sample"
                        ? recorder.current?.stop()
                        : void startRecord("clone-sample")
                    }
                  >
                    {recording === "clone-sample"
                      ? `Stop · ${recordSecs.toFixed(0)}s`
                      : "Record voice sample"}
                  </button>
                  {state.clone && <small>Clone: {state.clone.status}</small>}
                </div>
              </>
            )}
            {((state as any).suggestions ?? [])
              .filter((proposal: any) => proposal.status === "pending")
              .map((proposal: any) => (
                <div className="proposal-card" key={proposal.id}>
                  <span className="section-label">
                    SUGGESTED · BASE R{proposal.baseRev}
                  </span>
                  <h4>{proposal.label}</h4>
                  <p>{proposal.ops.length} proposed operations</p>
                  <div>
                    <button
                      disabled={locked}
                      onClick={() =>
                        run("proposal_accept", { id: proposal.id })
                      }
                    >
                      <Check size={12} />
                      Accept
                    </button>
                    <button
                      disabled={locked}
                      onClick={() =>
                        run("proposal_reject", { id: proposal.id })
                      }
                    >
                      <X size={12} />
                      Reject
                    </button>
                  </div>
                </div>
              ))}
            {pendingProposal && (
              <div className="proposal-all">
                <button
                  disabled={locked}
                  onClick={() => run("proposal_accept_all")}
                >
                  Accept all
                </button>
                <button
                  disabled={locked}
                  onClick={() => run("proposal_reject_all")}
                >
                  Reject all
                </button>
              </div>
            )}
          </div>
          {state.briefStudio?.waiting&&!locked&&<div className="lease-banner" role="status">Waiting for your brief or answer · Studio is editable</div>}
          {locked && (
            <div className="lease-banner">
              <LoaderCircle size={15} className="spin" />
              <div>
                <b>{state.lease?.owner} is editing</b>
                <small>{state.lease?.label}</small>
                {project.pipeline&&<small aria-label="Pipeline progress">{project.mode?.toUpperCase()} · {project.pipeline.stage}{project.pipeline.pass?` · ${project.pipeline.pass}`:""}</small>}
                {activeTouched.length === 0 && (
                  <small>Waiting for this turn’s first project change</small>
                )}
                {activeTouched.length > 0 && (
                  <ul className="touched-layers">
                    {activeTouched.map((id) => (
                      <li key={id}>
                        <button
                          onClick={() => {
                            setSelected(id);
                            const touched = items.find((i) => i.id === id);
                            if (touched) seek(touched.at);
                          }}
                        >
                          {id}
                        </button>
                      </li>
                    ))}
                  </ul>
                )}
              </div>
              <button title="Cancel agent" onClick={() => run("agent_cancel")}>
                <X size={15} />
              </button>
            </div>
          )}
          {state.agentError && (
            <div className="error-box">{state.agentError}</div>
          )}
        </aside>
      </div>
      <div className="timeline-resizer" {...workspace.separator("timeline")}/><section className="timeline">
        <div className="timeline-toolbar">
          <div className="timeline-title">
            <Layers size={15} />
            <b>Timeline</b>
            <span>{items.length} elements</span>
          </div>
          <div className="timeline-tools">
            <IconButton title="Select tool" active={!rangeTool} onClick={() => setRangeTool(false)}>
              <MousePointer2 size={15} />
            </IconButton>
            <IconButton
              title="Split selected (S)"
              disabled={locked || !item}
              onClick={() =>
                item && run("item_split", { itemId: item.id, at: time })
              }
            >
              <Scissors size={15} />
            </IconButton>
            <IconButton
              title="Duplicate selected"
              disabled={locked || !item}
              onClick={() => item && run("item_duplicate", { itemId: item.id })}
            >
              <Copy size={15} />
            </IconButton>
            <IconButton
              title="Delete selected"
              disabled={locked || !item}
              onClick={() => item && run("item_delete", { itemId: item.id })}
            >
              <Trash2 size={15} />
            </IconButton>
            <IconButton title="Range tool (I / O)" active={rangeTool} onClick={()=>setRangeTool(v=>!v)}><span>↔</span></IconButton>
            <span className="tool-divider" />
            <span className="snap-status">⌁ Snap · Shift-drag range</span>
            {range && (
              <button
                className="range-control"
                onClick={() => {
                  setLoop((v) => !v);
                  seek(range.start);
                }}
              >
                {loop ? "Looping" : "Loop"} {formatTime(range.start)}–
                {formatTime(range.end)}
              </button>
            )}
            {range && (
              <button
                className="text-button"
                onClick={() => {
                  setRange(null);
                  setLoop(false);
                  setRangeEnd("");
                }}
              >
                Clear range
              </button>
            )}
          </div>
          <div className="zoom">
            <IconButton
              title="Zoom out"
              onClick={() => setZoom((z) => Math.max(20, z - 15))}
            >
              <ZoomOut size={14} />
            </IconButton>
            <input
              aria-label="Timeline zoom"
              type="range"
              min={20}
              max={180}
              value={zoom}
              onChange={(e) => setZoom(Number(e.target.value))}
            />
            <IconButton
              title="Zoom in"
              onClick={() => setZoom((z) => Math.min(180, z + 15))}
            >
              <ZoomIn size={14} />
            </IconButton>
          </div>
        </div>
        <FrameSelection range={range} time={time} fps={project.settings.fps} total={total} onChange={setRange} onComment={(selectedRange)=>{frozenAnchor.current={...anchor(),...selectedRange?{tStart:selectedRange.start,tEnd:selectedRange.end}:{}};setRight("comments");workspace.show("inspector");requestAnimationFrame(()=>document.querySelector<HTMLTextAreaElement>('[aria-label="New comment"]')?.focus())}}/>
        <div className="timeline-scroll">
          <div
            className="timeline-content"
            style={{ minWidth: Math.max(900, (total + 3) * zoom + 140) }}
          >
            <div className="ruler">
              <div className="track-heading">
                <span>TRACKS</span>
              </div>
              <div
                className="ruler-marks"
                onPointerDown={scrubTimeline}
                title="Drag to scrub; Shift-drag to select a range"
              >
                {Array.from({ length: Math.ceil(total + 3) }, (_, i) => (
                  <span style={{ left: i * zoom }} key={i}>
                    {formatTime(i)}
                  </span>
                ))}
              </div>
            </div>
            {project.tracks.map((track, ti) => {
              const lanes = trackLanes(track.items);
              return (
                <div
                  className="track-row"
                  key={track.id}
                  style={{ height: lanes.count * 34 + 4 }}
                >
                  <div className="track-heading">
                    <span className={`track-color track-${track.kind}`} />
                    {track.kind === "audio" || track.kind === "voice" ? (
                      <AudioLines size={13} />
                    ) : track.kind === "caption" ? (
                      <span className="cc">CC</span>
                    ) : (
                      <Film size={13} />
                    )}
                    <b>{track.name}</b>
                    <small>{String(ti + 1).padStart(2, "0")}</small>
                  </div>
                  <div
                    className="track-lane"
                    data-track-id={track.id}
                    onPointerDown={(e) => {
                      if (e.target === e.currentTarget) {
                        const rect = e.currentTarget.getBoundingClientRect();
                        seek((e.clientX - rect.left) / zoom);
                        setSelected(undefined);
                      }
                    }}
                  >
                    {track.items.map((original) => {
                      const i =
                        gesture?.id === original.id
                          ? { ...original, ...gesture }
                          : original;
                      const label = i.clipId
                        ? (assets.find((a) => a.id === i.clipId)?.name ??
                          i.clipId)
                        : i.slotId
                          ? project.voiceSlots.find((s) => s.id === i.slotId)
                              ?.text
                          : i.src
                            ? project.sources[i.src]?.name
                            : (i.words?.map((w) => w.text).join(" ") ?? i.type);
                      return (
                        <div
                          role="button"
                          tabIndex={0}
                          aria-label={`${label}, ${formatTime(i.at)}, duration ${i.dur} seconds`}
                          key={i.id}
                          className={`timeline-clip clip-${i.type} ${selected === i.id ? "selected" : ""} ${historyHighlight.includes(i.id) ? "history-highlight" : ""}`}
                          style={{
                            left: i.at * zoom,
                            width: Math.max(6, i.dur * zoom),
                            top: 3 + lanes.positions[i.id] * 34,
                            height: 28,
                          }}
                          onPointerDown={(e) => beginGesture(e, i, "move")}
                          onKeyDown={(e) =>
                            e.key === "Enter" && setSelected(i.id)
                          }
                        >
                          <span
                            className="trim-handle left"
                            onPointerDown={(e) => beginGesture(e, i, "left")}
                          />
                          <span className="clip-label">
                            {i.type === "voice" || i.type === "audio" ? (
                              <AudioLines size={13} />
                            ) : i.type === "clip" ? (
                              <Sparkles size={12} />
                            ) : (
                              <Film size={12} />
                            )}{" "}
                            {label}
                          </span>
                          <div
                            className={`clip-pattern ${i.type === "audio" || i.type === "voice" ? "audio-pattern" : ""}`}
                          />
                          <span
                            className="trim-handle right"
                            onPointerDown={(e) => beginGesture(e, i, "right")}
                          />
                        </div>
                      );
                    })}
                    {pendingProposal?.previewProject?.tracks
                      .find((candidate: any) => candidate.id === track.id)
                      ?.items.filter(
                        (candidate: Item) =>
                          JSON.stringify(
                            items.find(
                              (current) => current.id === candidate.id,
                            ),
                          ) !== JSON.stringify(candidate),
                      )
                      .map((candidate: Item) => (
                        <div
                          className="ghost-timeline-item"
                          key={`ghost-${candidate.id}`}
                          style={{
                            left: candidate.at * zoom,
                            width: candidate.dur * zoom,
                          }}
                          title={`Proposed ${candidate.clipId ?? candidate.type}`}
                        >
                          {candidate.clipId ?? candidate.type} · proposed
                        </div>
                      ))}
                  </div>
                </div>
              );
            })}
            {range && (
              <div
                className="timeline-range"
                style={{
                  left: 140 + range.start * zoom,
                  width: (range.end - range.start) * zoom,
                }}
              />
            )}
            {range&&<FrameHandles range={range} fps={project.settings.fps} zoom={zoom} total={total} onChange={setRange}/>}
            {project.comments.map((c) => (
              <button
                className={`timeline-comment ${c.status} ${c.anchor.tEnd!==undefined?"range-band":""}`}
                key={c.id}
                title={`${c.text}${c.anchor.tEnd ? ` · to ${formatTime(c.anchor.tEnd)}` : ""}`}
                aria-label={`Comment: ${c.text}`}
                style={{ left: 140 + c.anchor.tStart * zoom,...c.anchor.tEnd!==undefined?{width:Math.max(18,(c.anchor.tEnd-c.anchor.tStart)*zoom)}:{} }}
                onClick={() => {
                  seek(c.anchor.tStart);
                  setSelected(affectedItems(project,[c.anchor])[0]?.id??c.anchor.layerId);
                  setRight("comments");
                  if (c.anchor.tEnd)
                    setRange({ start: c.anchor.tStart, end: c.anchor.tEnd });
                }}
              >
                <MessageSquare size={11} />
              </button>
            ))}
            {locked && typeof (state as any).agentPosition === "number" && (
              <div
                className="agent-marker"
                style={{ left: 140 + (state as any).agentPosition * zoom }}
              >
                <Sparkles size={12} />
                <span>Agent position</span>
              </div>
            )}
            <div className="playhead" style={{ left: 140 + time * zoom }}>
              <span />
            </div>
            {!items.length && (
              <div className="timeline-empty">
                Your next great story starts here. Add an element or drop in
                your footage.
              </div>
            )}
          </div>
        </div>
        <footer className="statusbar">
          <span>
            <span className="dot" />
            {locked
              ? `${state.lease?.owner} owns the edit lease`
              : "You’re in control"}
            <span className="status-divider">/</span>Revision {project.rev}
          </span>
          <span>
            {state.jobs.some(
              (j) => j.status === "running" || j.status === "queued",
            ) ? (
              <>
                <LoaderCircle size={11} className="spin" />
                Processing
              </>
            ) : (
              <>
                <Check size={11} />
                All changes saved
              </>
            )}
            <span className="status-divider">/</span>Local project
          </span>
        </footer>
      </section>
      {exportDialog && (
        <div className="modal-backdrop">
          <section
            className="export-dialog"
            role="dialog"
            aria-modal="true"
            aria-label="Export video settings"
          >
            <div className="modal-heading">
              <h2>Ready for the next screen.</h2>
              <button
                aria-label="Close export settings"
                onClick={() => setExportDialog(false)}
              >
                <X size={18} />
              </button>
            </div>
            <p>
              Export original media at revision {project.rev}. Browser preview
              voices are excluded.
            </p>
            <label className="field">
              <span>Format</span>
              <select
                aria-label="Export preset"
                value={exportPreset}
                onChange={(e) => setExportPreset(e.target.value)}
              >
                <option value="vertical-720">Vertical · 720 × 1280</option>
                <option value="vertical-1080">Vertical · 1080 × 1920</option>
                <option value="square">Square · 1080 × 1080</option>
                <option value="landscape">Landscape · 1280 × 720</option>
              </select>
            </label>
            <label className="field">
              <span>Quality</span>
              <select
                aria-label="Export quality"
                value={exportQuality}
                onChange={(e) => setExportQuality(e.target.value)}
              >
                <option value="draft">Draft</option>
                <option value="standard">Standard</option>
                <option value="high">High</option>
              </select>
            </label>
            <details className="export-licenses" open>
              <summary>Used assets & licenses</summary>
              {licenses ? (
                <>
                  <ul>
                    {[
                      ...(licenses.clips ?? []),
                      ...(licenses.assets ?? []),
                    ].map((entry: any, index: number) => (
                      <li key={index}>
                        <b>{entry.name}</b>
                        <span>{entry.license}</span>
                        {entry.sourceUrl && (
                          <a
                            href={entry.sourceUrl}
                            target="_blank"
                            rel="noreferrer"
                          >
                            Source
                          </a>
                        )}
                      </li>
                    ))}
                  </ul>
                  {licenses.notices?.map((notice: string, index: number) => (
                    <p key={index}>{notice}</p>
                  ))}
                  {!licenses.clips?.length && !licenses.assets?.length && (
                    <p>No licensed catalog assets used.</p>
                  )}
                </>
              ) : (
                <p>Loading actual used-license list…</p>
              )}
            </details>
            <button
              className="primary full"
              disabled={locked || !licenses}
              onClick={() => {
                run("export_start", {
                  preset: exportPreset,
                  quality: exportQuality,
                });
                setExportDialog(false);
              }}
            >
              <ArrowDownToLine size={14} />
              Render video
            </button>
          </section>
        </div>
      )}
      {compare && latestExport && originalVideo && (
        <div className="modal-backdrop">
          <section
            className="compare-dialog"
            role="dialog"
            aria-modal="true"
            aria-label="Compare raw footage and export"
          >
            <div className="modal-heading">
              <h2>From first frame to final cut.</h2>
              <button
                aria-label="Close comparison"
                onClick={() => setCompare(false)}
              >
                <X size={18} />
              </button>
            </div>
            <div className="comparison-videos">
              <figure>
                <figcaption>Raw · {originalVideo.name}</figcaption>
                <video
                  controls
                  src={`/media/${encodeURIComponent(originalVideo.id)}?variant=original&token=${encodeURIComponent(token)}`}
                />
              </figure>
              <figure>
                <figcaption>
                  Export · revision{" "}
                  {String(latestExport.result?.revision ?? "saved")}
                </figcaption>
                <video controls src={String(latestExport.result?.url)} />
              </figure>
            </div>
          </section>
        </div>
      )}
      {toast && (
        <div className="toast" role="status">
          <span>{toast}</span>
          <button
            aria-label="Dismiss notification"
            onClick={() => setToast("")}
          >
            <X size={14} />
          </button>
        </div>
      )}
      {countdown > 0 && (
        <div className="countdown-overlay">
          <b>{countdown}</b>
          <span>Get ready to read your line</span>
        </div>
      )}
    </div>
  );
}
function VoiceCard({
  slot,
  index,
  locked,
  provider,
  clone,
  op,
  recording,
  onRecord,
  recordSecs,
}: {
  slot: VoiceSlot;
  index: number;
  locked: boolean;
  provider: boolean;
  clone?: string;
  op: (name: string, args?: Record<string, unknown>) => void;
  recording: boolean;
  onRecord: () => void;
  recordSecs: number;
}) {
  const [voiceRef, setVoiceRef] = useState(slot.voiceRef ?? ""),
    [speed, setSpeed] = useState(1);
  useEffect(() => setVoiceRef(slot.voiceRef ?? ""), [slot.voiceRef]);
  return (
    <article className={`voice-card ${recording ? "recording" : ""}`}>
      <div className="voice-meta">
        <span>LINE {String(index + 1).padStart(2, "0")}</span>
        <small>
          {formatTime(slot.targetWindow.start)} →{" "}
          {formatTime(slot.targetWindow.end)}
        </small>
      </div>
      <p>{slot.text}</p>
      <fieldset disabled={locked}>
        <div className="field-row">
          <Field
            label="Window start (s)"
            type="number"
            min={0}
            step={0.1}
            value={slot.targetWindow.start}
            onCommit={(start) =>
              op("voice_window", {
                slotId: slot.id,
                start,
                end: slot.targetWindow.end,
              })
            }
          />
          <Field
            label="Window end (s)"
            type="number"
            min={slot.targetWindow.start + 0.1}
            step={0.1}
            value={slot.targetWindow.end}
            onCommit={(end) =>
              op("voice_window", {
                slotId: slot.id,
                start: slot.targetWindow.start,
                end,
              })
            }
          />
        </div>
      </fieldset>
      <div className="voice-modes">
        {(["ai", "recorded", "cloned"] as const).map((mode) => (
          <button
            key={mode}
            disabled={locked}
            className={slot.mode === mode ? "selected" : ""}
            onClick={() => op("voice_mode", { slotId: slot.id, mode })}
          >
            {mode === "ai"
              ? "AI voice"
              : mode === "recorded"
                ? "Record"
                : "My clone"}
          </button>
        ))}
      </div>
      {slot.mode !== "recorded" && (
        <fieldset disabled={locked} className="voice-generation-settings">
          {slot.mode === "ai" && (
            <Field
              label="Fish voice ID (optional)"
              value={voiceRef}
              onCommit={setVoiceRef}
            />
          )}
          <Field
            label="Voice speed"
            type="number"
            min={0.5}
            max={2}
            step={0.05}
            value={speed}
            onCommit={setSpeed}
          />
        </fieldset>
      )}
      {slot.mode === "recorded" ? (
        <button
          className={recording ? "recording-action" : "record-action"}
          disabled={locked}
          onClick={onRecord}
        >
          {recording ? (
            <Square size={12} fill="currentColor" />
          ) : (
            <Mic size={13} />
          )}{" "}
          {recording
            ? `Stop recording · ${recordSecs.toFixed(0)}s`
            : "Record this line"}
        </button>
      ) : (
        <button
          className="record-action"
          disabled={locked || !provider || (slot.mode === "cloned" && !clone)}
          onClick={() =>
            op("voice_generate", {
              slotId: slot.id,
              speed,
              ...(slot.mode === "cloned"
                ? { voiceRef: clone }
                : voiceRef.trim()
                  ? { voiceRef: voiceRef.trim() }
                  : {}),
            })
          }
        >
          <Sparkles size={13} />
          Generate {slot.mode === "cloned" ? "with my voice" : "voice"}
        </button>
      )}
      {!provider && (
        <button
          className="record-action browser-voice"
          disabled={
            locked || recording || typeof window.speechSynthesis === "undefined"
          }
          onClick={() => {
            window.speechSynthesis.cancel();
            const utterance = new SpeechSynthesisUtterance(slot.text);
            utterance.rate = speed;
            window.speechSynthesis.speak(utterance);
          }}
        >
          <Play size={12} />
          Browser voice · preview only
        </button>
      )}
      {!provider && (
        <small className="preview-only-note">
          Temporary browser speech is never included in exports.{" "}
          <button
            className="text-button"
            onClick={() => window.speechSynthesis?.cancel()}
          >
            Stop preview
          </button>
        </small>
      )}
      {recording && <div className="teleprompter">{slot.text}</div>}
      {slot.takes.some(
        (t) =>
          t.id === slot.activeTakeByMode[slot.mode] &&
          Math.abs(
            t.durationSec - (slot.targetWindow.end - slot.targetWindow.start),
          ) > 0.3,
      ) && (
        <div className="take-mismatch">
          Take length differs from its window.
          <div className="fit-options">
            {(["stretch", "extend", "ripple", "rerecord"] as const).map(
              (strategy) => (
                <button
                  disabled={locked}
                  key={strategy}
                  onClick={() => {
                    op("voice_fit", { slotId: slot.id, strategy });
                    if (strategy === "rerecord") onRecord();
                  }}
                >
                  {strategy === "stretch"
                    ? "Fit audio"
                    : strategy === "extend"
                      ? "Extend visuals"
                      : strategy === "ripple"
                        ? "Ripple timeline"
                        : "Record again"}
                </button>
              ),
            )}
          </div>
        </div>
      )}
      {slot.takes.length > 0 && (
        <label className="field">
          <span>Active take · {slot.status}</span>
          <select
            disabled={locked}
            value={slot.activeTakeByMode[slot.mode] ?? ""}
            onChange={(e) =>
              op("voice_take", { slotId: slot.id, takeId: e.target.value })
            }
          >
            <option value="" disabled>
              Choose a take
            </option>
            {slot.takes
              .filter((t) => t.mode === slot.mode)
              .map((t, i) => (
                <option key={t.id} value={t.id}>
                  Take {i + 1} · {t.durationSec.toFixed(1)}s
                </option>
              ))}
          </select>
        </label>
      )}
      {slot.takes.length > 0 && (
        <details className="take-list">
          <summary>{slot.takes.length} saved takes · provenance</summary>
          {slot.takes.map((t) => (
            <div key={t.id}>
              <b>
                {t.mode === "recorded"
                  ? "Human recording"
                  : t.mode === "cloned"
                    ? "AI · cloned voice"
                    : "AI · generated voice"}
              </b>
              <span>
                {t.durationSec.toFixed(1)}s ·{" "}
                {new Date(t.createdAt).toLocaleTimeString()}
              </span>
              <small>{t.sourceId}</small>
            </div>
          ))}
        </details>
      )}
    </article>
  );
}
createRoot(document.getElementById("root")!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
