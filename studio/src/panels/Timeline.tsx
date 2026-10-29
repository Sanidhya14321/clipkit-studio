// Tracks and their items laid out by time (spec 5.2). The agent's active layers carry the ghost
// cursor and a selection outline; layers the last patch touched flash once. Drag, trim and split
// are Phase 6 (spec 5.4).
import type { Project } from "../generated/types";
import type { Revision } from "../state";
import { duration, span } from "../timeline";

export function Timeline({
  project,
  agentLayers,
  lastRevision,
  selected,
  onSelect,
}: {
  project: Project;
  /** Layers the agent is working on: its active layers plus its live locks. */
  agentLayers: Set<string>;
  lastRevision: Revision | null;
  selected: string | null;
  onSelect: (layerId: string) => void;
}) {
  // Leave room after the last item; an empty project still gets a 10 s ruler.
  const total = Math.max(10, duration(project) * 1.1);
  const pct = (t: number) => `${(t / total) * 100}%`;
  const touched = new Set(lastRevision?.touched ?? []);

  return (
    <section className="panel timeline" aria-label="Timeline" data-ck-timeline>
      {project.tracks.map((track) => (
        <div
          key={track.id}
          className="track"
          data-ck-track={track.id}
          data-ck-kind={track.kind}
        >
          <span className="track-label">{track.id}</span>
          <div className="track-lane">
            {track.items.map((item) => {
              // Captions follow their source; until timing exists they span the whole ruler.
              const s = span(project, item) ?? { start: 0, end: total };
              const agent = agentLayers.has(item.id);
              const flash = touched.has(item.id);
              return (
                <button
                  type="button"
                  // A new key per revision restarts the flash animation.
                  key={flash ? `${item.id}@${lastRevision?.rev}` : item.id}
                  className={`item kind-${track.kind}${flash ? " flash" : ""}`}
                  style={{ left: pct(s.start), width: pct(s.end - s.start) }}
                  data-ck-layer={item.id}
                  data-ck-selected={selected === item.id}
                  data-ck-agent={agent}
                  data-ck-flash={flash}
                  aria-pressed={selected === item.id}
                  onClick={() => onSelect(item.id)}
                  title={item.purpose || item.name}
                >
                  {agent && (
                    <span className="ghost-cursor" data-ck-ghost aria-hidden>
                      ▲
                    </span>
                  )}
                  {item.name || item.id}
                </button>
              );
            })}
          </div>
        </div>
      ))}
      <div className="track" data-ck-pins>
        <span className="track-label">pins</span>
        <div className="track-lane">
          {project.comments
            .filter((c) => c.status !== "resolved" && c.anchor.tStart != null)
            .map((c) => (
              <span
                key={c.id}
                className="pin"
                data-ck-pin={c.id}
                title={c.text}
                style={{ left: pct(c.anchor.tStart ?? 0) }}
              />
            ))}
        </div>
      </div>
      {project.tracks.length === 0 && <p className="muted">No tracks yet.</p>}
    </section>
  );
}
