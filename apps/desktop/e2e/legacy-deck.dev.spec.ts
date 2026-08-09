import { expect, test } from "@playwright/test";

import { completeOnboarding } from "./helpers";

test("both developer flags activate the legacy Deck in development", async ({ page }) => {
  await completeOnboarding(page, {
    desktop_shell_v1: "true",
    "pytxo-developer-deck-v1": "true",
  });
  await page.goto("/");
  await expect(page.getByRole("navigation", { name: "Open workspaces" })).toBeVisible();
});
