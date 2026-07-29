import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  reporter: "line",
  use: {
    baseURL: "http://localhost:3101",
    channel: process.env.PLAYWRIGHT_CHANNEL || "chromium",
    trace: "retain-on-failure",
  },
  webServer: {
    command: "npm run dev -- -H 0.0.0.0 -p 3101",
    url: "http://localhost:3101",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
