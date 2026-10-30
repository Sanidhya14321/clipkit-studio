import { useState } from "react";
import { request } from "../api";
import type { Project } from "../generated/types";
export function VoiceSetup({
  token,
  project,
  readOnly,
  onApplied,
}: {
  token: string;
  project: Project;
  readOnly: boolean;
  onApplied: () => Promise<void>;
}) {
  const [script, setScript] = useState(""),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  return (
    <details className="voice-setup" open={!project.voiceSlots.length}>
      <summary>
        {project.voiceSlots.length
          ? "Update narration script"
          : "Create your narration"}
      </summary>
      <p className="muted">
        One line per take. New slots use recorded narration; you can record each
        line and select takes below. Unchanged lines keep their recordings.
      </p>
      <form
        onSubmit={(e) => {
          e.preventDefault();
          setBusy(true);
          setError("");
          void request(token, "/api/voice/script", {
            method: "POST",
            body: JSON.stringify({
              mode: "recorded",
              lines: script
                .split("\n")
                .map((text) => text.trim())
                .filter(Boolean)
                .map((text) => ({ text })),
            }),
          })
            .then(onApplied)
            .then(() => setScript(""))
            .catch((e) =>
              setError(
                e instanceof Error ? e.message : "Could not create the script.",
              ),
            )
            .finally(() => setBusy(false));
        }}
      >
        <textarea
          aria-label="Narration script"
          placeholder="Meet your new way to edit.&#10;Build the story, one scene at a time."
          rows={5}
          value={script}
          onChange={(e) => setScript(e.currentTarget.value)}
        />
        <button
          className="primary-action"
          disabled={readOnly || busy || !script.trim()}
        >
          {busy ? "Creating slots…" : "Create recorded voice slots"}
        </button>
        {error && (
          <p className="error" role="alert">
            {error}
          </p>
        )}
      </form>
      <small>
        Recorded takes are included in the export; live sequence playback does
        not yet mix narration. AI speech and voice cloning require a working
        provider integration. Recording works locally with microphone
        permission.
      </small>
    </details>
  );
}
