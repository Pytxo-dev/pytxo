import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("experimental Routing account stays out of Settings unless it is available or already connected", async ({ page }, info) => {
  await completeOnboarding(page);
  await page.goto("/#/settings");
  await page.getByRole("button", { name: "Account", exact: true }).click();
  await expect(page.locator(".setting-row").filter({ hasText: "Experimental Routing account" })).toHaveCount(0);
  const account = page.getByTestId("account-status");
  await expect(account).toContainText("Optional. Everything local works without an account");
  await expect(account.getByRole("button", { name: "Sign in" })).toBeEnabled();
  await expect(page.getByTestId("account-perks")).toContainText("Share a receipt as a public page");
  await expect(page.getByText("Tier", { exact: true })).toHaveCount(0);
  await account.scrollIntoViewIfNeeded();
  await page.screenshot({ path: info.outputPath("account-signed-out.png") });
});
