import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("New work assigns tasks across a mixed CLI team and requires a fresh plan after reassignment", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/flow");
  await page.getByLabel("What should Pytxo do?").fill("Update the Desktop flow and orchestration, then verify the contract");

  // Every ready agent joins by default; narrow the team to Codex + Claude Code.
  await expect(page.locator(".stepper output")).toHaveText("3");
  await page.getByRole("checkbox", { name: "Cursor Agent" }).uncheck();
  await expect(page.locator(".stepper output")).toHaveText("2");

  await page.getByRole("button", { name: "Build plan" }).click();
  await expect(page.getByLabel("Agent for desktop-flow")).toHaveValue("codex");
  await expect(page.getByLabel("Agent for orchestration-flow")).toHaveValue("claude");
  await expect(page.locator(".plan-summary")).toContainText("OpenAI Codex + Claude Code");

  await page.getByLabel("Agent for contract-tests").selectOption("claude");
  await expect(page.getByText("Plan is stale")).toBeVisible();
  await page.getByRole("button", { name: "Build plan" }).click();
  await expect(page.getByLabel("Agent for contract-tests")).toHaveValue("claude");
  await expect(page.getByText("Plan is stale")).toHaveCount(0);
});

test("the worker stepper stays within one to eight", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/flow");
  await page.getByRole("button", { name: "Lead only" }).click();
  const fewer = page.getByRole("button", { name: "Fewer workers at once" });
  const more = page.getByRole("button", { name: "More workers at once" });
  await expect(fewer).toBeDisabled();
  for (let i = 0; i < 7; i += 1) await more.click();
  await expect(page.locator(".stepper output")).toHaveText("8");
  await expect(more).toBeDisabled();
});
