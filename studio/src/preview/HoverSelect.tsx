// Hover-select on the preview (spec 8.4): hovering any element that carries `data-ck-part` or
// `data-ck-layer` outlines it with a label; a click selects its layer and opens a mini inspector
// whose Ask agent sends a comment anchored to that exact element.
import { type ReactNode, useRef, useState } from "react";
import { ApiError, request } from "../api";
import type { CommentAddRequest, Project } from "../generated/types";

export interface Target {
  layerId: string;
  path: string | null;
  box: { left: number; top: number; width: number; height: number };
}

/** The innermost instrumented element under `el`, as layer id and part path. */
export function targetOf(el: Element | null, root: Element): Target | null {
  const part = el?.closest("[data-ck-part]");
  const layer = el?.closest("[data-ck-layer]");
  if (!layer || !root.contains(layer)) return null;
  const shown = part && layer.contains(part) ? part : layer;
  const r = shown.getBoundingClientRect();
  const base = root.getBoundingClientRect();
  return {
    layerId: layer.getAttribute("data-ck-layer") ?? "",
    path:
      part && layer.contains(part) ? part.getAttribute("data-ck-part") : null,
    box: {
      left: r.left - base.left,
      top: r.top - base.top,
      width: r.width,
      height: r.height,
    },
  };
}

export function HoverSelect({
  token,
  project,
  children,
  onHover,
  onPick,
}: {
  token: string;
  project: Project;
  children: ReactNode;
  onHover: (t: Target | null) => void;
  onPick: (t: Target) => void;
}) {
  const root = useRef<HTMLDivElement>(null);
  const [hover, setHover] = useState<Target | null>(null);
  const [picked, setPicked] = useState<Target | null>(null);
  const [ask, setAsk] = useState("");
  const [error, setError] = useState<string | null>(null);

  const at = (e: React.MouseEvent): Target | null => {
    if (!root.current) return null;
    // The outline sits on top; look beneath it.
    const el = document
      .elementsFromPoint(e.clientX, e.clientY)
      .find((n) => !n.closest("[data-ck-hover-ui]"));
    return targetOf(el ?? null, root.current);
  };

  const name = (id: string) =>
    project.tracks.flatMap((t) => t.items).find((i) => i.id === id)?.name || id;

  const send = async () => {
    if (!picked || !ask.trim()) return;
    const req: CommentAddRequest = {
      text: ask.trim(),
      anchor: {
        layerId: picked.layerId,
        ...(picked.path ? { propPath: picked.path } : {}),
      },
      author: "human",
    };
    try {
      await request(token, "/api/comments/add", {
        method: "POST",
        body: JSON.stringify(req),
      });
      setAsk("");
      setPicked(null);
      setError(null);
    } catch (e) {
      setError(e instanceof ApiError ? e.message : String(e));
    }
  };

  return (
    <div
      ref={root}
      className="hover-select"
      data-ck-hover-root
      onMouseMove={(e) => {
        const t = at(e);
        setHover(t);
        onHover(t);
      }}
      onMouseLeave={() => {
        setHover(null);
        onHover(null);
      }}
      onClickCapture={(e) => {
        if (
          e.target instanceof Element &&
          e.target.closest("[data-ck-hover-ui]")
        )
          return;
        const t = at(e);
        if (!t) return;
        // Picking an element must not also toggle playback.
        e.stopPropagation();
        setPicked(t);
        onPick(t);
      }}
    >
      {children}
      {hover && (
        <div
          data-ck-hover-ui
          data-ck-hover={hover.layerId}
          data-ck-hover-path={hover.path ?? ""}
          className="hover-outline"
          style={hover.box}
        >
          <span className="hover-label">
            {name(hover.layerId)}
            {hover.path ? ` · ${hover.path}` : ""}
          </span>
        </div>
      )}
      {picked && (
        <div
          data-ck-hover-ui
          data-ck-mini-inspector={picked.layerId}
          className="mini-inspector"
          style={{
            left: picked.box.left,
            top: picked.box.top + picked.box.height + 4,
          }}
        >
          <strong>{name(picked.layerId)}</strong>
          {picked.path && <span className="muted"> · {picked.path}</span>}
          {error && (
            <p role="alert" className="error">
              {error}
            </p>
          )}
          <input
            aria-label="Ask agent"
            data-ck-ask-text
            value={ask}
            onChange={(e) => setAsk(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") void send();
              if (e.key === "Escape") setPicked(null);
            }}
          />
          <button type="button" data-ck-ask onClick={() => void send()}>
            Ask agent
          </button>
          <button type="button" onClick={() => setPicked(null)}>
            Close
          </button>
        </div>
      )}
    </div>
  );
}
