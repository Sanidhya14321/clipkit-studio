// What the agent is doing (spec 5.2): status, stage and pass, and its plain-words label.
import { request } from "../api";
import type { Pass, Project } from "../generated/types";

const PASS_LABEL: Record<Pass, string> = {
  structure: "Structure pass",
  audio: "Audio pass",
  text: "Text pass",
  graphicsMotion: "Graphics and motion pass",
  transitions: "Transitions pass",
  look: "Look pass",
  polish: "Polish pass",
};

export function Banner({
  token,
  project,
  stopped,
}: {
  token: string;
  project: Project;
  stopped: boolean;
}) {
  const { agent, pipeline } = project;
  const status = stopped ? "stopped" : agent.status;
  const where =
    pipeline.stage === "passes" && pipeline.pass
      ? PASS_LABEL[pipeline.pass]
      : pipeline.stage;

  let text: string;
  if (stopped) text = "The agent stopped. You have full control.";
  else if (agent.status === "idle") text = "Agent idle. You have full control.";
  else if (agent.stopRequested) text = "Stopping after the current step…";
  else text = agent.label ? `${where}: ${agent.label}` : where;

  return (
    <div className="banner" role="status" data-ck-banner data-ck-agent={status}>
      <span className="banner-who">Agent</span>
      <span className="banner-status">{status}</span>
      <span className="banner-text">{text}</span>
      {!stopped && agent.status !== "idle" && !agent.stopRequested && (
        <button
          type="button"
          data-ck-stop-agent
          onClick={() =>
            void request(token, "/api/stop", { method: "POST" }).catch(
              () => undefined,
            )
          }
        >
          Stop
        </button>
      )}
    </div>
  );
}
