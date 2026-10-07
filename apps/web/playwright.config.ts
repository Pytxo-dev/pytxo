import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  workers: 1,
  reporter: "line",
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL || "http://127.0.0.1:3101",
    storageState: process.env.PLAYWRIGHT_STORAGE_STATE || undefined,
    channel: process.env.PLAYWRIGHT_CHANNEL || "chromium",
    trace: "retain-on-failure",
  },
  webServer: process.env.PLAYWRIGHT_BASE_URL ? undefined : {
    command: "npx next start -H 0.0.0.0 -p 3101",
    url: "http://127.0.0.1:3101",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
    // Keep Clerk middleware off unless an e2e run explicitly provides a secret.
    // Local `.env.local` keys would otherwise handshake to an unresolved Clerk host.
    env: {
      ...process.env,
      CLERK_SECRET_KEY: process.env.PLAYWRIGHT_CLERK_SECRET_KEY ?? "",
    },
  },
});
