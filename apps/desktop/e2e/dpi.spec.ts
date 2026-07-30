import { test, expect } from "@playwright/test";
import {
  completeOnboarding,
  expectRootZoom,
  openAppearance,
  rootOverflow,
  setBrowserPreviewScale,
} from "./helpers";

const DEVICE_SCALE_FACTORS = [1, 1.25, 1.5] as const;
const USER_SCALES = [
  { label: "90%" as const, value: 0.9 },
  { label: "100%" as const, value: 1 },
  { label: "110%" as const, value: 1.1 },
] as const;

for (const deviceScaleFactor of DEVICE_SCALE_FACTORS) {
  test.describe(`DPI contract at device scale factor ${deviceScaleFactor}`, () => {
    test.use({ viewport: { width: 860, height: 560 }, deviceScaleFactor });

    for (const userScale of USER_SCALES) {
      test(`keeps ${userScale.label} user zoom independent from simulated DPI`, async ({ page }) => {
        await completeOnboarding(page);
        await page.goto("/");
        await expect(page.getByRole("heading", { name: "Ops" })).toBeVisible();

        await openAppearance(page);
        await setBrowserPreviewScale(page, userScale.label);
        await expectRootZoom(page, userScale.value);
        expect(await page.evaluate(() => window.devicePixelRatio)).toBe(deviceScaleFactor);

        await page.getByRole("link", { name: "Ops" }).click();
        const pageOverflow = await rootOverflow(page);
        expect(pageOverflow.horizontal).toBeLessThanOrEqual(1);
        expect(pageOverflow.vertical).toBeLessThanOrEqual(1);

        await expect(page.getByRole("button", { name: "Minimize" })).toBeVisible();
        await expect(page.getByRole("button", { name: "Maximize" })).toBeVisible();
        await expect(page.getByRole("button", { name: "Close" })).toBeVisible();

        const activeRun = page.locator(".run-row").filter({ hasText: "run-8f2c" });
        await activeRun.scrollIntoViewIfNeeded();
        await expect(activeRun).toBeVisible();
        await expect(activeRun).toBeInViewport();

        const stopReview = page.getByRole("button", { name: "Review stop for run run-8f2c" });
        await stopReview.scrollIntoViewIfNeeded();
        await expect(stopReview).toBeVisible();
        await expect(stopReview).toBeInViewport();
        await stopReview.click();

        const dialog = page.getByRole("dialog", { name: "Stop active run?" });
        await expect(dialog).toBeVisible();
        const footerOverflow = await dialog.locator("footer").evaluate(
          (element) => element.scrollWidth - element.clientWidth,
        );
        expect(footerOverflow).toBeLessThanOrEqual(1);
        await expect(dialog.getByRole("button", { name: "Stop run" })).toBeVisible();
        await dialog.getByRole("button", { name: "Keep running" }).click();

        await page.getByRole("link", { name: "Approvals" }).click();
        await expect(page.getByRole("heading", { name: "Flush Blast Shield workspace" })).toBeVisible();
        const evidence = page.locator(".decision-evidence");
        const actions = page.locator(".decision-actions");
        await evidence.scrollIntoViewIfNeeded();
        await expect(evidence).toBeVisible();
        await expect(evidence).toBeInViewport();
        await actions.scrollIntoViewIfNeeded();
        await expect(actions).toBeVisible();
        await expect(actions).toBeInViewport();
        for (const overflow of await Promise.all(
          [evidence, actions].map((locator) => locator.evaluate((element) => element.scrollWidth - element.clientWidth)),
        )) {
          expect(overflow).toBeLessThanOrEqual(1);
        }
        const reviewLatestRun = page.getByRole("button", { name: "Review latest run" });
        await reviewLatestRun.scrollIntoViewIfNeeded();
        await expect(reviewLatestRun).toBeVisible();
        await expect(reviewLatestRun).toBeInViewport();
        const approveAndFlush = page.getByRole("button", { name: "Approve & flush" });
        await approveAndFlush.scrollIntoViewIfNeeded();
        await expect(approveAndFlush).toBeVisible();
        await expect(approveAndFlush).toBeInViewport();
      });
    }
  });
}
