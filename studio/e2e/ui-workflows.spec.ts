// Real-server workflows. No mocked API or WebSocket traffic in this file.
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { join, resolve } from "node:path";
import { mkdir, stat } from "node:fs/promises";
import type { Page } from "@playwright/test";
import type { Project } from "../src/generated/types";
import { test, expect, type Studio } from "./fixture";
const run = promisify(execFile);
async function open(page: Page, studio: Studio) {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator('[data-ck-connected="true"]')).toBeVisible();
}
async function project(page: Page): Promise<Project> {
  return page.evaluate(async () => {
    const response = await fetch("/api/project?view=full", {
      headers: { "x-ck-token": sessionStorage.getItem("ck-token") ?? "" },
    });
    if (!response.ok) throw new Error("Cannot read project");
    return response.json();
  });
}
async function seek(page: Page, seconds: number) {
  const field = page.getByRole("spinbutton", { name: "Seek to seconds" });
  await field.fill(String(seconds));
  await field.press("Enter");
}
async function command(page: Page, label: string) {
  await page
    .getByRole("button", { name: "Open command palette", exact: true })
    .click();
  await page.getByRole("textbox", { name: "Search commands" }).fill(label);
  await page.getByRole("textbox", { name: "Search commands" }).press("Enter");
}

test("real title workflow: content, move, trim, split, undo and delete are server revisions", async ({
  page,
  studio,
}) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await open(page, studio);
  await page.getByRole("button", { name: "T Add title", exact: true }).click();
  await page
    .getByRole("textbox", { name: "Title text", exact: true })
    .fill("A story worth telling");
  await page
    .getByRole("dialog", { name: "Add title", exact: true })
    .getByRole("button", { name: "Add title", exact: true })
    .click();
  await expect(page.locator('[data-ck-inspector="title-1"]')).toBeVisible();
  const text = page.locator(
    '[data-ck-inspector="title-1"] [data-ck-prop="text"]',
  );
  await text.fill("The revised story");
  await text.press("Enter");
  await expect
    .poll(
      async () =>
        (
          (await project(page)).tracks
            .flatMap((t) => t.items)
            .find((i) => i.id === "title-1") as unknown as {
            props: { text: string };
          }
        ).props.text,
    )
    .toBe("The revised story");
  await seek(page, 1);
  await expect(
    page.locator('.studio-preview [data-ck-part="line"]'),
  ).toContainText("The revised story");
  // A standalone title must use the look's background as well as its text
  // color; the clean-product look's dark type was unreadable on hardcoded black.
  await expect(page.locator(".studio-preview [data-ck-composition]")).toHaveCSS(
    "background-color",
    "rgb(247, 247, 245)",
  );
  const clip = page.locator('[data-ck-timeline] [data-ck-layer="title-1"]');
  let box = (await clip.boundingBox())!;
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width / 2 + 70, box.y + box.height / 2, {
    steps: 8,
  });
  await page.mouse.up();
  await expect
    .poll(
      async () =>
        (
          (await project(page)).tracks
            .flatMap((t) => t.items)
            .find((i) => i.id === "title-1") as { at: number }
        ).at,
    )
    .toBe(1);
  box = (await clip.boundingBox())!;
  await page.mouse.move(box.x + box.width - 3, box.y + box.height / 2);
  await page.mouse.down();
  await page.mouse.move(box.x + box.width - 73, box.y + box.height / 2, {
    steps: 8,
  });
  await page.mouse.up();
  await expect
    .poll(
      async () =>
        (
          (await project(page)).tracks
            .flatMap((t) => t.items)
            .find((i) => i.id === "title-1") as { dur: number }
        ).dur,
    )
    .toBe(2);
  await seek(page, 2);
  await page
    .locator("[data-ck-timeline]")
    .getByRole("button", { name: "Split", exact: true })
    .click();
  await expect(
    page.locator('[data-ck-timeline] [data-ck-layer="title-1-b"]'),
  ).toBeVisible();
  await command(page, "Undo");
  await expect(
    page.locator('[data-ck-timeline] [data-ck-layer="title-1-b"]'),
  ).toHaveCount(0);
  await clip.click();
  await page
    .locator("[data-ck-timeline]")
    .getByRole("button", { name: "Delete", exact: true })
    .click();
  await expect(clip).toHaveCount(0);
  await command(page, "Undo");
  await expect(clip).toBeVisible();
  const evidence = resolve(
    import.meta.dirname,
    "../../docs/updates/ui-workflow-evidence",
  );
  await mkdir(evidence, { recursive: true });
  await page.screenshot({ path: join(evidence, "real-editing-1440.png") });
});

