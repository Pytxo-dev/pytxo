// Screenshots of the main journeys for visual review in CI (desktop-ui-review.yml).
// Skipped unless UI_REVIEW=1, so the regular e2e run is unchanged.
import { test } from "@playwright/test";
import { clearOnboarding, completeOnboarding } from "./helpers";

const OUT = process.env.UI_REVIEW_OUT ?? "ui-review";
test.skip(!process.env.UI_REVIEW, "UI review screenshots run only in desktop-ui-review.yml");

for (const [width, height] of [[1440, 900], [1024, 680]]) {
  const at = `${width}x${height}`;

  test(`onboarding @${at}`, async ({ page }) => {
    await page.setViewportSize({ width, height });
    await clearOnboarding(page);
    await page.goto("/");
    await page.getByRole("button", { name: "Get started" }).waitFor();
    await page.screenshot({ path: `${OUT}/${at}-1-welcome.png` });
    await page.getByRole("button", { name: "Get started" }).click();
    await page.getByRole("button", { name: /^Continue/ }).waitFor();
    await page.waitForTimeout(600);
    await page.screenshot({ path: `${OUT}/${at}-2-agents.png` });
    await page.getByRole("button", { name: /^Continue/ }).click();
    await page.getByRole("button", { name: "Try the guided example" }).waitFor();
    await page.screenshot({ path: `${OUT}/${at}-3-project.png` });
    await page.getByRole("button", { name: "Try the guided example" }).click();
    await page.getByRole("button", { name: "Enter Pytxo Desktop" }).waitFor();
    await page.screenshot({ path: `${OUT}/${at}-4-ready.png` });
  });

  test(`work needing a decision @${at}`, async ({ page }) => {
    await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "long-mission" });
    await page.setViewportSize({ width, height });
    await page.goto("/#/work");
    await page.getByRole("region", { name: "Work", exact: true }).locator("h1").waitFor();
    await page.waitForTimeout(1500);
    await page.screenshot({ path: `${OUT}/${at}-5-work-decision.png` });
  });

  test(`fleet board @${at}`, async ({ page }) => {
    await completeOnboarding(page, { "pytxo-preview-fleet-v1": "1" });
    await page.setViewportSize({ width, height });
    await page.goto("/#/work");
    await page.getByTestId("fleet-board").waitFor();
    await page.waitForTimeout(2000);
    await page.screenshot({ path: `${OUT}/${at}-6-fleet.png` });
  });

  test(`review @${at}`, async ({ page }) => {
    await completeOnboarding(page, { "pytxo-preview-review-state-v1": "ready", "pytxo-preview-candidate-check-v1": "passed" });
    await page.setViewportSize({ width, height });
    await page.goto("/#/run-review");
    await page.locator(".line-diff, .exact-diff").first().waitFor();
    await page.waitForTimeout(800);
    await page.screenshot({ path: `${OUT}/${at}-7-review.png` });
  });
}
