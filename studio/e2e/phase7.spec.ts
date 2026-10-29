// Phase 7 step 3: record-along. A recorded slot shows its line on a teleprompter after a
// countdown, the fake microphone records a take, and ck turns it into the slot's active,
// recorded take. A slot with no speech offers the labeled browser placeholder.
import { writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { Page } from "@playwright/test";
import { expect, test, type Studio } from "./fixture";

async function open(page: Page, studio: Studio) {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
}

async function script(studio: Studio, mode: string, lines: string[]) {
  const file = join(studio.dir, "script.json");
  await writeFile(
    file,
    JSON.stringify({ lines: lines.map((text) => ({ text })), mode }),
  );
  return studio.ck("voice", "script", file);
}

test("record-along: countdown, teleprompter, take", async ({
  page,
  studio,
}) => {
  await open(page, studio);
  await script(studio, "recorded", ["Welcome to the studio.", "Second line."]);

  const slot = page.locator('[data-ck-slot="v1"]');
  await expect(slot.locator("[data-ck-mode]")).toHaveValue("recorded");
  await slot.locator('[data-ck-record="v1"]').click();

  const prompter = page.locator('[data-ck-teleprompter="v1"]');
  await expect(prompter).toContainText("Welcome to the studio.");
  await expect(prompter).toHaveAttribute("data-ck-phase", "countdown");
  await expect(prompter).toHaveAttribute("data-ck-phase", "recording", {
    timeout: 6000,
  });
  await page.waitForTimeout(1500);
  await prompter.locator("[data-ck-stop]").click();

  await expect(slot.locator('[data-ck-take="t1"]')).toContainText(
    "(recorded)",
    {
      timeout: 15000,
    },
  );
  await expect(slot).toHaveAttribute("data-ck-slot-status", "ready");
  await expect(slot.locator("[data-ck-fit]")).toBeVisible();
});

test("a slot without speech offers the labeled placeholder", async ({
  page,
  studio,
}) => {
  await open(page, studio);
  await script(studio, "ai", ["Hello there."]);
  const spoken = (await studio.ck("voice", "speak", "v1")) as {
    placeholder: boolean;
  };
  expect(spoken.placeholder).toBe(true);
  const slot = page.locator('[data-ck-slot="v1"]');
  await expect(slot).toHaveAttribute("data-ck-slot-status", "failed");
  await expect(slot.locator('[data-ck-placeholder="v1"]')).toContainText(
    "placeholder",
  );

  // The user switches the slot to recorded from its badge.
  await slot.locator("[data-ck-mode]").selectOption("recorded");
  await expect(slot.locator("[data-ck-mode]")).toHaveAttribute(
    "data-ck-mode",
    "recorded",
  );
});
