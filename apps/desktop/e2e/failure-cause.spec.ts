import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("a failed run names the failed task and the agent's reported cause", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-run-state-v1": "agent_failed" });
  await page.goto("/#/work");
  const summary = page.locator(".work-summary");
  await expect(summary).toHaveText(
    "ui failed. The agent reported: Browser fixture: the agent CLI reported an error before finishing.",
  );
  await expect(summary).toHaveCSS("white-space", "normal");
});

test("a starting run does not report a missing review contract as a check failure", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-run-state-v1": "starting" });
  await page.goto("/#/work");
  await expect(page.locator(".work-summary")).toHaveText("Getting the work ready. Your agents have not reported a result yet.");
  await expect(page.getByText("Checks could not be loaded", { exact: false })).toHaveCount(0);
});
