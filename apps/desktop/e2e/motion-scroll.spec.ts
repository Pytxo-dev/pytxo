import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

async function review(page: import("@playwright/test").Page) {
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-71ad"]').click();
  await page.getByRole("button", { name: "Review prepared changes", exact: true }).click();
  await expect(page.locator("#run-review-title")).toBeVisible();
}

for (const viewport of [{ width: 1280, height: 800 }, { width: 960, height: 640 }]) {
  test(`wheel reaches lower exact changes at ${viewport.width}x${viewport.height}`, async ({ page }) => {
    await page.setViewportSize(viewport);
    await completeOnboarding(page, { "pytxo-preview-review-substantial-v1": "true" });
    await review(page);
    const route = page.locator(".mission-content > .content");
    const content = page.locator(".review-scroll");
    const target = page.getByRole("button", { name: "Inspect exact content for assets/signal-mark.bin" });
    const box = await content.boundingBox();
    expect(box).not.toBeNull();
    expect(box!.y + box!.height).toBeLessThanOrEqual(viewport.height + 1);
    expect(await content.evaluate(e => e.scrollHeight - e.clientHeight)).toBeGreaterThan(100);
    expect(await route.evaluate(e => e.scrollHeight - e.clientHeight)).toBeLessThanOrEqual(1);
    const lowerEvidence = page.locator(".technical-evidence > summary");
    await expect(lowerEvidence).not.toBeInViewport();
    const chromeBefore = await page.locator(".app-bar").boundingBox();
    expect(chromeBefore).not.toBeNull();
    // Stay in the page gutter so nested code panes cannot consume the wheel.
    await page.mouse.move(box!.x + 7, box!.y + 100);
    await page.mouse.wheel(0, 480);
    await expect.poll(() => content.evaluate(e => e.scrollTop)).toBeGreaterThan(0);
    const wheelSamples = [];
    for (let i = 0; i < 32; i++) {
      const bounds = await lowerEvidence.boundingBox();
      wheelSamples.push({ bounds, scrollTop: await content.evaluate(e => e.scrollTop) });
      if (bounds && bounds.y >= box!.y && bounds.y + bounds.height <= box!.y + box!.height) break;
      await page.mouse.wheel(0, 300);
      await page.waitForTimeout(30);
    }
    await test.info().attach("wheel-samples", { body: JSON.stringify({ box, wheelSamples, target: await target.boundingBox(), viewport }), contentType: "application/json" });
    await page.screenshot({ path: test.info().outputPath("wheel-result.png") });
    const evidenceBounds = (await lowerEvidence.boundingBox())!;
    expect(evidenceBounds.y).toBeGreaterThanOrEqual(box!.y - 1);
    expect(evidenceBounds.y + evidenceBounds.height).toBeLessThanOrEqual(box!.y + box!.height + 1);
    expect(await page.locator(".app-bar").boundingBox()).toEqual(chromeBefore);
    expect(await page.evaluate(() => [document.documentElement, document.body, document.querySelector("#app")!, document.querySelector(".deck-shell__body")!, document.querySelector(".mission-content > .content")!].map(e => e.scrollTop))).toEqual([0, 0, 0, 0, 0]);
    await page.mouse.move(box!.x + 7, box!.y + 100);
    await page.mouse.wheel(0, -10000);
    await expect.poll(() => content.evaluate(e => e.scrollTop)).toBe(0);
    await expect(page.locator("#run-review-title")).toBeInViewport();
    await target.click();
    await expect(page.locator(".binary-content")).toContainText("00 ff 50 4e 47");
  });
}

test("motion override persists, applies to the document and respects the OS preference", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-review-state-v1": "preparing" });
  await page.emulateMedia({ reducedMotion: "no-preference" });
  await page.goto("/#/settings");
  await page.getByRole("button", { name: "General", exact: true }).click();
  const toggle = page.getByRole("button", { name: "Reduced motion", exact: true });
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("html")).toHaveAttribute("data-force-reduced-motion", "");
  await page.reload();
  await expect(toggle).toHaveAttribute("aria-pressed", "true");
  await expect(page.locator("html")).toHaveAttribute("data-force-reduced-motion", "");
  await review(page);
  await expect(page.getByLabel("Preparing immutable review")).toHaveCSS("animation-name", "none");
  await page.emulateMedia({ reducedMotion: "reduce" });
  await page.getByRole("link", { name: "Setup", exact: true }).click();
  await page.getByRole("button", { name: "General", exact: true }).click();
  await toggle.click();
  await expect(toggle).toHaveAttribute("aria-pressed", "false");
  await expect(page.locator("html")).not.toHaveAttribute("data-force-reduced-motion", "");
  await review(page);
  await expect(page.getByLabel("Preparing immutable review")).toHaveCSS("animation-name", "none");
});

test("display choices expose selection and respond to keyboard input", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/settings");
  await page.getByRole("button", { name: "Appearance", exact: true }).click();
  const comfortable = page.getByRole("button", { name: "Comfortable", exact: true });
  await comfortable.focus();
  await page.keyboard.press("Enter");
  await expect(comfortable).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByRole("button", { name: "Compact", exact: true })).toHaveAttribute("aria-pressed", "false");
  const scale = page.getByRole("button", { name: "110%", exact: true });
  await scale.focus();
  await page.keyboard.press("Enter");
  await expect(scale).toHaveAttribute("aria-pressed", "true");
  await expect(page.getByRole("button", { name: "100%", exact: true })).toHaveAttribute("aria-pressed", "false");
});
