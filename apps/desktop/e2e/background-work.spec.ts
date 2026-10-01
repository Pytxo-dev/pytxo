import { expect, test } from "@playwright/test";
import type { Page } from "@playwright/test";
import { completeOnboarding } from "./helpers";

const RECOVERY_AUDIT_MS = 5 * 60_000;

async function openUiOutput(page: Page) {
  const map = page.getByTestId("execution-map");
  await map.getByRole("button", { name: /^ui / }).click();
  await page.locator(".dock-panel:visible").getByRole("button", { name: "Open output", exact: true }).click();
  const output = page.getByRole("region", { name: "Recorded agent output", exact: true });
  await expect(output).toBeVisible();
  return output;
}

test("a late Setup response cannot erase a newer Work snapshot", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-delayed-light-snapshot-v1": "1" });
  await page.clock.install();
  await page.goto("/#/work");
  const output = await openUiOutput(page);
  await expect(output).toContainText("Browser fixture");
  await page.getByRole("link", { name: "Setup", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Setup", exact: true })).toBeVisible();
  await page.evaluate(() => {
    window.dispatchEvent(new Event("blur"));
    window.dispatchEvent(new Event("focus"));
  });
  // The lightweight read is pending while the route requests full Work rows.
  await page.getByRole("link", { name: "Work", exact: true }).click();
  await expect(output).toContainText("Browser fixture");
  await page.clock.runFor(1_000);
  await expect(output).toContainText("Browser fixture");
  await expect(page.getByTestId("execution-map").getByText("Worker not recorded", { exact: true })).toHaveCount(0);
});

test("returning from a lightweight Setup snapshot reloads Work records", async ({ page }) => {
  await completeOnboarding(page);
  await page.clock.install();
  await page.goto("/#/work");
  const output = await openUiOutput(page);
  await expect(output).toContainText("Browser fixture");
  await page.getByRole("link", { name: "Setup", exact: true }).click();
  await page.clock.runFor(61_000);
  await expect(page.getByText("Updated 1m ago", { exact: true })).toBeVisible();
  await page.getByRole("link", { name: "Work", exact: true }).click();
  await expect(page.getByText("Updated just now", { exact: true })).toBeVisible();
  await expect(output).toContainText("Browser fixture");
  await expect(page.getByTestId("execution-map").getByText("Worker not recorded", { exact: true })).toHaveCount(0);
});

test("background Work refresh keeps recorded workers and their evidence", async ({ page }) => {
  await completeOnboarding(page);
  await page.clock.install();
  await page.goto("/#/work");
  const map = page.getByTestId("execution-map");
  await expect(map.getByText("Active worker", { exact: true })).toHaveCount(1);
  const output = await openUiOutput(page);
  await expect(output).toContainText("Browser fixture: recorded output example.");

  // Exercise the real shell's idle integrity refresh while it is unfocused.
  await page.evaluate(() => window.dispatchEvent(new Event("blur")));
  await page.clock.runFor(31_000);
  await expect(page.getByText("Updated 30s ago", { exact: true })).toBeVisible();
  // Incremental domain polling handles normal updates. This slower full read is
  // the recovery audit for a missed notification or cursor signal.
  await page.clock.fastForward(RECOVERY_AUDIT_MS - 30_000);
  await expect(page.getByText("Updated just now", { exact: true })).toBeVisible();
  await expect(map.getByText("Active worker", { exact: true })).toHaveCount(1);
  await expect(map.getByText("Worker not recorded", { exact: true })).toHaveCount(0);
  await expect(output).toBeVisible();
  await expect(output).toContainText("Browser fixture: recorded output example.");

  await page.evaluate(() => window.dispatchEvent(new Event("focus")));
  await expect(output).toBeVisible();
});
