// Frontend contract fixture only. This does not replace real ck end-to-end tests.
import { createServer } from "node:http";
import { readFile } from "node:fs/promises";
import { resolve, extname } from "node:path";
import type { AddressInfo } from "node:net";
import type { Page, WebSocketRoute } from "@playwright/test";
import type { Project } from "../src/generated/types";

export function workspaceProject(): Project {
  return {
    schemaVersion: 1,
    id: "ui-preview",
    name: "ClipKit · UI contract fixture",
    rev: 1,
    mode: "generate",
    settings: { fps: 30, width: 1920, height: 1080 },
    brief: { fields: {}, status: "confirmed", checkpoints: [] },
    plan: null,
    tokens: {},
    sources: [],
    tracks: [
      {
        id: "V1",
        kind: "overlay",
        items: [
          {
            id: "opening",
            name: "Opening title",
            purpose: "Introduce the story",
            briefDeps: [],
            rev: 1,
            type: "clip",
            clipId: "kinetic-text",
            at: 0,
            dur: 5,
            props: {
              text: "Make something worth watching.",
              subtitle: "YOUR STORY. YOUR STUDIO.",
            },
          },
          {
            id: "closing",
            name: "Closing title",
            purpose: "Close the story",
            briefDeps: [],
            rev: 1,
            type: "clip",
            clipId: "kinetic-text",
            at: 5,
            dur: 5,
            props: { text: "Made with ClipKit" },
          },
        ],
      },
    ],
    voiceSlots: [],
    comments: [],
    assetsUsed: [],
    derived: [],
    locks: [],
    branches: [],
    pipeline: { stage: "handoff" },
    agent: { status: "idle", label: "", activeLayers: [] },
  };
}

export async function serveWorkspaceBuild() {
  const root = resolve(import.meta.dirname, "../dist");
  const server = createServer(async (req, res) => {
    const path = resolve(
      root,
      `.${new URL(req.url ?? "/", "http://localhost").pathname === "/" ? "/index.html" : new URL(req.url ?? "/", "http://localhost").pathname}`,
    );
    if (!path.startsWith(`${root}/`)) {
      res.writeHead(403).end();
      return;
    }
    try {
      const body = await readFile(path);
      res.setHeader(
        "content-type",
        (
          {
            ".html": "text/html",
            ".js": "text/javascript",
            ".css": "text/css",
          } as Record<string, string>
        )[extname(path)] ?? "application/octet-stream",
      );
      res.end(body);
    } catch {
      res.writeHead(404).end();
    }
  });
  await new Promise<void>((ok) => server.listen(0, "127.0.0.1", ok));
  return {
    url: `http://127.0.0.1:${(server.address() as AddressInfo).port}`,
    close: () =>
      new Promise<void>((ok, fail) =>
        server.close((e) => (e ? fail(e) : ok())),
      ),
  };
}

export async function mockWorkspace(page: Page, project = workspaceProject()) {
  let socket: WebSocketRoute | undefined;
  const calls: { path: string; body: unknown }[] = [];
  let refuse = false;
  let refreshFails = false;
  const snapshot = () =>
    socket?.send(JSON.stringify({ type: "snapshot", project }));
  await page.routeWebSocket(/\/ws\?/, (ws) => {
    socket = ws;
    snapshot();
  });
  await page.route("**/api/**", async (route) => {
    const req = route.request();
    const path = new URL(req.url()).pathname;
    if (req.method() === "POST") {
      const body = req.postDataJSON();
      calls.push({ path, body });
      if (refuse && path !== "/api/context") {
        await route.fulfill({
          status: 409,
          json: {
            code: "E_PATCH_STALE",
            message: "The project changed. Try again.",
            hint: "Refresh the revision.",
          },
        });
        return;
      }
      if (path === "/api/patch") {
        const op = body.ops[0];
        if (op.path === "/tracks/0/items/0/name") {
          project.tracks[0]!.items[0]!.name = op.value;
          project.rev++;
          project.tracks[0]!.items[0]!.rev = project.rev;
          snapshot();
        }
      }
      await route.fulfill({ json: { rev: project.rev } });
      return;
    }
    if (path === "/api/project" && refreshFails) {
      await route.fulfill({
        status: 503,
        json: {
          code: "E_SERVER",
          message: "Refresh unavailable",
          hint: "Reconnect.",
        },
      });
      return;
    }
    await route.fulfill({
      json:
        path === "/api/project"
          ? project
          : path === "/api/looks" || path === "/api/asks"
            ? []
            : path === "/api/branch/list"
              ? { branches: [] }
              : {
                  rev: project.rev,
                  lines: ["1 human: Created preview fixture"],
                },
    });
  });
  return {
    project,
    calls,
    snapshot,
    refuse: () => {
      refuse = true;
    },
    failRefresh: () => {
      refreshFails = true;
    },
    disconnect: () => socket?.close(),
  };
}
