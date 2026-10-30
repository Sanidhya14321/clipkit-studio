import { test, expect } from "@playwright/test";
import { mkdir } from "node:fs/promises";
import { resolve } from "node:path";
import { WORKSPACES } from "../src/workspace/layout";
import { mockWorkspace, serveWorkspaceBuild } from "./workspace-fixture";

let server: Awaited<ReturnType<typeof serveWorkspaceBuild>>;
test.beforeAll(async () => {
  server = await serveWorkspaceBuild();
});
test.afterAll(async () => {
  await server.close();
});

test("workspace commands, docking, resizing and layout restoration", async ({
  page,
}) => {
  await mockWorkspace(page);
  await page.goto(`${server.url}/#token=synthetic-ui-test`);
  await expect(
    page.getByRole("tab", { name: "Program monitor", exact: true }),
  ).toBeVisible();
  await page.getByRole("separator", { name: "Resize left panels" }).focus();
  await page.keyboard.press("ArrowRight");
  await expect(
    page.getByRole("separator", { name: "Resize left panels" }),
  ).toHaveAttribute("aria-valuenow", "280");
  await page
    .getByRole("button", { name: "Open command palette", exact: true })
    .click();
  await page
    .getByRole("textbox", { name: "Search commands" })
    .fill("Open Comments");
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("tab", { name: "Comments", exact: true }),
  ).toHaveAttribute("aria-selected", "true");
  await page.getByRole("button", { name: "Review", exact: true }).click();
  await expect(
    page.getByRole("tab", { name: "History", exact: true }),
  ).toHaveAttribute("aria-selected", "true");
  await page.getByRole("button", { name: "Edit", exact: true }).last().click();
  await expect(
    page.getByRole("tab", { name: "Comments", exact: true }),
  ).toHaveAttribute("aria-selected", "true");
  await page.getByLabel("Comments panel options").click();
  await page.getByRole("button", { name: "Dock left", exact: true }).click();
  await expect(
    page.locator('[data-ck-zone="left"] #tab-comments'),
  ).toHaveAttribute("aria-selected", "true");
  await page.reload();
  await expect(
    page.locator('[data-ck-zone="left"] #tab-comments'),
  ).toHaveAttribute("aria-selected", "true");
  await page.getByLabel("Comments panel options").click();
  await page.getByRole("button", { name: "Close panel", exact: true }).click();
  await expect(
    page.getByRole("tab", { name: "Comments", exact: true }),
  ).toHaveCount(0);
  await page
    .getByRole("button", { name: "Open command palette", exact: true })
    .click();
  await page
    .getByRole("textbox", { name: "Search commands" })
    .fill("Open Comments");
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("tab", { name: "Comments", exact: true }),
  ).toBeVisible();
});

test("selection edits retain revisioned API requests, lock refusal and command errors", async ({
  page,
}) => {
  const fixture = await mockWorkspace(page);
  await page.goto(`${server.url}/#token=synthetic-ui-test`);
  await page.locator('[data-ck-timeline] [data-ck-layer="opening"]').click();
  const name = page.locator(
    '[data-ck-inspector="opening"] [data-ck-prop="name"]',
  );
  await name.fill("Revised opening");
  await name.press("Tab");
  await expect
    .poll(() => fixture.calls.find((c) => c.path === "/api/patch")?.body)
    .toMatchObject({
      baseRev: 1,
      layerRevs: { opening: 1 },
      author: "human",
      ops: [
        {
          op: "replace",
          path: "/tracks/0/items/0/name",
          value: "Revised opening",
        },
      ],
    });
  await expect(
    page.locator('[data-ck-timeline] [data-ck-layer="opening"]'),
  ).toContainText("Revised opening");
  fixture.project.locks = [
    {
      layerId: "opening",
      holder: "agent",
      acquiredAtMs: Date.now(),
      heartbeatAtMs: Date.now(),
      ttlMs: 60_000,
    },
  ];
  fixture.project.agent.status = "working";
  fixture.snapshot();
  await expect(name).toBeDisabled();
  await page.locator('[data-ck-timeline] [data-ck-layer="closing"]').click();
  await expect(
    page.locator('[data-ck-inspector="closing"] input').first(),
  ).toBeEnabled();
  await expect(page).toHaveTitle(/working/);
  fixture.refuse();
  await page
    .getByRole("button", { name: "Open command palette", exact: true })
    .click();
  await page.getByRole("textbox", { name: "Search commands" }).fill("Undo");
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("alert").filter({ hasText: "E_PATCH_STALE" }),
  ).toBeVisible();
  await page.getByLabel("Search project").focus();
  await page.keyboard.press("Control+z");
  expect(fixture.calls.filter((c) => c.path === "/api/undo")).toHaveLength(1);
});

