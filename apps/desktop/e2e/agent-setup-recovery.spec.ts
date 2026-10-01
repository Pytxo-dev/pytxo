import { expect, test } from "@playwright/test";
import { clearOnboarding, completeOnboarding } from "./helpers";

test("failed onboarding recheck withdraws previously reported readiness", async ({ page }, info) => {
  await clearOnboarding(page);
  await page.goto("/");
  await page.getByRole("button", { name: "Get started" }).click();
  await expect(page.getByText("ChatGPT connected", { exact: true })).toBeVisible();
  await page.evaluate(() => localStorage.setItem("pytxo-preview-ade-state-v1", "detection-error"));
  await page.getByRole("button", { name: "Check again", exact: true }).click();
  await expect(page.getByRole("alert")).toContainText("Agent detection failed");
  await expect(page.getByText("ChatGPT connected", { exact: true })).toBeHidden();
  await expect(page.getByRole("button", { name: "Continue", exact: true })).toBeHidden();
  await expect(page.getByRole("button", { name: "Set up agents later", exact: true })).toBeVisible();
  await page.screenshot({ path: info.outputPath("detection-error.png") });
  await page.evaluate(() => localStorage.setItem("pytxo-preview-ade-state-v1", "codex-only"));
  await page.getByRole("button", { name: "Check again", exact: true }).click();
  await expect(page.getByRole("alert")).toHaveCount(0);
  await expect(page.getByText("ChatGPT connected", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Continue", exact: true })).toBeEnabled();
});

test("onboarding can defer agent setup without claiming an agent is ready", async ({ page }, info) => {
  await page.setViewportSize({ width: 860, height: 560 });
  await clearOnboarding(page);
  await page.addInitScript(() => localStorage.setItem("pytxo-preview-ade-state-v1", "no-agents"));
  await page.goto("/");
  await page.getByRole("button", { name: "Get started" }).click();
  await expect(page.locator(".agent").first()).toContainText("OpenAI Codex");
  await expect(page.locator(".agent").first().getByRole("link", { name: "Install guide" })).toBeVisible();
  await expect(page.locator(".summary[role=status]")).toContainText("0 installed · 0 available");
  const skip = page.getByRole("button", { name: "Set up agents later", exact: true });
  await expect(skip).toBeInViewport();
  await page.screenshot({ path: info.outputPath("no-agent-compact.png") });
  await skip.click();
  await expect(page.getByRole("button", { name: "Try the guided example" })).toBeVisible();
});

for (const state of ["no-agents", "codex-signed-out", "codex-unknown"]) {
  test(`Setup provides recovery without readiness for ${state}`, async ({ page }) => {
    await completeOnboarding(page, { "pytxo-preview-ade-state-v1": state });
    await page.goto("/#/setup");
    await page.getByRole("button", { name: "Agents & permissions", exact: true }).click();
    const codex = page.getByRole("group", { name: "OpenAI Codex", exact: true });
    await expect(codex).toBeVisible();
    await expect(codex.getByRole("button", { name: "Use in new work" })).toHaveCount(0);
    if (state === "no-agents") {
      await expect(codex.getByRole("link", { name: "Install guide" })).toBeVisible();
      await expect(codex.getByRole("button", { name: "Connect with ChatGPT" })).toHaveCount(0);
    } else {
      await page.evaluate(() => localStorage.setItem("pytxo-preview-login-error-v1", "true"));
      await codex.getByRole("button", { name: "Connect with ChatGPT" }).click();
      await expect(page.getByRole("alert")).toContainText("Could not open vendor sign-in");
      await page.evaluate(() => localStorage.removeItem("pytxo-preview-login-error-v1"));
      await codex.getByRole("button", { name: "Connect with ChatGPT" }).click();
      await expect(page.getByRole("status").filter({ hasText: "sign-in opened" })).toBeVisible();
      await expect(codex.getByRole("button", { name: "Use in new work" })).toHaveCount(0);
    }
    await page.evaluate(() => localStorage.setItem("pytxo-preview-ade-state-v1", "codex-only"));
    await page.getByRole("button", { name: "Recheck all" }).click();
    await expect(codex.getByText("ChatGPT connected", { exact: true })).toBeVisible();
    await codex.getByRole("button", { name: "Use in new work" }).click();
    await expect(page.getByLabel("Agent CLI", { exact: true })).toHaveValue("codex");
  });
}
