// Phase 3 proof (spec 12), headless stand-in for a Claude Code session: the plugin's own hook
// commands (plugin/hooks/hooks.json) run through `sh -c` the way Claude Code runs them, the
// studio opens on a video prompt, and the agent patches a layer through `ck mcp` (the command
// the plugin's .mcp.json registers) while the page watches. No studio is started by the test.
import { spawn } from "node:child_process";
import {
  chmod,
  mkdir,
  mkdtemp,
  readFile,
  rm,
  symlink,
  writeFile,
} from "node:fs/promises";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { createInterface } from "node:readline";
import { addTitleOps, CK_BIN, expect, test } from "./fixture";

const HOOKS = resolve(
  import.meta.dirname,
  "..",
  "..",
  "plugin",
  "hooks",
  "hooks.json",
);

type HookEvent = "UserPromptSubmit" | "PreToolUse" | "Stop" | "SessionEnd";
type HookRun = { code: number | null; stdout: string; stderr: string };

/** A fake agent machine: `ck` on PATH, a fake browser, and a project folder with no studio. */
async function machine() {
  const root = await mkdtemp(join(tmpdir(), "ck-e2e3-"));
  const bin = join(root, "bin");
  const dir = join(root, "project");
  await mkdir(bin);
  await mkdir(dir);
  await symlink(CK_BIN, join(bin, "ck"));
  const opened = join(root, "opened.txt");
  const browser = join(bin, "browser.sh");
  await writeFile(browser, `#!/bin/sh\nprintf '%s\\n' "$1" >> '${opened}'\n`);
  await chmod(browser, 0o755);
  const env = {
    ...process.env,
    PATH: `${bin}:${process.env.PATH ?? ""}`,
    BROWSER: browser,
    CLAUDE_PROJECT_DIR: dir,
  };
  const hooks = JSON.parse(await readFile(HOOKS, "utf8")) as {
    hooks: Record<HookEvent, { hooks: { command: string }[] }[]>;
  };

  /** Runs the plugin's command for `event` with the hook JSON on stdin. */
  const hook = (event: HookEvent, input: object) =>
    new Promise<HookRun>((ok, fail) => {
      const command = hooks.hooks[event][0]?.hooks[0]?.command;
      if (!command)
        return fail(new Error(`hooks.json has no ${event} command`));
      const child = spawn("sh", ["-c", command], { env, cwd: dir });
      let stdout = "";
      let stderr = "";
      child.stdout.on("data", (d: Buffer) => (stdout += d.toString()));
      child.stderr.on("data", (d: Buffer) => (stderr += d.toString()));
      child.once("error", fail);
      child.once("close", (code) => ok({ code, stdout, stderr }));
      child.stdin.end(
        JSON.stringify({
          session_id: "e2e",
          cwd: dir,
          hook_event_name: event,
          ...input,
        }),
      );
    });

  /** The URLs the fake browser was asked to open, waiting up to `waitMs` for `count`. */
  const openedUrls = async (count: number, waitMs = 5000) => {
    const until = Date.now() + waitMs;
    for (;;) {
      const text = await readFile(opened, "utf8").catch(() => "");
      const lines = text.split("\n").filter(Boolean);
      if (lines.length >= count || Date.now() > until) return lines;
      await new Promise((r) => setTimeout(r, 20));
    }
  };

  /** Stops the background studio through the pid in server.json, then removes the machine. */
  const cleanup = async () => {
    const info = await readFile(join(dir, ".clipkit", "server.json"), "utf8")
      .then((t) => JSON.parse(t) as { pid: number })
      .catch(() => null);
    if (info) {
      try {
        process.kill(info.pid, "SIGTERM");
      } catch {
        // already gone
      }
    }
    await rm(root, { recursive: true, force: true });
  };

  return { dir, env, hook, openedUrls, cleanup };
}

/** A `ck mcp` session started like the plugin's .mcp.json entry (`ck mcp --dir <project>`). */
function mcp(dir: string, env: NodeJS.ProcessEnv) {
  const child = spawn("ck", ["mcp", "--dir", dir], { env });
  const lines = createInterface({ input: child.stdout });
  const waiting = new Map<number, (v: Record<string, unknown>) => void>();
  lines.on("line", (line) => {
    const msg = JSON.parse(line) as { id?: number } & Record<string, unknown>;
    if (msg.id !== undefined) waiting.get(msg.id)?.(msg);
  });
  let next = 0;
  const request = (method: string, params: object) =>
    new Promise<Record<string, unknown>>((ok) => {
      const id = ++next;
      waiting.set(id, ok);
      child.stdin.write(
        JSON.stringify({ jsonrpc: "2.0", id, method, params }) + "\n",
      );
    });
  const call = async (name: string, args: object) => {
    const res = (await request("tools/call", { name, arguments: args })) as {
      result: { isError?: boolean; content: { text: string }[] };
    };
    const text = res.result.content[0]?.text ?? "";
    expect(res.result.isError ?? false, text).toBe(false);
    return JSON.parse(text) as Record<string, unknown>;
  };
  const start = async () => {
    await request("initialize", {
      protocolVersion: "2025-06-18",
      capabilities: {},
      clientInfo: { name: "ck-e2e", version: "0" },
    });
    child.stdin.write(
      JSON.stringify({
        jsonrpc: "2.0",
        method: "notifications/initialized",
      }) + "\n",
    );
  };
  const close = () => child.kill("SIGTERM");
  return { start, call, close };
}

