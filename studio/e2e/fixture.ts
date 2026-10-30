// Per-test fixture: a fresh project folder, a real `ck studio` serving it, and helpers to drive
// the real `ck` CLI against it. The token is read from server.json and never printed.
import { test as base } from "@playwright/test";
import { execFile, spawn, type ChildProcess } from "node:child_process";
import { mkdtemp, readFile, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { createInterface } from "node:readline";
import { promisify } from "node:util";
import type { Page } from "@playwright/test";
import { PANELS, type PanelId } from "../src/workspace/layout";

const run = promisify(execFile);

/** The debug `ck` built by `just e2e`; override with CK_BIN. */
export const CK_BIN =
  process.env.CK_BIN ??
  resolve(import.meta.dirname, "..", "..", "target", "debug", "ck");

export type Studio = {
  dir: string;
  url: string;
  token: string;
  /** Runs `ck <args> --dir <project>` and returns its parsed JSON stdout. */
  ck: (...args: string[]) => Promise<unknown>;
};

export const test = base.extend<{ studio: Studio }>({
  studio: async ({}, use) => {
    const dir = await mkdtemp(join(tmpdir(), "ck-e2e-"));
    // Studios under test learn into a scratch taste profile, never the user's.
    process.env.CK_PROFILE = join(dir, "taste-profile.json");
    // The studio token never reaches server.json; the fixture fixes it.
    process.env.CK_STUDIO_TOKEN = "e2e-studio-token";
    const child = spawn(CK_BIN, ["studio", "--dir", dir], {
      stdio: ["ignore", "pipe", "inherit"],
    });
    try {
      const { url } = (await firstLine(child)) as { url: string };
      const info = JSON.parse(
        await readFile(join(dir, ".clipkit", "server.json"), "utf8"),
      ) as { token: string };
      const ck = async (...args: string[]) => {
        const { stdout } = await run(CK_BIN, [...args, "--dir", dir]);
        return JSON.parse(stdout) as unknown;
      };
      await use({ dir, url, token: "e2e-studio-token", ck });
    } finally {
      child.kill("SIGTERM");
      await new Promise((r) => child.once("exit", r));
      await rm(dir, { recursive: true, force: true });
    }
  },
});

export { expect } from "@playwright/test";

/** Navigate the real workspace instead of assuming every panel is visible at once. */
export async function openPanel(page: Page, id: PanelId) {
  await page
    .getByRole("button", { name: "Open command palette", exact: true })
    .click();
  await page
    .getByRole("textbox", { name: "Search commands" })
    .fill(`Open ${PANELS[id]}`);
  await page.getByRole("textbox", { name: "Search commands" }).press("Enter");
  await base
    .expect(page.getByRole("tab", { name: PANELS[id], exact: true }))
    .toHaveAttribute("aria-selected", "true");
}

/** `ck studio` prints one JSON line ({url, port}) once it listens. */
function firstLine(child: ChildProcess): Promise<unknown> {
  return new Promise((ok, fail) => {
    if (!child.stdout) return fail(new Error("ck studio has no stdout"));
    const lines = createInterface({ input: child.stdout });
    child.once("exit", (code) =>
      fail(new Error(`ck studio exited early (code ${code})`)),
    );
    lines.once("line", (line) => ok(JSON.parse(line)));
  });
}

/** A clip on a new overlay track, as the agent would add it. */
export function addTitleOps(id: string, name: string) {
  return [
    {
      op: "add",
      path: "/tracks/-",
      value: {
        id: "t1",
        kind: "overlay",
        items: [
          {
            id,
            name,
            purpose: "",
            briefDeps: [],
            rev: 0,
            type: "clip",
            at: 0,
            dur: 2,
            props: {},
          },
        ],
      },
    },
  ];
}
