import { expect, test } from "@playwright/test";
import { clearOnboarding, completeOnboarding } from "./helpers";

for (const viewport of [{ width: 1440, height: 1000 }, { width: 390, height: 844 }]) {
  test.describe(`Beta workflow at ${viewport.width}px`, () => {
    test.use({ viewport });

    test("onboarding rechecks detected sessions after vendor login", async ({ page }, testInfo) => {
      await clearOnboarding(page);
      await page.goto("/");
      await page.getByRole("button", { name: "Get started" }).click();
      await page.getByRole("button", { name: "Continue with Desktop" }).click();
      await expect(page.getByText("One coding agent CLI is enough.", { exact: false })).toBeVisible();
      await expect(page.getByRole("status")).toContainText("5 installed · 3 ready");
      await page.getByRole("button", { name: "Connect an OpenCode provider" }).click();
      await expect(page.getByText("OpenCode sign-in opened. Finish the vendor flow, then recheck.")).toBeVisible();
      // Simulate a changed vendor-owned probe result, not a successful real login.
      await page.evaluate(() => localStorage.setItem("pytxo-preview-ade-state-v1", "codex-only"));
      await page.getByRole("button", { name: "Check again", exact: true }).click();
      await expect(page.locator(".summary")).toContainText("1 installed · 1 ready");
      await expect(page.getByText("ChatGPT connected", { exact: true })).toBeVisible();
      await expect(page.getByText("OpenCode sign-in opened. Finish the vendor flow, then recheck.")).not.toBeVisible();
      await page.screenshot({ path: testInfo.outputPath("onboarding.png"), fullPage: true });
      await page.getByRole("button", { name: "Continue", exact: true }).click();
      await expect(page.getByRole("heading", { name: "Open a Workspace" })).toBeVisible();
    });

    test("Flow and Review distinguish task checks from candidate proof", async ({ page }, testInfo) => {
      await completeOnboarding(page, { "pytxo-preview-ade-state-v1": "codex-only" });
      await page.goto("/#/flow");
      await expect(page.getByLabel("Agent CLI")).toHaveValue("codex");
      await page.getByLabel("Mission outcome").fill("Improve src/api.ts and its tests");
      await page.getByRole("button", { name: "Build plan", exact: true }).click();
      await expect(page.getByText("Per-task verification commands", { exact: true })).toBeVisible();
      await expect(page.getByText("These checks run in each task's workspace. Passing task checks does not prove the combined candidate passes.")).toBeVisible();
      await page.getByText("Per-task verification commands", { exact: true }).scrollIntoViewIfNeeded();
      await page.screenshot({ path: testInfo.outputPath("flow-task-checks.png"), fullPage: true });

      await page.goto("/#/history");
      await page.locator(".history .row", { hasText: "run-71ad" }).first().click();
      await page.getByRole("button", { name: /Review package/ }).click();
      await expect(page.getByRole("heading", { name: "Run Review" })).toBeVisible();
      await expect(page.getByText("Combined candidate checks", { exact: true })).toBeVisible();
      await expect(page.getByText("Not verified. Task checks ran in separate workspaces; this package binds the reviewed bytes, not a passing combined check.")).toBeVisible();
      await page.getByText("Combined candidate checks", { exact: true }).scrollIntoViewIfNeeded();
      await page.screenshot({ path: testInfo.outputPath("review-candidate-checks.png"), fullPage: true });
    });

    test("a missing Git error keeps onboarding retry and skip available", async ({ page }, testInfo) => {
      await clearOnboarding(page);
      await page.goto("/");
      await page.getByRole("button", { name: "Get started" }).click();
      await page.getByRole("button", { name: "Continue with Desktop" }).click();
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
      await expect(page.getByRole("heading", { name: "Set your display" })).toBeVisible();
      await page.getByRole("button", { name: "Back", exact: true }).click();
      await page.getByRole("button", { name: "Try the guided example" }).click();
      await expect(page.getByRole("alert")).toContainText("Git was not found");
      await page.evaluate(() => localStorage.removeItem("pytxo-preview-example-error-v1"));
      await page.getByRole("button", { name: "Try the guided example" }).click();
      await expect(page.getByRole("alert")).toHaveCount(0);
      await expect(page.getByText("Guided local Git example ready. Its baseline tests need no API key.")).toBeVisible();
      await page.getByRole("button", { name: "Continue", exact: true }).click();
      await expect(page.getByRole("heading", { name: "Set your display" })).toBeVisible();
    });

    test("Review reports the combined receipt without promoting failed or empty checks", async ({ page }, testInfo) => {
      await completeOnboarding(page);
      await page.goto("/");
      await page.evaluate(() => localStorage.setItem("pytxo-preview-candidate-check-v1", "passed"));
      await page.goto("/#/history");
      await page.locator(".history .row", { hasText: "run-71ad" }).first().click();
      await page.getByRole("button", { name: /Review package/ }).click();
      await expect(page.locator(".candidate-checks")).toContainText("Passed · 1 command on this combined candidate.");
      await expect(page.locator(".candidate-checks")).toContainText("npm run check");
      await page.locator(".candidate-checks").scrollIntoViewIfNeeded();
      await page.screenshot({ path: testInfo.outputPath("review-combined-passed.png"), fullPage: true });
      await page.locator(".candidate-checks").screenshot({ path: testInfo.outputPath("combined-receipt.png") });
      for (const state of ["failed", "empty"]) {
        await page.evaluate((value) => localStorage.setItem("pytxo-preview-candidate-check-v1", value), state);
        await page.reload();
        await page.goto("/#/history");
        await page.locator(".history .row", { hasText: "run-71ad" }).first().click();
        await page.getByRole("button", { name: /Review package/ }).click();
        await expect(page.locator(".candidate-checks")).toContainText("Not verified. The combined candidate receipt is incomplete, unsupported, or includes a failed check.");
      }
    });
  });
}
