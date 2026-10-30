import {
  memo,
  useMemo,
  useState,
  type RefCallback,
  type CSSProperties,
} from "react";
import { Player, type PlayerRef } from "@remotion/player";
import type { Project, Look } from "../generated/types";
import type { Revision } from "../state";
import { HoverSelect, type Target } from "../preview/HoverSelect";
import { ProjectComposition } from "../preview/ProjectComposition";
import { ClockTime, type PlaybackClock } from "../editor/clock";
import { durationInFrames } from "../timeline";
export const ProgramMonitor = memo(function ProgramMonitor({
  token,
  project,
  looks,
  agentLayers,
  lastRevision,
  media,
  attachPlayer,
  getPlayer,
  clock,
  onHover,
  onPick,
  onSeek,
}: {
  token: string;
  project: Project;
  looks: Look[];
  agentLayers: Set<string>;
  lastRevision: Revision | null;
  media: Record<string, string>;
  attachPlayer: RefCallback<PlayerRef>;
  getPlayer: () => PlayerRef | null;
  clock: PlaybackClock;
  onHover: (t: Target | null) => void;
  onPick: (t: Target) => void;
  onSeek: (s: number) => void;
}) {
  const [safe, setSafe] = useState(false),
    [grid, setGrid] = useState(false);
  const inputProps = useMemo(
    () => ({
      project,
      looks,
      agentLayers: [...agentLayers],
      flash: lastRevision?.touched ?? [],
      flashRev: lastRevision?.rev ?? 0,
      media,
    }),
    [project, looks, agentLayers, lastRevision, media],
  );
  return (
    <main className="studio-preview">
      <div className="monitor-meta">
        <span>{project.name}</span>
        <div>
          <button
            aria-pressed={safe}
            onClick={() => setSafe((v) => !v)}
            title="Title and action safe guides"
          >
            Safe areas
          </button>
          <button
            aria-pressed={grid}
            onClick={() => setGrid((v) => !v)}
            title="Rule of thirds guide"
          >
            Grid
          </button>
        </div>
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
            onHover={onHover}
            onPick={onPick}
          >
            <Player
              ref={attachPlayer}
              component={ProjectComposition}
              inputProps={inputProps}
              durationInFrames={durationInFrames(project)}
              fps={project.settings.fps}
              compositionWidth={project.settings.width}
              compositionHeight={project.settings.height}
              style={{ width: "100%" }}
              controls={false}
            />
            {safe && (
              <div className="monitor-safe" aria-hidden>
                <div />
                <div />
              </div>
            )}
            {grid && <div className="monitor-grid" aria-hidden />}
          </HoverSelect>
        </div>
      </div>
      <div className="monitor-transport">
        <button
          aria-label="Go to sequence start"
          title="Go to start"
          onClick={() => onSeek(0)}
        >
          │◀
        </button>
        <button
          aria-label="Previous frame"
          title="Previous frame"
          onClick={() =>
            onSeek(Math.max(0, clock.get() - 1 / project.settings.fps))
          }
        >
          ‹
        </button>
        <button
          aria-label="Toggle playback"
          title="Play / pause (Space)"
          onClick={() => {
            const player = getPlayer();
            if (player?.isPlaying()) player.pause();
            else player?.play();
          }}
        >
          ▶ / Ⅱ
        </button>
        <button
          aria-label="Next frame"
          title="Next frame"
          onClick={() => onSeek(clock.get() + 1 / project.settings.fps)}
        >
          ›
        </button>
        <ClockTime clock={clock} />
        <input
          type="number"
          aria-label="Seek to seconds"
          placeholder="Go to seconds"
          min="0"
          step=".001"
          onKeyDown={(e) => {
            if (e.key === "Enter") {
              onSeek(Number(e.currentTarget.value));
              e.currentTarget.blur();
            }
          }}
        />
        <span className="toolbar-spacer" />
        <small>
          {project.settings.width} × {project.settings.height} ·{" "}
          {project.settings.fps} fps
        </small>
      </div>
      {project.tracks.some((t) =>
        t.items.some(
          (i) =>
            i.type === "voice" || i.type === "audio" || i.type === "caption",
        ),
      ) && (
        <div className="preview-capability-note">
          Video preview plays footage audio. Narration and music are mixed on
          export. Captions export as SRT; burned-in captions are not available
          yet.
        </div>
      )}
    </main>
  );
});
