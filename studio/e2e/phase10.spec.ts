// Phase 10 step 4: the storyboard checkpoint. The brief asks for one; the agent renders a
// still per scene (render worker); the studio shows the sheet; the user's confirm reaches the
// agent's `ck wait` as a checkpoint event.
import { existsSync } from "node:fs";
import { writeFile } from "node:fs/promises";
import { join, resolve } from "node:path";
import { expect, test, openPanel, advanceToVerify } from "./fixture";

const BUNDLE = resolve(
  import.meta.dirname,
  "..",
  "..",
  "render-worker",
  "bundle",
  "index.html",
);

test("the user confirms the storyboard; the agent's wait sees it", async ({
  page,
  studio,
}) => {
  test.skip(!existsSync(BUNDLE), "run `just worker` first");
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

  await openPanel(page, "storyboard");
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

test("history: undo, redo, save a branch and switch back", async ({
  page,
  studio,
}) => {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
  const file = join(studio.dir, "r.json");
  await writeFile(
    file,
    JSON.stringify({ recipe: "hook-title", slots: { headline: "One" } }),
  );
  await studio.ck("recipe", "apply", file);

  await openPanel(page, "history");
  const panel = page.locator("[data-ck-history]");
  await expect(
    panel.locator("[data-ck-history-lines] li").first(),
  ).toContainText("recipe: hook-title");
  await panel.locator("[data-ck-branch-name]").fill("first");
  await panel.locator("[data-ck-branch-save]").click();
  await expect(panel.locator('[data-ck-branch="first"]')).toBeVisible();

  // Undo removes the scene; the branch brings it back as one revision.
  await panel.locator("[data-ck-undo]").click();
  const timeline = page.locator("[data-ck-timeline]");
  await expect(
    timeline.locator('[data-ck-layer="hook-title-backdrop"]'),
  ).toHaveCount(0);
  await panel.locator('[data-ck-branch-switch="first"]').click();
  await expect(
    timeline.locator('[data-ck-layer="hook-title-backdrop"]'),
  ).toHaveCount(1);
  await expect(
    panel.locator("[data-ck-history-lines] li").first(),
  ).toContainText("human switch to branch first");
});

test("Stop: the agent's next write is refused until it hands back", async ({
  page,
  studio,
}) => {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
  const file = join(studio.dir, "r.json");
  await writeFile(
    file,
    JSON.stringify({ recipe: "hook-title", slots: { headline: "One" } }),
  );
  // The agent is working: it holds a lock (the banner shows it working).
  await studio.ck("recipe", "apply", file);
  await studio.ck("lock", "hook-title-headline");
  const banner = page.locator("[data-ck-banner]");
  await expect(banner).toHaveAttribute("data-ck-agent", "working");
  await banner.locator("[data-ck-stop-agent]").click();
  await expect(banner).toContainText("Stopping after the current step");

  const again = join(studio.dir, "r2.json");
  await writeFile(
    again,
    JSON.stringify({
      recipe: "outro-cta",
      at: 4,
      slots: { action: "Go", handle: "x.dev" },
    }),
  );
  const refused = await studio
    .ck("recipe", "apply", again)
    .catch((e: { stderr?: string }) => e.stderr ?? "");
  expect(String(refused)).toContain("E_STAGE");
  await advanceToVerify(page);
  await studio.ck(
    "handoff",
    "--override-reason",
    "The user stopped this unfinished proof sequence; return control without claiming completion.",
  );
  await expect(banner).toHaveAttribute("data-ck-agent", "idle");
});
