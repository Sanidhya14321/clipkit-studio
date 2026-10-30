// Storyboard checkpoint (spec 4 PLAN): when the brief asks for one, the agent renders a still
// per scene and waits; the user looks at the sheet here and confirms. Only the user confirms.
import { useEffect, useState } from "react";
import { ApiError, request } from "../api";
import type { Project } from "../generated/types";

export const CHECKPOINT = "storyboard";
export const CONFIRMED = "storyboard:confirmed";

export function StoryboardPanel({
  token,
  project,
}: {
  token: string;
  project: Project;
}) {
  const [missing, setMissing] = useState(false);
  const [attempt, setAttempt] = useState(0);
  const [error, setError] = useState<string | null>(null);
  // The sheet is rendered after the checkpoint is asked for (no new revision), so retry.
  useEffect(() => {
    if (!missing) return;
    const t = setTimeout(() => {
      setMissing(false);
      setAttempt((a) => a + 1);
    }, 2000);
    return () => clearTimeout(t);
  }, [missing]);
  const checkpoints = project.brief.checkpoints ?? [];
  if (!checkpoints.includes(CHECKPOINT) || checkpoints.includes(CONFIRMED)) {
    return null;
  }
  const src = `/media/storyboard/latest.png?token=${encodeURIComponent(token)}&v=${project.rev}-${attempt}`;
  return (
    <section
      className="panel storyboard"
      aria-label="Storyboard"
      data-ck-storyboard
    >
      <h2>Storyboard</h2>
      {missing && <p className="muted">Waiting for the agent's storyboard…</p>}
      {!missing && (
        <img
          src={src}
          alt="One still per scene"
          data-ck-storyboard-sheet
          onError={() => setMissing(true)}
          onLoad={() => setMissing(false)}
          style={{ width: "100%" }}
        />
      )}
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
      <button
        type="button"
        data-ck-storyboard-confirm
        disabled={missing}
        onClick={async () => {
          try {
            await request(token, "/api/storyboard/confirm", { method: "POST" });
            setError(null);
          } catch (e) {
            setError(e instanceof ApiError ? e.message : String(e));
          }
        }}
      >
        Looks right: build it
      </button>
    </section>
  );
}
