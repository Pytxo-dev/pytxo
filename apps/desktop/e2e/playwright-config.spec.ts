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
  const missionFlow = readFileSync(new URL("../src/components/desktop2/MissionFlowScreen.svelte", import.meta.url), "utf8");
  const flowIpc = readFileSync(new URL("../src-tauri/src/ipc_flow.rs", import.meta.url), "utf8");

  expect(flow).not.toContain("backend.loadSnapshot");
  expect(flow).not.toContain("watchRun(");
  expect(missionFlow).toContain("runs={snapshot.runs}");
  expect(flowIpc).toContain("emit_domain_changed(&app");
});

test("mission tabs expose roving tab semantics and keyboard navigation", () => {
  const missionFlow = readFileSync(
    new URL("../src/components/desktop2/MissionFlowScreen.svelte", import.meta.url),
    "utf8",
  );

  expect(missionFlow).toContain('role="tablist"');
  expect(missionFlow).toContain('role="tab"');
  expect(missionFlow).toContain('role="tabpanel"');
  expect(missionFlow).toContain("aria-selected");
  expect(missionFlow).toContain("handleTabKeydown");
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
