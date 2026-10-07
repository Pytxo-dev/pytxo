import { expect, test } from '@playwright/test';

test("homepage sections keep their images and links inside the viewport width", async ({ page }) => {
  for (const width of [390, 1440]) {
    await page.setViewportSize({ width, height: 900 });
    await page.goto("/");
    for (const id of ["how-it-works", "compare-section", "proof-section", "compatibility-section"]) {
      const section = page.getByTestId(id);
      await section.scrollIntoViewIfNeeded();
      const box = (await section.boundingBox())!;
      expect(box.x).toBeGreaterThanOrEqual(0);
      expect(box.x + box.width).toBeLessThanOrEqual(width + 1);
    }
    expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
  }
});

for (const width of [390, 768, 1280, 1440, 1920]) {
  test(`fleet hero and docs stay readable at ${width}`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 900 });
    await page.emulateMedia({ reducedMotion: 'reduce' });
    await page.goto('/');
    const heading = page.getByRole('heading', { level: 1 });
    await expect(heading).toBeInViewport();
    // The headline must keep a readable size and never run under the fleet capture.
    expect(parseFloat(await heading.evaluate(node => getComputedStyle(node).fontSize))).toBeGreaterThanOrEqual(40);
    const headingBox = (await heading.boundingBox())!;
    const fleetBox = (await page.getByTestId('hero-fleet').boundingBox())!;
    expect(headingBox.y + headingBox.height).toBeLessThan(fleetBox.y);
    expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
    await page.screenshot({ path: testInfo.outputPath(`fleet-home-${width}.png`), fullPage: true });
    await page.goto('/docs/getting-started/first-mission');
    await expect(page.getByRole('heading', { level: 1, name: 'First mission' })).toBeVisible();
    await expect(page.locator('article')).toContainText('unpublished');
    expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
    await page.screenshot({ path: testInfo.outputPath(`first-mission-${width}.png`), fullPage: true });
  });
}
