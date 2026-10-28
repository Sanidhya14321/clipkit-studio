// Phase 2 proof (spec 12): a CLI patch appears in the browser in under 300 ms, and human edits
// are refused while the agent holds the lock, both in the DOM and by the server.
import type { Page } from "@playwright/test";
import type { Project } from "../src/generated/types";
import { fieldPatch } from "../src/state";
import { addTitleOps, expect, test, type Studio } from "./fixture";

const PATCH_BUDGET_MS = 300;

async function open(page: Page, studio: Studio) {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
  await expect(page.locator("[data-ck-timeline]")).toBeVisible();
}

/** Adds the "title" clip as the agent would: lock, patch through the CLI, keep the lock. */
async function agentAddsTitle(studio: Studio) {
  await studio.ck("lock", "t1", "title");
  const { rev } = (await studio.ck("status")) as { rev: number };
  return studio.ck(
    "project",
    "patch",
    JSON.stringify({
      baseRev: rev,
      ops: addTitleOps("title", "Hook title"),
      author: "agent",
      label: "Text pass: adding the hook title",
    }),
  );
}

/** Calls the API from inside the page, so the request carries the studio's own Origin. */
function pageFetch(page: Page, path: string, body?: unknown) {
  return page.evaluate(
    async ([path, body]) => {
      // The studio moves the token from the URL fragment into sessionStorage (src/api.ts).
      const token = sessionStorage.getItem("ck-token");
      const res = await fetch(path, {
        method: body === undefined ? "GET" : "POST",
        headers: {
          "x-ck-token": token ?? "",
          "content-type": "application/json",
        },
        body: body === undefined ? undefined : JSON.stringify(body),
      });
      return { status: res.status, body: await res.json() };
    },
    [path, body] as const,
  );
}

test("a CLI patch appears in the timeline and the preview in under 300 ms", async ({
  page,
  studio,
}) => {
  await open(page, studio);
  await studio.ck("lock", "t1", "title");
  const { rev } = (await studio.ck("status")) as { rev: number };

  // Timestamp the layer's arrival in the DOM from inside the page (same machine clock). The
  // probe is installed, and finds nothing yet, before the CLI runs.
  const absent = await page.evaluate(() => {
    const seen: { timeline?: number; preview?: number } = {};
    (window as unknown as { ckSeen: typeof seen }).ckSeen = seen;
    const check = () => {
      const now = Date.now();
      if (
        !seen.timeline &&
        document.querySelector('[data-ck-timeline] [data-ck-layer="title"]')
      )
        seen.timeline = now;
      if (
        !seen.preview &&
        document.querySelector('.studio-preview [data-ck-layer="title"]')
      )
        seen.preview = now;
      if (seen.timeline && seen.preview) observer.disconnect();
    };
    const observer = new MutationObserver(check);
    observer.observe(document.body, {
      childList: true,
      subtree: true,
      attributes: true,
    });
    check();
    return seen.timeline === undefined && seen.preview === undefined;
  });
  expect(absent).toBe(true);

  const started = Date.now();
  await studio.ck(
    "project",
    "patch",
    JSON.stringify({
      baseRev: rev,
      ops: addTitleOps("title", "Hook title"),
      author: "agent",
      label: "Text pass: adding the hook title",
    }),
  );
  const at = (await (
    await page.waitForFunction(() => {
      const seen = (
        window as unknown as { ckSeen: { timeline?: number; preview?: number } }
      ).ckSeen;
      return seen.timeline && seen.preview ? seen : null;
    })
  ).jsonValue()) as { timeline: number; preview: number };
  const timelineMs = at.timeline - started;
  const previewMs = at.preview - started;
  test.info().annotations.push({
    type: "latency",
    description: `CLI spawn to timeline ${timelineMs} ms, to preview ${previewMs} ms`,
  });
  console.log(`latency: timeline ${timelineMs} ms, preview ${previewMs} ms`);
  expect(timelineMs).toBeLessThan(PATCH_BUDGET_MS);
  expect(previewMs).toBeLessThan(PATCH_BUDGET_MS);

  await expect(
    page.locator('[data-ck-timeline] [data-ck-layer="title"]'),
  ).toHaveText(/Hook title/);
  await expect(page.locator("[data-ck-feed-rev]").first()).toHaveAttribute(
    "data-ck-author",
    "agent",
  );
  // The touched layer flashes; the agent's locked layer carries the ghost cursor.
  const layer = page.locator('[data-ck-timeline] [data-ck-layer="title"]');
  await expect(layer).toHaveAttribute("data-ck-flash", "true");
  await expect(layer.locator("[data-ck-ghost]")).toBeVisible();

  // The banner shows the agent's state. Patches cannot set it (`/agent` is read-only); the
  // working state arrives with the pipeline commands (Phase 3).
  await expect(page.locator("[data-ck-banner]")).toHaveAttribute(
    "data-ck-agent",
    "idle",
  );
});

test("human edits are refused while the agent holds the lock, then land after unlock", async ({
  page,
  studio,
}) => {
  await open(page, studio);
  await agentAddsTitle(studio);

  const layer = page.locator('[data-ck-timeline] [data-ck-layer="title"]');
  await layer.click();
  const inspector = page.locator('[data-ck-inspector="title"]');
  const name = inspector.locator('[data-ck-prop="name"]');

  // DOM: read-only, every control disabled, ghost cursor on the agent's layer.
  await expect(inspector).toHaveAttribute("data-ck-readonly", "true");
  for (const input of await inspector.locator("input").all())
    await expect(input).toBeDisabled();
  await expect(layer.locator("[data-ck-ghost]")).toBeVisible();

  // Server: the studio's own edit request is refused with E_LOCK_HELD.
  const full = await pageFetch(page, "/api/project?view=full");
  expect(full.status, JSON.stringify(full.body)).toBe(200);
  const project = full.body as Project;
  const req = fieldPatch(project, "title", "name", "Mine");
  expect(req).not.toBeNull();
  const refused = await pageFetch(page, "/api/patch", req);
  expect(refused.status).toBe(423);
  expect(refused.body.code).toBe("E_LOCK_HELD");
  expect(
    (
      (await studio.ck("project", "get", "--layer", "title")) as {
        name: string;
      }
    ).name,
  ).toBe("Hook title");

  // After the agent releases the layer the inspector unlocks and a real UI edit lands.
  await studio.ck("unlock");
  await expect(inspector).toHaveAttribute("data-ck-readonly", "false");
  await expect(name).toBeEnabled();
  await name.fill("Mine");
  await name.press("Enter");
  await expect(layer).toHaveText(/Mine/);
  await expect(page.locator("[data-ck-feed-rev]").first()).toHaveAttribute(
    "data-ck-author",
    "human",
  );
  expect(
    (
      (await studio.ck("project", "get", "--layer", "title")) as {
        name: string;
      }
    ).name,
  ).toBe("Mine");
});
