import { test, expect, openPanel } from "./fixture";
import { resolve, join } from "node:path";
import { readFile, stat, mkdir } from "node:fs/promises";
import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { createHash } from "node:crypto";
const run = promisify(execFile);

test("unsupported derived components are refused by export even with a gate override", async ({
  page,
  studio,
}) => {
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
  const result = await page.evaluate(async () => {
    const headers = {
      "x-ck-token": sessionStorage.getItem("ck-token") ?? "",
      "content-type": "application/json",
    };
    const response = await fetch("/api/recipe/apply", {
      method: "POST",
      headers,
      body: JSON.stringify({
        recipe: "hook-title",
        slots: { headline: "Refusal proof" },
      }),
    });
    if (!response.ok) throw new Error(JSON.stringify(await response.json()));
    const p = await (await fetch("/api/project?view=full", { headers })).json();
    const ti = p.tracks.findIndex((t: { items: { id: string }[] }) =>
      t.items.some((i) => i.id === "hook-title-headline"),
    );
    const patch = await fetch("/api/patch", {
      method: "POST",
      headers,
      body: JSON.stringify({
        baseRev: p.rev,
        author: "human",
        label: "Unsupported adapter fixture",
        ops: [
          {
            op: "add",
            path: "/derived/-",
            value: {
              id: "unknown",
              sourceAsset: "unverified/component",
              nativeAdapter: "external.js",
            },
          },
          {
            op: "add",
            path: `/tracks/${ti}/items/0/derivedId`,
            value: "unknown",
          },
        ],
      }),
    });
    if (!patch.ok) throw new Error(JSON.stringify(await patch.json()));
    const exported = await fetch("/api/export", {
      method: "POST",
      headers,
      body: JSON.stringify({
        format: "mp4",
        preset: "source",
        forceReason: "Refusal proof: attempt a gate override",
      }),
    });
    return { status: exported.status, body: await exported.json() };
  });
  expect(result.status).toBeGreaterThanOrEqual(400);
  expect(result.body.code).toBe("E_RENDER");
  expect(result.body.message).toContain("no registered renderer");
  await page.getByRole("button", { name: "Export ↗", exact: true }).click();
  await expect(
    page.getByRole("button", { name: "Export sequence", exact: true }),
  ).toBeDisabled();
});

