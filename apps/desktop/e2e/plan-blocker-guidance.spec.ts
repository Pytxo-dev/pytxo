import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

for (const width of [1920, 1280, 860, 390]) {
  for (const theme of ["void", "light"]) {
    test(`blocked plan explains recovery beside Run at ${width} in ${theme}`, async ({ page }, info) => {
      await completeOnboarding(page, { "pytxo-deck-theme": theme });
      await page.setViewportSize({ width, height: width === 1280 ? 720 : 760 });
      await page.goto("/#/flow");
      for (const label of ["Project", "Agent CLI"]) {
        const selector = page.getByLabel(label, { exact: true });
        await expect(selector).toBeInViewport({ ratio: 1 });
        expect((await selector.boundingBox())!.width).toBeGreaterThan(130);
      }
      await page.getByLabel("What should Pytxo do?").fill("Add a regression test for the parser.");
      await page.getByLabel("Agent CLI").selectOption("claude");
      await page.getByRole("button", { name: "Build plan", exact: true }).click();
      await expect(page.getByRole("heading", { name: "Plan blocked", exact: true })).toBeVisible();
      const reason = page.locator("#run-disabled-reason");
      await expect(reason).toHaveText("Desktop Beta runs Codex. Select Codex and build a new plan.");
      await expect(reason).toBeInViewport({ ratio: 1 });
      await expect(page.getByRole("button", { name: "Run", exact: true })).toBeDisabled();
      await expect(page.getByText("Desktop Beta runs Codex. Select Codex and build a new plan.", { exact: true })).toHaveCount(1);
      expect(await page.locator(".work-content").evaluate(el => el.scrollHeight - el.clientHeight)).toBeLessThanOrEqual(1);
      await page.screenshot({ path: info.outputPath("blocked-plan.png") });
      // A changed input must explain that the old plan is stale, not keep presenting its obsolete blocker.
      await page.getByLabel("Agent CLI").selectOption("codex");
      await expect(reason).toContainText("plan no longer matches");
      await expect(page.getByRole("button", { name: "Run", exact: true })).toBeDisabled();
    });
  }
}
