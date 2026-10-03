import { expect, test } from "@playwright/test";
import { clearOnboarding, completeOnboarding } from "./helpers";

for (const viewport of [{ width: 1440, height: 1000 }, { width: 390, height: 844 }]) {
  test.describe(`Beta workflow at ${viewport.width}px`, () => {
    test.use({ viewport });

    test("run-specific checks require a fresh preview and keep one-worker task scope", async ({ page }, testInfo) => {
      await completeOnboarding(page, { "pytxo-preview-ade-state-v1": "codex-only", "pytxo-preview-flow-verification-v1": "none" });
      await page.goto("/#/flow");
      await page.getByLabel("What should Pytxo do?").fill("Fix src/api.ts and its tests");
      await page.getByRole("button", { name: "Build plan", exact: true }).click();
      await expect(page.getByRole("button", { name: "Run", exact: true })).toBeDisabled();
      if (viewport.width < 1100) await page.getByRole("button", { name: "Request", exact: true }).click();
      await page.locator(".checks-editor > summary").click();
      await page.getByLabel("Additional verification commands", { exact: true }).fill("npm test");
      if (viewport.width < 1100) await page.getByRole("button", { name: "Plan", exact: true }).click();
      await expect(page.getByText("Plan is stale", { exact: true })).toBeVisible();
      if (viewport.width < 1100) await page.getByRole("button", { name: "Request", exact: true }).click();
      await page.getByRole("button", { name: "Build plan", exact: true }).first().click();
      await expect(page.getByRole("button", { name: "Run", exact: true })).toBeEnabled();
      await expect(page.locator(".plan-wave")).toHaveCount(3);
      await expect(page.locator(".plan-explainer").first()).toHaveText("Up to 1 agent at once. Edit any step before you run it.");
      await expect(page.locator(".contract-list code")).toContainText(["npm test"]);
      if (viewport.width < 1100) await page.getByRole("button", { name: "Request", exact: true }).click();
      await page.getByLabel("Additional verification commands", { exact: true }).fill("npm run test:integration");
      if (viewport.width < 1100) await page.getByRole("button", { name: "Plan", exact: true }).click();
      await expect(page.getByRole("button", { name: "Run", exact: true })).toBeDisabled();
      await page.screenshot({ path: testInfo.outputPath("checks-require-fresh-review.png"), fullPage: true });
    });

    test("onboarding rechecks detected sessions after vendor login", async ({ page }, testInfo) => {
      await clearOnboarding(page);
      await page.goto("/");
      await page.getByRole("button", { name: "Get started" }).click();
      await expect(page.getByText("Pytxo runs the coding CLIs you already use", { exact: false })).toBeVisible();
      await expect(page.getByRole("status")).toContainText("3 of 5 ready · 4 installed");
      await page.getByRole("button", { name: "Connect an OpenCode provider" }).click();
      await expect(page.getByText("OpenCode sign-in opened. Finish the vendor flow, then recheck.")).toBeVisible();
      // Simulate a changed vendor-owned probe result, not a successful real login.
      await page.evaluate(() => localStorage.setItem("pytxo-preview-ade-state-v1", "codex-only"));
      await page.getByRole("button", { name: "Check again", exact: true }).click();
      await expect(page.locator(".summary")).toContainText("1 of 5 ready · 1 installed");
      await expect(page.getByText("ChatGPT connected", { exact: true })).toBeVisible();
      await expect(page.getByText("OpenCode sign-in opened. Finish the vendor flow, then recheck.")).not.toBeVisible();
      await page.screenshot({ path: testInfo.outputPath("onboarding.png"), fullPage: true });
      await page.getByRole("button", { name: "Continue", exact: true }).click();
      await expect(page.getByRole("heading", { name: "Choose your project" })).toBeVisible();
    });

    test("Flow and Review distinguish task checks from candidate proof", async ({ page }, testInfo) => {
      await completeOnboarding(page, { "pytxo-preview-ade-state-v1": "codex-only" });
      await page.goto("/#/flow");
      await expect(page.getByLabel("Agent CLI")).toHaveValue("codex");
      await page.getByLabel("What should Pytxo do?").fill("Improve src/api.ts and its tests");
      await page.getByRole("button", { name: "Build plan", exact: true }).click();
      await page.getByText("Permissions and technical details", { exact: true }).click();
      await expect(page.getByText("Per-task verification commands", { exact: true })).toBeVisible();
      await expect(page.getByText("These checks run in each task's copy, then again on all the changes together before you can Apply.")).toBeVisible();
      await page.getByText("Per-task verification commands", { exact: true }).scrollIntoViewIfNeeded();
      await page.screenshot({ path: testInfo.outputPath("flow-task-checks.png"), fullPage: true });

      await page.goto("/#/history");
      await page.locator('.history .row[data-run-id="run-71ad"]').click();
      await page.getByRole("button", { name: /Review prepared changes/ }).click();
      await page.locator(".technical-evidence > summary").click();
      await expect(page.locator("#run-review-title")).toBeVisible();
      await expect(page.getByText("Checks on the combined changes", { exact: true })).toBeVisible();
      await expect(page.getByText("Not verified. Checks ran only in each agent's own copy, not on the combined changes.")).toBeVisible();
      await page.getByText("Checks on the combined changes", { exact: true }).scrollIntoViewIfNeeded();
      await page.screenshot({ path: testInfo.outputPath("review-candidate-checks.png"), fullPage: true });
    });

    test("a missing Git error keeps onboarding retry and skip available", async ({ page }, testInfo) => {
      await clearOnboarding(page);
      await page.goto("/");
      await page.getByRole("button", { name: "Get started" }).click();
      await page.getByRole("button", { name: "Continue", exact: true }).click();
      await expect(page.getByText("Requires Git. Includes tests that use Node.js; no API key required.")).toBeVisible();
      // Exercise UI error recovery with a preview backend response. The native
      // Rust regression separately checks a genuinely absent Git executable.
      await page.evaluate(() => localStorage.setItem("pytxo-preview-example-error-v1", "missing-git"));
      await page.getByRole("button", { name: "Try the guided example" }).click();
      await expect(page.getByRole("alert")).toContainText("Git was not found. Install Git, restart Pytxo Desktop");
      await expect(page.getByRole("button", { name: "Try the guided example" })).toBeEnabled();
      await expect(page.getByRole("button", { name: "Skip for now" })).toBeEnabled();
      await page.getByRole("alert").scrollIntoViewIfNeeded();
      await page.screenshot({ path: testInfo.outputPath("missing-git.png"), fullPage: true });
      await page.getByRole("button", { name: "Skip for now" }).click();
      await expect(page.getByRole("heading", { name: "Desktop setup complete" })).toBeVisible();
      await page.getByRole("button", { name: "Back", exact: true }).click();
      await page.getByRole("button", { name: "Try the guided example" }).click();
      await expect(page.getByRole("alert")).toContainText("Git was not found");
      await page.evaluate(() => localStorage.removeItem("pytxo-preview-example-error-v1"));
      await page.getByRole("button", { name: "Try the guided example" }).click();
      await expect(page.getByRole("alert")).toHaveCount(0);
      await expect(page.getByText("Guided local Git example ready. Its baseline tests need no API key.")).toBeVisible();
      await page.getByRole("button", { name: "Continue", exact: true }).click();
      await expect(page.getByRole("heading", { name: "Desktop setup complete" })).toBeVisible();
    });

    test("Review reports the combined receipt without promoting failed or empty checks", async ({ page }, testInfo) => {
      await completeOnboarding(page);
      await page.goto("/");
      await page.evaluate(() => localStorage.setItem("pytxo-preview-candidate-check-v1", "passed"));
      await page.goto("/#/history");
      await page.locator('.history .row[data-run-id="run-71ad"]').click();
      await page.getByRole("button", { name: /Review prepared changes/ }).click();
      await page.locator(".technical-evidence > summary").click();
      await expect(page.locator(".candidate-checks")).toContainText("Passed · 1 command run by Pytxo on these exact files.");
      await expect(page.locator(".candidate-checks")).toContainText("npm run check");
      await page.locator(".candidate-checks").scrollIntoViewIfNeeded();
      await page.screenshot({ path: testInfo.outputPath("review-combined-passed.png"), fullPage: true });
      await page.locator(".candidate-checks").screenshot({ path: testInfo.outputPath("combined-receipt.png") });
      for (const state of ["failed", "empty"]) {
        await page.evaluate((value) => localStorage.setItem("pytxo-preview-candidate-check-v1", value), state);
        await page.reload();
        await page.goto("/#/history");
        await page.locator('.history .row[data-run-id="run-71ad"]').click();
        await page.getByRole("button", { name: /Review prepared changes/ }).click();
        await page.locator(".technical-evidence > summary").click();
        await expect(page.locator(".candidate-checks")).toContainText("Not verified. A check failed, did not finish, or is not supported on the combined changes.");
      }
    });
  });
}
