import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("Review shows changed lines first and remembers the exact before/after view", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-review-state-v1": "ready", "pytxo-preview-candidate-check-v1": "passed" });
  await page.goto("/#/run-review");
  const view = page.getByRole("group", { name: "Comparison view" });
  const changes = page.getByRole("region", { name: "Changes in crates/pytxo-signal/src/lib.rs" });
  await expect(changes.locator(".diff-line.del")).toContainText("source.to_owned()");
  await expect(changes.locator(".diff-line.add")).toContainText("parse(source).structural_skeleton()");
  await expect(page.locator(".diff-count")).toHaveText("+1 −1");
  await expect(page.locator(".exact-diff")).toHaveCount(0);

  await view.getByRole("button", { name: "Before & after" }).click();
  await expect(page.locator(".diff-side.after .text-content")).toContainText("structural_skeleton()");
  await page.reload();
  await expect(view.getByRole("button", { name: "Before & after" })).toHaveAttribute("aria-pressed", "true");
  await view.getByRole("button", { name: "Changes" }).click();
  await expect(changes).toBeVisible();

  // An added file is all additions; a binary file falls back to its exact bytes.
  await page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/tests/skeleton.rs" }).click();
  await expect(page.getByRole("region", { name: "Changes in crates/pytxo-signal/tests/skeleton.rs" }).locator(".diff-line.del")).toHaveCount(0);
  await page.getByRole("button", { name: "Inspect exact content for assets/signal-mark.bin" }).click();
  await expect(page.locator(".binary-content")).toBeVisible();
});
