import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  testIgnore: "**/*.dev.spec.ts",
  workers: 4,
  use: {
    baseURL: "http://127.0.0.1:5173",
    channel: process.env.PLAYWRIGHT_CHANNEL || "chrome",
  },
  webServer: {
    command: "npm run preview:e2e",
    url: "http://127.0.0.1:5173",
    reuseExistingServer: false,
    timeout: 180_000,
  },
});
