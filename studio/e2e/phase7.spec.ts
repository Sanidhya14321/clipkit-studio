// Phase 7 step 3: record-along. A recorded slot shows its line on a teleprompter after a
// countdown, the fake microphone records a take, and ck turns it into the slot's active,
// recorded take. A slot with no speech offers the labeled browser placeholder.
import { writeFile } from "node:fs/promises";
import { join } from "node:path";
import type { Page } from "@playwright/test";
import { expect, test, openPanel, type Studio } from "./fixture";

async function open(page: Page, studio: Studio) {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
  await openPanel(page, "agent");
}

/** Runs a ck command that must fail; returns its error code from stderr. */
async function errorCode(studio: Studio, ...args: string[]): Promise<string> {
  try {
    await studio.ck(...args);
  } catch (e) {
    const stderr = (e as { stderr?: string }).stderr ?? "";
    return (JSON.parse(stderr) as { code: string }).code;
  }
  throw new Error(`ck ${args.join(" ")} should fail`);
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

  await openPanel(page, "voice");
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

  // Cloning is refused until the user consents in the studio; then only the provider stub
  // (no Fish Audio key in tests) stands in the way.
  expect(await errorCode(studio, "voice", "clone", "v1")).toBe("E_REQUEST");
  // A controlled checkbox: it shows checked once the server's snapshot arrives.
  await slot.locator('[data-ck-consent="t1"]').click();
  await expect(slot.locator('[data-ck-consent="t1"]')).toBeChecked();
  expect(await errorCode(studio, "voice", "clone", "v1")).toBe("E_VOICE");
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
  await openPanel(page, "voice");
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

test("push-to-talk: seek, split here, undo locally; other phrases become comments", async ({
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

  const timeline = page.locator("[data-ck-timeline]");
  await timeline.locator('[data-ck-layer="hook-title-backdrop"]').click();
  const say = async (phrase: string) => {
    const box = page.locator("[data-ck-ptt-text]");
    await box.fill(phrase);
    await box.press("Enter");
    await expect(page.locator("[data-ck-ptt-last]")).toContainText(phrase);
  };

  await say("go to 1 second");
  await say("split here");
  await expect(
    timeline.locator('[data-ck-layer="hook-title-backdrop-b"]'),
  ).toHaveCount(1);
  await say("undo");
  await expect(
    timeline.locator('[data-ck-layer="hook-title-backdrop-b"]'),
  ).toHaveCount(0);

  await say("Make the background warmer");
  const bundle = (await studio.ck("comments", "list")) as {
    comments: { author: string; text: string; anchor: { layerId?: string } }[];
  };
  expect(bundle.comments[0]).toMatchObject({
    author: "voice",
    text: "Make the background warmer",
    anchor: { layerId: "hook-title-backdrop" },
  });
});

test("ck ask shows a card; the user's answer is the tool result", async ({
  page,
  studio,
}) => {
  // Asked before the studio opens: the card still shows once it connects.
  const early = (await studio.ck(
    "ask",
    "Keep the intro?",
    "--timeout",
    "0",
  )) as {
    id: string;
    answered: boolean;
  };
  expect(early).toEqual({ id: "q1", answered: false });
  await open(page, studio);
  await expect(page.locator('[data-ck-question="q1"]')).toContainText(
    "Keep the intro?",
  );

  const asked = studio.ck(
    "ask",
    "Which logo?",
    "--option",
    "Old",
    "--option",
    "New",
    "--timeout",
    "20",
  );
  const card = page.locator('[data-ck-question="q2"]');
  await card.locator('[data-ck-ask-option="New"]').click();
  expect(await asked).toEqual({ id: "q2", answered: true, answer: "New" });
  await expect(card).toHaveCount(0);

  // A typed answer to the earlier question; the agent picks it up with --id.
  const first = page.locator('[data-ck-question="q1"]');
  await first.locator("[data-ck-ask-answer]").fill("Yes, shorter");
  await first.locator("[data-ck-ask-answer]").press("Enter");
  await expect(first).toHaveCount(0);
  expect(await studio.ck("ask", "--id", "q1", "--timeout", "0")).toEqual({
    id: "q1",
    answered: true,
    answer: "Yes, shorter",
  });
});
