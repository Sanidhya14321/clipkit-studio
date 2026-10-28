// Browser end-to-end tests (spec 10.1). Each test starts its own `ck studio` on a fresh project
// folder (see e2e/fixture.ts), so tests run in parallel. `just e2e` builds ck and the studio first.
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "e2e",
  forbidOnly: !!process.env.CI,
  retries: 0,
  reporter: process.env.CI ? "github" : "list",
  timeout: 30_000,
  use: { trace: "retain-on-failure" },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
});
