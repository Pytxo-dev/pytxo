import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

const REQUEST = "Add search and status filtering with tests, a dark theme that follows the system, and a Spanish translation.";

test("a ready agent splits one request into owned task lines that plan as separate tasks, and Undo restores it", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-split-delay-v1": "300" });
  await page.goto("/#/flow");
  const request = page.getByLabel("What should Pytxo do?");
  await request.fill(REQUEST);
  await page.getByRole("button", { name: "Split with OpenAI Codex" }).click();
  await expect(page.getByText("OpenAI Codex proposed 4 tasks, each owning its files.")).toBeVisible();
  await expect(request).toHaveValue(/\| files: src\/model\.mjs\n/);

  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await expect(page.locator(".plan-summary")).toContainText("4 tasks");
  await expect(page.getByText("src/i18n/es.json")).toBeVisible();

  await page.getByRole("button", { name: "Undo" }).click();
  await expect(request).toHaveValue(REQUEST);
});

test("cancelling a split keeps the request and explains it", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-split-delay-v1": "20000" });
  await page.goto("/#/flow");
  const request = page.getByLabel("What should Pytxo do?");
  await request.fill(REQUEST);
  await page.getByRole("button", { name: "Split with OpenAI Codex" }).click();
  await expect(page.getByText("OpenAI Codex is reading the project to split this job")).toBeVisible();
  await expect(request).toHaveAttribute("readonly", "");
  await expect(page.getByRole("button", { name: "Build plan", exact: true })).toBeDisabled();
  await page.getByRole("button", { name: "Cancel" }).click();
  await expect(page.getByRole("alert")).toHaveText("Split cancelled. Your request was not changed.");
  await expect(request).toHaveValue(REQUEST);
  await expect(request).not.toHaveAttribute("readonly", "");
});
