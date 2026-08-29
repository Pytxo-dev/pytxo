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
        await expect(page.getByRole("heading", { level: 1, name: "Work" })).toBeVisible();

        await openAppearance(page);
        await setBrowserPreviewScale(page, userScale.label);
        await expectRootZoom(page, userScale.value);
        expect(await page.evaluate(() => window.devicePixelRatio)).toBe(deviceScaleFactor);

        await page.getByRole("link", { name: "Work" }).click();
        const pageOverflow = await rootOverflow(page);
        expect(pageOverflow.horizontal).toBeLessThanOrEqual(1);
        expect(pageOverflow.vertical).toBeLessThanOrEqual(1);

        await expect(page.getByRole("button", { name: "Minimize" })).toBeVisible();
        await expect(page.getByRole("button", { name: "Maximize" })).toBeVisible();
        await expect(page.getByRole("button", { name: "Close" })).toBeVisible();

        // The ledger row is the operator's primary target, so it has to stay in
        // the viewport at every DPI and zoom combination.
        const ledgerRow = page.locator(".ledger .row").first();
        await ledgerRow.scrollIntoViewIfNeeded();
        await expect(ledgerRow).toBeVisible();
        await expect(ledgerRow).toBeInViewport();

        const stop = page.getByRole("button", { name: "Stop", exact: true });
        await stop.scrollIntoViewIfNeeded();
        await expect(stop).toBeInViewport();
        await stop.click();

        const stopDialog = page.locator("dialog.confirm-dialog");
        await expect(stopDialog).toBeVisible();
        const actionsOverflow = await stopDialog.locator(".dialog-actions").evaluate(
          (element) => element.scrollWidth - element.clientWidth,
        );
        expect(actionsOverflow).toBeLessThanOrEqual(1);
        await expect(stopDialog.getByRole("button", { name: "Stop run" })).toBeVisible();
        await stopDialog.getByRole("button", { name: "Keep running" }).click();

        // Approvals are an overlay now, reachable from the title bar.
        await page.getByRole("button", { name: /^Approvals inbox/ }).click();
        const inbox = page.getByRole("dialog", { name: "Approvals inbox" });
        await expect(inbox).toBeVisible();
        await expect(inbox.getByRole("heading", { name: "Apply reviewed workspace changes" })).toBeVisible();

        const evidence = inbox.locator(".evidence");
        const actions = inbox.locator(".actions");
        for (const region of [evidence, actions]) {
          await region.scrollIntoViewIfNeeded();
          await expect(region).toBeVisible();
          await expect(region).toBeInViewport();
          expect(
            await region.evaluate((element) => element.scrollWidth - element.clientWidth),
          ).toBeLessThanOrEqual(1);
        }

        const reviewRun = inbox.getByRole("button", { name: "Review run" });
        await reviewRun.scrollIntoViewIfNeeded();
        await expect(reviewRun).toBeInViewport();
        const approveAndApply = inbox.getByRole("button", { name: /Approve and apply/ });
        await approveAndApply.scrollIntoViewIfNeeded();
        await expect(approveAndApply).toBeInViewport();
      });
    }
  });
}
