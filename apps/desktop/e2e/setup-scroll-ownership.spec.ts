import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

for (const viewport of [{ width: 1920, height: 1020 }, { width: 1280, height: 800 }, { width: 860, height: 560 }]) {
  test(`Setup keeps navigation stationary while long content scrolls at ${viewport.width}`, async ({ page }) => {
    await page.setViewportSize(viewport);
    await completeOnboarding(page);
    await page.goto("/#/settings");
    await page.getByRole("button", { name: "Agents & permissions", exact: true }).click();
    const main = page.locator(".settings-main");
    const rail = page.locator(".settings-rail");
    const railBefore = await rail.boundingBox();
    await page.screenshot({ path: test.info().outputPath(`setup-top-${viewport.width}.png`) });
    await page.getByText("Additional agents", { exact: true }).click();
    await page.getByText("Change default permissions", { exact: true }).click();
    const box = await main.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.y + box!.height).toBeLessThanOrEqual(viewport.height + 1);
    expect(Math.abs(box!.x + box!.width - viewport.width)).toBeLessThanOrEqual(1);
    const contentBox = await page.locator(".settings-content").boundingBox();
    expect(contentBox).not.toBeNull();
    expect(contentBox!.width).toBeLessThanOrEqual(960);
    expect(await main.evaluate((element) => getComputedStyle(element, "::-webkit-scrollbar").width)).toBe("10px");
    await page.mouse.move(box!.x + box!.width / 2, box!.y + box!.height / 2);
    await page.mouse.wheel(0, 10000);
    await expect.poll(() => main.evaluate(e => e.scrollTop)).toBeGreaterThan(0);
    await expect(page.getByRole("button", { name: /Supernova Full host privileges/ })).toBeInViewport();
    expect(await rail.boundingBox()).toEqual(railBefore);
    expect(await page.locator(".mission-content > .content").evaluate(e => e.scrollTop)).toBe(0);
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBeLessThanOrEqual(1);
    await page.screenshot({ path: test.info().outputPath(`setup-${viewport.width}.png`) });
    await page.getByRole("button", { name: "General", exact: true }).click();
    await expect(page.getByRole("heading", { name: "General", exact: true })).toBeInViewport();
    await page.getByRole("link", { name: "History", exact: true }).click();
    await expect(page.getByRole("heading", { name: "History", exact: true })).toBeVisible();
    await page.screenshot({ path: test.info().outputPath(`history-${viewport.width}.png`) });
    await page.getByRole("link", { name: "Work", exact: true }).click();
    await page.screenshot({ path: test.info().outputPath(`work-${viewport.width}.png`) });
  });
}
