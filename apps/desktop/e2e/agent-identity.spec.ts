import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("agent inspection uses recorded launcher and folder, never the task alias", async ({ page }, info) => {
  await completeOnboarding(page, { "pytxo-preview-agent-identity-v1": "1" });
  await page.goto("/#/work");
  await page.locator('.ledger').getByRole('button', { name: /architect/ }).click();
  const panel = page.locator('.dock-panel:visible');
  await expect(panel.getByLabel('Recorded launcher', { exact: true })).toHaveText('OpenAI Codex');
  await panel.getByRole('button', { name: 'Task & receipt', exact: true }).click();
  await expect(panel.locator('.facts')).toContainText('C:/browser-fixture/isolated/worker');
  await expect(panel.locator('.facts')).toContainText('OpenAI Codex');
  await page.screenshot({ path: info.outputPath('recorded-identity.png') });
  await page.locator('.ledger').getByRole('button', { name: /desktop/ }).click();
  await expect(panel.getByLabel('Recorded launcher', { exact: true })).toHaveCount(0);
  await panel.getByRole('button', { name: 'Task & receipt', exact: true }).click();
  await expect(panel.locator('.facts')).toContainText('Not identified');
  await expect(panel.locator('.facts')).toContainText('Not recorded');
  await expect(panel.locator('.facts')).not.toContainText('OpenAI Codex');
});
