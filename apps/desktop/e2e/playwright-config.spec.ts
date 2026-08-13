import { expect, test } from "@playwright/test";
import { readFileSync } from "node:fs";
import config from "../playwright.config";

test("the release E2E gate owns a production-preview server", () => {
  expect(config.workers).toBe(4);
  expect(config.testIgnore).toBe("**/*.dev.spec.ts");
  expect(config.webServer).toMatchObject({
    command: "npm run preview:e2e",
    reuseExistingServer: false,
    timeout: 180_000,
  });
});

test("Flow follows the shell snapshot and native dispatch emits a domain event", () => {
  const flow = readFileSync(new URL("../src/components/desktop2/FlowScreen.svelte", import.meta.url), "utf8");
  const missions = readFileSync(new URL("../src/components/desktop2/MissionsScreen.svelte", import.meta.url), "utf8");
  const flowIpc = readFileSync(new URL("../src-tauri/src/ipc_flow.rs", import.meta.url), "utf8");

  expect(flow).not.toContain("backend.loadSnapshot");
  expect(flow).not.toContain("watchRun(");
  expect(missions).toContain("RunReviewScreen");
  expect(flowIpc).toContain("emit_domain_changed(&app");
});

test("mission tabs expose Plan Live Review panes", () => {
  const missionDetail = readFileSync(
    new URL("../src/components/desktop2/MissionDetail.svelte", import.meta.url),
    "utf8",
  );

  expect(missionDetail).toContain('role="tablist"');
  expect(missionDetail).toContain("Plan");
  expect(missionDetail).toContain("Live");
  expect(missionDetail).toContain("Review");
});

test("Desktop and Storybook share a checked-in Chrome channel contract", () => {
  const playwrightConfig = readFileSync(
    new URL("../playwright.config.ts", import.meta.url),
    "utf8",
  );
  const storybookConfig = readFileSync(
    new URL("../.storybook/test-runner-jest.config.cjs", import.meta.url),
    "utf8",
  );

  expect(playwrightConfig).toContain('process.env.PLAYWRIGHT_CHANNEL || "chrome"');
  expect(storybookConfig).toContain('process.env.PLAYWRIGHT_CHANNEL || "chrome"');
  expect(storybookConfig).toContain('rootDir: path.resolve(__dirname, "..")');
  expect(storybookConfig).toContain("launchOptions");
});