test("a video prompt opens the studio and the agent's MCP patch appears while the user watches", async ({
  page,
}) => {
  const m = await machine();
  try {
    // UserPromptSubmit: the hook starts the studio, opens one tab and adds one context line.
    const prompt = await m.hook("UserPromptSubmit", {
      prompt: "Make a 15 second launch video for our app",
    });
    expect(prompt.code, prompt.stderr).toBe(0);
    const lines = prompt.stdout.trim().split("\n");
    expect(lines).toHaveLength(1);
    expect(lines[0]).toMatch(/^ClipKit studio: http:\/\/127\.0\.0\.1:\d+/);
    expect(lines[0]).not.toContain("token");
    const [url = ""] = await m.openedUrls(1);
    expect(url).toMatch(/^http:\/\/127\.0\.0\.1:\d+\/?#token=/);

    // The user's tab: the URL the hook opened, live.
    await page.goto(url);
    await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
    await expect(page.locator("[data-ck-timeline]")).toBeVisible();

    // The agent: lock and patch a layer through MCP, as the plugin registers it.
    const agent = mcp(m.dir, m.env);
    try {
      await agent.start();
      await page.evaluate(async () => {
        const headers = {
          "x-ck-token": sessionStorage.getItem("ck-token") ?? "",
          "content-type": "application/json",
        };
        const project = await (
          await fetch("/api/project?view=full", { headers })
        ).json();
        const response = await fetch("/api/patch", {
          method: "POST",
          headers,
          body: JSON.stringify({
            baseRev: project.rev,
            author: "human",
            label: "Confirm proof brief",
            ops: [{ op: "replace", path: "/brief/status", value: "confirmed" }],
          }),
        });
        if (!response.ok) throw new Error("Studio brief confirmation failed");
      });
      await agent.call("lock", { holder: "agent", layerIds: ["t1", "title"] });
      const { rev } = (await agent.call("status", {})) as { rev: number };
      await agent.call("project_patch", {
        baseRev: rev,
        ops: addTitleOps("title", "Hook title"),
        author: "agent",
        label: "Text pass: adding the hook title",
      });
    } finally {
      agent.close();
    }
    const layer = page.locator('[data-ck-timeline] [data-ck-layer="title"]');
    await expect(layer).toHaveText(/Hook title/);
    await expect(layer).toHaveAttribute("data-ck-flash", "true");
    await expect(layer.locator("[data-ck-ghost]")).toBeVisible();
    await expect(page.locator("[data-ck-feed-rev]").first()).toHaveAttribute(
      "data-ck-author",
      "agent",
    );

    // PreToolUse: a direct ffmpeg call in the project is blocked with a reason.
    const guard = await m.hook("PreToolUse", {
      tool_name: "Bash",
      tool_input: { command: "ffmpeg -i in.mp4 out.mp4" },
    });
    expect(guard.code).toBe(2);
    expect(guard.stderr).toContain("ck");

    // Stop: blocks once while the agent holds locks, then steps aside.
    const stop = await m.hook("Stop", { stop_hook_active: false });
    expect(stop.code).toBe(2);
    expect(stop.stderr).toContain("title");
    expect((await m.hook("Stop", { stop_hook_active: true })).code).toBe(0);

    // SessionEnd: the agent's locks are released and the user can take over.
    expect((await m.hook("SessionEnd", { reason: "exit" })).code).toBe(0);
    await expect(layer.locator("[data-ck-ghost]")).toHaveCount(0);
    await layer.click();
    await expect(page.locator('[data-ck-inspector="title"]')).toHaveAttribute(
      "data-ck-readonly",
      "false",
    );

    // A second prompt in the same project reuses the studio and opens no new tab.
    const again = await m.hook("UserPromptSubmit", {
      prompt: "make it punchier",
    });
    expect(again.code).toBe(0);
    // The opener runs detached: give a stray second open a second to show up.
    expect(await m.openedUrls(2, 1000)).toHaveLength(1);
  } finally {
    await m.cleanup();
  }
});
