import { Player } from "@remotion/player";
import { useMemo, useState } from "react";
import { ActivityFeed } from "./panels/ActivityFeed";
import { Banner } from "./panels/Banner";
import { Inspector } from "./panels/Inspector";
import { Timeline } from "./panels/Timeline";
import { ProjectComposition } from "./preview/ProjectComposition";
import { agentLocked, agentStopped, canEdit } from "./state";
import { durationInFrames } from "./timeline";
import { useNow, useStudio } from "./useStudio";

export function App({ token }: { token: string }) {
  const { connected, project, locks, lastRevision, feed } = useStudio(token);
  const now = useNow();
  const [selected, setSelected] = useState<string | null>(null);

  const stopped = agentStopped(locks, now);
  const agentLayers = useMemo(() => {
    const ids = agentLocked(locks, now);
    if (project && project.agent.status !== "idle" && !stopped) {
      for (const id of project.agent.activeLayers) ids.add(id);
    }
    return ids;
  }, [locks, now, project, stopped]);

  return (
    <div className="studio" data-ck-connected={connected}>
      <header className="studio-header">
        <strong>ClipKit Studio</strong>
        {project && (
          <span data-ck-project={project.id}>
            {project.name} · rev <span data-ck-rev>{project.rev}</span>
          </span>
        )}
        {!connected && <span role="status">Connecting to ck…</span>}
      </header>
      {project && <Banner project={project} stopped={stopped} />}
      {project && (
        <div className="studio-body">
          <main className="studio-preview">
            <Player
              component={ProjectComposition}
              inputProps={{
                project,
                agentLayers: [...agentLayers],
                flash: lastRevision?.touched ?? [],
                flashRev: lastRevision?.rev ?? 0,
              }}
              durationInFrames={durationInFrames(project)}
              fps={project.settings.fps}
              compositionWidth={project.settings.width}
              compositionHeight={project.settings.height}
              style={{ width: "100%", maxHeight: "100%" }}
              controls
            />
          </main>
          <aside className="studio-side">
            <Inspector
              key={selected ?? ""}
              token={token}
              project={project}
              layerId={selected}
              readOnly={selected !== null && !canEdit(locks, selected, now)}
            />
            <ActivityFeed feed={feed} />
          </aside>
          <Timeline
            project={project}
            agentLayers={agentLayers}
            lastRevision={lastRevision}
            selected={selected}
            onSelect={setSelected}
          />
        </div>
      )}
    </div>
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