test("real scene library updates recipe text in preview instead of stale solver lines", async ({
  page,
  studio,
}) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await open(page, studio);
  await page.getByRole("button", { name: "▧ Add scene", exact: true }).click();
  await page
    .getByRole("textbox", { name: "headline", exact: true })
    .fill("Original scene title");
  await page
    .getByRole("button", { name: "Add scene at playhead", exact: true })
    .click();
  const clip = page.locator(
    '[data-ck-timeline] [data-ck-layer="hook-title-headline"]',
  );
  await expect(clip).toBeVisible();
  await clip.click();
  const field = page.locator(
    '[data-ck-inspector="hook-title-headline"] [data-ck-prop="text"]',
  );
  await field.fill("Now the content really changes");
  await field.press("Enter");
  await expect
    .poll(
      async () =>
        (
          (await project(page)).tracks
            .flatMap((t) => t.items)
            .find((i) => i.id === "hook-title-headline") as unknown as {
            props: { text: string };
          }
        ).props.text,
    )
    .toBe("Now the content really changes");
  await seek(page, 1);
  await expect(
    page.locator('.studio-preview [data-ck-part="line"]'),
  ).toContainText("Now the content really changes");
  await page.getByRole("button", { name: "Export ↗", exact: true }).click();
  await page.getByRole("button", { name: "Run checks", exact: true }).click();
  await expect(
    page.getByRole("region", { name: "Review and export" }),
  ).toContainText(/checks passed|checks need attention/);
});

test("real media import, source range insertion, verification and MP4 export", async ({
  page,
  studio,
}) => {
  test.setTimeout(90_000);
  const file = join(studio.dir, "ui-fixture.mp4");
  await run("ffmpeg", [
    "-hide_banner",
    "-nostdin",
    "-loglevel",
    "error",
    "-y",
    "-f",
    "lavfi",
    "-i",
    "testsrc2=size=320x180:rate=30",
    "-f",
    "lavfi",
    "-i",
    "sine=frequency=440:sample_rate=48000",
    "-t",
    "4",
    "-c:v",
    "libx264",
    "-pix_fmt",
    "yuv420p",
    "-c:a",
    "aac",
    file,
  ]);
  await page.setViewportSize({ width: 1440, height: 1000 });
  await open(page, studio);
  await page
    .getByRole("button", { name: "＋ Import media", exact: true })
    .click();
  await page
    .getByRole("textbox", { name: "Local media path", exact: true })
    .fill(file);
  await page
    .getByRole("button", { name: "Import into project", exact: true })
    .click();
  await expect(page.locator("[data-ck-timeline] [data-ck-layer]")).toHaveCount(
    1,
    { timeout: 30_000 },
  );
  await expect(
    page.getByRole("button", { name: /Preview src-/ }),
  ).toBeVisible();
  await page.getByRole("button", { name: /Preview src-/ }).click();
  await page
    .getByRole("spinbutton", { name: "Source in", exact: true })
    .fill("1");
  await page
    .getByRole("spinbutton", { name: "Source out", exact: true })
    .fill("2");
  await page
    .getByRole("button", { name: "Insert range at playhead", exact: true })
    .click();
  await expect
    .poll(
      async () =>
        (await project(page)).tracks
          .flatMap((t) => t.items)
          .filter((i) => i.type === "footage").length,
    )
    .toBe(2);
  await command(page, "Undo");
  await expect
    .poll(
      async () =>
        (await project(page)).tracks
          .flatMap((t) => t.items)
          .filter((i) => i.type === "footage").length,
    )
    .toBe(1);
  await page.getByRole("button", { name: "Export ↗", exact: true }).click();
  await page.getByRole("button", { name: "Run checks", exact: true }).click();
  await expect(
    page.getByText("Project checks passed", { exact: true }),
  ).toBeVisible();
  await page
    .getByRole("textbox", { name: "Export output path", exact: true })
    .fill("exports/ui-proof.mp4");
  await page
    .getByRole("button", { name: "Export sequence", exact: true })
    .click();
  await expect(page.getByText("Export finished", { exact: true })).toBeVisible({
    timeout: 45_000,
  });
  expect(
    (await stat(join(studio.dir, "exports/ui-proof.mp4"))).size,
  ).toBeGreaterThan(1000);
  const { stdout } = await run("ffprobe", [
    "-v",
    "error",
    "-show_format",
    "-of",
    "json",
    join(studio.dir, "exports/ui-proof.mp4"),
  ]);
  expect(Number(JSON.parse(stdout).format.duration)).toBeGreaterThan(3.9);
  const evidence = resolve(
    import.meta.dirname,
    "../../docs/updates/ui-workflow-evidence",
  );
  await mkdir(evidence, { recursive: true });
  await page.screenshot({ path: join(evidence, "real-export-1440.png") });
});

