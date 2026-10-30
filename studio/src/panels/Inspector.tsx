import { memo, useRef, useState } from "react";
import { request } from "../api";
import type {
  JsonValue,
  Look,
  PatchRequest,
  Project,
} from "../generated/types";
import { locate } from "../state";
import { itemRequest } from "../editor/edits";
import { span } from "../timeline";

function ValueField({
  label,
  value,
  type = "text",
  disabled,
  save,
  name,
  min,
  max,
  step,
}: {
  label: string;
  value: string | number;
  type?: string;
  disabled: boolean;
  save: (value: string) => Promise<boolean>;
  name: string;
  min?: number;
  max?: number;
  step?: number;
}) {
  const [raw, setRaw] = useState(String(value));
  const [saving, setSaving] = useState(false);
  const cancelled = useRef(false);
  return (
    <label className="field inspector-value">
      <span>{label}</span>
      <input
        type={type}
        data-ck-prop={name}
        value={raw}
        min={min}
        max={max}
        step={step ?? "any"}
        disabled={disabled || saving}
        onChange={(e) => setRaw(e.currentTarget.value)}
        onBlur={() => {
          if (cancelled.current) {
            cancelled.current = false;
            return;
          }
          if (raw === String(value)) return;
          setSaving(true);
          void save(raw)
            .then((ok) => {
              if (!ok) setRaw(String(value));
            })
            .finally(() => setSaving(false));
        }}
        onKeyDown={(e) => {
          if (e.key === "Enter") e.currentTarget.blur();
          if (e.key === "Escape") {
            cancelled.current = true;
            setRaw(String(value));
            e.currentTarget.blur();
          }
        }}
      />
    </label>
  );
}
export const Inspector = memo(function Inspector({
  token,
  project,
  layerId,
  readOnly,
  onCommit,
  onDuplicate,
  onDelete,
  onSeek,
  looks = [],
}: {
  token: string;
  project: Project;
  layerId: string | null;
  readOnly: boolean;
  onCommit?: (patch: PatchRequest | null) => Promise<boolean>;
  onDuplicate?: () => void;
  onDelete?: () => void;
  onSeek?: (seconds: number) => void;
  looks?: Look[];
}) {
  const [error, setError] = useState("");
  const item = layerId ? locate(project, layerId)?.item : undefined;
  if (!item || !layerId)
    return (
      <section className="panel inspector" aria-label="Inspector">
        <div className="inspector-empty-icon" aria-hidden>
          ↖
        </div>
        <h2>Select something to make it yours</h2>
        <p className="muted">
          Click a layer in the timeline, project list or video. Edit its text,
          timing and supported appearance here.
        </p>
      </section>
    );
  const range = span(project, item);
  const props =
    item.type === "clip" &&
    item.props &&
    typeof item.props === "object" &&
    !Array.isArray(item.props)
      ? item.props
      : {};
  const commit = async (values: Record<string, JsonValue>, label: string) => {
    const patch = itemRequest(project, layerId, values, label);
    if (!patch) return false;
    try {
      const ok = onCommit
        ? await onCommit(patch)
        : await request(token, "/api/patch", {
            method: "POST",
            body: JSON.stringify(patch),
          }).then(() => true);
      setError("");
      return ok;
    } catch (e) {
      setError(
        e instanceof Error
          ? e.message
          : "Edit failed. Previous value restored.",
      );
      return false;
    }
  };
  const field = (
    name: string,
    label: string,
    value: string | number,
    options: {
      type?: string;
      min?: number;
      max?: number;
      step?: number;
      prop?: boolean;
    } = {},
  ) => (
    <ValueField
      key={`${item.id}@${item.rev}:${name}`}
      name={name}
      label={label}
      value={value}
      type={options.type}
      min={options.min}
      max={options.max}
      step={options.step}
      disabled={readOnly}
      save={async (raw) => {
        let parsed: JsonValue = raw;
        if (typeof value === "number") {
          const number = Number(raw);
          if (
            raw.trim() === "" ||
            !Number.isFinite(number) ||
            (options.min != null && number < options.min) ||
            (options.max != null && number > options.max)
          ) {
            setError(`${label} is outside its allowed range.`);
            return false;
          }
          parsed = number;
        }
        const nextProps = { ...props, [name]: parsed };
        if (
          options.prop &&
          name === "color" &&
          typeof parsed === "string" &&
          !parsed.startsWith("$") &&
          !CSS.supports("color", parsed)
        ) {
          setError(
            "Use a valid color, such as #ffffff, or a project color token.",
          );
          return false;
        }
        if (name === "text" || name === "quote" || name === "fontSize") {
          delete nextProps.lines;
          delete nextProps.textCut;
        }
        return commit(
          options.prop ? { props: nextProps } : { [name]: parsed },
          `Edit ${label.toLowerCase()}`,
        );
      }}
    />
  );
  const textKeys =
    item.type === "clip"
      ? [
          ...new Set([
            ...(item.clipId === "kinetic-text"
              ? ["text", "subtitle", "kicker"]
              : []),
            ...Object.keys(props).filter(
              (key) =>
                typeof props[key] === "string" &&
                !String(props[key]).startsWith("$") &&
                ![
                  "color",
                  "font",
                  "align",
                  "easing",
                  "enter",
                  "exit",
                  "accent",
                  "background",
                  "source",
                ].includes(key),
            ),
          ]),
        ]
      : [];
  return (
    <section
      className="panel inspector"
      aria-label="Inspector"
      data-ck-inspector={layerId}
      data-ck-readonly={readOnly}
    >
      <div className="inspector-selection">
        <span className="selection-type">{item.type}</span>
        <strong>{item.name || item.id}</strong>
        <small>
          {range
            ? `${(range.end - range.start).toFixed(2)} seconds`
            : "Follows source timing"}
        </small>
      </div>
      {readOnly && (
        <p className="editor-notice" role="note">
          Editing is paused while saving, disconnected or locked by the agent.
          Playback stays available.
        </p>
      )}
      {error && (
        <p className="error" role="alert" data-ck-error>
          {error}
        </p>
      )}
      <details open className="property-group">
        <summary>Layer</summary>
        {field("name", "Name", item.name)}
        {field("purpose", "Purpose", item.purpose)}
      </details>
      {textKeys.length > 0 && (
        <details open className="property-group">
          <summary>Text & content</summary>
          {textKeys.map((key) =>
            field(
              key,
              key.charAt(0).toUpperCase() + key.slice(1),
              String(props[key] ?? ""),
              { prop: true },
            ),
          )}
        </details>
      )}
      {item.type === "clip" &&
        (item.clipId === "kinetic-text" || "color" in props) && (
          <details open className="property-group">
            <summary>Appearance</summary>
            {field(
              "color",
              item.clipId === "backdrop" ? "Background color" : "Text color",
              String(props.color ?? "#ffffff"),
              {
                prop: true,
              },
            )}
            {item.clipId !== "backdrop" &&
              field(
                "fontSize",
                "Type size (frame fraction)",
                typeof props.fontSize === "number" ? props.fontSize : 0.07,
                { type: "number", min: 0.005, max: 1, prop: true },
              )}
            {item.clipId === "kinetic-text" && (
              <label className="field">
                <span>Alignment</span>
                <select
                  aria-label="Text alignment"
                  value={String(props.align ?? "center")}
                  disabled={readOnly}
                  onChange={(e) =>
                    void commit(
                      { props: { ...props, align: e.currentTarget.value } },
                      "Align text",
                    )
                  }
                >
                  <option value="left">Left</option>
                  <option value="center">Center</option>
                  <option value="right">Right</option>
                </select>
              </label>
            )}
            {looks.length > 0 && item.clipId !== "backdrop" && (
              <label className="field">
                <span>Project font</span>
                <select
                  aria-label="Project font"
                  value={String(props.font ?? "$fonts.display")}
                  disabled={readOnly}
                  onChange={(e) =>
                    void commit(
                      { props: { ...props, font: e.currentTarget.value } },
                      "Change project font",
                    )
                  }
                >
                  <option value="$fonts.display">Look display font</option>
                  <option value="$fonts.text">Look body font</option>
                  {typeof props.font === "string" &&
                    !String(props.font).startsWith("$") && (
                      <option value={props.font}>{props.font}</option>
                    )}
                </select>
              </label>
            )}
            <p className="muted">
              Appearance uses the native clip's supported properties. Fonts
              follow the project look.
            </p>
          </details>
        )}
      <details open className="property-group">
        <summary>Timing</summary>
        {"at" in item &&
          field("at", "Start (s)", item.at, {
            type: "number",
            min: 0,
            step: 1 / project.settings.fps,
          })}
        {item.type === "clip" &&
          field("dur", "Duration (s)", item.dur, {
            type: "number",
            min: 1 / project.settings.fps,
            step: 1 / project.settings.fps,
          })}
        {item.type === "footage" && (
          <>
            {field("srcIn", "Source in (s)", item.srcIn, {
              type: "number",
              min: 0,
              max: item.srcOut - 1 / project.settings.fps,
            })}
            {field("srcOut", "Source out (s)", item.srcOut, {
              type: "number",
              min: item.srcIn + 1 / project.settings.fps,
              max:
                project.sources.find((s) => s.id === item.src)?.durationSec ??
                undefined,
            })}
            <label className="field">
              <span>Playback speed</span>
              <input
                value={`${item.speed}×`}
                disabled
                title="Speed changes are not supported consistently by the current export engine."
              />
              <small>
                Speed editing requires preview/export engine parity.
              </small>
            </label>
          </>
        )}
        {range && (
          <button onClick={() => onSeek?.(range.start)}>
            Go to layer start
          </button>
        )}
      </details>
      {"volume" in item && (
        <details open className="property-group">
          <summary>Audio</summary>
          {field("volume", "Volume", item.volume, {
            type: "number",
            min: 0,
            max: 4,
            step: 0.05,
          })}
          {item.type === "audio" && (
            <p className="muted">
              Audio range gain is applied during export. This preview currently
              plays footage audio only.
            </p>
          )}
          {item.type === "audio" && (
            <label className="checkbox-field">
              <input
                type="checkbox"
                checked={item.duck}
                disabled
                title="Automatic music ducking is not implemented in the current mix engine."
              />
              Duck under voice
            </label>
          )}
        </details>
      )}
      <details className="property-group">
        <summary>Source & provenance</summary>
        <dl>
          <dt>Layer</dt>
          <dd>{item.id}</dd>
          <dt>Revision</dt>
          <dd>{item.rev}</dd>
          {item.provenance && (
            <>
              <dt>Asset</dt>
              <dd>{item.provenance.sourceAsset}</dd>
              <dt>License</dt>
              <dd>{item.provenance.license}</dd>
            </>
          )}
          {"src" in item && (
            <>
              <dt>Source</dt>
              <dd>{item.src}</dd>
            </>
          )}
        </dl>
      </details>
      <div className="inspector-actions">
        <button disabled={readOnly || !("at" in item)} onClick={onDuplicate}>
          Duplicate
        </button>
        <button disabled={readOnly} onClick={onDelete}>
          Delete layer
        </button>
      </div>
    </section>
  );
});
