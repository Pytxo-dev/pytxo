import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("a mixed-CLI run opens on the fleet board with each worker's vendor, output and shared-path order", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-fleet-v1": "1" });
  await page.goto("/#/work");

  const board = page.getByTestId("fleet-board");
  await expect(board).toBeVisible();
  await expect(page.getByRole("button", { name: "Fleet" })).toHaveAttribute("aria-pressed", "true");

  for (const vendor of ["Claude Code", "Cursor Agent", "OpenCode", "Antigravity"]) {
    await expect(board.locator(".worker .vendor", { hasText: vendor })).toHaveCount(1);
  }
  await expect(board.locator(".worker .vendor", { hasText: "OpenAI Codex" })).toHaveCount(2);

  await expect(board.getByRole("log", { name: "Recent output from Claude Code" })).toContainText("Update(");
  await expect(board.getByRole("log", { name: "Recent output from OpenAI Codex" }).first()).toContainText("Task checks passed");
  await expect(board).toContainText("Shares src/app.js with Claude Code.");
  await expect(board.locator(".facts")).toContainText("1 task ordered for shared paths");

  // The monitor lists every task with its CLI and state; queued tasks have no output pane yet.
  const monitor = board.getByRole("list", { name: "Worker activity from recorded events" });
  await expect(monitor.getByRole("listitem")).toHaveCount(6);
  await expect(monitor.getByRole("button", { name: /^1\. OpenAI Codex, Add search and status filtering.*: passed/ })).toBeEnabled();
  await expect(monitor.getByRole("button", { name: /^6\. Antigravity, Document the new features.*: queued/ })).toBeDisabled();
  await expect(board.locator(".worker.compact")).toHaveCount(2);
  await expect(board.getByRole("log")).toHaveCount(4);

  await page.getByRole("button", { name: "Canvas" }).click();
  await expect(board).toHaveCount(0);
  await page.getByRole("button", { name: "Fleet" }).click();
  await expect(page.getByTestId("fleet-board")).toBeVisible();
});

test("a single-CLI run opens on the canvas and still offers the fleet board", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/work");
  await expect(page.getByRole("button", { name: "Canvas" })).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByTestId("fleet-board")).toHaveCount(0);
  await page.getByRole("button", { name: "Fleet" }).click();
  await expect(page.getByTestId("fleet-board")).toBeVisible();
});

test("dragging from a worker pans the canvas without selecting it", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/work");
  const node = page.locator(".task-node").filter({ hasText: "tests" }).locator("button");
  const scene = page.locator(".scene");
  const before = await scene.evaluate((el) => (el as HTMLElement).style.transform);
  const box = (await node.boundingBox())!;
  await page.mouse.move(box.x + 20, box.y + 20);
  await page.mouse.down();
  await page.mouse.move(box.x - 100, box.y + 40, { steps: 8 });
  await page.mouse.up();
  expect(await scene.evaluate((el) => (el as HTMLElement).style.transform)).not.toBe(before);
  expect(await page.locator(".canvas-grid").evaluate((el) => (el as HTMLElement).style.backgroundPosition)).not.toBe("");
  await expect(page.locator(".task-node.chosen")).not.toContainText("tests");
});

test("number keys jump to that worker's pane and leave text fields alone", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 720 });
  await completeOnboarding(page, { "pytxo-preview-fleet-v1": "1" });
  await page.goto("/#/work");
  const board = page.getByTestId("fleet-board");
  await expect(board).toBeVisible();
  await expect(board.getByRole("button", { name: /^6\. Antigravity/ })).toHaveAttribute("aria-keyshortcuts", "6");

  await page.keyboard.press("6");
  const queued = board.locator(".worker").last();
  await expect(queued).toHaveClass(/jumped/);
  await expect(queued).toBeInViewport();

  await page.keyboard.press("1");
  await expect(board.locator(".worker").first().locator("button.head")).toBeFocused();

  await page.keyboard.press("Control+k");
  const field = page.getByRole("combobox", { name: "Command search" });
  await field.pressSequentially("3");
  await expect(field).toHaveValue("3");
  await expect(board.locator(".worker.jumped")).toHaveCount(1);
});