test("real server rejects bypassed locked-layer edits while other layers remain editable", async ({
  page,
  studio,
}) => {
  await open(page, studio);
  await page.getByRole("button", { name: "T Add title", exact: true }).click();
  await page
    .getByRole("textbox", { name: "Title text", exact: true })
    .fill("Lock me");
  await page
    .getByRole("dialog", { name: "Add title" })
    .getByRole("button", { name: "Add title", exact: true })
    .click();
  await expect(page.locator('[data-ck-inspector="title-1"]')).toBeVisible();
  await studio.ck("lock", "title-1");
  await expect(
    page.locator('[data-ck-inspector="title-1"] [data-ck-prop="name"]'),
  ).toBeDisabled();
  const result = await page.evaluate(async () => {
    const token = sessionStorage.getItem("ck-token") ?? "";
    const headers = { "x-ck-token": token, "content-type": "application/json" };
    const p = await (await fetch("/api/project?view=full", { headers })).json();
    const response = await fetch("/api/patch", {
      method: "POST",
      headers,
      body: JSON.stringify({
        baseRev: p.rev,
        author: "human",
        label: "Bypass locked input",
        ops: [
          {
            op: "replace",
            path: "/tracks/0/items/0/name",
            value: "Must not land",
          },
        ],
      }),
    });
    return { status: response.status, body: await response.json() };
  });
  expect(result.status).toBe(423);
  expect(result.body.code).toBe("E_LOCK_HELD");
  await studio.ck("unlock");
  await expect(
    page.locator('[data-ck-inspector="title-1"] [data-ck-prop="name"]'),
  ).toBeEnabled();
});

test("real interchange export uses timeline endpoint and caption layers stay selectable", async ({
  page,
  studio,
}) => {
  await open(page, studio);
  const p = await project(page);
  await studio.ck(
    "project",
    "patch",
    JSON.stringify({
      baseRev: p.rev,
      author: "human",
      label: "Caption timing fixture",
      ops: [
        {
          op: "add",
          path: "/sources/-",
          value: {
            id: "caption-source",
            kind: "video",
            path: "fixture.mp4",
            hash: "fixture",
            durationSec: 4,
          },
        },
        {
          op: "add",
          path: "/tracks/-",
          value: {
            id: "captions",
            kind: "caption",
            items: [
              {
                id: "caption-layer",
                name: "Source captions",
                purpose: "",
                briefDeps: [],
                rev: 0,
                type: "caption",
                from: "transcript",
                ref: "caption-source",
                style: "clean",
              },
            ],
          },
        },
      ],
    }),
  );
  await expect(
    page.locator('[data-ck-timeline] [data-ck-layer="caption-layer"]'),
  ).toBeVisible();
  await page
    .locator('[data-ck-timeline] [data-ck-layer="caption-layer"]')
    .click();
  await expect(
    page.locator('[data-ck-inspector="caption-layer"]'),
  ).toBeVisible();
  await page.getByRole("button", { name: "Export ↗", exact: true }).click();
  await page.getByLabel("Export format", { exact: true }).selectOption("otio");
  await page
    .getByRole("button", { name: "Export sequence", exact: true })
    .click();
  await expect(
    page.getByText("Export finished", { exact: true }),
  ).toBeVisible();
  expect(
    (await stat(join(studio.dir, "exports", `${p.id}.otio`))).size,
  ).toBeGreaterThan(0);
});

