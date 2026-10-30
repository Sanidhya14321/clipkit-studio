import { useEffect, useRef, useState } from "react";
import { matchesCommand, type Command } from "./commands";

export function Palette({
  commands,
  onRun,
  onClose,
}: {
  commands: Command[];
  onRun: (command: Command) => void;
  onClose: () => void;
}) {
  const [query, setQuery] = useState("");
  const [index, setIndex] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const previousFocus = useRef(document.activeElement);
  const results = commands.filter((c) => matchesCommand(c, query));
  useEffect(() => {
    input.current?.focus();
    const previous = previousFocus.current;
    return () => {
      if (previous instanceof HTMLElement) previous.focus();
    };
  }, []);
  return (
    <div className="palette-backdrop" onClick={onClose}>
      <section
        className="command-palette"
        role="dialog"
        aria-modal="true"
        aria-label="Command palette"
        onClick={(e) => e.stopPropagation()}
        onKeyDown={(e) => {
          if (e.key === "Escape") {
            e.stopPropagation();
            onClose();
          }
          if (
            e.target === input.current &&
            (e.key === "ArrowDown" || e.key === "ArrowUp")
          ) {
            e.preventDefault();
            setIndex((i) =>
              Math.max(
                0,
                Math.min(
                  results.length - 1,
                  i + (e.key === "ArrowDown" ? 1 : -1),
                ),
              ),
            );
          }
          if (
            e.target === input.current &&
            e.key === "Enter" &&
            results[index] &&
            !results[index].reason
          ) {
            e.preventDefault();
            onRun(results[index]);
          }
          if (e.key === "Tab") {
            const focusable = [
              ...e.currentTarget.querySelectorAll<HTMLElement>(
                "input,button:not(:disabled)",
              ),
            ];
            const next =
              (focusable.indexOf(document.activeElement as HTMLElement) +
                (e.shiftKey ? -1 : 1) +
                focusable.length) %
              focusable.length;
            e.preventDefault();
            focusable[next]?.focus();
          }
        }}
      >
        <div className="palette-search">
          <span aria-hidden>⌕</span>
          <input
            ref={input}
            aria-label="Search commands"
            placeholder="Search commands, panels, workspaces…"
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setIndex(0);
            }}
          />
          <button onClick={onClose} aria-label="Close command palette">
            Esc
          </button>
        </div>
        <div className="palette-results">
          {results.map((c, i) => (
            <button
              key={c.id}
              disabled={Boolean(c.reason)}
              title={c.reason}
              data-active={i === index}
              onClick={() => onRun(c)}
            >
              <span>
                <small>{c.group}</small>
                {c.label}
                {c.reason && <em>{c.reason}</em>}
              </span>
              <kbd>{c.shortcut}</kbd>
            </button>
          ))}
          {!results.length && <p className="muted">No commands found.</p>}
        </div>
      </section>
    </div>
  );
}
