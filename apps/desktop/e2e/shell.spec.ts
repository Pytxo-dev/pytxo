import { test, expect } from "@playwright/test";

test.describe("Pytxo Desktop shell", () => {
  test("setup wizard renders on fresh profile", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.removeItem("pytxo-deck-setup-v1");
      localStorage.removeItem("pytxo-deck-tabs-v1");
    });
    await page.goto("/");
    await expect(page.getByText("Welcome to Pytxo Desktop")).toBeVisible();
    await expect(page.getByRole("button", { name: "Get started" })).toBeVisible();
    await expect(page.getByLabel("Setup progress")).toBeVisible();
  });

  test("setup skip-all reaches ready step", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.removeItem("pytxo-deck-setup-v1");
      localStorage.removeItem("pytxo-deck-tabs-v1");
    });
    await page.goto("/");
    await page.getByRole("button", { name: "Get started" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await expect(page.getByRole("heading", { name: "You're ready" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Enter Pytxo Desktop" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Back" })).toBeVisible();
  });

  test("workspace home shares tab chrome after setup", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem("pytxo-deck-setup-v1", "complete");
      localStorage.removeItem("pytxo-deck-tabs-v1");
    });
    await page.goto("/");
    await expect(page.getByText("Pytxo Desktop")).toBeVisible();
    await expect(page.getByRole("navigation", { name: "Open workspaces" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Workspaces" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Open folder" })).toBeVisible();
  });

  test("project tabs bar is visible when a workspace tab exists", async ({ page }) => {
    await page.addInitScript(() => {
      localStorage.setItem("pytxo-deck-setup-v1", "complete");
      localStorage.setItem(
        "pytxo-deck-tabs-v1",
        JSON.stringify({
          tabs: [
            {
              id: "tab-1",
              domainId: "/tmp/demo",
              label: "demo",
              projectId: null,
              selectedRunId: null,
              selectedAgentId: null,
              dispatchRepo: "",
            },
          ],
          activeTabId: "tab-1",
        }),
      );
    });
    await page.goto("/");
    await expect(page.getByRole("navigation", { name: "Open workspaces" })).toBeVisible();
  });
});
