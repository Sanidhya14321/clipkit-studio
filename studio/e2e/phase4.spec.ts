// Phase 4 step 4: the Brief page. The agent proposes through the CLI, the studio shows each
// field with its source, the pacing slider shows its numbers, a look card is picked, and the
// confirm reaches the agent's `ck brief wait` as a human patch. While the agent holds the
// project lock the page is read-only and the server refuses the confirm.
import { writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { Page } from "@playwright/test";
import type { Project } from "../src/generated/types";
import { confirmPatch } from "../src/brief";
import { expect, test, type Studio } from "./fixture";

async function propose(studio: Studio) {
  const file = join(studio.dir, "propose.json");
  await writeFile(
    file,
    JSON.stringify({
      fields: {
        platform: "tiktok",
        length: 15,
        pacing: "punchy",
        look: "cinematic",
      },
      confidence: { platform: 0.9 },
      questions: [{ id: "price", text: "Show the price?", affects: ["cta"] }],
    }),
  );
  return studio.ck("brief", "propose", file);
}

async function open(page: Page, studio: Studio) {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
}

type Wait = {
  ready: boolean;
  brief?: { fields: Record<string, { value: unknown; source: string }> };
};

test("the user adjusts and confirms the brief; the agent's wait returns it", async ({
  page,
  studio,
}) => {
  await open(page, studio);
  await propose(studio);

  const brief = page.locator('[data-ck-brief="proposed"]');
  await expect(brief).toBeVisible();
  await expect(
    brief.locator('[data-ck-field="platform"] [data-ck-source]'),
  ).toHaveAttribute("data-ck-source", "inferred");
  await expect(
    brief.locator('[data-ck-field="goal"] [data-ck-source]'),
  ).toHaveAttribute("data-ck-source", "default");
  await expect(brief.locator('[data-ck-prop="platform"]')).toHaveValue(
    "tiktok",
  );

  // "punchy" is level 4: the slider sits there and shows the numbers it maps to.
  const slider = brief.locator('[data-ck-prop="pacing"]');
  await expect(slider).toHaveValue("4");
  await expect(brief.locator("[data-ck-pacing-numbers]")).toHaveText(
    /avg shot 1\.5 s \(0\.8 to 3 s\), 9 to 10 cuts per 15 s/,
  );
  await slider.fill("2");
  await expect(brief.locator("[data-ck-pacing-numbers]")).toHaveText(
    /steady: avg shot 3\.2 s/,
  );

  // Four look cards; the proposed one is selected, the user picks another.
  await expect(brief.locator("[data-ck-look]")).toHaveCount(4);
  await expect(brief.locator('[data-ck-look="cinematic"]')).toHaveAttribute(
    "aria-checked",
    "true",
  );
  await brief.locator('[data-ck-look="bold-social"]').click();
  await expect(
    brief.locator('[data-ck-field="look"] [data-ck-source]'),
  ).toHaveAttribute("data-ck-source", "user");

  await brief.locator('[data-ck-question="price"] input').fill("yes");
  await brief.locator("[data-ck-brief-confirm]").click();
  await expect(page.locator('[data-ck-brief="confirmed"]')).toBeVisible();

  const done = (await studio.ck("brief", "wait", "--timeout", "0")) as Wait;
  expect(done.ready).toBe(true);
  const fields = done.brief?.fields ?? {};
  expect(fields.pacing).toEqual({ value: 2, source: "user" });
  expect(fields.look).toEqual({ value: "bold-social", source: "user" });
  expect(fields.platform).toEqual({ value: "tiktok", source: "inferred" });
  await expect(page.locator("[data-ck-feed-rev]").first()).toHaveAttribute(
    "data-ck-author",
    "human",
  );
});

test("the brief is read-only while the agent holds the project", async ({
  page,
  studio,
}) => {
  await open(page, studio);
  await propose(studio);
  await studio.ck("lock", "@project");

  const brief = page.locator("[data-ck-brief]");
  await expect(brief).toHaveAttribute("data-ck-readonly", "true");
  await expect(brief.locator("[data-ck-brief-confirm]")).toBeDisabled();
  await expect(brief.locator('[data-ck-prop="pacing"]')).toBeDisabled();

  // The server refuses the studio's confirm too.
  const refused = await page.evaluate(async () => {
    const token = sessionStorage.getItem("ck-token") ?? "";
    const headers = { "x-ck-token": token, "content-type": "application/json" };
    const project = (await (
      await fetch("/api/project?view=full", { headers })
    ).json()) as unknown;
    return { project };
  });
  // Serialized here: a typed PatchRequest argument is too deep for page.evaluate's types.
  const body = JSON.stringify(confirmPatch(refused.project as Project, {}));
  const res = await page.evaluate(async (body: string) => {
    const token = sessionStorage.getItem("ck-token") ?? "";
    const r = await fetch("/api/patch", {
      method: "POST",
      headers: { "x-ck-token": token, "content-type": "application/json" },
      body,
    });
    return { status: r.status, body: (await r.json()) as { code: string } };
  }, body);
  expect(res.status).toBe(423);
  expect(res.body.code).toBe("E_LOCK_HELD");

  await studio.ck("unlock");
  await expect(brief).toHaveAttribute("data-ck-readonly", "false");
  await brief.locator("[data-ck-brief-confirm]").click();
  await expect(page.locator('[data-ck-brief="confirmed"]')).toBeVisible();
});
