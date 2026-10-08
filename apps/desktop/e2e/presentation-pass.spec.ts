import { expect, test } from '@playwright/test';
import { completeOnboarding } from './helpers';

async function recordPanelMotion(page: import('@playwright/test').Page) {
  await page.addInitScript(() => {
    const animate = Element.prototype.animate;
    const records: Animation[] = [];
    Object.assign(window, { panelMotion: records });
    Element.prototype.animate = function (...args) {
      const animation = animate.apply(this, args);
      if (this.classList.contains('dock-panel')) records.push(animation);
      return animation;
    };
  });
}

async function revealInspectionTools(page: import('@playwright/test').Page) {
  const toggle = page.getByRole('button', { name: 'Inspection tools', exact: true });
  if (await toggle.getAttribute('aria-expanded') !== 'true') await toggle.click();
}

test('a sparse pointer gesture resolves the destination at release', async ({ page }) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/#/work');
  await revealInspectionTools(page);
  const source = page.getByRole('button', { name: 'Files', description: 'Click to open, or drag to the right or bottom edge', exact: true });
  await expect(source).toBeEnabled();
  const from = (await source.boundingBox())!;
  const grid = (await page.locator('.dock-grid').boundingBox())!;
  await page.mouse.move(from.x + from.width / 2, from.y + from.height / 2);
  await page.mouse.down();
  // One movement event reaches the destination. There need not be another
  // pointermove after the newly visible drop targets have rendered.
  await page.mouse.move(grid.x + grid.width / 2, grid.y + grid.height - 50);
  await page.mouse.up();
  await expect(page.getByRole('tablist', { name: 'bottom dock' })).toContainText('Files');
  await expect(page.getByRole('status').filter({ hasText: 'Files docked below.' })).toBeVisible();
});

test('evidence motion runs, interrupts immediately, and leaves no transform or stale panel', async ({ page }, testInfo) => {
  await completeOnboarding(page);
  await recordPanelMotion(page);
  await page.emulateMedia({ reducedMotion: 'no-preference' });
  await page.goto('/#/work');
  await revealInspectionTools(page);
  const files = page.getByRole('button', { name: 'Files', description: 'Click to open, or drag to the right or bottom edge', exact: true });
  await files.click();
  await expect.poll(() => page.evaluate(() => (window as any).panelMotion.length)).toBeGreaterThan(0);
  const panel = page.locator('.dock-panel:not([hidden])');
  for (let i = 0; i < 3; i++) {
    await page.getByRole('button', { name: 'Hide bottom dock', exact: true }).click();
    await expect(panel).toHaveCount(0);
    await files.click();
    await expect(panel).toBeVisible();
  }
  await expect.poll(() => panel.evaluate(node => node.getAnimations().length)).toBe(0);
  await expect(panel).toHaveCSS('transform', 'none');
  await expect(panel).toHaveCSS('opacity', '1');
  await expect(files).toBeFocused();
  await page.screenshot({ path: testInfo.outputPath('evidence-settled-browser.png') });
  await page.getByRole('link', { name: 'History', exact: true }).click();
  await expect(panel).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).panelMotion.every((animation: Animation) => animation.playState !== 'running'))).toBe(true);
});

for (const preference of ['os', 'app'] as const) {
  test(`evidence does not animate under ${preference} reduced motion`, async ({ page }) => {
    await completeOnboarding(page, preference === 'app' ? { 'pytxo-desktop-reduced-motion-v1': 'true' } : {});
    await recordPanelMotion(page);
    await page.emulateMedia({ reducedMotion: preference === 'os' ? 'reduce' : 'no-preference' });
    await page.goto('/#/work');
    await revealInspectionTools(page);
    await page.getByRole('button', { name: 'Files', description: 'Click to open, or drag to the right or bottom edge', exact: true }).click();
    await expect(page.locator('.dock-panel:not([hidden])')).toBeVisible();
    expect(await page.evaluate(() => (window as any).panelMotion.length)).toBe(0);
  });
}

test('review keeps exact identity one interaction away without repeating its blocker', async ({ page }, testInfo) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto('/#/history');
  await page.locator('.history .row[data-run-id="run-71ad"]').click();
  await page.getByRole('button', { name: 'Review prepared changes', exact: true }).click();
  const apply = page.getByRole('button', { name: 'Apply reviewed changes', exact: true });
  await expect(apply).toBeDisabled();
  await expect(apply).toHaveAccessibleDescription(/./);
  await expect(page.locator('.package-identity code')).toBeHidden();
  await expect(page.locator('.verification-summary')).toHaveText('Not verified');
  await expect(page.getByRole('heading', { name: 'Prepared changes', exact: true })).toBeInViewport();
  await page.screenshot({ path: testInfo.outputPath('review-after-browser-1280x800.png') });
  await page.getByText('Prepared change ID', { exact: true }).click();
  await expect(page.locator('.package-identity code')).toHaveText('pkg-71ad-immutable');
  await expect(apply).toBeDisabled();
});
