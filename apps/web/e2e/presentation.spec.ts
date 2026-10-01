import { expect, test } from '@playwright/test';

for (const width of [1440, 390]) {
  test(`product walkthrough is keyboard operable at ${width}`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 900 });
    const errors: string[] = [];
    page.on('pageerror', error => errors.push(error.message));
    await page.goto('/');
    const walkthrough = page.getByTestId('product-walkthrough');
    const tabs = walkthrough.getByRole('tab');
    await expect(tabs).toHaveCount(3);
    const imageOffset = async () => {
      const panel = walkthrough.getByRole('tabpanel');
      const image = await (width < 1280
        ? panel.getByTestId('mobile-product-details').locator('img').first().locator('..')
        : panel.getByRole('img')).boundingBox();
      const frame = await walkthrough.boundingBox();
      return image!.y - frame!.y;
    };
    const initialImageOffset = await imageOffset();
    await walkthrough.screenshot({ path: testInfo.outputPath(`walkthrough-review-${width}.png`) });
    await tabs.first().focus();
    await page.keyboard.press('ArrowRight');
    await expect(tabs.nth(1)).toBeFocused();
    await expect(tabs.nth(1)).toHaveAttribute('aria-selected', 'true');
    const panel = walkthrough.getByRole('tabpanel');
    await expect(panel).toHaveAccessibleName('Review & Apply');
    expect(Math.abs(await imageOffset() - initialImageOffset)).toBeLessThanOrEqual(1);
    await expect(panel.getByRole('img').first()).toHaveJSProperty('naturalWidth', 1600);
    await expect(panel.getByRole('link')).toHaveAttribute('href', '/product/run-review-1600x1000.png');
    await page.keyboard.press('End');
    await expect(panel).toHaveAccessibleName('Recorded outcome');
    expect(Math.abs(await imageOffset() - initialImageOffset)).toBeLessThanOrEqual(1);
    await walkthrough.screenshot({ path: testInfo.outputPath(`walkthrough-history-${width}.png`) });
    await page.keyboard.press('ArrowRight');
    await expect(tabs.first()).toBeFocused();
    await expect(panel).toHaveAccessibleName('Execution');
    // Focus must remain usable beneath the sticky site navigation, not merely
    // exist on an element obscured by another layer.
    await expect.poll(() => tabs.first().evaluate(node => {
      const box = node.getBoundingClientRect();
      const top = document.elementFromPoint(box.x + box.width / 2, box.y + box.height / 2);
      return top !== null && node.contains(top);
    })).toBe(true);
    await expect(walkthrough).toContainText('browser fixtures, not a recorded mission');
    expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
    await page.screenshot({ path: testInfo.outputPath(`walkthrough-focus-${width}.png`) });
    await walkthrough.screenshot({ path: testInfo.outputPath(`walkthrough-${width}.png`) });
    expect(errors).toEqual([]);
  });
}

test('walkthrough rapid selection and reduced motion retain the selected content', async ({ page }) => {
  await page.emulateMedia({ reducedMotion: 'reduce' });
  await page.goto('/');
  const walkthrough = page.getByTestId('product-walkthrough');
  const tabs = walkthrough.getByRole('tab');
  for (const index of [1, 2, 0, 2, 1]) await tabs.nth(index).click();
  await expect(walkthrough.getByRole('tabpanel')).toHaveAccessibleName('Review & Apply');
  expect(await walkthrough.evaluate(node => node.getAnimations({ subtree: true }).filter(animation => animation.playState === 'running').length)).toBe(0);
  await expect(walkthrough.getByRole('tabpanel')).toHaveCSS('opacity', '1');
});

test('homepage explains verification and does not claim coordinated modular Apply', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByTestId('product-walkthrough').getByRole('tab')).toHaveCount(3);
  await expect(page.getByTestId('compatibility-section')).toContainText('One repository.');
  await expect(page.getByTestId('boundary-section')).toContainText('does not control every host or network side effect');
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