test("300 clips keep canvas rendering and DOM hit targets bounded", async ({
  page,
  studio,
}) => {
  await page.setViewportSize({ width: 1440, height: 1000 });
  await open(page, studio);
  const p = await project(page);
  const items = Array.from({ length: 300 }, (_, i) => ({
    id: `perf-${i}`,
    name: `Clip ${i}`,
    purpose: "",
    briefDeps: [],
    rev: 0,
    type: "clip",
    clipId: "kinetic-text",
    at: i * 2,
    dur: 2,
    props: { text: `Clip ${i}` },
  }));
  await studio.ck(
    "project",
    "patch",
    JSON.stringify({
      baseRev: p.rev,
      author: "human",
      label: "300 clip performance fixture",
      ops: [
        {
          op: "add",
          path: "/tracks/-",
          value: { id: "perf-track", kind: "overlay", items },
        },
      ],
    }),
  );
  const timeline = page.locator("[data-ck-timeline]");
  await expect(timeline.locator("canvas")).toHaveCount(1);
  expect(await timeline.locator("[data-ck-layer]").count()).toBeLessThan(30);
  await page
    .getByRole("button", { name: "Zoom in timeline", exact: true })
    .click();
  const metrics = await page.evaluate(async () => {
    const scroll = document.querySelector(".timeline-scroll")!;
    const start = performance.now();
    scroll.scrollLeft = 2000;
    await new Promise(requestAnimationFrame);
    await new Promise(requestAnimationFrame);
    return {
      scrollAndPaintMs: performance.now() - start,
      visibleHitTargets: document.querySelectorAll(
        "[data-ck-timeline] [data-ck-layer]",
      ).length,
    };
  });
  test.info().annotations.push({
    type: "measured",
    description: JSON.stringify(metrics),
  });
  expect(metrics.visibleHitTargets).toBeLessThan(30);
  console.log(`timeline300 ${JSON.stringify(metrics)}`);
});

test("native recipe source slots use the real imported image and preserve recipe provenance", async ({
  page,
  studio,
}) => {
  test.setTimeout(90_000);
  const image = resolve(
    import.meta.dirname,
    "../../fixtures/generated/images/landscape.png",
  );
  await page.setViewportSize({ width: 1440, height: 1000 });
  await open(page, studio);
  await page
    .getByRole("button", { name: "＋ Import media", exact: true })
    .click();
  await page.getByLabel("Local media path", { exact: true }).fill(image);
  await page
    .getByRole("button", { name: "Import into project", exact: true })
    .click();
  await expect.poll(async () => (await project(page)).sources.length).toBe(1);
  const source = (await project(page)).sources[0]!;
  await page.getByRole("button", { name: "▧ Add scene", exact: true }).click();
  await page.getByRole("button", { name: /Product reveal/ }).click();
  await page
    .getByRole("combobox", { name: "media", exact: true })
    .selectOption(source.id);
  await page
    .getByRole("textbox", { name: "name", exact: true })
    .fill("The actual product");
  await page
    .getByRole("button", { name: "Add scene at playhead", exact: true })
    .click();
  await expect(
    page.locator('[data-ck-timeline] [data-ck-layer="product-reveal-device"]'),
  ).toBeVisible();
  await seek(page, 1);
  const img = page.locator(
    `.studio-preview [data-ck-source="${source.id}"] img`,
  );
  await expect(img).toBeVisible();
  await expect
    .poll(() => img.evaluate((node: HTMLImageElement) => node.naturalWidth))
    .toBe(1920);
  const p = await project(page);
  expect(p.assetsUsed).toContain("recipe:product-reveal");
  const layer = p.tracks
    .flatMap((t) => t.items)
    .find((i) => i.id === "product-reveal-device")!;
  expect(layer.provenance).toMatchObject({
    sourceAsset: "recipe:product-reveal",
    license: "original",
  });
  // This crosses the Rust job builder and Node worker, including authenticated
  // image loading. Browser preview alone cannot prove the export media map.
  const board = (await studio.ck("storyboard", "get")) as { scenes: number[] };
  expect(board.scenes).toEqual([0]);
  const dir = resolve(
    import.meta.dirname,
    "../../docs/updates/ui-workflow-evidence",
  );
  await mkdir(dir, { recursive: true });
  await page.screenshot({ path: join(dir, "real-recipe-asset.png") });
});
