import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("a mixed-CLI run opens on the fleet board with each worker's vendor, output and shared-path order", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-fleet-v1": "1" });
  await page.goto("/#/work");

  const board = page.getByTestId("fleet-board");
  await expect(board).toBeVisible();
  await expect(page.getByRole("button", { name: "Fleet" })).toHaveAttribute("aria-pressed", "true");

  for (const vendor of ["Claude Code", "Cursor Agent", "OpenCode", "Antigravity"]) {
    await expect(board.locator(".worker strong", { hasText: vendor })).toHaveCount(1);
  }
  await expect(board.locator(".worker strong", { hasText: "OpenAI Codex" })).toHaveCount(2);

  await expect(board.getByRole("log", { name: "Recent output from Claude Code" })).toContainText("Update(src/style.css)");
  await expect(board.getByRole("log", { name: "Recent output from OpenAI Codex" }).first()).toContainText("Task checks passed");
  await expect(board).toContainText("Shares src/app.js with Claude Code.");
  await expect(board.locator(".facts")).toContainText("1 task ordered for shared paths");

  await page.getByRole("button", { name: "Canvas" }).click();
  await expect(board).toHaveCount(0);
  await page.getByRole("button", { name: "Fleet" }).click();
  await expect(page.getByTestId("fleet-board")).toBeVisible();
});

test("a single-CLI run keeps the canvas and offers no fleet board", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/work");
  await expect(page.getByRole("button", { name: "Canvas" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Fleet" })).toHaveCount(0);
  await expect(page.getByTestId("fleet-board")).toHaveCount(0);
});
