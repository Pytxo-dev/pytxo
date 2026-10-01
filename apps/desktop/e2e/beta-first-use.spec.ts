import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

for (const width of [1280, 860]) {
  test(`beta setup leads with one clear agent at ${width}`, async ({ page }, info) => {
    await page.setViewportSize({ width, height: 800 });
    await completeOnboarding(page);
    await page.goto("/#/setup");
    await page.getByRole("button", { name: "Agents & permissions", exact: true }).click();
    const codex = page.getByRole("group", { name: "OpenAI Codex", exact: true });
    await expect(codex).toBeVisible();
    await expect(page.locator(".agent-row:visible")).toHaveCount(5);
    await expect(codex.getByText("Installation", { exact: true })).toBeVisible();
    await expect(codex.getByText("Agent account", { exact: true })).toBeVisible();
    await expect(codex.getByRole("button", { name: "Use in new work" })).toBeInViewport();
    await expect(page.getByText("Orbit for new folders", { exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Supernova Full host privileges", exact: true })).toBeHidden();
    expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
    await page.screenshot({ path: info.outputPath("beta-agents.png") });
    await codex.getByRole("button", { name: "Use in new work" }).click();
    const picker = page.getByLabel("Agent CLI", { exact: true });
    await expect(picker).toHaveValue("codex");
    await expect(picker.locator('optgroup[label="Beta agents"] option')).toHaveCount(5);
    await expect(picker.locator('optgroup[label="Additional agents"] option')).not.toHaveCount(0);
  });
}

test("existing non-default permissions remain visible and unchanged", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-default-permission-profile-v1": "galaxy" });
  await page.goto("/#/setup");
  await page.getByRole("button", { name: "Agents & permissions", exact: true }).click();
  await expect(page.getByText("Galaxy for new folders", { exact: true })).toBeVisible();
  await page.getByText("Change default permissions", { exact: true }).click();
  await expect(page.getByRole("button", { name: "Galaxy Host tools + HITL", exact: true })).toHaveAttribute("aria-pressed", "true");
  expect(await page.evaluate(() => localStorage.getItem("pytxo-default-permission-profile-v1"))).toBe("galaxy");
});