test("storage fallback retains session authentication and opens a project", async ({
  page,
}) => {
  await mockWorkspace(page);
  await page.addInitScript(() => {
    Storage.prototype.setItem = () => {
      throw new DOMException("Storage unavailable", "QuotaExceededError");
    };
  });
  await page.goto(`${server.url}/#token=synthetic-ui-test`);
  await expect(
    page
      .getByRole("status")
      .filter({ hasText: "Layout storage is unavailable" }),
  ).toBeVisible();
  await expect(
    page.getByRole("tab", { name: "Program monitor", exact: true }),
  ).toBeVisible();
  await page.getByRole("tab", { name: "Comments", exact: true }).click();
  await page
    .getByRole("navigation", { name: "Workspaces" })
    .getByRole("button", { name: "Review", exact: true })
    .click();
  await page
    .getByRole("navigation", { name: "Workspaces" })
    .getByRole("button", { name: "Edit", exact: true })
    .click();
  await expect(
    page.getByRole("tab", { name: "Comments", exact: true }),
  ).toHaveAttribute("aria-selected", "true");
});

test("palette buttons keep native Enter and Space behavior", async ({
  page,
}) => {
  const fixture = await mockWorkspace(page);
  await page.goto(`${server.url}/#token=synthetic-ui-test`);
  const trigger = page.getByRole("button", {
    name: "Open command palette",
    exact: true,
  });
  await trigger.focus();
  await page.keyboard.press("Space");
  await expect(
    page.getByRole("dialog", { name: "Command palette" }),
  ).toBeVisible();
  await page.getByRole("textbox", { name: "Search commands" }).fill("Undo");
  await page.keyboard.press("Tab");
  await expect(
    page.getByRole("button", { name: "Close command palette", exact: true }),
  ).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(
    page.getByRole("dialog", { name: "Command palette" }),
  ).toHaveCount(0);
  expect(fixture.calls.filter((c) => c.path === "/api/undo")).toHaveLength(0);
});

test("a panel render failure is contained and reloadable", async ({ page }) => {
  const fixture = await mockWorkspace(page);
  await page.goto(`${server.url}/#token=synthetic-ui-test`);
  await page.getByRole("tab", { name: "Brief & styles", exact: true }).click();
  fixture.project.brief.fields = null as never;
  fixture.snapshot();
  await expect(
    page
      .getByRole("alert")
      .filter({ hasText: "Brief & styles could not render" }),
  ).toBeVisible();
  await expect(page.locator("[data-ck-timeline]")).toBeVisible();
  fixture.project.brief.fields = {};
  fixture.snapshot();
  await page.getByRole("button", { name: "Reload panel", exact: true }).click();
  await expect(page.locator("[data-ck-brief]")).toBeVisible();
});

test("draft fields survive docking and responsive layout changes; closed panels cannot blank maximize", async ({
  page,
}) => {
  await mockWorkspace(page);
  await page.goto(`${server.url}/#token=synthetic-ui-test`);
  await page.getByRole("tab", { name: "Brief & styles", exact: true }).click();
  const goal = page.locator('[data-ck-brief] [data-ck-prop="goal"]');
  await goal.fill("A draft that must survive docking");
  await page
    .getByRole("navigation", { name: "Workspaces" })
    .getByRole("button", { name: "Generate", exact: true })
    .click();
  await expect(goal).toHaveValue("A draft that must survive docking");
  await page.setViewportSize({ width: 720, height: 800 });
  await page.getByRole("tab", { name: "Brief & styles", exact: true }).click();
  await expect(goal).toHaveValue("A draft that must survive docking");
  await page.setViewportSize({ width: 1440, height: 900 });
  await expect(page.locator('[data-ck-zone="left"] #tab-brief')).toBeVisible();
  await page.getByLabel("Brief & styles panel options").click();
  await page.getByRole("button", { name: "Dock right", exact: true }).click();
  await expect(goal).toHaveValue("A draft that must survive docking");
  await goal.focus();
  await page.keyboard.press("Tab");
  await page.getByLabel("Brief & styles panel options").click();
  await page.getByRole("button", { name: "Close panel", exact: true }).click();
  await page
    .getByRole("button", { name: "Open command palette", exact: true })
    .click();
  await page
    .getByRole("textbox", { name: "Search commands" })
    .fill("Toggle focused panel maximized");
  await page.keyboard.press("Enter");
  await expect(page.getByRole("tabpanel")).toHaveCount(1);
});

