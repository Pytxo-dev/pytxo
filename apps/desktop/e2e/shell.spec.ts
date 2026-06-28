import { test, expect } from "@playwright/test";

test.describe("Reality Deck shell", () => {
  test("setup wizard renders on fresh profile", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.removeItem("pytxo-deck-setup-v1");
    });
    await page.goto("/");
    await expect(page.getByText("Welcome to Reality Deck")).toBeVisible();
    await expect(page.getByRole("button", { name: "Get started" })).toBeVisible();
  });

  test("dashboard renders after setup complete", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem("pytxo-deck-setup-v1", "complete");
    });
    await page.goto("/");
    await expect(page.getByText("Reality Deck")).toBeVisible();
    await expect(page.getByText("Structural topology")).toBeVisible();
  });
});
