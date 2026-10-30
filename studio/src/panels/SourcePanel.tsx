import { memo, useRef, useState } from "react";
import type { Project, PatchRequest } from "../generated/types";
import { addItemRequest, uniqueId } from "../editor/edits";
export const SourcePanel = memo(function SourcePanel({
  token,
  project,
  sourceId,
  getPlayhead,
  onCommit,
  readOnly,
}: {
  token: string;
  project: Project;
  sourceId: string | null;
  getPlayhead: () => number;
  onCommit: (r: PatchRequest | null) => Promise<boolean>;
  readOnly: boolean;
}) {
  const source = project.sources.find((s) => s.id === sourceId);
  const video = useRef<HTMLVideoElement>(null);
  const [start, setStart] = useState("0"),
    [end, setEnd] = useState(""),
    [error, setError] = useState("");
  if (!source)
    return (
      <section className="panel empty-state" aria-label="Source monitor">
        <strong>Preview before adding</strong>
        <p>
          Select a source in Project → Media. Mark a range and insert it at the
          sequence playhead.
        </p>
      </section>
    );
  const url =
    source.kind === "video" && source.proxyPath
      ? `/media/${source.hash}.mp4?token=${encodeURIComponent(token)}`
      : null;
  const insert = async () => {
    const a = Number(start),
      b = end === "" ? (source.durationSec ?? 0) : Number(end);
    if (
      !Number.isFinite(a) ||
      !Number.isFinite(b) ||
      a < 0 ||
      b <= a ||
      (source.durationSec != null && b > source.durationSec)
    ) {
      setError("Choose a source range within the media duration.");
      return;
    }
    const at = getPlayhead(),
      id = uniqueId(project, "insert");
    const common = {
      id,
      name: source.id,
      purpose: "Inserted source range",
      briefDeps: [],
      rev: 0,
    };
    const req =
      source.kind === "video"
        ? addItemRequest(
            project,
            "video",
            {
              ...common,
              type: "footage",
              src: source.id,
              srcIn: a,
              srcOut: b,
              at,
              speed: 1,
              volume: 1,
            },
            "Insert source range",
          )
        : source.kind === "audio"
          ? addItemRequest(
              project,
              "audio",
              {
                ...common,
                type: "audio",
                src: source.id,
                at,
                ranges: [{ start: a, end: b }],
                volume: 1,
                duck: false,
              },
              "Insert audio range",
            )
          : null;
    if (!req) {
      setError(
        "Image placement needs a supported image component. Ask your agent to place this image.",
      );
      return;
    }
    const ok = await onCommit(req);
    if (ok) setError("");
  };
  return (
    <section className="panel source-monitor" aria-label="Source monitor">
      <div className="panel-intro">
        <span className="eyebrow">SOURCE MONITOR</span>
        <h2>{source.id}</h2>
        <p>
          {source.kind} · {source.durationSec?.toFixed(2) ?? "—"} seconds
        </p>
      </div>
      {url ? (
        <video
          ref={video}
          src={url}
          controls
          preload="metadata"
          aria-label="Source preview"
        />
      ) : (
        <div className="empty-state">
          <strong>
            {source.kind === "audio" ? "Audio source" : "Preview unavailable"}
          </strong>
          <p>
            {source.kind === "video"
              ? "No browser proxy is ready. Importing video through ck creates a proxy."
              : "This server does not expose a browser preview for this source format."}
          </p>
        </div>
      )}
      <form
        onSubmit={(e) => {
          e.preventDefault();
          void insert();
        }}
      >
        <div className="range-inputs">
          <label className="field">
            <span>Source in (s)</span>
            <input
              aria-label="Source in"
              type="number"
              min="0"
              step=".01"
              value={start}
              onChange={(e) => setStart(e.currentTarget.value)}
            />
          </label>
          <label className="field">
            <span>Source out (s)</span>
            <input
              aria-label="Source out"
              type="number"
              min="0"
              max={source.durationSec ?? undefined}
              step=".01"
              placeholder={String(source.durationSec ?? 0)}
              value={end}
              onChange={(e) => setEnd(e.currentTarget.value)}
            />
          </label>
        </div>
        {url && (
          <div className="row">
            <button
              type="button"
              onClick={() =>
                setStart((video.current?.currentTime ?? 0).toFixed(3))
              }
            >
              Mark in
            </button>
            <button
              type="button"
              onClick={() =>
                setEnd((video.current?.currentTime ?? 0).toFixed(3))
              }
            >
              Mark out
            </button>
          </div>
        )}
        {error && (
          <p className="error" role="alert">
            {error}
          </p>
        )}
        <button
          className="primary-action"
          disabled={readOnly || source.kind === "image"}
          title={
            source.kind === "image"
              ? "Image placement requires a supported component"
              : undefined
          }
        >
          Insert range at playhead
        </button>
      </form>
      <details className="property-group">
        <summary>Source details</summary>
        <dl>
          <dt>File</dt>
          <dd>{source.path}</dd>
          <dt>Proxy</dt>
          <dd>{source.proxyPath ? "Ready" : "Unavailable"}</dd>
          <dt>Transcript</dt>
          <dd>{source.transcriptPath ? "Available" : "Not transcribed"}</dd>
        </dl>
      </details>
    </section>
  );
});
