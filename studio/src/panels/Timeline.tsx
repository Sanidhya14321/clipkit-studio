import {
  memo,
  useEffect,
  useMemo,
  useRef,
  useState,
  type PointerEvent as ReactPointerEvent,
} from "react";
import type { PatchRequest, Project } from "../generated/types";
import type { Revision } from "../state";
import type { PlaybackClock } from "../editor/clock";
import { duration, span } from "../timeline";
import { moveRequest, trimRequest, snapTime } from "../editor/edits";

const ROW = 48;
const RULER = 30;
export const Timeline = memo(function Timeline({
  project,
  agentLayers,
  lastRevision,
  selected,
  onSelect,
  clock,
  onSeek,
  onCommit,
  onSplit,
  onDelete,
  readOnly = false,
  pending = false,
}: {
  project: Project;
  agentLayers: Set<string>;
  lastRevision: Revision | null;
  selected: string | null;
  onSelect: (id: string) => void;
  clock?: PlaybackClock;
  onSeek?: (seconds: number) => void;
  onCommit?: (request: PatchRequest | null) => Promise<boolean>;
  onSplit?: () => void;
  onDelete?: (ripple: boolean) => void;
  readOnly?: boolean;
  pending?: boolean;
}) {
  const [zoom, setZoom] = useState(70);
  const [snapping, setSnapping] = useState(true);
  const [scroll, setScroll] = useState(0);
  const [width, setWidth] = useState(800);
  const [themeVersion, setThemeVersion] = useState(0);
  const [gesture, setGesture] = useState<{
    id: string;
    at: number;
    end: number;
  } | null>(null);
  const [notice, setNotice] = useState("");
  const viewport = useRef<HTMLDivElement>(null),
    canvas = useRef<HTMLCanvasElement>(null),
    playhead = useRef<HTMLDivElement>(null),
    headers = useRef<HTMLDivElement>(null);
  const total = Math.max(10, duration(project) + 2);
  const contentWidth = Math.max(width, total * zoom);
  const tracks = useMemo(
    () =>
      project.tracks.map((track) => ({
        ...track,
        entries: track.items
          .map((item) => ({
            item,
            range:
              span(project, item) ??
              (item.type === "caption"
                ? { start: 0, end: Math.max(1, duration(project)) }
                : item.type === "voice"
                  ? {
                      start: item.at,
                      end:
                        item.at +
                        Math.max(
                          0.5,
                          project.voiceSlots
                            .find((s) => s.id === item.slotId)
                            ?.text.split(/\s+/).length ?? 1,
                        ) /
                          2.5,
                    }
                  : null),
          }))
          .filter((entry) => entry.range !== null),
      })),
    [project],
  );
  const points = useMemo(
    () =>
      tracks.flatMap((track) =>
        track.entries.flatMap((e) => [e.range!.start, e.range!.end]),
      ),
    [tracks],
  );
  useEffect(() => {
    const el = canvas.current?.closest(".workbench");
    if (!el) return;
    const observer = new MutationObserver(() => setThemeVersion((v) => v + 1));
    observer.observe(el, { attributes: true, attributeFilter: ["data-theme"] });
    return () => observer.disconnect();
  }, []);
  useEffect(() => {
    const el = viewport.current;
    if (!el) return;
    const observer = new ResizeObserver(() => setWidth(el.clientWidth));
    observer.observe(el);
    return () => observer.disconnect();
  }, []);
  useEffect(() => {
    if (!clock) return;
    const update = () => {
      if (playhead.current)
        playhead.current.style.transform = `translateX(${clock.get() * zoom}px)`;
    };
    update();
    return clock.subscribe(update);
  }, [clock, zoom]);
  useEffect(() => {
    const el = canvas.current;
    if (!el) return;
    const ratio = window.devicePixelRatio || 1,
      height = RULER + Math.max(1, tracks.length) * ROW + ROW;
    el.width = Math.ceil(width * ratio);
    el.height = Math.ceil(height * ratio);
    el.style.width = `${width}px`;
    el.style.height = `${height}px`;
    const ctx = el.getContext("2d");
    if (!ctx) return;
    ctx.scale(ratio, ratio);
    const theme = getComputedStyle(el),
      color = (name: string) => theme.getPropertyValue(name).trim();
    ctx.fillStyle = color("--ck-panel");
    ctx.fillRect(0, 0, width, height);
    const interval = zoom >= 100 ? 0.5 : zoom >= 50 ? 1 : zoom >= 20 ? 5 : 10;
    ctx.font = "10px ui-monospace, monospace";
    for (
      let t = Math.floor(scroll / zoom / interval) * interval;
      t <= (scroll + width) / zoom;
      t += interval
    ) {
      const x = t * zoom - scroll;
      ctx.strokeStyle = color("--ck-border");
      ctx.beginPath();
      ctx.moveTo(x, 20);
      ctx.lineTo(x, height);
      ctx.stroke();
      ctx.fillStyle = color("--ck-muted");
      ctx.fillText(`${t.toFixed(interval < 1 ? 1 : 0)}s`, x + 4, 13);
    }
    tracks.forEach((track, row) => {
      const y = RULER + row * ROW;
      ctx.strokeStyle = color("--ck-border");
      ctx.beginPath();
      ctx.moveTo(0, y + ROW);
      ctx.lineTo(width, y + ROW);
      ctx.stroke();
      for (const { item, range } of track.entries) {
        const active =
          gesture?.id === item.id
            ? { start: gesture.at, end: gesture.end }
            : range!;
        const x = active.start * zoom - scroll,
          w = Math.max(2, (active.end - active.start) * zoom);
        if (x + w < 0 || x > width) continue;
        ctx.fillStyle = color(`--ck-${track.kind}`);
        ctx.fillRect(x, y + 7, w, ROW - 14);
        ctx.strokeStyle =
          selected === item.id
            ? color("--ck-accent")
            : agentLayers.has(item.id)
              ? color("--ck-agent")
              : color("--ck-border");
        ctx.lineWidth = selected === item.id ? 2 : 1;
        ctx.strokeRect(x + 1, y + 8, Math.max(1, w - 2), ROW - 16);
        ctx.save();
        ctx.beginPath();
        ctx.rect(x + 7, y + 8, Math.max(0, w - 14), ROW - 16);
        ctx.clip();
        ctx.fillStyle = color("--ck-text");
        ctx.font = "11px system-ui, sans-serif";
        ctx.fillText(
          `${agentLayers.has(item.id) ? "▣ " : ""}${item.name || item.id}`,
          x + 8,
          y + 28,
        );
        ctx.restore();
      }
    });
  }, [
    tracks,
    width,
    scroll,
    zoom,
    selected,
    agentLayers,
    gesture,
    themeVersion,
  ]);

  const start = (
    e: ReactPointerEvent<HTMLButtonElement>,
    id: string,
    edge: "start" | "end" | "move",
  ) => {
    onSelect(id);
    if (!onCommit) return;
    if (readOnly || pending || agentLayers.has(id)) {
      setNotice(
        agentLayers.has(id)
          ? "The agent is editing this layer. You can edit other unlocked layers."
          : "Editing is unavailable while disconnected, locked or saving.",
      );
      return;
    }
    const track = project.tracks.find((t) => t.items.some((i) => i.id === id));
    const item = track?.items.find((i) => i.id === id);
    if (!item) return;
    const range = span(project, item);
    if (!range || !("at" in item)) return;
    if (edge !== "move" && item.type !== "clip" && item.type !== "footage") {
      setNotice("Trim this source through its take or audio range controls.");
      return;
    }
    e.preventDefault();
    e.currentTarget.setPointerCapture(e.pointerId);
    const button = e.currentTarget,
      origin = e.clientX;
    let target = edge === "end" ? range.end : range.start;
    let moved = false;
    const move = (event: PointerEvent) => {
      if (Math.abs(event.clientX - origin) < 3 && !moved) return;
      moved = true;
      const value =
        (edge === "end" ? range.end : range.start) +
        (event.clientX - origin) / zoom;
      const snapPoints = snapping
        ? [
            ...points.filter(
              (point) => point !== range.start && point !== range.end,
            ),
            clock?.get() ?? 0,
          ]
        : [];
      target = snapTime(
        value,
        snapPoints,
        project.settings.fps,
        snapping ? 7 / zoom : 0,
      );
      const request =
        edge === "move"
          ? moveRequest(project, id, target)
          : trimRequest(project, id, edge, target);
      if (!request) return;
      setGesture({
        id,
        at: edge === "end" ? range.start : target,
        end:
          edge === "move"
            ? target + range.end - range.start
            : edge === "start"
              ? range.end
              : target,
      });
    };
    const finish = () => {
      button.removeEventListener("pointermove", move);
      button.removeEventListener("pointerup", finish);
      button.removeEventListener("pointercancel", cancel);
      setGesture(null);
      if (moved)
        void onCommit(
          edge === "move"
            ? moveRequest(project, id, target)
            : trimRequest(project, id, edge, target),
        );
    };
    const cancel = () => {
      moved = false;
      finish();
    };
    button.addEventListener("pointermove", move);
    button.addEventListener("pointerup", finish);
    button.addEventListener("pointercancel", cancel);
  };
  const fit = () => setZoom(Math.max(5, Math.min(240, width / total)));
  return (
    <section
      className="timeline-editor"
      aria-label="Timeline"
      data-ck-timeline
      tabIndex={0}
      onKeyDown={(e) => {
        if (
          e.target instanceof Element &&
          e.target.closest("input,textarea,select")
        )
          return;
        if (e.key.toLowerCase() === "s") {
          e.preventDefault();
          setSnapping((s) => !s);
        }
        if (e.key === "+" || e.key === "=") {
          e.preventDefault();
          setZoom((z) => Math.min(300, z * 1.25));
        }
        if (e.key === "-") {
          e.preventDefault();
          setZoom((z) => Math.max(5, z / 1.25));
        }
        if (e.key === "\\") {
          e.preventDefault();
          fit();
        }
        if (e.key === "Delete" || e.key === "Backspace") {
          e.preventDefault();
          onDelete?.(e.shiftKey);
        }
        if (e.key === "ArrowLeft" || e.key === "ArrowRight") {
          e.preventDefault();
          onSeek?.(
            Math.max(
              0,
              (clock?.get() ?? 0) +
                ((e.key === "ArrowRight" ? 1 : -1) * (e.shiftKey ? 10 : 1)) /
                  project.settings.fps,
            ),
          );
        }
      }}
    >
      <div className="timeline-toolbar">
        <strong>Sequence</strong>
        <button
          onClick={onSplit}
          disabled={!selected || readOnly || pending}
          title="Split selected layer at playhead"
        >
          Split
        </button>
        <button
          onClick={() => onDelete?.(false)}
          disabled={!selected || readOnly || pending}
          title="Delete selected layer (Delete)"
        >
          Delete
        </button>
        <button
          onClick={() => onDelete?.(true)}
          disabled={!selected || readOnly || pending}
          title="Delete and close this track's gap (Shift Delete)"
        >
          Ripple delete
        </button>
        <span className="toolbar-spacer" />
        <button
          aria-pressed={snapping}
          onClick={() => setSnapping((v) => !v)}
          title="Snap to clip edges and playhead (S)"
        >
          Snap
        </button>
        <button
          aria-label="Zoom out timeline"
          onClick={() => setZoom((z) => Math.max(5, z / 1.25))}
        >
          −
        </button>
        <button onClick={fit} title="Fit sequence (\\)">
          Fit
        </button>
        <button
          aria-label="Zoom in timeline"
          onClick={() => setZoom((z) => Math.min(300, z * 1.25))}
        >
          +
        </button>
        <span className="timeline-duration">
          {duration(project).toFixed(2)}s · {project.settings.fps} fps
        </span>
      </div>
      {notice && (
        <div className="editor-notice" role="status">
          {notice}
          <button
            aria-label="Dismiss timeline notice"
            onClick={() => setNotice("")}
          >
            ×
          </button>
        </div>
      )}
      <div className="timeline-grid">
        <div className="timeline-headers" ref={headers}>
          <div className="timeline-ruler-label">TRACKS</div>
          {tracks.map((track) => (
            <div
              key={track.id}
              className="timeline-track-header"
              data-ck-track={track.id}
            >
              <span className={`track-kind kind-${track.kind}`}>
                {track.kind === "audio"
                  ? "A"
                  : track.kind === "caption"
                    ? "C"
                    : "V"}
              </span>
              <span>
                {track.id}
                <small>{track.items.length} layers</small>
              </span>
            </div>
          ))}
          <div className="timeline-track-header">
            <small>Comments</small>
          </div>
        </div>
        <div
          className="timeline-scroll"
          ref={viewport}
          onScroll={(e) => {
            setScroll(e.currentTarget.scrollLeft);
            if (headers.current)
              headers.current.scrollTop = e.currentTarget.scrollTop;
          }}
          onWheel={(e) => {
            if (e.ctrlKey || e.metaKey) {
              e.preventDefault();
              setZoom((z) =>
                Math.max(5, Math.min(300, z * (e.deltaY < 0 ? 1.1 : 0.9))),
              );
            }
          }}
        >
          <div
            className="timeline-surface"
            style={{
              width: contentWidth,
              height: RULER + (Math.max(1, tracks.length) + 1) * ROW,
            }}
          >
            <canvas
              ref={canvas}
              className="timeline-canvas"
              style={{ left: scroll }}
              aria-hidden
            />
            <button
              className="timeline-ruler-hit"
              aria-label="Seek timeline"
              style={{ width: contentWidth, height: RULER }}
              onPointerDown={(e) => {
                const rect = e.currentTarget.getBoundingClientRect();
                const seek = (event: PointerEvent) =>
                  onSeek?.(Math.max(0, (event.clientX - rect.left) / zoom));
                onSeek?.(Math.max(0, (e.clientX - rect.left) / zoom));
                e.currentTarget.setPointerCapture(e.pointerId);
                const button = e.currentTarget;
                const end = () => {
                  button.removeEventListener("pointermove", seek);
                  button.removeEventListener("pointerup", end);
                };
                button.addEventListener("pointermove", seek);
                button.addEventListener("pointerup", end);
              }}
            />
            {tracks.map((track, row) =>
              track.entries
                .filter(
                  ({ range }) =>
                    range!.end * zoom >= scroll - 100 &&
                    range!.start * zoom <= scroll + width + 100,
                )
                .map(({ item, range }) => (
                  <button
                    key={item.id}
                    className="timeline-clip-hit"
                    style={{
                      left: range!.start * zoom,
                      width: Math.max(3, (range!.end - range!.start) * zoom),
                      top: RULER + row * ROW + 7,
                      height: ROW - 14,
                    }}
                    data-ck-layer={item.id}
                    data-ck-selected={selected === item.id}
                    data-ck-agent={agentLayers.has(item.id)}
                    data-ck-flash={
                      lastRevision?.touched.includes(item.id) ?? false
                    }
                    aria-pressed={selected === item.id}
                    aria-label={item.name || item.id}
                    title={`${item.name || item.id} · ${range!.start.toFixed(2)}–${range!.end.toFixed(2)}s${agentLayers.has(item.id) ? " · Agent locked" : " · Drag to move; drag edges to trim"}`}
                    onClick={() => onSelect(item.id)}
                    onPointerDown={(e) =>
                      start(
                        e,
                        item.id,
                        ((e.target as HTMLElement).dataset.edge as
                          "start" | "end") ?? "move",
                      )
                    }
                  >
                    <span className="timeline-sr-label">
                      {item.name || item.id}
                    </span>
                    {agentLayers.has(item.id) && (
                      <span
                        data-ck-ghost
                        aria-label="Agent locked"
                        className="timeline-lock-badge"
                      >
                        ▣
                      </span>
                    )}
                    {(item.type === "clip" || item.type === "footage") && (
                      <>
                        <span
                          data-edge="start"
                          className="trim-edge trim-start"
                          title="Trim start"
                        />
                        <span
                          data-edge="end"
                          className="trim-edge trim-end"
                          title="Trim end"
                        />
                      </>
                    )}
                  </button>
                )),
            )}
            <div className="timeline-playhead" ref={playhead} aria-hidden />
            {project.comments
              .filter((c) => c.status !== "resolved" && c.anchor.tStart != null)
              .map((c) => (
                <button
                  className="timeline-comment-pin"
                  key={c.id}
                  data-ck-pin={c.id}
                  style={{
                    left: c.anchor.tStart! * zoom,
                    top: RULER + tracks.length * ROW + 10,
                  }}
                  title={c.text}
                  aria-label={`Comment: ${c.text}`}
                  onClick={() => onSeek?.(c.anchor.tStart!)}
                >
                  ◆
                </button>
              ))}
            {!tracks.length && (
              <div className="timeline-empty">
                Your story starts here. Import footage or add a title from the
                toolbar.
              </div>
            )}
          </div>
        </div>
      </div>
    </section>
  );
});
