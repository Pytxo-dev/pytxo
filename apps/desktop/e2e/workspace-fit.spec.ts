import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

for (const size of [{ width: 1536, height: 816 }, { width: 1280, height: 800 }, { width: 860, height: 560 }]) {
  test(`workspace paths fit beside docks at ${size.width}`, async ({ page }, info) => {
    await page.setViewportSize(size);
    await completeOnboarding(page, { "pytxo-preview-layout-fixture-v1": "long" });
    await page.goto("/#/work");
    await page.getByRole("button", { name: "Preview", exact: true }).click();
    await page.goto("/#/setup");
    const picker = page.getByRole("combobox", { name: "Settings section", exact: true });
    if (await picker.isVisible()) await picker.selectOption("workspaces");
    else await page.getByRole("button", { name: "Workspaces", exact: true }).click();
    const table = page.locator(".workspace-table");
    await expect(table).toBeVisible();
    expect(await table.evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
    const row = table.locator('.table-row:not(.table-header)').first();
    const path = row.locator(".workspace-path");
    expect(await path.evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
    const box = (await path.boundingBox())!;
    const policy = (await row.locator('.workspace-policy').boundingBox())!;
    expect(policy.y >= box.y + box.height - 1 || policy.x >= box.x + box.width - 1).toBeTruthy();
    await row.getByRole("button", { name: "Settings", exact: true }).scrollIntoViewIfNeeded();
    await expect(row.getByRole("button", { name: "Settings", exact: true })).toBeInViewport();
    expect(await page.locator('.content').evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
    await page.screenshot({ path: info.outputPath('workspaces.png') });
  });
}

for (const fixture of ["short", "long"]) {
  test(`History scrolls only overflowing content with ${fixture} history`, async ({ page }, info) => {
    await completeOnboarding(page, { "pytxo-preview-layout-fixture-v1": fixture });
    await page.setViewportSize({ width: 1536, height: 900 });
    await page.goto("/#/history");
    await expect(page.locator('.history .boundary .candidate')).toBeVisible();
    const overflow = () => page.locator('.content').evaluate(el => el.scrollHeight - el.clientHeight);
    expect(await overflow()).toBeLessThanOrEqual(1);
    const rows = page.locator('.history .rows');
    const delta = await rows.evaluate(el => el.scrollHeight - el.clientHeight);
    if (fixture === "short") expect(delta).toBeLessThanOrEqual(1);
    else {
      expect(delta).toBeGreaterThan(100);
      await rows.locator('.row').last().scrollIntoViewIfNeeded();
      await expect(rows.locator('.row').last()).toBeInViewport();
    }
    await page.screenshot({ path: info.outputPath('history-wide.png') });
    await page.setViewportSize({ width: 860, height: 560 });
    await page.locator('.history .boundary footer').scrollIntoViewIfNeeded();
    await expect(page.locator('.history .boundary footer')).toBeInViewport();
    expect(await overflow()).toBeLessThanOrEqual(1);
    expect(await page.locator('.content').evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
    await page.screenshot({ path: info.outputPath('history-small.png') });
  });
}

test('agent activity separates recorded controls and preserves output reading position', async ({ page }, info) => {
  await completeOnboarding(page, { "pytxo-preview-layout-fixture-v1": "long" });
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto('/#/work');
  await page.locator('.ledger').getByRole('button', { name: /architect/ }).click();
  const panel = page.locator('.dock-panel:visible');
  const output = panel.getByRole('region', { name: 'Recorded agent output', exact: true });
  await expect(output).toContainText('worker prose 78');
  await output.focus();
  await page.keyboard.press('Control+Home');
  await expect(panel.getByRole('button', { name: 'Follow output', exact: true })).toBeVisible();
  await expect.poll(() => output.evaluate(el => el.scrollTop)).toBe(0);
  const position = await output.evaluate(el => el.scrollTop);
  // Cross a real poll interval: polling must not pull the reader back down.
  await page.waitForTimeout(2300);
  expect(await output.evaluate(el => el.scrollTop)).toBe(position);
  await panel.getByRole('button', { name: 'Follow output', exact: true }).click();
  await expect.poll(() => output.evaluate(el => el.scrollHeight - el.clientHeight - el.scrollTop)).toBeLessThanOrEqual(2);
  await panel.getByRole('button', { name: 'Activity', exact: true }).click();
  const activity = panel.getByLabel('Recorded agent activity', { exact: true });
  await expect(activity).toContainText('agent-start');
  await expect(activity).toContainText('verify-ok');
  await expect(activity).not.toContainText('worker prose');
  await expect(panel.locator('.source')).toContainText('Run run-8f2c');
  await expect(panel.locator('header .chip')).toContainText('Completed');
  await page.screenshot({ path: info.outputPath('activity.png') });
});
