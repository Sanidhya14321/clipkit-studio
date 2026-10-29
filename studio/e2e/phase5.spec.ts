// Phase 5 step 7: the transcript panel. Deleting a word cuts it from the footage as a human
// revision; the same key on a cut word restores it at the clip edge. While the agent holds the
// video track the panel is read-only and the server refuses the edit.
import { copyFile, mkdir, writeFile } from "node:fs/promises";
import { existsSync } from "node:fs";
import { join, resolve } from "node:path";
import type { Page } from "@playwright/test";
import { expect, test, type Studio } from "./fixture";

const FIXTURES = resolve(
  import.meta.dirname,
  "..",
  "..",
  "fixtures",
  "generated",
);

type Imported = { source: { id: string; hash: string } };

/** Import av.mp4 and attach the fixture transcript as if `ck transcribe` had run (the real
 * transcriber needs a Fish Audio key). */
async function importWithTranscript(page: Page, studio: Studio) {
  const { source } = (await studio.ck(
    "import",
    join(FIXTURES, "av.mp4"),
  )) as Imported;
  const rel = `.clipkit/cache/transcripts/${source.hash}.json`;
  await mkdir(join(studio.dir, ".clipkit", "cache", "transcripts"), {
    recursive: true,
  });
  await copyFile(join(FIXTURES, "transcript.json"), join(studio.dir, rel));
  const res = await page.evaluate(async (rel: string) => {
    const token = sessionStorage.getItem("ck-token") ?? "";
    const headers = { "x-ck-token": token, "content-type": "application/json" };
    const p = (await (await fetch("/api/status", { headers })).json()) as {
      rev: number;
    };
    const r = await fetch("/api/patch", {
      method: "POST",
      headers,
      body: JSON.stringify({
        baseRev: p.rev,
        ops: [{ op: "add", path: "/sources/0/transcriptPath", value: rel }],
        author: "human",
        label: "attach transcript",
      }),
    });
    return r.status;
  }, rel);
  expect(res).toBe(200);
  return source.id;
}

async function postCut(page: Page, source: string, deleted: number[]) {
  return page.evaluate(
    async ({ source, deleted }) => {
      const token = sessionStorage.getItem("ck-token") ?? "";
      const r = await fetch("/api/cut/words", {
        method: "POST",
        headers: { "x-ck-token": token, "content-type": "application/json" },
        body: JSON.stringify({ source, deleted }),
      });
      return { status: r.status, body: (await r.json()) as { code?: string } };
    },
    { source, deleted },
  );
}

test.skip(!existsSync(join(FIXTURES, "av.mp4")), "run `just fixtures` first");

test("deleting a word cuts the footage; deleting it again restores it", async ({
  page,
  studio,
}) => {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
  const id = await importWithTranscript(page, studio);

  const panel = page.locator(`[data-ck-transcript="${id}"]`);
  const um = panel.locator('[data-ck-word="1"]');
  await expect(um).toHaveText("um");
  await expect(um).toHaveAttribute("data-ck-deleted", "false");

  await um.focus();
  await page.keyboard.press("Delete");
  await expect(um).toHaveAttribute("data-ck-deleted", "true");
  await expect(panel.locator('[data-ck-word="0"]')).toHaveAttribute(
    "data-ck-deleted",
    "false",
  );
  // Two footage segments now sit on the video track, split around "um".
  await expect(page.locator(`[data-ck-layer="${id}-seg-2"]`)).toHaveCount(1);
  await expect(page.locator(`[data-ck-layer="${id}-seg-3"]`)).toHaveCount(0);

  // Restore at the clip edge.
  await page.keyboard.press("Backspace");
  await expect(um).toHaveAttribute("data-ck-deleted", "false");
  await expect(page.locator(`[data-ck-layer="${id}-seg-2"]`)).toHaveCount(0);
});

test("while the agent holds the video track the panel is read-only", async ({
  page,
  studio,
}) => {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
  const id = await importWithTranscript(page, studio);
  await studio.ck("cut", "apply", id, "--ids", "cut-1");
  await studio.ck("lock", "footage-1");

  const panel = page.locator(`[data-ck-transcript="${id}"]`);
  await expect(panel).toHaveAttribute("data-ck-readonly", "true");
  const word = panel.locator('[data-ck-word="2"]');
  await word.focus();
  await page.keyboard.press("Delete");
  await expect(word).toHaveAttribute("data-ck-deleted", "false");

  const refused = await postCut(page, id, [2]);
  expect(refused.status).toBe(423);
  expect(refused.body.code).toBe("E_LOCK_HELD");

  await studio.ck("unlock");
  await expect(panel).toHaveAttribute("data-ck-readonly", "false");
});

test("hybrid: footage plays under generated layers", async ({
  page,
  studio,
}) => {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
  const file = join(studio.dir, "apply.json");
  await writeFile(
    file,
    JSON.stringify({ recipe: "hook-title", slots: { headline: "Hi" } }),
  );
  await studio.ck("recipe", "apply", file);
  const { source } = (await studio.ck(
    "import",
    join(FIXTURES, "av.mp4"),
  )) as Imported;

  const status = (await studio.ck("status")) as { mode: string };
  expect(status.mode).toBe("hybrid");
  const footage = page.locator(`[data-ck-footage="${source.id}"]`);
  await expect(footage).toHaveCount(1);
  // The proxy loads through the token-in-query media route.
  await expect
    .poll(() =>
      footage.locator("video").evaluate((v: HTMLVideoElement) => v.readyState),
    )
    .toBeGreaterThanOrEqual(1);
  // The generated backdrop is still there (preview and timeline both carry its id).
  await expect(
    page.locator('[data-ck-layer="hook-title-backdrop"]').first(),
  ).toBeAttached();
});
