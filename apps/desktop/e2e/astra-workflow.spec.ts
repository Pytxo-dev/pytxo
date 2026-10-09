import { expect, test } from "@playwright/test";
import { completeOnboarding, rootOverflow } from "./helpers";

for (const viewport of [{ width: 1440, height: 1000 }, { width: 390, height: 844 }]) {
  test.describe(`Scoped mission at ${viewport.width}px`, () => {
    test.use({ viewport });

    test("templates require real scope and readiness can be rechecked", async ({ page }, testInfo) => {
      await completeOnboarding(page, { "pytxo-preview-ade-state-v1": "codex-only" });
      await page.goto("/#/flow");
      await page.getByRole("button", { name: "Fix a failure", exact: true }).click();
      await expect(page.getByLabel("What should Pytxo do?")).toHaveValue(/\[existing file path\]/);
      await expect(page.getByRole("button", { name: "Build plan", exact: true })).toBeDisabled();
      await expect(page.getByLabel("What should Pytxo do?")).toBeFocused();
      await page.getByLabel("What should Pytxo do?").fill("Fix src/parser.rs and add a regression test");
      await expect(page.getByRole("button", { name: "Build plan", exact: true })).toBeEnabled();
      await page.getByRole("button", { name: "Check again", exact: true }).click();
      await expect(page.getByLabel("Agent CLI")).toHaveValue("codex");
      await expect(page.getByText("Each task runs in its own isolated copy of the project.", { exact: false })).toBeVisible();
      expect((await rootOverflow(page)).horizontal).toBeLessThanOrEqual(1);
      await page.screenshot({ path: testInfo.outputPath("composer.png"), fullPage: true });
      await page.getByRole("button", { name: "Build plan", exact: true }).click();
      await expect(page.getByLabel("Task desktop-flow prompt")).toHaveJSProperty("tagName", "TEXTAREA");
      await expect(page.getByText("after desktop-flow, orchestration-flow", { exact: true })).toBeVisible();
      await page.screenshot({ path: testInfo.outputPath("plan.png"), fullPage: true });
    });

    test("using a draft restores its workspace and revokes the previous plan", async ({ page }) => {
      await completeOnboarding(page, { "pytxo-preview-ade-state-v1": "codex-only", "pytxo-preview-flow-history-v1": "workspace" });
      await page.goto("/#/flow");
      await expect(page.getByLabel("Project", { exact: true })).toHaveValue("pytxo");
      await page.getByLabel("What should Pytxo do?").fill("Improve src/api.ts and its tests");
      await page.getByRole("button", { name: "Build plan", exact: true }).click();
      await expect(page.getByRole("button", { name: "Run", exact: true })).toBeEnabled();
      await page.locator(".flow-history > summary").click();
      await page.getByRole("button", { name: /Use as new request: Fix the parser regression/ }).click();
      await expect(page.getByLabel("Project", { exact: true })).toHaveValue("signal-lab");
      await expect(page.getByRole("button", { name: "Run", exact: true })).not.toBeVisible();
      await expect(page.getByText(/Build a fresh plan before running/)).toBeVisible();
    });

    test("an unavailable draft workspace requires an explicit new selection", async ({ page }) => {
      await completeOnboarding(page, { "pytxo-preview-ade-state-v1": "codex-only", "pytxo-preview-flow-history-v1": "missing" });
      await page.goto("/#/flow");
      await page.locator(".flow-history > summary").click();
      await page.getByRole("button", { name: /Use as new request: Fix the parser regression/ }).click();
      await expect(page.getByLabel("Project", { exact: true })).toHaveValue("");
      await expect(page.getByRole("button", { name: "Build plan", exact: true })).toBeDisabled();
      await expect(page.getByText(/Original workspace is unavailable/)).toBeVisible();
      await page.getByLabel("Project", { exact: true }).selectOption("pytxo");
      await expect(page.getByRole("button", { name: "Build plan", exact: true })).toBeEnabled();
    });

    test("rechecking does not replace an unavailable original harness", async ({ page }) => {
      await completeOnboarding(page, { "pytxo-preview-ade-state-v1": "codex-only", "pytxo-preview-flow-history-v1": "unavailable-cli", "pytxo-flow-ade-v1": "codex" });
      await page.goto("/#/flow");
      await page.locator(".flow-history > summary").click();
      await page.getByRole("button", { name: /Use as new request: Fix the parser regression/ }).click();
      await expect(page.getByLabel("Agent CLI")).toHaveValue("");
      await page.getByRole("button", { name: "Check again", exact: true }).click();
      await expect(page.getByLabel("Agent CLI")).toHaveValue("");
      await expect(page.getByRole("button", { name: "Build plan", exact: true })).toBeDisabled();
      await page.getByLabel("Agent CLI").selectOption("codex");
      await expect(page.getByRole("button", { name: "Build plan", exact: true })).toBeEnabled();
    });

    test("long original workspace paths remain readable", async ({ page }, testInfo) => {
      await completeOnboarding(page, { "pytxo-preview-ade-state-v1": "codex-only", "pytxo-preview-flow-history-v1": "long-path" });
      await page.goto("/#/flow");
      await page.locator(".flow-history > summary").click();
      const path = page.locator(".flow-history small");
      await path.scrollIntoViewIfNeeded();
      await expect(path).toContainText("C:/workspaces/");
      const bounds = await path.boundingBox();
      const panel = await page.locator(".flow-history").boundingBox();
      expect(bounds!.x + bounds!.width).toBeLessThanOrEqual(panel!.x + panel!.width);
      expect((await rootOverflow(page)).horizontal).toBeLessThanOrEqual(1);
      await page.screenshot({ path: testInfo.outputPath("long-workspace.png"), fullPage: true });
    });
  });
}
