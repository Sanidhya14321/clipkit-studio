// UI preferences only. Project data and edits continue to belong to ck.
export const PANELS = {
  project: "Project",
  program: "Program monitor",
  timeline: "Timeline",
  inspector: "Inspector",
  brief: "Brief & styles",
  agent: "Agent",
  comments: "Comments",
  history: "History",
  transcript: "Transcript",
  voice: "Voice",
  storyboard: "Storyboard",
  source: "Source monitor",
  library: "Scene library",
  delivery: "Review & export",
} as const;
export type PanelId = keyof typeof PANELS;
export const WORKSPACES = [
  "Edit",
  "Generate",
  "Audio",
  "Captions",
  "Review",
] as const;
export type WorkspaceId = (typeof WORKSPACES)[number];
export const ZONES = ["left", "center", "right", "bottom"] as const;
export type Zone = (typeof ZONES)[number];
export type Layout = {
  groups: Record<Zone, PanelId[]>;
  active: Record<Zone, PanelId | null>;
  left: number;
  right: number;
  bottom: number;
};

export function defaultLayout(workspace: WorkspaceId): Layout {
  const groups: Layout["groups"] = {
    left: ["project", "library", "transcript"],
    center: ["program", "source"],
    right: [
      "inspector",
      "brief",
      "agent",
      "comments",
      "history",
      "voice",
      "storyboard",
      "delivery",
    ],
    bottom: ["timeline"],
  };
  if (workspace === "Generate") {
    groups.left = ["brief", "storyboard", "project"];
    groups.right = [
      "agent",
      "inspector",
      "comments",
      "history",
      "voice",
      "transcript",
    ];
  } else if (workspace === "Audio") {
    groups.left = ["voice", "project", "transcript"];
    groups.right = [
      "agent",
      "inspector",
      "brief",
      "comments",
      "history",
      "storyboard",
    ];
  } else if (workspace === "Captions") {
    groups.left = ["transcript", "voice", "project"];
    groups.right = [
      "inspector",
      "brief",
      "agent",
      "comments",
      "history",
      "storyboard",
    ];
  } else if (workspace === "Review") {
    groups.left = ["history", "project", "transcript"];
    groups.right = [
      "comments",
      "agent",
      "inspector",
      "brief",
      "voice",
      "storyboard",
    ];
  }
  for (const id of Object.keys(PANELS) as PanelId[]) {
    if (!ZONES.some((zone) => groups[zone].includes(id)))
      groups[id === "source" || id === "library" ? "left" : "right"].push(id);
  }
  return {
    groups,
    active: {
      left: groups.left[0]!,
      center: "program",
      right: groups.right[0]!,
      bottom: "timeline",
    },
    left: 260,
    right: 330,
    bottom: 260,
  };
}

export function movePanel(
  layout: Layout,
  id: PanelId,
  zone: Zone | null,
): Layout {
  const groups = Object.fromEntries(
    ZONES.map((z) => [z, layout.groups[z].filter((p) => p !== id)]),
  ) as Layout["groups"];
  if (zone) groups[zone].push(id);
  const active = { ...layout.active };
  for (const z of ZONES)
    if (!groups[z].includes(active[z]!)) active[z] = groups[z][0] ?? null;
  if (zone) active[zone] = id;
  return { ...layout, groups, active };
}

export function parseLayout(
  raw: string | null,
  workspace: WorkspaceId,
): Layout {
  const fallback = defaultLayout(workspace);
  if (!raw) return fallback;
  try {
    const value = JSON.parse(raw) as Layout;
    const seen = new Set<string>();
    for (const z of ZONES) {
      if (!Array.isArray(value.groups?.[z])) return fallback;
      for (const id of value.groups[z]) {
        if (
          typeof id !== "string" ||
          !Object.hasOwn(PANELS, id) ||
          seen.has(id)
        )
          return fallback;
        seen.add(id);
      }
      if (
        value.groups[z].length
          ? !value.groups[z].includes(value.active?.[z]!)
          : value.active?.[z] !== null
      )
        return fallback;
    }
    if (
      !Number.isFinite(value.left) ||
      !Number.isFinite(value.right) ||
      !Number.isFinite(value.bottom)
    )
      return fallback;
    return {
      ...value,
      left: Math.max(200, Math.min(420, value.left)),
      right: Math.max(250, Math.min(480, value.right)),
      bottom: Math.max(160, Math.min(500, value.bottom)),
    };
  } catch {
    return fallback;
  }
}

export function isTyping(target: EventTarget | null): boolean {
  return (
    typeof HTMLElement !== "undefined" &&
    target instanceof HTMLElement &&
    Boolean(
      target.closest(
        "input, textarea, select, [contenteditable=true], [role=textbox]",
      ),
    )
  );
}
