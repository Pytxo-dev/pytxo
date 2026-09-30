import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("Work and History show the same durable routed attempt record", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-routing-summary-v1": "1" });
  await page.goto("/#/work");

  const work = page.getByRole("region", { name: "Routing and worker attempts" });
  await expect(work.locator("summary")).toContainText("3 tasks · 3 attempts · Shadow mode");
  await work.locator("summary").click();
  await expect(work).toContainText("Worker attempt 1");
  await expect(work).toContainText("Worker attempt 2");
  await expect(work).toContainText("Shadow recorded");
  await expect(work).not.toContainText("Apply recorded");

  await page.goto("/#/history");
  await page.locator(".history .row").filter({ hasText: "run-8f2c" }).click();
  const history = page.getByRole("region", { name: "Routing and worker attempts" });
  await expect(history.locator("summary")).toContainText("3 tasks · 3 attempts · Shadow mode");
  await history.locator("summary").click();
  await expect(history).toContainText("Worker attempt 2");
});
