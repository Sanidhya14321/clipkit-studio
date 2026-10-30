// Frontend-only contracts; real ck integration remains in playwright.config.ts.
import { defineConfig } from "@playwright/test";
export default defineConfig({
  testDir: "e2e",
  testMatch: "ui-workspace.spec.ts",
  timeout: 60_000,
  workers: 1,
  reporter: "list",
  use: { channel: "chrome", trace: "retain-on-failure" },
});