test("narrow panels and visual workspace matrix", async ({ page }) => {
  const errors: string[] = [];
  page.on("pageerror", (e) => errors.push(e.message));
  await mockWorkspace(page);
  await page.goto(`${server.url}/#token=synthetic-ui-test`);
  await page.getByRole("tab", { name: "Agent", exact: true }).click();
  await page
    .getByLabel("Voice command", { exact: true })
    .fill("go to 1 seconds");
  await page.getByLabel("Voice command", { exact: true }).press("Enter");
  await expect(page.locator("[data-ck-playhead]")).toHaveText("1.000 s");
  await page.getByRole("tab", { name: "Inspector", exact: true }).click();
  const evidence = resolve(
    import.meta.dirname,
    "../../docs/updates/ui-workflow-evidence/layouts",
  );
  await mkdir(evidence, { recursive: true });
  for (const width of [1440, 1920]) {
    await page.setViewportSize({ width, height: 1080 });
    for (const theme of ["dark", "high-contrast"]) {
      if (theme === "high-contrast") {
        await page
          .getByRole("button", { name: "Open command palette", exact: true })
          .click();
        await page
          .getByRole("textbox", { name: "Search commands" })
          .fill("Use high contrast");
        await page.keyboard.press("Enter");
      }
      for (const workspace of WORKSPACES) {
        await page
          .getByRole("navigation", { name: "Workspaces" })
          .getByRole("button", { name: workspace, exact: true })
          .click();
        await expect(
          page.getByRole("tab", { name: "Program monitor", exact: true }),
        ).toBeVisible();
        await page.screenshot({
          path: `${evidence}/${workspace.toLowerCase()}-${width}-${theme}.png`,
        });
        expect(
          await page.evaluate(() => document.documentElement.scrollHeight),
        ).toBe(1080);
        expect(
          await page.evaluate(() => document.documentElement.scrollWidth),
        ).toBe(width);
      }
      if (theme === "high-contrast") {
        await page
          .getByRole("button", { name: "Open command palette", exact: true })
          .click();
        await page
          .getByRole("textbox", { name: "Search commands" })
          .fill("Use dark theme");
        await page.keyboard.press("Enter");
      }
    }
  }
  await page.setViewportSize({ width: 720, height: 800 });
  await page
    .getByRole("button", { name: "Open command palette", exact: true })
    .click();
  await page
    .getByRole("textbox", { name: "Search commands" })
    .fill("Open Project");
  await page.keyboard.press("Enter");
  await expect(
    page.locator('[data-ck-zone="right"] #tab-project'),
  ).toBeVisible();
  await expect(page.getByLabel("Search project")).toBeVisible();
  expect(errors).toEqual([]);
});

test("Escape cancels inspector edits and accepted edits survive failed refresh without retry", async ({
  page,
}) => {
  const fixture = await mockWorkspace(page);
  await page.goto(`${server.url}/#token=synthetic-ui-test`);
  await page.locator('[data-ck-timeline] [data-ck-layer="opening"]').click();
  const field = page.locator(
    '[data-ck-inspector="opening"] [data-ck-prop="name"]',
  );
  await field.fill("This must be cancelled");
  await field.press("Escape");
  expect(fixture.calls.filter((c) => c.path === "/api/patch")).toHaveLength(0);
  await expect(field).toHaveValue("Opening title");
  fixture.failRefresh();
  await field.fill("Saved once");
  await field.press("Enter");
  await expect(
    page.getByRole("alert").filter({ hasText: "Your edit was saved by ck" }),
  ).toBeVisible();
  expect(fixture.calls.filter((c) => c.path === "/api/patch")).toHaveLength(1);
  await expect(field).toHaveValue("Saved once");
});
