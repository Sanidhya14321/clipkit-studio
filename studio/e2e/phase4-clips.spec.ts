// Phase 4 step 7: recipe layers render through the native core clips in the studio Player.
// The agent applies a recipe with the CLI; the preview shows each clip, instrumented with
// `data-ck-layer`, `data-ck-clip` and `data-ck-part`, placed by the solver's `box`.
import { readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { expect, test } from "./fixture";

function hexToRgb(hex: string): string {
  const n = parseInt(hex.slice(1), 16);
  return `rgb(${(n >> 16) & 255}, ${(n >> 8) & 255}, ${n & 255})`;
}

test("an applied recipe renders through its core clips in the Player", async ({
  page,
  studio,
}) => {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();

  const file = join(studio.dir, "apply.json");
  await writeFile(
    file,
    JSON.stringify({
      recipe: "hook-title",
      slots: { headline: "Edit video by talking", kicker: "New" },
    }),
  );
  const applied = (await studio.ck("recipe", "apply", file)) as {
    layers: string[];
  };
  expect(applied.layers).toEqual([
    "hook-title-backdrop",
    "hook-title-headline",
  ]);

  const preview = page.locator(".studio-preview");
  const backdrop = preview.locator('[data-ck-layer="hook-title-backdrop"]');
  await expect(backdrop).toHaveAttribute("data-ck-clip", "backdrop");
  // Token refs resolve against the default look (clean-product background).
  const look = JSON.parse(
    await readFile(
      join(
        import.meta.dirname,
        "..",
        "..",
        "assets",
        "looks",
        "clean-product.json",
      ),
      "utf8",
    ),
  ) as { colors: { background: string } };
  const fill = backdrop.locator('[data-ck-part="fill"]');
  await expect(fill).toBeVisible();
  const bg = await fill.evaluate((el) => getComputedStyle(el).backgroundColor);
  expect(bg).toBe(hexToRgb(look.colors.background));

  // The headline enters 0.1 s into the scene; the Player sits at frame 0, so play it. (A click
  // on the canvas picks an element for hover-select since Phase 6, so use the play control.)
  await preview.getByRole("button", { name: "Toggle playback" }).click();
  const headline = preview.locator('[data-ck-layer="hook-title-headline"]');
  await expect(headline).toHaveAttribute("data-ck-clip", "kinetic-text");
  await expect(headline.locator('[data-ck-part="kicker"]')).toContainText(
    "New",
  );
  await expect(
    headline.locator('[data-ck-part="line"]').first(),
  ).toBeAttached();
  // The solver's `lines` are drawn one part each.
  expect(
    (await headline.locator('[data-ck-part="line"]').allTextContents()).join(
      " ",
    ),
  ).toBe("Edit video by talking");

  // The text sits in the solver's box (inside the frame, not full-bleed).
  const frame = await backdrop.boundingBox();
  const text = await headline
    .locator('[data-ck-part="line"]')
    .first()
    .boundingBox();
  expect(frame && text).toBeTruthy();
  if (frame && text) {
    expect(text.x).toBeGreaterThanOrEqual(frame.x);
    expect(text.x + text.width).toBeLessThanOrEqual(frame.x + frame.width + 1);
    expect(text.width).toBeLessThan(frame.width);
  }
});
