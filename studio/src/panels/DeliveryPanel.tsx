import { memo, useState } from "react";
import { request } from "../api";
import type { JsonValue, Project } from "../generated/types";
import { object, text, list } from "../editor/json";
import { unsupportedAssetLayers } from "../editor/assets";
export const DeliveryPanel = memo(function DeliveryPanel({
  token,
  project,
  onSelect,
  onSeek,
  connected,
}: {
  token: string;
  project: Project;
  onSelect: (id: string) => void;
  onSeek: (s: number) => void;
  connected: boolean;
}) {
  const [verification, setVerification] = useState<JsonValue>(null),
    [result, setResult] = useState<JsonValue>(null),
    [busy, setBusy] = useState(""),
    [error, setError] = useState(""),
    [preset, setPreset] = useState("source"),
    [format, setFormat] = useState("mp4"),
    [out, setOut] = useState("");
  const report = object(verification),
    exported = object(result),
    stale = typeof report.rev === "number" && report.rev !== project.rev;
  const unsupported = unsupportedAssetLayers(project);
  const verify = async () => {
    setBusy("verify");
    setError("");
    try {
      const value = await request<JsonValue>(token, "/api/verify");
      setVerification(value);
    } catch (e) {
      setError(e instanceof Error ? e.message : "Checks failed to run.");
    } finally {
      setBusy("");
    }
  };
  const deliver = async () => {
    if (format === "mp4" && unsupported.length) {
      setError(
        `These layers have no renderer: ${unsupported.join(", ")}. Replace them with registered native clips before exporting video.`,
      );
      return;
    }
    setBusy("export");
    setError("");
    setResult(null);
    try {
      const value = await request<JsonValue>(
        token,
        format === "mp4" ? "/api/export" : "/api/export/timeline",
        {
          method: "POST",
          body: JSON.stringify({
            format,
            ...(format === "mp4" ? { preset } : {}),
            ...(out.trim() ? { out: out.trim() } : {}),
          }),
        },
      );
      setResult(value);
    } catch (e) {
      setError(
        e instanceof Error
          ? e.message
          : "Export failed. Check media and render-worker setup.",
      );
    } finally {
      setBusy("");
    }
  };
  return (
    <section className="panel delivery-panel" aria-label="Review and export">
      <div className="panel-intro">
        <span className="eyebrow">FINISH & DELIVER</span>
        <h2>Know what you're sending</h2>
        <p>
          Review project checks, fix flagged layers and export through the same
          renderer used by your agent.
        </p>
      </div>
      <section className="delivery-step">
        <div className="section-heading">
          <h3>1. Check your sequence</h3>
          <button
            disabled={!connected || Boolean(busy)}
            onClick={() => void verify()}
          >
            {busy === "verify" ? "Checking…" : "Run checks"}
          </button>
        </div>
        {verification !== null && (
          <>
            <div
              className={`verification-summary ${report.ok ? "passed" : "failed"}`}
            >
              {stale
                ? "Project changed — run checks again"
                : report.ok
                  ? "Project checks passed"
                  : `${list(report.failed).length} checks need attention`}
            </div>
            <div className="gate-chips">
              {list(report.passed).map((gate) => (
                <span key={text(gate)}>✓ {text(gate)}</span>
              ))}
            </div>
            <ul className="verification-issues">
              {list(report.issues).map((raw, i) => {
                const issue = object(raw);
                return (
                  <li key={i}>
                    <strong>{text(issue.gate)}</strong>
                    <p>{text(issue.message)}</p>
                    {list(issue.ids).map((id) => (
                      <button
                        key={text(id)}
                        onClick={() => {
                          onSelect(text(id));
                          const item = project.tracks
                            .flatMap((t) => t.items)
                            .find((item) => item.id === id);
                          if (item && "at" in item) onSeek(item.at);
                        }}
                      >
                        {text(id)} ↗
                      </button>
                    ))}
                  </li>
                );
              })}
            </ul>
            {Number(report.omitted) > 0 && (
              <p className="muted">
                {String(report.omitted)} additional issues were omitted by the
                server's response cap.
              </p>
            )}
          </>
        )}
        {verification === null && (
          <p className="muted">
            Check timing, layout, contrast, motion and asset provenance before
            export.
          </p>
        )}
      </section>
      <section className="delivery-step">
        <h3>2. Export</h3>
        {unsupported.length > 0 && (
          <div className="editor-notice">
            Video export is blocked for unregistered component layers:{" "}
            {unsupported.join(", ")}. Ingested cards and derived source files
            need a vetted render adapter; they cannot render as labels.
          </div>
        )}
        <form
          onSubmit={(e) => {
            e.preventDefault();
            void deliver();
          }}
        >
          <label className="field">
            <span>Format</span>
            <select
              aria-label="Export format"
              value={format}
              onChange={(e) => setFormat(e.currentTarget.value)}
            >
              <option value="mp4">Video · MP4</option>
              <option value="otio">Editable timeline · OpenTimelineIO</option>
              <option value="srt">Subtitles · SRT</option>
            </select>
          </label>
          {format === "mp4" && (
            <label className="field">
              <span>Frame preset</span>
              <select
                aria-label="Export preset"
                value={preset}
                onChange={(e) => setPreset(e.currentTarget.value)}
              >
                <option value="source">
                  Project · {project.settings.width} × {project.settings.height}
                </option>
                <option value="youtube">YouTube · 1920 × 1080</option>
                <option value="reels">Reels · 1080 × 1920</option>
                <option value="shorts">Shorts · 1080 × 1920</option>
                <option value="tiktok">TikTok · 1080 × 1920</option>
              </select>
            </label>
          )}
          <label className="field">
            <span>Output path (optional)</span>
            <input
              aria-label="Export output path"
              placeholder="exports/my-video.mp4"
              value={out}
              onChange={(e) => setOut(e.currentTarget.value)}
            />
            <small>
              Relative to the project folder. Leave empty to use the server
              default.
            </small>
          </label>
          <button
            className="primary-action"
            disabled={
              !connected ||
              Boolean(busy) ||
              !project.tracks.some((t) => t.items.length) ||
              (format === "mp4" && unsupported.length > 0)
            }
          >
            {busy === "export"
              ? "Rendering & writing output…"
              : "Export sequence"}
          </button>
          {busy === "export" && (
            <div role="status" className="export-progress">
              <progress aria-label="Export in progress" />
              <span>
                The server is rendering. Keep this session open; completion or a
                specific error will appear here.
              </span>
            </div>
          )}
        </form>
      </section>
      {error && (
        <p className="error" role="alert">
          {error}
        </p>
      )}
      {result !== null && (
        <section className="export-result" role="status">
          <strong>Export finished</strong>
          <p>
            <code>
              {text(exported.path, "The server completed the export.")}
            </code>
          </p>
          {exported.durationSec != null && (
            <small>
              {String(exported.durationSec)} seconds · {String(exported.width)}{" "}
              × {String(exported.height)}
            </small>
          )}
          {exported.licenseReport && (
            <p>
              License report: <code>{text(exported.licenseReport)}</code>
            </p>
          )}
          <p className="muted">
            Output is saved in your project folder. This server does not expose
            exported files as browser downloads.
          </p>
          <button
            onClick={() => {
              void navigator.clipboard
                ?.writeText(text(exported.path))
                .catch(() =>
                  setError(
                    "Clipboard unavailable. Select and copy the output path above.",
                  ),
                );
            }}
          >
            Copy output path
          </button>
        </section>
      )}
    </section>
  );
});
