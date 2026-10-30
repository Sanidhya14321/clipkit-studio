import { useEffect, useRef, useState } from "react";
export function TitleDialog({
  onAdd,
  onClose,
}: {
  onAdd: (text: string) => Promise<boolean>;
  onClose: () => void;
}) {
  const [text, setText] = useState(""),
    [busy, setBusy] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  const previous = useRef(document.activeElement);
  useEffect(() => {
    input.current?.focus();
    const el = previous.current;
    return () => {
      if (el instanceof HTMLElement) el.focus();
    };
  }, []);
  return (
    <div className="palette-backdrop" onClick={onClose}>
      <form
        className="title-dialog"
        role="dialog"
        aria-modal="true"
        aria-label="Add title"
        onClick={(e) => e.stopPropagation()}
        onSubmit={(e) => {
          e.preventDefault();
          setBusy(true);
          void onAdd(text.trim())
            .then((ok) => {
              if (ok) onClose();
            })
            .finally(() => setBusy(false));
        }}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            e.stopPropagation();
            onClose();
          }
          if (e.key === "Tab") {
            const fields = [
              ...e.currentTarget.querySelectorAll<HTMLElement>(
                "input,button:not(:disabled)",
              ),
            ];
            const next =
              (fields.indexOf(document.activeElement as HTMLElement) +
                (e.shiftKey ? -1 : 1) +
                fields.length) %
              fields.length;
            e.preventDefault();
            fields[next]?.focus();
          }
        }}
      >
        <span className="eyebrow">ADD AT PLAYHEAD</span>
        <h2>Say it with a title</h2>
        <label className="field">
          <span>Title text</span>
          <input
            ref={input}
            aria-label="Title text"
            value={text}
            maxLength={240}
            onChange={(e) => setText(e.currentTarget.value)}
            required
            placeholder="Your next great opening line"
          />
        </label>
        <p className="muted">
          A three-second title will be added on an overlay track. Change its
          content, color and timing in the inspector.
        </p>
        <div className="dialog-actions">
          <button type="button" onClick={onClose}>
            Cancel
          </button>
          <button className="primary-action" disabled={!text.trim() || busy}>
            {busy ? "Adding…" : "Add title"}
          </button>
        </div>
      </form>
    </div>
  );
}
