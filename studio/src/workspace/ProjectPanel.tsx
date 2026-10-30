import { memo, useState } from "react";
import { request } from "../api";
import type { Project } from "../generated/types";
import { duration, span } from "../timeline";

export const ProjectPanel = memo(function ProjectPanel({
  project,
  selected,
  onSelect,
  token,
  readOnly = false,
  onApplied,
  onSource,
  onStartBrief,
  showImport = false,
}: {
  project: Project;
  selected: string | null;
  onSelect: (id: string) => void;
  token?: string;
  readOnly?: boolean;
  onApplied?: () => Promise<void>;
  onSource?: (id: string) => void;
  onStartBrief?: () => void;
  showImport?: boolean;
}) {
  const [query, setQuery] = useState("");
  const [view, setView] = useState<"layers" | "media">("layers");
  const [importing, setImporting] = useState(false),
    [path, setPath] = useState(""),
    [error, setError] = useState("");
  const items = project.tracks.flatMap((track) =>
    track.items.map((item) => ({ item, track })),
  );
  const match = (name: string) =>
    name.toLowerCase().includes(query.trim().toLowerCase());
  return (
    <section className="panel project-bin" aria-label="Project">
      <div className="bin-heading">
        <span className="bin-project-icon" aria-hidden>
          ▤
        </span>
        <div>
          <strong>{project.name}</strong>
          <small>
            {items.length} layers · {duration(project).toFixed(1)} seconds
          </small>
        </div>
      </div>
      <input
        className="bin-search"
        type="search"
        aria-label="Search project"
        placeholder="Search this project"
        value={query}
        onChange={(e) => setQuery(e.target.value)}
      />
      {token && (
        <details
          className="import-form"
          open={showImport || project.sources.length === 0}
        >
          <summary>Import media</summary>
          <p className="muted">
            Paste the absolute path of a file on this computer. The server
            probes it, creates a proxy and adds video to the sequence.
          </p>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              setImporting(true);
              setError("");
              void request(token, "/api/import", {
                method: "POST",
                body: JSON.stringify({ path: path.trim() }),
              })
                .then(() => onApplied?.())
                .then(() => {
                  setPath("");
                  setView("media");
                })
                .catch((e) =>
                  setError(e instanceof Error ? e.message : "Import failed."),
                )
                .finally(() => setImporting(false));
            }}
          >
            <input
              aria-label="Local media path"
              placeholder="/Users/you/Movies/clip.mp4"
              value={path}
              onChange={(e) => setPath(e.currentTarget.value)}
              required
            />
            <button
              className="primary-action"
              disabled={readOnly || importing || !path.trim().startsWith("/")}
            >
              {importing ? "Probing & building proxy…" : "Import into project"}
            </button>
            {importing && <progress aria-label="Import in progress" />}
            {error && (
              <p className="error" role="alert">
                {error}
              </p>
            )}
          </form>
        </details>
      )}
      <div className="bin-views" aria-label="Project view">
        <button
          aria-pressed={view === "layers"}
          onClick={() => setView("layers")}
        >
          Sequence
        </button>
        <button
          aria-pressed={view === "media"}
          onClick={() => setView("media")}
        >
          Media <span>{project.sources.length}</span>
        </button>
      </div>
      {view === "layers" ? (
        <>
          <div className="bin-list-heading">
            <span>NAME</span>
            <span>DURATION</span>
          </div>
          <ul className="bin-list">
            {items
              .filter(({ item }) => match(item.name || item.id))
              .map(({ item, track }) => {
                const time = span(project, item);
                return (
                  <li key={item.id}>
                    <button
                      className="bin-item"
                      aria-pressed={selected === item.id}
                      onClick={() => onSelect(item.id)}
                    >
                      <span
                        className={`bin-type type-${track.kind}`}
                        aria-hidden
                      >
                        {track.kind === "audio" ? "♪" : "▧"}
                      </span>
                      <span>
                        <strong>{item.name || item.id}</strong>
                        <small>
                          {item.type} · {track.id}
                        </small>
                      </span>
                      <time>
                        {time ? `${(time.end - time.start).toFixed(1)}s` : "—"}
                      </time>
                    </button>
                  </li>
                );
              })}
          </ul>
          {items.length === 0 && (
            <div className="empty-state">
              <strong>Your sequence starts here</strong>
              <p>
                Import footage above, or start with a brief and choose a scene
                recipe.
              </p>
              <button onClick={onStartBrief}>Start from a brief</button>
            </div>
          )}
        </>
      ) : (
        <>
          <ul className="bin-list">
            {project.sources
              .filter((s) => match(s.id))
              .map((source) => (
                <li key={source.id} className="source-item">
                  <span className="bin-type" aria-hidden>
                    ▧
                  </span>
                  <div>
                    <strong>{source.id}</strong>
                    <small>
                      {source.kind} · {source.durationSec?.toFixed(1) ?? "—"} s
                    </small>
                    <small>
                      {source.proxyPath ? "Proxy available" : "No proxy"} ·{" "}
                      {source.transcriptPath
                        ? "Transcript available"
                        : "No transcript"}
                    </small>
                  </div>
                  <button
                    aria-label={`Preview ${source.id}`}
                    onClick={() => onSource?.(source.id)}
                  >
                    Preview ↗
                  </button>
                </li>
              ))}
          </ul>
          {project.sources.length === 0 && (
            <div className="empty-state">
              <strong>No media imported</strong>
              <p>
                Ask your agent to import footage with ck import. File import
                controls arrive with the media bin update.
              </p>
            </div>
          )}
        </>
      )}
      <div className="bin-footnote">PROJECT · {project.mode.toUpperCase()}</div>
    </section>
  );
});
