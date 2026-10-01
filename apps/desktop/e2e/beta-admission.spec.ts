import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("additional agent remains selectable but cannot start a Desktop beta run", async ({ page }, info) => {
  await completeOnboarding(page);
  await page.goto("/#/flow");
  await page.getByLabel("What should Pytxo do?").fill("Fix src/parser.rs and add a regression test for empty input.");
  const picker = page.getByLabel("Agent CLI", { exact: true });
  await picker.selectOption("claude");
  await page.getByRole("button", { name: "Build plan", exact: true }).first().click();
  await expect(page.getByText("Desktop Beta runs Codex. Select Codex and build a new plan.", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Run", exact: true })).toBeDisabled();
  await expect(picker).toHaveValue("claude");
  await page.getByRole("button", { name: "Run", exact: true }).scrollIntoViewIfNeeded();
  await page.screenshot({ path: info.outputPath("beta-admission-blocked.png") });
  await picker.selectOption("codex");
  await expect(page.getByRole("button", { name: "Run", exact: true })).toBeDisabled();
  await page.getByRole("button", { name: "Build plan", exact: true }).first().click();
  await expect(page.getByRole("button", { name: "Run", exact: true })).toBeEnabled();
});
