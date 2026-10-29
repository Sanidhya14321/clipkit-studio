// Tuning chips under a confirmed brief (spec 5.4): each shows how many layers it re-plans
// before you click; the click is one human patch the agent's `ck wait` picks up.
import { useState } from "react";
import { ApiError, request } from "../api";
import type { Project } from "../generated/types";
import { CHIPS, CHIP_LABEL, chipEdits, chipPatch, impacted } from "../tune";

export function TuningChips({
  token,
  project,
  readOnly,
}: {
  token: string;
  project: Project;
  readOnly: boolean;
}) {
  const [error, setError] = useState<string | null>(null);

  return (
    <div className="chips" data-ck-chips>
      {CHIPS.map((chip) => {
        const edits = chipEdits(project, chip);
        const layers = edits ? impacted(project, Object.keys(edits)) : [];
        return (
          <button
            key={chip}
            type="button"
            className="chip"
            data-ck-chip={chip}
            data-ck-impact={layers.join(" ")}
            disabled={readOnly || edits === null}
            title={
              edits
                ? `Re-plans ${layers.length} layer(s): ${layers.join(", ") || "none"}`
                : "Already at the end of its range"
            }
            onClick={async () => {
              if (!edits) return;
              try {
                await request(token, "/api/patch", {
                  method: "POST",
                  body: JSON.stringify(chipPatch(project, chip, edits)),
                });
                setError(null);
              } catch (e) {
                setError(e instanceof ApiError ? e.message : String(e));
              }
            }}
          >
            {CHIP_LABEL[chip]} <small className="muted">{layers.length}</small>
          </button>
        );
      })}
      {error && (
        <p className="error" role="alert">
          {error}
        </p>
      )}
    </div>
  );
}
