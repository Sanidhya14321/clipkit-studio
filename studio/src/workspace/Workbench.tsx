import {
  useEffect,
  useRef,
  useState,
  type CSSProperties,
  type ReactNode,
} from "react";
import type { Project } from "../generated/types";
import { ClockTime, type PlaybackClock } from "../editor/clock";
import { Boundary } from "./Boundary";
import { Palette } from "./Palette";
import { executeCommand, type Command } from "./commands";
import {
  PANELS,
  WORKSPACES,
  ZONES,
  defaultLayout,
  isTyping,
  movePanel,
  parseLayout,
  type Layout,
  type PanelId,
  type WorkspaceId,
  type Zone,
} from "./layout";

const DESCRIPTIONS: Partial<Record<WorkspaceId, string>> = {
  Audio: "Voice & takes · track mixer arrives in UI-5",
  Captions: "Transcript & voice captions",
};

export function Workbench({
  project,
  connected,
  agentStatus,
  panels,
  banner,
  commands: actions,
  selection,
  playhead,
  clock,
  openRequest,
  editError,
  onDismissError,
  pending = false,
}: {
  project: Project;
  connected: boolean;
  agentStatus: string;
  panels: Record<PanelId, ReactNode>;
  banner: ReactNode;
  commands: Command[];
  selection: string | null;
  playhead: number;
  clock?: PlaybackClock;
  openRequest?: { panel: PanelId; nonce: number } | null;
  editError?: string;
  onDismissError?: () => void;
  pending?: boolean;
}) {
  const [workspace, setWorkspace] = useState<WorkspaceId>("Edit");
  const [layout, setLayout] = useState<Layout>(() => defaultLayout("Edit"));
  const [loadedKey, setLoadedKey] = useState("");
  const [storageWarning, setStorageWarning] = useState(false);
  const [palette, setPalette] = useState(false);
  const [maximized, setMaximized] = useState<PanelId | null>(null);
  const [focused, setFocused] = useState<PanelId>("program");
  const [message, setMessage] = useState("");
  const [contrast, setContrast] = useState(false);
  const [comfortable, setComfortable] = useState(false);
  const [menu, setMenu] = useState<string | null>(null);
  const root = useRef<HTMLDivElement>(null);
  const memoryLayouts = useRef(new Map<string, Layout>());
  const [compact, setCompact] = useState(
    () => window.matchMedia("(max-width: 1150px)").matches,
  );
  const [narrow, setNarrow] = useState(
    () => window.matchMedia("(max-width: 800px)").matches,
  );
  useEffect(() => {
    const query = window.matchMedia("(max-width: 800px)");
    const compactQuery = window.matchMedia("(max-width: 1150px)");
    const update = () => {
      setNarrow(query.matches);
      setCompact(compactQuery.matches);
    };
    query.addEventListener("change", update);
    compactQuery.addEventListener("change", update);
    return () => {
      query.removeEventListener("change", update);
      compactQuery.removeEventListener("change", update);
    };
  }, []);
  const key = `ck-layout-v2:${project.id}:${workspace}`;
  useEffect(() => {
    if (menu)
      root.current
        ?.querySelector<HTMLButtonElement>(
          `[role="menu"][aria-label="${menu}"] button:not(:disabled)`,
        )
        ?.focus();
  }, [menu]);
  useEffect(() => {
    let raw: string | null = null;
    try {
      raw = localStorage.getItem(key);
    } catch {
      setStorageWarning(true);
    }
    setLayout(memoryLayouts.current.get(key) ?? parseLayout(raw, workspace));
    setLoadedKey(key);
    setMaximized(null);
  }, [key, workspace]);
  useEffect(() => {
    if (loadedKey !== key) return;
    memoryLayouts.current.set(key, layout);
    try {
      localStorage.setItem(key, JSON.stringify(layout));
    } catch {
      setStorageWarning(true);
    }
  }, [layout, loadedKey, key]);
  useEffect(() => {
    document.title = `${connected ? (agentStatus === "idle" ? project.name : agentStatus) : "Disconnected"} · ClipKit`;
  }, [connected, agentStatus, project.name]);
  useEffect(() => {
    if (!message) return;
    const timer = setTimeout(() => setMessage(""), 6000);
    return () => clearTimeout(timer);
  }, [message]);

  const open = (id: PanelId) => {
    const zone = ZONES.find((z) => layout.groups[z].includes(id));
    if (zone) setLayout((l) => ({ ...l, active: { ...l.active, [zone]: id } }));
    else
      setLayout((l) =>
        movePanel(
          l,
          id,
          id === "timeline" ? "bottom" : id === "program" ? "center" : "right",
        ),
      );
    setMaximized(null);
    setFocused(id);
  };
  useEffect(() => {
    if (openRequest) open(openRequest.panel);
  }, [openRequest]);
  const commands: Command[] = [
    ...actions,
    ...Object.entries(PANELS).map(([id, label]): Command => ({
      id: `panel-${id}`,
      label: `Open ${label}`,
      group: "Window",
      run: () => open(id as PanelId),
    })),
    ...WORKSPACES.map((id): Command => ({
      id: `workspace-${id}`,
      label: `${id} workspace`,
      group: "Window",
      run: () => setWorkspace(id),
    })),
    {
      id: "reset-layout",
      label: "Reset workspace",
      group: "Window",
      run: () => {
        setLayout(defaultLayout(workspace));
        setMaximized(null);
      },
    },
    {
      id: "maximize",
      label: "Toggle focused panel maximized",
      shortcut: "`",
      group: "Window",
      run: () => {
        const live = ZONES.flatMap((z) => layout.groups[z]);
        const candidate = live.includes(focused)
          ? focused
          : ZONES.map((z) => layout.active[z]).find(
              (id): id is PanelId => id !== null,
            );
        setMaximized((m) => (m ? null : (candidate ?? null)));
      },
    },
    {
      id: "contrast",
      label: contrast ? "Use dark theme" : "Use high contrast theme",
      group: "Window",
      run: () => setContrast((v) => !v),
    },
    {
      id: "density",
      label: comfortable ? "Use compact density" : "Use comfortable density",
      group: "Window",
      run: () => setComfortable((v) => !v),
    },
    {
      id: "shortcuts",
      label: "Keyboard shortcuts",
      group: "Help",
      run: () =>
        setMessage(
          "⌘/Ctrl K: commands · ⌘/Ctrl Z: undo · ⇧⌘/Ctrl Z: redo · Space: playback · `: maximize focused panel · arrow keys: resize focused splitter",
        ),
    },
    {
      id: "diagnostics",
      label: "Connection diagnostics",
      group: "Help",
      run: () =>
        setMessage(
          `ck ${connected ? "connected" : "disconnected"} · schema ${project.schemaVersion} · rev ${project.rev} · preview: Remotion · agent: ${agentStatus}`,
        ),
    },
  ];
  const run = (command: Command) => {
    setPalette(false);
    setMenu(null);
    void executeCommand(command, setMessage);
  };
  useEffect(() => {
    const outside = (event: PointerEvent) => {
      if (
        event.target instanceof Element &&
        !event.target.closest(".menu-root")
      )
        setMenu(null);
    };
    document.addEventListener("pointerdown", outside);
    return () => document.removeEventListener("pointerdown", outside);
  }, []);
  useEffect(() => {
    const handler = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        setMenu(null);
        setMaximized(null);
        return;
      }
      if (isTyping(e.target) || e.altKey || e.repeat) return;
      if (
        e.key === " " &&
        e.target instanceof Element &&
        e.target.closest(
          "button, a, summary, [role=button], [role=tab], [role=menuitem]",
        )
      )
        return;
      const mod = e.ctrlKey || e.metaKey;
      const id =
        mod && e.key.toLowerCase() === "z"
          ? e.shiftKey
            ? "redo"
            : "undo"
          : mod && e.key.toLowerCase() === "k"
            ? "palette"
            : mod && e.key.toLowerCase() === "b"
              ? "split"
              : !mod && e.key === "`"
                ? "maximize"
                : !mod && e.key === " "
                  ? "play"
                  : null;
      if (!id) return;
      e.preventDefault();
      if (id === "palette") setPalette((v) => !v);
      else {
        const command = commands.find((c) => c.id === id);
        if (command) run(command);
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  });

  const resize = (zone: "left" | "right" | "bottom", delta: number) =>
    setLayout((l) => ({
      ...l,
      [zone]: Math.max(
        zone === "bottom" ? 160 : zone === "left" ? 200 : 250,
        Math.min(zone === "bottom" ? 500 : 480, l[zone] + delta),
      ),
    }));
  const splitter = (zone: "left" | "right" | "bottom") => (
    <div
      className={`splitter split-${zone}`}
      hidden={zone === "bottom" ? narrow : compact}
      role="separator"
      tabIndex={0}
      aria-label={`Resize ${zone} panels`}
      aria-orientation={zone === "bottom" ? "horizontal" : "vertical"}
      aria-valuemin={zone === "bottom" ? 160 : zone === "left" ? 200 : 250}
      aria-valuemax={zone === "bottom" ? 500 : 480}
      aria-valuenow={layout[zone]}
      onKeyDown={(e) => {
        if (
          ["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown"].includes(e.key)
        ) {
          e.preventDefault();
          resize(
            zone,
            e.key === "ArrowRight" || e.key === "ArrowUp" ? 20 : -20,
          );
        }
      }}
      onPointerDown={(e) => {
        const el = e.currentTarget;
        el.setPointerCapture(e.pointerId);
        let last = zone === "bottom" ? e.clientY : e.clientX;
        const move = (event: PointerEvent) => {
          const pos = zone === "bottom" ? event.clientY : event.clientX;
          resize(zone, (pos - last) * (zone === "left" ? 1 : -1));
          last = pos;
        };
        const end = () => {
          el.removeEventListener("pointermove", move);
          el.removeEventListener("pointerup", end);
          el.removeEventListener("pointercancel", end);
        };
        el.addEventListener("pointermove", move);
        el.addEventListener("pointerup", end);
        el.addEventListener("pointercancel", end);
      }}
    />
  );

  const groups = narrow
    ? {
        ...layout.groups,
        left: [],
        right: [...layout.groups.right, ...layout.groups.left],
      }
    : layout.groups;
  const active = narrow
    ? {
        ...layout.active,
        left: null,
        right: groups.right.includes(focused)
          ? focused
          : (layout.active.right ?? groups.right[0] ?? null),
      }
    : layout.active;
  const group = (zone: Zone) => (
    <section
      className={`dock-group dock-${zone}`}
      data-ck-zone={zone}
      aria-label={`${zone} panel group`}
      hidden={Boolean(maximized && !groups[zone].includes(maximized))}
    >
      <div className="dock-bar">
        <div className="dock-tabs" role="tablist" aria-label={`${zone} panels`}>
          {groups[zone].map((id) => (
            <button
              id={`tab-${id}`}
              key={id}
              role="tab"
              aria-selected={active[zone] === id}
              aria-controls={`panel-${id}`}
              tabIndex={active[zone] === id ? 0 : -1}
              onClick={() => {
                const wasMaximized = maximized !== null;
                open(id);
                if (wasMaximized) setMaximized(id);
              }}
              onKeyDown={(e) => {
                if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
                e.preventDefault();
                const ids = groups[zone];
                const next =
                  ids[
                    (ids.indexOf(id) +
                      (e.key === "ArrowRight" ? 1 : -1) +
                      ids.length) %
                      ids.length
                  ]!;
                open(next);
                document.getElementById(`tab-${next}`)?.focus();
              }}
            >
              {PANELS[id]}
            </button>
          ))}
        </div>
        {active[zone] && (
          <details className="panel-options">
            <summary
              aria-label={`${PANELS[active[zone]!]} panel options`}
              title="Panel options"
            >
              ⋯
            </summary>
            <div>
              {ZONES.filter((z) => z !== zone).map((z) => (
                <button
                  key={z}
                  onClick={(e) => {
                    e.currentTarget.closest("details")?.removeAttribute("open");
                    setLayout((l) => movePanel(l, active[zone]!, z));
                  }}
                >
                  Dock {z}
                </button>
              ))}
              <button
                onClick={() => setMaximized((m) => (m ? null : active[zone]))}
              >
                {maximized ? "Restore panel" : "Maximize panel"}
              </button>
              <button
                onClick={() => {
                  setLayout((l) => movePanel(l, active[zone]!, null));
                  setMaximized(null);
                }}
              >
                Close panel
              </button>
            </div>
          </details>
        )}
      </div>
      <div className="dock-content">
        {!groups[zone].length && (
          <div className="dock-empty">
            Open a panel from Window or the command palette.
          </div>
        )}
      </div>
    </section>
  );
  const style = {
    "--dock-left": `${layout.left}px`,
    "--dock-right": `${layout.right}px`,
    "--dock-bottom": `${layout.bottom}px`,
  } as CSSProperties;
  return (
    <div
      ref={root}
      className="workbench"
      data-ck-connected={connected}
      data-theme={contrast ? "high-contrast" : "dark"}
      data-density={comfortable ? "comfortable" : "compact"}
      style={style}
    >
      <header className="workbench-header">
        <div className="brand">
          <span className="brand-mark" aria-hidden>
            c<span>k</span>
          </span>
          <strong>
            ClipKit<span> STUDIO</span>
          </strong>
        </div>
        <nav className="menubar" aria-label="Application menus">
          {(
            ["File", "Edit", "Sequence", "Agent", "Window", "Help"] as const
          ).map((name) => (
            <div className="menu-root" key={name}>
              <button
                aria-haspopup="menu"
                aria-expanded={menu === name}
                onClick={() => setMenu((m) => (m === name ? null : name))}
              >
                {name}
              </button>
              {menu === name && (
                <div
                  className="menu-popover"
                  role="menu"
                  aria-label={name}
                  onKeyDown={(e) => {
                    if (e.key === "Escape") {
                      setMenu(null);
                      e.currentTarget.parentElement
                        ?.querySelector("button")
                        ?.focus();
                    }
                    if (e.key === "ArrowDown" || e.key === "ArrowUp") {
                      e.preventDefault();
                      const items = [
                        ...e.currentTarget.querySelectorAll<HTMLButtonElement>(
                          "button:not(:disabled)",
                        ),
                      ];
                      const n = items.indexOf(
                        document.activeElement as HTMLButtonElement,
                      );
                      items[
                        (n + (e.key === "ArrowDown" ? 1 : -1) + items.length) %
                          items.length
                      ]?.focus();
                    }
                  }}
                >
                  {commands
                    .filter((c) => c.group === name)
                    .map((c) => (
                      <button
                        role="menuitem"
                        key={c.id}
                        disabled={Boolean(c.reason)}
                        title={c.reason}
                        onClick={() => run(c)}
                      >
                        <span>
                          {c.label}
                          {c.reason && <small>{c.reason}</small>}
                        </span>
                        <kbd>{c.shortcut}</kbd>
                      </button>
                    ))}
                </div>
              )}
            </div>
          ))}
        </nav>
        <div className="project-heading" data-ck-project={project.id}>
          <span className="saved-dot" />
          {project.name}
          <small>
            {connected
              ? `Revision ${project.rev} synced`
              : "Last received revision"}
          </small>
        </div>
        <button
          className="command-trigger"
          title="Command palette (⌘/Ctrl K)"
          onClick={() => setPalette(true)}
          aria-label="Open command palette"
        >
          ⌕ <kbd>⌘ K</kbd>
        </button>
      </header>
      <div className="workspace-bar">
        <span className="workspace-label">WORKSPACE</span>
        <nav aria-label="Workspaces">
          {WORKSPACES.map((id) => (
            <button
              key={id}
              aria-pressed={workspace === id}
              onClick={() => setWorkspace(id)}
            >
              {id}
            </button>
          ))}
        </nav>
        <div
          className="agent-pill"
          data-state={connected ? agentStatus : "disconnected"}
          role="status"
        >
          <span />
          {connected ? `Agent ${agentStatus}` : "Not connected"}
        </div>
        <button
          className="export-trigger"
          disabled={!connected}
          onClick={() => open("delivery")}
          title="Review project checks and export"
        >
          Export ↗
        </button>
      </div>
      <div className="workflow-actions" aria-label="Editing actions">
        <button
          onClick={() => {
            open("project");
            const c = commands.find((c) => c.id === "import");
            if (c) run(c);
          }}
        >
          ＋ Import media
        </button>
        <button
          onClick={() => {
            const c = commands.find((c) => c.id === "add-title");
            if (c) run(c);
          }}
          disabled={!connected || pending}
        >
          T Add title
        </button>
        <button onClick={() => open("library")}>▧ Add scene</button>
        <span className="toolbar-spacer" />
        {(
          [
            ["brief", "Brief"],
            ["voice", "Narration"],
            ["transcript", "Transcript"],
            ["comments", "Comments"],
            ["history", "History"],
          ] as [PanelId, string][]
        ).map(([id, label]) => (
          <button key={id} onClick={() => open(id)}>
            {label}
          </button>
        ))}
        <span className="save-state" role="status">
          {pending ? "Saving edit…" : connected ? "Project synced" : "Offline"}
        </span>
      </div>
      {editError && (
        <div className="edit-error-bar" role="alert">
          {editError}
          <button onClick={onDismissError} aria-label="Dismiss edit error">
            ×
          </button>
        </div>
      )}
      <div
        className="workbench-agent"
        data-working={connected && agentStatus === "working"}
      >
        {banner}
      </div>
      {!connected && (
        <div className="connection-notice" role="status">
          Connection lost. Reconnecting to ck… Last received project stays
          visible. Editing commands are disabled.
        </div>
      )}
      {storageWarning && (
        <div className="connection-notice" role="status">
          Layout storage is unavailable. This session keeps your layout in
          memory.
        </div>
      )}
      <div className="dock-layout" data-maximized={Boolean(maximized)}>
        {group("left")}
        {splitter("left")}
        {group("center")}
        {splitter("right")}
        {group("right")}
        {splitter("bottom")}
        {group("bottom")}
        {(Object.keys(PANELS) as PanelId[]).map((id) => {
          const zone = ZONES.find((z) => groups[z].includes(id));
          const shown =
            zone && (maximized ? maximized === id : active[zone] === id);
          return (
            <div
              className={`dock-panel positioned-panel panel-zone-${zone ?? "right"} ${id === "program" ? "program-panel" : ""}`}
              id={`panel-${id}`}
              key={id}
              role="tabpanel"
              aria-labelledby={zone ? `tab-${id}` : undefined}
              aria-label={zone ? undefined : PANELS[id]}
              tabIndex={0}
              hidden={!shown}
              onFocus={() => setFocused(id)}
              onPointerDown={() => setFocused(id)}
            >
              <Boundary name={PANELS[id]}>{panels[id]}</Boundary>
            </div>
          );
        })}
      </div>
      <footer className="statusbar">
        <span className={`connection-dot ${connected ? "online" : ""}`} />
        {connected ? "Connected to ck" : "Reconnecting"}
        <span className="status-divider" />
        <span>Remotion preview</span>
        <span className="status-secondary">
          {project.settings.width} × {project.settings.height} ·{" "}
          {project.settings.fps} fps
        </span>
        <span className="status-grow">
          {DESCRIPTIONS[workspace] ??
            (selection
              ? `Selected: ${selection}`
              : "Select a layer to inspect · ⌘/Ctrl K for commands")}
        </span>
        {clock ? (
          <ClockTime clock={clock} instrument />
        ) : (
          <span className="timecode" data-ck-playhead>
            {playhead.toFixed(3)} s
          </span>
        )}
        <span>
          rev <b data-ck-rev>{project.rev}</b>
        </span>
      </footer>
      {message && (
        <div className="toast" role="status">
          {message}
          <button
            aria-label="Dismiss notification"
            onClick={() => setMessage("")}
          >
            ×
          </button>
        </div>
      )}
      {palette && (
        <Palette
          commands={commands}
          onRun={run}
          onClose={() => setPalette(false)}
        />
      )}
    </div>
  );
}