test("licensed SFX imports, plays in source monitor, analyzes, renders and exports attribution", async ({
  page,
  studio,
}) => {
  test.setTimeout(90_000);
  const id = "sfx-elements-audio/whoosh-c2efd9d902a5";
  const implementationHash = createHash("sha256")
    .update(
      await readFile(resolve(import.meta.dirname, "../src/clips/clips.tsx")),
    )
    .digest("hex");
  const file = resolve(
    import.meta.dirname,
    "../../assets",
    id,
    "media/whoosh.mp3",
  );
  await page.setViewportSize({ width: 1440, height: 1000 });
  await page.goto(`${studio.url}#token=${studio.token}`);
  await expect(page.locator("[data-ck-connected=true]")).toBeVisible();
  await page
    .getByRole("button", { name: "＋ Import media", exact: true })
    .click();
  await page.getByLabel("Local media path", { exact: true }).fill(file);
  await page
    .getByRole("button", { name: "Import into project", exact: true })
    .click();
  await page.getByRole("button", { name: /Preview src-/ }).click();
  const audio = page
    .getByRole("region", { name: "Source monitor" })
    .locator("audio");
  await expect(audio).toBeVisible();
  await expect
    .poll(() => audio.evaluate((el: HTMLAudioElement) => el.readyState))
    .toBeGreaterThan(0);
  await audio.evaluate(async (el: HTMLAudioElement) => {
    await el.play();
  });
  await expect
    .poll(() => audio.evaluate((el: HTMLAudioElement) => el.currentTime))
    .toBeGreaterThan(0);
  await audio.evaluate((el: HTMLAudioElement) => el.pause());
  await page
    .getByRole("button", { name: "Analyze rhythm", exact: true })
    .click();
  await expect(
    page.getByText(/Timing candidates require confirmation/),
  ).toBeVisible();
  await page
    .getByRole("spinbutton", { name: "Source out", exact: true })
    .fill("0.5");
  await page
    .getByRole("button", { name: "Insert range at playhead", exact: true })
    .click();
  await expect(page.locator("[data-ck-timeline] [data-ck-layer]")).toHaveCount(
    1,
  );
  // Native recipe is executable; agent records the actual audio asset and its
  // trim/gain adaptation before asking the existing export path to mix it.
  await page.evaluate(async () => {
    const headers = {
      "x-ck-token": sessionStorage.getItem("ck-token") ?? "",
      "content-type": "application/json",
    };
    const apply = await fetch("/api/recipe/apply", {
      method: "POST",
      headers,
      body: JSON.stringify({
        recipe: "hook-title",
        at: 0,
        duration: 3,
        slots: { headline: "Hear the difference" },
      }),
    });
    if (!apply.ok) throw new Error(JSON.stringify(await apply.json()));
  });
  const derived = (await studio.ck(
    "assets",
    "derive",
    "rendercomp-free/glitch-text",
    "--as",
    "ui-glitch",
  )) as { nativeAdapter: string };
  expect(derived.nativeAdapter).toBe("glitch-text-v1");
  const response = await page.evaluate(
    async ({ id, implementationHash }) => {
      const headers = {
        "x-ck-token": sessionStorage.getItem("ck-token") ?? "",
        "content-type": "application/json",
      };
      const p = await (
        await fetch("/api/project?view=full", { headers })
      ).json();
      const ti = p.tracks.findIndex(
        (t: { kind: string }) => t.kind === "audio",
      );
      const source = p.sources[0];
      const titleTrack = p.tracks.findIndex((t: { items: { id: string }[] }) =>
        t.items.some((i) => i.id === "hook-title-headline"),
      );
      const titleIndex = p.tracks[titleTrack].items.findIndex(
        (i: { id: string }) => i.id === "hook-title-headline",
      );
      const patch = await fetch("/api/patch", {
        method: "POST",
        headers,
        body: JSON.stringify({
          baseRev: p.rev,
          author: "human",
          label: "Adapt CC0 whoosh: trim and gain for title accent",
          ops: [
            {
              op: "add",
              path: `/tracks/${titleTrack}/items/${titleIndex}/derivedId`,
              value: "ui-glitch",
            },
            {
              op: "replace",
              path: `/tracks/${titleTrack}/items/${titleIndex}/clipId`,
              value: "glitch-text",
            },
            {
              op: "add",
              path: `/tracks/${titleTrack}/items/${titleIndex}/provenance`,
              value: {
                sourceAsset: "rendercomp-free/glitch-text",
                hash: implementationHash,
                mode: "native-reimplementation",
                adaptations: [
                  "brief headline",
                  "native frame-driven adaptation",
                  "three-second timing",
                ],
                license: "MIT",
              },
            },
            { op: "add", path: "/assetsUsed/-", value: id },
            {
              op: "add",
              path: `/tracks/${ti}/items/0/provenance`,
              value: {
                sourceAsset: id,
                hash: source.hash,
                mode: "direct",
                adaptations: ["trim to 0.5s", "gain 0.25"],
                license: "CC0-1.0",
              },
            },
            {
              op: "replace",
              path: `/tracks/${ti}/items/0/volume`,
              value: 0.25,
            },
          ],
        }),
      });
      return patch.status;
    },
    { id, implementationHash },
  );
  expect(response).toBe(200);
  await openPanel(page, "program");
  const seek = page.getByRole("spinbutton", { name: "Seek to seconds" });
  await seek.fill("1");
  await seek.press("Enter");
  await expect(
    page.locator(
      '.studio-preview [data-ck-clip="glitch-text"] [data-ck-part="line"]',
    ),
  ).toContainText("Hear the difference");
  const checked = (await studio.ck(
    "render",
    "check",
    "--frame",
    "16",
    "--frame",
    "17",
    "--frame",
    "45",
  )) as {
    deterministic: boolean;
  };
  expect(checked.deterministic).toBe(true);
  await page.getByRole("button", { name: "Export ↗", exact: true }).click();
  await page
    .getByLabel("Export output path", { exact: true })
    .fill("exports/licensed-sfx.mp4");
  const exported = page.waitForResponse(
    (r) => r.url().endsWith("/api/export") && r.request().method() === "POST",
  );
  await page
    .getByRole("button", { name: "Export sequence", exact: true })
    .click();
  const result = await (await exported).json();
  await expect(page.getByText("Export finished", { exact: true })).toBeVisible({
    timeout: 60_000,
  });
  expect(
    (await stat(join(studio.dir, "exports/licensed-sfx.mp4"))).size,
  ).toBeGreaterThan(1000);
  expect(result.licenseReport).toBeTruthy();
  const report = JSON.parse(
    await readFile(join(studio.dir, result.licenseReport), "utf8"),
  );
  expect(report.assets).toEqual(
    expect.arrayContaining([
      expect.objectContaining({ id, spdx: "CC0-1.0", allowed: true }),
    ]),
  );
  const { stdout } = await run("ffprobe", [
    "-v",
    "error",
    "-show_streams",
    "-of",
    "json",
    join(studio.dir, "exports/licensed-sfx.mp4"),
  ]);
  expect(
    JSON.parse(stdout).streams.some(
      (s: { codec_type: string }) => s.codec_type === "audio",
    ),
  ).toBe(true);
  const evidence = resolve(
    import.meta.dirname,
    "../../docs/updates/u6-evidence",
  );
  await mkdir(evidence, { recursive: true });
  await page.screenshot({ path: join(evidence, "licensed-sfx-export.png") });
});
