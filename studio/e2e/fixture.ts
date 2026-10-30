// Per-test fixture: a fresh project folder, a real `ck studio` serving it, and helpers to drive
// the real `ck` CLI against it. Human actions use a fixed test-only studio credential.
import { test as base } from "@playwright/test";
import { execFile, spawn, type ChildProcess } from "node:child_process";
import { mkdtemp, rm } from "node:fs/promises";
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
      const ck = async (...args: string[]) => {
        // Some proofs explicitly simulate a human patch. Use the studio credential
        // in this harness rather than asking an agent-token CLI to impersonate it.
        if (args[0] === "project" && args[1] === "patch" && args[2]) {
          const patch = JSON.parse(args[2]);
          if (patch.author === "human") {
            const response = await fetch(`${url}api/patch`, {
              method: "POST",
              headers: {
                "x-ck-token": "e2e-studio-token",
                "content-type": "application/json",
              },
              body: args[2],
            });
            const body = await response.json();
            if (!response.ok) throw new Error(JSON.stringify(body));
            return body;
          }
        }
        const { stdout } = await run(CK_BIN, [...args, "--dir", dir]);
        return JSON.parse(stdout) as unknown;
      };
      // Editing fixtures start with a confirmed empty brief. Tests which exercise
      // proposal/confirmation replace that status through the real propose route.
      await ck(
        "project",
        "patch",
        JSON.stringify({
          baseRev: 0,
          author: "human",
          label: "Confirm test fixture brief",
          ops: [{ op: "replace", path: "/brief/status", value: "confirmed" }],
        }),
      );
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

/** Finish fixture passes through the server's stage API, without bypassing gates. */
export async function advanceToVerify(page: Page) {
  await page.evaluate(async () => {
    const headers = {
      "x-ck-token": sessionStorage.getItem("ck-token") ?? "",
      "content-type": "application/json",
    };
    for (let n = 0; n < 12; n++) {
      const project = await (
        await fetch("/api/project?view=full", { headers })
      ).json();
      if (["verify", "handoff", "iterate"].includes(project.pipeline.stage))
        return;
      const response = await fetch("/api/pipeline/next", {
        method: "POST",
        headers,
      });
      if (!response.ok) throw new Error(JSON.stringify(await response.json()));
    }
    throw new Error("Fixture pipeline did not reach verify");
  });
}
