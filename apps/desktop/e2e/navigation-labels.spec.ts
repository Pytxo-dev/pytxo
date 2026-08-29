import { expect, test } from "@playwright/test";
import { ROUTE_LABELS, type CanonicalRoute } from "../src/lib/navigation.svelte";
import { completeOnboarding } from "./helpers";

const ROUTES = Object.keys(ROUTE_LABELS) as CanonicalRoute[];

test.describe("Navigation labels", () => {
  test.beforeEach(async ({ page }) => {
    await completeOnboarding(page);
  });

  // Trunk test: a nav item that reads "Ops" while its screen says "Operations"
  // teaches an operator that labels are approximate.
  test("the nav label equals the page title for every destination", async ({ page }) => {
    for (const route of ROUTES) {
      await page.goto(`/#/${route}`);
      const label = ROUTE_LABELS[route];

      await expect(
        page.locator(`aside a[href="#/${route}"]`),
        `nav item for ${route}`,
      ).toHaveText(label);

      await expect(
        page.getByRole("heading", { level: 1, name: label, exact: true }),
        `page title for ${route}`,
      ).toBeVisible();
    }
  });

  test("exactly three destinations are offered", async ({ page }) => {
    await page.goto("/#/work");
    await expect(page.locator("aside nav a")).toHaveCount(ROUTES.length);
  });

  test("aria-current marks the active destination", async ({ page }) => {
    await page.goto("/#/history");
    await expect(page.locator('aside a[href="#/history"]')).toHaveAttribute("aria-current", "page");
  });
});
