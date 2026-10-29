// The Brief page essentials (spec 8.1): every field with its source, a pacing slider that shows
// the numbers it maps to, look cards from the built-in looks, the agent's dynamic questions, and
// a confirm that goes to the server as one human patch on `/brief`. While the agent holds the
// project lock every control is disabled, and the server refuses the patch anyway.
import { useEffect, useState } from "react";
import { ApiError, request } from "../api";
import {
  ESSENTIALS,
  ESSENTIAL_LABEL,
  PACING,
  VOICE_OPTIONS,
  confirmPatch,
  display,
  pacingLevel,
  type Edits,
  type Essential,
} from "../brief";
import type { JsonValue, Look, PatchResult, Project } from "../generated/types";

export function BriefPanel({
  token,
  project,
  readOnly,
}: {
  token: string;
  project: Project;
  readOnly: boolean;
}) {
  const [looks, setLooks] = useState<Look[]>([]);
  const [edits, setEdits] = useState<Edits>({});
  const [answers, setAnswers] = useState<Record<string, string>>({});
  const [error, setError] = useState<string | null>(null);
  const brief = project.brief;

  useEffect(() => {
    request<Look[]>(token, "/api/looks").then(setLooks, () =>
      setError("The looks did not load from ck."),
    );
  }, [token]);

  const value = (name: string): JsonValue | undefined =>
    name in edits ? edits[name] : brief.fields[name]?.value;
  const set = (name: string, v: JsonValue) =>
    setEdits((e) => ({ ...e, [name]: v }));

  const confirm = async () => {
    try {
      await request<PatchResult>(token, "/api/patch", {
        method: "POST",
        body: JSON.stringify(confirmPatch(project, edits, answers)),
      });
      setEdits({});
      setError(null);
    } catch (e) {
      setError(
        e instanceof ApiError
          ? `${e.body.code}: ${e.body.message}`
          : "The confirm did not reach ck.",
      );
    }
  };

  const level = pacingLevel(value("pacing"));
  const pacing = PACING[level - 1];

  const control = (name: Essential) => {
    switch (name) {
      case "pacing":
        return (
          <span className="pacing">
            <input
              type="range"
              min={1}
              max={PACING.length}
              step={1}
              value={level}
              data-ck-prop="pacing"
              disabled={readOnly}
              onChange={(e) => set("pacing", Number(e.currentTarget.value))}
            />
            {pacing && (
              <span data-ck-pacing-numbers className="muted">
                {pacing.level} {pacing.name}: avg shot {pacing.avgShot} s (
                {pacing.minShot} to {pacing.maxShot} s), {pacing.cutsPer15s}{" "}
                cuts per 15 s
              </span>
            )}
          </span>
        );
      case "look":
        return (
          <span className="look-cards" role="radiogroup" aria-label="Look">
            {looks.map((l) => (
              <button
                key={l.id}
                type="button"
                role="radio"
                aria-checked={value("look") === l.id}
                data-ck-look={l.id}
                disabled={readOnly}
                className="look-card"
                style={{
                  background: l.colors.background,
                  color: l.colors.text,
                }}
                onClick={() => set("look", l.id)}
              >
                <span className="swatches" aria-hidden>
                  {[l.colors.surface, l.colors.accent, l.colors.mutedText].map(
                    (c) => (
                      <span key={c} style={{ background: c }} />
                    ),
                  )}
                </span>
                <strong>{l.name}</strong>
                <small>{l.motion.personality}</small>
              </button>
            ))}
          </span>
        );
      case "voice":
        return (
          <select
            data-ck-prop="voice"
            value={display(value("voice"))}
            disabled={readOnly}
            onChange={(e) => set("voice", e.currentTarget.value)}
          >
            <option value="" />
            {VOICE_OPTIONS.map((v) => (
              <option key={v} value={v}>
                {v}
              </option>
            ))}
          </select>
        );
      case "length":
        return (
          <input
            data-ck-prop="length"
            inputMode="decimal"
            value={display(value("length"))}
            disabled={readOnly}
            onChange={(e) => {
              const raw = e.currentTarget.value;
              const n = Number(raw);
              set("length", raw.trim() !== "" && Number.isFinite(n) ? n : raw);
            }}
          />
        );
      default:
        return (
          <input
            data-ck-prop={name}
            value={display(value(name))}
            disabled={readOnly}
            onChange={(e) => set(name, e.currentTarget.value)}
          />
        );
    }
  };

  return (
    <section
      className="panel brief"
      aria-label="Brief"
      data-ck-brief={brief.status}
      data-ck-readonly={readOnly}
    >
      <h2>Brief</h2>
      <p className="muted">
        {brief.status === "proposed"
          ? "The agent is waiting for you: adjust anything, then confirm."
          : `Brief ${brief.status}.`}
      </p>
      {readOnly && (
        <p className="readonly-note" role="note">
          Read-only: the agent is working on the project.
        </p>
      )}
      {ESSENTIALS.map((name) => (
        <div key={name} className="field" data-ck-field={name}>
          <span>
            {ESSENTIAL_LABEL[name]}{" "}
            <small
              className="source"
              data-ck-source={
                name in edits
                  ? "user"
                  : (brief.fields[name]?.source ?? "default")
              }
            >
              {name in edits
                ? "user"
                : (brief.fields[name]?.source ?? "default")}
            </small>
          </span>
          {control(name)}
        </div>
      ))}
      {(brief.questions ?? []).map((q) => (
        <label key={q.id} className="field" data-ck-question={q.id}>
          <span>
            {q.text}{" "}
            <small className="muted">affects {q.affects.join(", ")}</small>
          </span>
          <input
            defaultValue={display(q.answer ?? undefined)}
            disabled={readOnly}
            onChange={(e) => {
              const v = e.currentTarget.value;
              setAnswers((a) => ({ ...a, [q.id]: v }));
            }}
          />
        </label>
      ))}
      <button
        type="button"
        data-ck-brief-confirm
        disabled={readOnly}
        onClick={() => void confirm()}
      >
        {brief.status === "proposed" ? "Confirm brief" : "Save brief"}
      </button>
      {error && (
        <p className="error" role="alert" data-ck-error>
          {error}
        </p>
      )}
    </section>
  );
}
