import { test, expect } from "@playwright/test";

test.describe("Reality Deck shell", () => {
  test("marketing title renders in dev build", async ({ page }) => {
    await page.goto("/");
    await expect(page.locator("h1")).toContainText("Pytxo Reality Deck");
  });
});
