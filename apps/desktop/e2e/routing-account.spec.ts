import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test("experimental Routing account does not offer a dead Connect action by default", async ({ page }, info) => {
  await completeOnboarding(page);
  await page.goto("/#/settings");
  await page.getByRole("button", { name: "Account & billing" }).click();
  const routing = page.locator(".setting-row").filter({ hasText: "Experimental Routing account" });
  await expect(routing).toContainText("The account bridge is off in this build");
  await expect(routing.getByRole("button", { name: "Connect" })).toBeDisabled();
  await routing.scrollIntoViewIfNeeded();
  await page.screenshot({ path: info.outputPath("routing-bridge-off.png") });
});
