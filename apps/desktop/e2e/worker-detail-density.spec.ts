import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

for (const height of [800, 720]) {
  test(`long worker requests retain visible output access at 1280x${height}`, async ({ page }, testInfo) => {
    await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "long-worker-request" });
    await page.setViewportSize({ width: 1280, height });
    await page.goto("/#/work");
    const worker = page.getByTestId("execution-map").getByRole("button", { name: /^ui / });
    await expect(worker).toContainText("Update the parser");
    await worker.click();
    const panel = page.getByRole("complementary", { name: "Selected worker details" });
    const output = panel.getByRole("button", { name: "Open output", exact: true });
    await expect(output).toBeVisible();
    const bounds = (await panel.boundingBox())!;
    const action = (await output.boundingBox())!;
    expect(action.y + action.height).toBeLessThanOrEqual(bounds.y + bounds.height + 1);
    expect(action.y + action.height).toBeLessThanOrEqual(height);
    expect((await panel.locator(".inspection-title").boundingBox())!.height).toBeLessThan(60);
    if (height === 800) {
      await panel.getByText("Full worker request", { exact: true }).click();
      await expect(panel.locator(".worker-request p")).toContainText("mixed-case input");
      const expandedAction = (await output.boundingBox())!;
      expect(expandedAction.y + expandedAction.height).toBeLessThanOrEqual(bounds.y + bounds.height + 1);
    }
    await output.click();
    await expect(page.getByRole("region", { name: "Recorded agent output", exact: true })).toBeVisible();
    try {
      await expect.poll(async () => (await page.getByTestId("execution-map").locator(".map-body").boundingBox())!.height).toBeGreaterThan(120);
    } finally {
      await page.screenshot({ path: testInfo.outputPath("worker-output.png") });
      await testInfo.attach("layout", { body: JSON.stringify(await page.locator(".dock-grid,.mission-content,.work,.command-strip,.map-body,.dock-panel:visible").evaluateAll(elements => elements.map(e => ({class:e.className,height:e.getBoundingClientRect().height,style:e.getAttribute("style")})))), contentType: "application/json" });
    }
  });
}
