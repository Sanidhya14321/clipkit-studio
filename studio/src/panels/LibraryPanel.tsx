import { memo, useMemo, useState } from "react";
import { request } from "../api";
import type { JsonValue, Project } from "../generated/types";
import { object, text, list } from "../editor/json";
const catalog = Object.values(
  import.meta.glob("../../../assets/recipes/*.json", {
    eager: true,
    import: "default",
  }),
).map((value) => object(value as JsonValue));
export const LibraryPanel = memo(function LibraryPanel({
  token,
  project,
  getPlayhead,
  onApplied,
  readOnly,
}: {
  token: string;
  project: Project;
  getPlayhead: () => number;
  onApplied: () => Promise<void>;
  readOnly: boolean;
}) {
  const [query, setQuery] = useState(""),
    [selected, setSelected] = useState("hook-title"),
    [values, setValues] = useState<Record<string, string>>({}),
    [busy, setBusy] = useState(false),
    [error, setError] = useState("");
  const recipes = useMemo(
    () =>
      catalog.filter((recipe) =>
        `${text(recipe.name)} ${text(recipe.description)} ${text(recipe.useWhen)}`
          .toLowerCase()
          .includes(query.toLowerCase()),
      ),
    [query],
  );
  const recipe =
    catalog.find((recipe) => text(recipe.id) === selected) ?? catalog[0]!;
  const slots = list(recipe.slots).map(object);
  const [duration, setDuration] = useState("");
  const confirmed =
    project.brief.status === "confirmed" || project.brief.status === "auto";
  const apply = async () => {
    setBusy(true);
    setError("");
    try {
      const filled: Record<string, JsonValue> = {};
      for (const slot of slots) {
        const name = text(slot.name),
          value = values[name]?.trim();
        if (!value) continue;
        filled[name] =
          slot.kind === "number"
            ? Number(value)
            : slot.kind === "list"
              ? value.split("\n").filter(Boolean)
              : value;
      }
      await request(token, "/api/recipe/apply", {
        method: "POST",
        body: JSON.stringify({
          recipe: text(recipe.id),
          at: getPlayhead(),
          slots: filled,
          ...(duration ? { duration: Number(duration) } : {}),
        }),
      });
      await onApplied();
    } catch (e) {
      setError(e instanceof Error ? e.message : "Could not add the scene.");
    } finally {
      setBusy(false);
    }
  };
  return (
    <section className="panel library-panel" aria-label="Scene library">
      <div className="panel-intro">
        <span className="eyebrow">BUILT-IN SCENES</span>
        <h2>A starting point for your story</h2>
        <p>
          Choose a recipe, fill its content and add it at the playhead. These
          are the same original recipes your agent uses.
        </p>
      </div>
      <details className="library-inventory">
        <summary>Project asset use · {project.assetsUsed.length}</summary>
        <p className="muted">
          10 original recipes use 8 registered native clips. The external source
          catalog is agent-searchable. GlitchText has a registered native
          adaptation; other external components require a render adapter.
        </p>
        <small>
          License mode: {project.brief.licenseMode ?? "commercial-safe"}
        </small>
        <ul>
          {project.assetsUsed.map((id) => (
            <li key={id}>
              <code>{id}</code>
            </li>
          ))}
        </ul>
        {!project.assetsUsed.length && (
          <p className="muted">
            No recipe or derived asset has been registered in this project yet.
          </p>
        )}
      </details>
      <input
        type="search"
        aria-label="Search recipes"
        placeholder="Search titles, stats, quotes…"
        value={query}
        onChange={(e) => setQuery(e.currentTarget.value)}
      />
      <div className="recipe-list">
        {recipes.map((entry) => (
          <button
            className="recipe-card"
            key={text(entry.id)}
            aria-pressed={selected === text(entry.id)}
            onClick={() => {
              setSelected(text(entry.id));
              setValues({});
              setDuration("");
            }}
          >
            <span className="recipe-icon" aria-hidden>
              ▧
            </span>
            <strong>{text(entry.name)}</strong>
            <small>{text(entry.useWhen)}</small>
          </button>
        ))}
      </div>
      {!recipes.length && (
        <p className="muted">No matching recipes. Try a shorter search.</p>
      )}
      <form
        className="recipe-form"
        onSubmit={(e) => {
          e.preventDefault();
          void apply();
        }}
      >
        <h3>{text(recipe.name)}</h3>
        <p className="muted">{text(recipe.description)}</p>
        {slots.map((slot) => (
          <label className="field" key={text(slot.name)}>
            <span>
              {text(slot.name).replaceAll("-", " ")}
              {slot.required ? " *" : ""}
            </span>
            {slot.kind === "source" ? (
              <select
                aria-label={text(slot.name)}
                value={values[text(slot.name)] ?? ""}
                required={Boolean(slot.required)}
                onChange={(e) => {
                  const value = e.currentTarget.value;
                  setValues((v) => ({ ...v, [text(slot.name)]: value }));
                }}
              >
                <option value="">Choose imported source</option>
                {project.sources.map((source) => (
                  <option key={source.id} value={source.id}>
                    {source.id}
                  </option>
                ))}
              </select>
            ) : slot.kind === "list" ? (
              <textarea
                aria-label={text(slot.name)}
                placeholder="One item per line"
                value={values[text(slot.name)] ?? ""}
                required={Boolean(slot.required)}
                onChange={(e) => {
                  const value = e.currentTarget.value;
                  setValues((v) => ({ ...v, [text(slot.name)]: value }));
                }}
              />
            ) : (
              <input
                aria-label={text(slot.name)}
                type={slot.kind === "number" ? "number" : "text"}
                value={values[text(slot.name)] ?? ""}
                required={Boolean(slot.required)}
                maxLength={
                  typeof slot.maxChars === "number" ? slot.maxChars : undefined
                }
                onChange={(e) => {
                  const value = e.currentTarget.value;
                  setValues((v) => ({ ...v, [text(slot.name)]: value }));
                }}
              />
            )}
            <small>{text(slot.description)}</small>
          </label>
        ))}
        <label className="field">
          <span>Duration (seconds)</span>
          <input
            type="number"
            aria-label="Recipe duration"
            min={Number(object(recipe.duration).min) || 0.1}
            max={Number(object(recipe.duration).max) || 120}
            step=".1"
            placeholder={String(object(recipe.duration).default ?? 3)}
            value={duration}
            onChange={(e) => setDuration(e.currentTarget.value)}
          />
        </label>
        {!confirmed && (
          <p className="editor-notice">
            Confirm the project brief before generating a scene.
          </p>
        )}
        {error && (
          <p className="error" role="alert">
            {error}
          </p>
        )}
        <button
          className="primary-action"
          disabled={readOnly || busy || !confirmed}
        >
          {busy ? "Adding scene…" : "Add scene at playhead"}
        </button>
      </form>
    </section>
  );
});
