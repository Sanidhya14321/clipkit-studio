// The selected layer's fields. Each change goes to the server as one JSON patch built against
// the revision the studio last saw; the next snapshot shows the result. While the agent holds
// the layer every control is disabled, and the server refuses the edit anyway (E_LOCK_HELD).
// The schema-driven inspector for clip props is Phase 6 (spec 5.4).
import { useState } from "react";
import { ApiError, request } from "../api";
import type { Item, PatchResult, Project } from "../generated/types";
import {
  editableFields,
  fieldPatch,
  locate,
  type EditableField,
} from "../state";

const FIELD_LABEL: Record<EditableField, string> = {
  name: "Name",
  purpose: "Purpose",
  at: "Start (s)",
  dur: "Duration (s)",
};

export function Inspector({
  token,
  project,
  layerId,
  readOnly,
}: {
  token: string;
  project: Project;
  layerId: string | null;
  readOnly: boolean;
}) {
  const [error, setError] = useState<string | null>(null);
  const item = layerId ? locate(project, layerId)?.item : undefined;

  if (!layerId || !item) {
    return (
      <section className="panel inspector" aria-label="Inspector">
        <h2>Inspector</h2>
        <p className="muted">Select a layer on the timeline.</p>
      </section>
    );
  }

  const commit = async (field: EditableField, raw: string) => {
    const value = parse(field, raw);
    if (value === null) {
      setError(`${FIELD_LABEL[field]} must be a number of seconds, 0 or more.`);
      return;
    }
    if (value === current(item, field)) return;
    const req = fieldPatch(project, layerId, field, value);
    if (!req) return;
    try {
      await request<PatchResult>(token, "/api/patch", {
        method: "POST",
        body: JSON.stringify(req),
      });
      setError(null);
    } catch (e) {
      setError(
        e instanceof ApiError
          ? `${e.body.code}: ${e.body.message}`
          : "The edit did not reach ck.",
      );
    }
  };

  return (
    <section
      className="panel inspector"
      aria-label="Inspector"
      data-ck-inspector={layerId}
      data-ck-readonly={readOnly}
    >
      <h2>Inspector</h2>
      <p className="muted">
        {item.type} · {item.id} · rev {item.rev}
      </p>
      {readOnly && (
        <p className="readonly-note" role="note">
          Read-only: the agent is working on this layer.
        </p>
      )}
      {editableFields(item).map((field) => (
        <label key={field} className="field">
          <span>{FIELD_LABEL[field]}</span>
          <input
            // Remount on every layer revision so the input shows the server's value.
            key={`${layerId}@${item.rev}`}
            data-ck-prop={field}
            defaultValue={String(current(item, field))}
            inputMode={field === "at" || field === "dur" ? "decimal" : "text"}
            disabled={readOnly}
            onBlur={(e) => void commit(field, e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") e.currentTarget.blur();
            }}
          />
        </label>
      ))}
      {error && (
        <p className="error" role="alert" data-ck-error>
          {error}
        </p>
      )}
    </section>
  );
}

function current(item: Item, field: EditableField): string | number {
  switch (field) {
    case "name":
      return item.name;
    case "purpose":
      return item.purpose;
    case "at":
      return "at" in item ? item.at : 0;
    case "dur":
      return item.type === "clip" ? item.dur : 0;
  }
}

function parse(field: EditableField, raw: string): string | number | null {
  if (field === "name" || field === "purpose") return raw;
  const n = Number(raw.trim());
  return raw.trim() !== "" && Number.isFinite(n) && n >= 0 ? n : null;
}
