import { Player } from "@remotion/player";
import { ProjectComposition } from "./preview/ProjectComposition";
import { durationInFrames } from "./timeline";
import { useStudio } from "./useStudio";

export function App({ token }: { token: string }) {
  const { connected, project } = useStudio(token);

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
      <main className="studio-preview">
        {project && (
          <Player
            component={ProjectComposition}
            inputProps={{ project }}
            durationInFrames={durationInFrames(project)}
            fps={project.settings.fps}
            compositionWidth={project.settings.width}
            compositionHeight={project.settings.height}
            style={{ width: "100%", maxHeight: "100%" }}
            controls
          />
        )}
      </main>
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
