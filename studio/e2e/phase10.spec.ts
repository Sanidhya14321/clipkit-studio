// Phase 10 step 4: the storyboard checkpoint. The brief asks for one; the agent renders a
// still per scene (render worker); the studio shows the sheet; the user's confirm reaches the
// agent's `ck wait` as a checkpoint event.
import { existsSync } from "node:fs";
import { writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { expect, test } from "./fixture";

const BUNDLE = resolve(
  import.meta.dirname,
  "..",
  "..",
  "render-worker",
  "bundle",
  "index.html",
);

test.skip(!existsSync(BUNDLE), "run `just worker` first");

test("the user confirms the storyboard; the agent's wait sees it", async ({
  page,
  studio,
}) => {
  test.setTimeout(90_000);
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
  for (const [i, recipe] of [
    { recipe: "hook-title", at: 0, duration: 3, slots: { headline: "Hello" } },
    {
      recipe: "outro-cta",
      at: 3,
      duration: 3,
      slots: { action: "Try it", handle: "x.dev" },
    },
  ].entries()) {
    const file = join(studio.dir, `r${i}.json`);
    await writeFile(file, JSON.stringify(recipe));
    await studio.ck("recipe", "apply", file);
  }
  const status = (await studio.ck("status")) as { rev: number };
  await studio.ck(
    "project",
    "patch",
    JSON.stringify({
      baseRev: status.rev,
      author: "human",
      ops: [{ op: "add", path: "/brief/checkpoints/-", value: "storyboard" }],
    }),
  );
  const board = (await studio.ck("storyboard", "get")) as {
    scenes: number[];
    requested: boolean;
    confirmed: boolean;
  };
  expect(board).toMatchObject({
    scenes: [0, 3],
    requested: true,
    confirmed: false,
  });
  const since = ((await studio.ck("status")) as { rev: number }).rev;

  const panel = page.locator("[data-ck-storyboard]");
  await expect(panel.locator("[data-ck-storyboard-sheet]")).toBeVisible();
  await expect
    .poll(() =>
      panel
        .locator("[data-ck-storyboard-sheet]")
        .evaluate((img: HTMLImageElement) => img.naturalWidth),
    )
    .toBe(960);
  await panel.locator("[data-ck-storyboard-confirm]").click();
  await expect(panel).toHaveCount(0);

  const waited = (await studio.ck(
    "wait",
    "--since",
    String(since),
    "--timeout",
    "0",
  )) as {
    events: { kind: string }[];
  };
  expect(waited.events.map((e) => e.kind)).toContain("checkpoint");
});
