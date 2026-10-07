import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("ambient ASCII rotates without claiming execution and respects reduced motion", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-legacy-retry-v1": "1" });
  await page.goto("/#/work");
  const orb = page.locator(".aperture-glyph");
  await expect(orb).toHaveAttribute("data-animated", "true");
  await expect(orb).toHaveAttribute("data-active", "false");
  await expect(orb).toHaveAttribute("data-motion", "ambient");
  const pixels = () => orb.locator("canvas").evaluate((el: HTMLCanvasElement) => el.toDataURL());
  await expect.poll(() => orb.locator("canvas").evaluate((el: HTMLCanvasElement) => {
    const data = el.getContext("2d")!.getImageData(0, 0, el.width, el.height).data;
    return data.some((value, index) => index % 4 === 3 && value > 0);
  })).toBe(true);
  const before = await pixels();
  await expect.poll(pixels).not.toBe(before);
  await expect(page.locator(".work-summary")).toContainText("saved to your project");
  await page.emulateMedia({ reducedMotion: "reduce" });
  await expect(orb).toHaveAttribute("data-animated", "false");
  await expect(orb.locator("svg")).not.toHaveClass(/concealed/);
});

for (const width of [1280, 1920]) test(`flat task surfaces at ${width}`, async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "ready" });
  await page.setViewportSize({ width, height: width === 1920 ? 1080 : 800 });
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.goto("/#/flow");
  await expect(page.getByLabel("What should Pytxo do?")).toBeVisible();
  await expect(page.getByText("One useful change.")).toHaveCount(0);
  await expect(page.locator(".flow-history")).not.toHaveAttribute("open");
  await page.locator(".flow-history > summary").click();
  await expect(page.getByRole("button", { name: /Use as new request:/ }).first()).toBeVisible();
  await page.locator(".flow-history > summary").click();
  for (const route of ["flow", "work", "history"]) {
    await page.goto(`/#/${route}`);
    await expect(page.locator("h1").first()).toBeVisible();
    await page.locator("h1").first().scrollIntoViewIfNeeded();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await page.screenshot({ path: `../../target/beta-regression-repair-20260919/refined-${route}-${width}.png` });
  }
});
