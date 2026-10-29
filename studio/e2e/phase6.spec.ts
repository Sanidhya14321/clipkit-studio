// Phase 6 step 5: comments and pins, hover-select with Ask agent, and the context the studio
// reports for `ck context`. Comments go through while the agent holds the project lock.
import { writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { Page } from "@playwright/test";
import { expect, test, type Studio } from "./fixture";

/** The studio posts context 250 ms after it settles; allow for a loaded machine. */
const CONTEXT_POLL = { timeout: 15_000 };

async function open(page: Page, studio: Studio) {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
}

type Bundle = {
  comments: {
    id: string;
    text: string;
    author: string;
    anchor: { tStart?: number; layerId?: string; propPath?: string };
  }[];
};

test("the user pins a comment while the agent works; the agent sees it", async ({
  page,
  studio,
}) => {
  await open(page, studio);
  await studio.ck("lock", "@project");

  const panel = page.locator("[data-ck-comments]");
  await panel.locator("[data-ck-comment-text]").fill("Make the intro faster");
  await panel.locator("[data-ck-send]").click();
  await expect(panel.locator('[data-ck-comment="c1"]')).toHaveAttribute(
    "data-ck-status",
    "open",
  );
  // Pinned to the playhead: a pin on the timeline.
  await expect(page.locator('[data-ck-pin="c1"]')).toHaveCount(1);

  const bundle = (await studio.ck("comments", "list")) as Bundle;
  expect(bundle.comments[0]).toMatchObject({
    id: "c1",
    author: "human",
    text: "Make the intro faster",
    anchor: { tStart: 0 },
  });

  // The agent starts work: the studio shows it working, then resolved.
  await studio.ck("comments", "reply", "c1", "On it", "--scope", "");
  await expect(panel.locator('[data-ck-comment="c1"]')).toHaveAttribute(
    "data-ck-status",
    "working",
  );
  await studio.ck("comments", "resolve", "c1", "--text", "Done");
  await expect(panel.locator('[data-ck-comment="c1"]')).toHaveAttribute(
    "data-ck-status",
    "resolved",
  );
  await expect(page.locator('[data-ck-pin="c1"]')).toHaveCount(0);
});

test("hover-select outlines an element; Ask agent anchors a comment to it", async ({
  page,
  studio,
}) => {
  await open(page, studio);
  const file = join(studio.dir, "apply.json");
  await writeFile(
    file,
    JSON.stringify({ recipe: "hook-title", slots: { headline: "Hi there" } }),
  );
  await studio.ck("recipe", "apply", file);

  const root = page.locator("[data-ck-hover-root]");
  const part = root.locator("[data-ck-part]").first();
  await expect(part).toBeVisible();
  const layer = await part.evaluate(
    (el) => el.closest("[data-ck-layer]")?.getAttribute("data-ck-layer") ?? "",
  );
  await part.hover();
  // The outline names the topmost part under the pointer, inside that layer.
  const outline = page.locator("[data-ck-hover]");
  await expect(outline).toHaveAttribute("data-ck-hover", layer);
  const path = await outline.getAttribute("data-ck-hover-path");
  expect(path).toBeTruthy();

  // The studio reports what the user points at.
  await expect
    .poll(async () => {
      const ctx = (await studio.ck("context")) as {
        hovered?: { layerId: string };
      };
      return ctx.hovered?.layerId;
    }, CONTEXT_POLL)
    .toBe(layer);

  await part.click();
  const mini = page.locator(`[data-ck-mini-inspector="${layer}"]`);
  await expect(mini).toBeVisible();
  await mini.locator("[data-ck-ask-text]").fill("Use the brand blue here");
  await mini.locator("[data-ck-ask]").click();
  await expect(mini).toHaveCount(0);

  const bundle = (await studio.ck("comments", "list")) as Bundle;
  expect(bundle.comments[0]?.anchor).toEqual({
    layerId: layer,
    propPath: path,
  });
  await expect
    .poll(
      async () =>
        ((await studio.ck("context")) as { selection?: string }).selection,
      CONTEXT_POLL,
    )
    .toBe(layer);
});

test("tuning chips show their impact and re-plan only dependent layers", async ({
  page,
  studio,
}) => {
  await open(page, studio);
  const brief = join(studio.dir, "brief.json");
  await writeFile(
    brief,
    JSON.stringify({
      fields: {
        platform: "youtube",
        length: 15,
        pacing: "lively",
        look: "clean-product",
      },
      questions: [],
    }),
  );
  const proposed = (await studio.ck("brief", "propose", brief)) as {
    rev: number;
  };
  await studio.ck(
    "project",
    "patch",
    JSON.stringify({
      baseRev: proposed.rev,
      author: "human",
      ops: [{ op: "replace", path: "/brief/status", value: "confirmed" }],
    }),
  );
  const file = join(studio.dir, "apply.json");
  await writeFile(
    file,
    JSON.stringify({
      recipe: "kinetic-list",
      slots: { title: "Why", items: ["One", "Two"] },
    }),
  );
  await studio.ck("recipe", "apply", file);
  await studio.ck("handoff");
  const before = ((await studio.ck("status")) as { rev: number }).rev;

  const chips = page.locator("[data-ck-chips]");
  const punchier = chips.locator('[data-ck-chip="punchier"]');
  await expect(punchier).toBeEnabled();
  // Only kinetic-list's pacing-bound layer depends on pacing.
  const impact = (await punchier.getAttribute("data-ck-impact")) ?? "";
  expect(impact.split(" ").filter(Boolean).length).toBe(1);
  await punchier.click();

  await expect
    .poll(async () => {
      const diff = (await studio.ck(
        "brief",
        "diff",
        "--since",
        String(before),
      )) as {
        changes: { field: string; to: unknown }[];
        affected: string[];
      };
      return { changes: diff.changes, affected: diff.affected.join(" ") };
    })
    .toEqual({
      changes: [{ field: "pacing", from: "lively", to: 4, source: "user" }],
      affected: impact,
    });

  // More premium re-themes at once through the stored look.
  await chips.locator('[data-ck-chip="premium"]').click();
  await expect(chips.locator('[data-ck-chip="premium"]')).toBeDisabled();
});
