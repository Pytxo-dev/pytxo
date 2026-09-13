import { expect, test, type Page } from "@playwright/test";
import { writeFile } from "node:fs/promises";
import { completeOnboarding } from "./helpers";

async function openReview(page: Page) {
  await page.goto("/#/history");
  await page.locator(".history .row", { hasText: "run-71ad" }).first().click();
  await page.getByRole("button", { name: "Review changes", exact: true }).click();
  await expect(page.locator(".diff-side.after .text-content")).toContainText("structural_skeleton()");
}

test("ownership identifies recorded workers instead of planned CLI labels", async ({ page }) => {
  await completeOnboarding(page);
  await openReview(page);
  const ownership = page.locator(".task-list");
  await expect(ownership.locator("div", { has: page.getByText("signal-core", { exact: true }) }).locator("small"))
    .toHaveText("agent-0 · crates/pytxo-signal/src/lib.rs");
  await expect(ownership.locator("div", { has: page.getByText("contract-tests", { exact: true }) }).locator("small"))
    .toHaveText("agent-1 · crates/pytxo-signal/tests/skeleton.rs");
});

test("ownership stays unknown when worker records are missing", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-agent-verification-v1": "missing" });
  await openReview(page);
  const labels = page.locator(".task-list small");
  await expect(labels).toHaveCount(2);
  for (const label of await labels.all()) await expect(label).toContainText("Worker not recorded");
  await expect(page.getByRole("button", { name: "Apply reviewed changes", exact: true })).toBeDisabled();
});

test("unverified combined changes make verification the next action", async ({ page }) => {
  await completeOnboarding(page);
  await openReview(page);
  await expect(page.getByText("Verify before Apply", { exact: true })).toBeVisible();
  const verify = page.getByRole("button", { name: "Verify candidate", exact: true });
  await expect(verify).toBeEnabled();
  await expect(page.getByRole("button", { name: "Apply reviewed changes", exact: true })).toBeDisabled();
  await verify.click();
  await expect(page.getByText("Combined checks: passed", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Apply reviewed changes", exact: true })).toBeEnabled();
});

test("hierarchy acceptance keeps changes and verification in the first viewport", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await completeOnboarding(page);
  await openReview(page);
  await expect(page.getByRole("heading", { name: "Prepared changes", exact: true })).toBeInViewport();
  await expect(page.locator(".diff-side.after .text-content")).toBeInViewport({ ratio: 1 });
  const decision = page.getByRole("region", { name: "Review decision" });
  await expect(decision.getByText("Combined checks: not verified", { exact: true })).toBeInViewport();
  await expect(decision.getByRole("button", { name: "Apply reviewed changes", exact: true })).toBeInViewport();
  await expect(decision.getByText("pkg-71ad-immutable", { exact: true })).toBeInViewport();
});

test("hierarchy acceptance exposes the Work review action without scrolling", async ({ page }) => {
  await page.setViewportSize({ width: 960, height: 800 });
  await completeOnboarding(page);
  await page.goto("/#/work");
  const review = page.getByRole("region", { name: "Focused run" }).getByRole("button", { name: "Review changes", exact: true });
  await expect(review).toBeInViewport();
  await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeInViewport();
  await openReview(page);
  await page.getByRole("button", { name: "Back to runs", exact: true }).click();
  await page.getByRole("link", { name: "Work", exact: true }).click();
  await expect(review).toBeEnabled();
  await review.click();
  await expect(page.getByRole("heading", { name: "Run Review", exact: true })).toBeVisible();
});

test("mobile review paths and ownership remain readable beside their action", async ({ page }) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await completeOnboarding(page, { "pytxo-deck-theme": "light" });
  await openReview(page);
  for (const header of await page.locator(".file-row > header").all()) {
    const labels = header.locator("div");
    expect(await labels.evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
    const labelBox = await labels.boundingBox();
    const actionBox = await header.getByRole("button").boundingBox();
    expect(labelBox!.y).toBeGreaterThanOrEqual(actionBox!.y + actionBox!.height);
    await expect(labels.locator("strong")).toHaveCSS("white-space", "normal");
  }
  expect(await page.locator(".mission-content > .content").evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
});

for (const width of [1280, 390]) {
  test(`supporting review evidence stays fully readable at ${width}`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 844 });
    await completeOnboarding(page, { "pytxo-preview-candidate-check-v1": "passed" });
    await openReview(page);
    const details = page.locator(".digest-details").first();
    await details.locator("summary").click();
    for (const value of await details.locator("dd").all()) {
      await expect(value).toHaveCSS("white-space", "normal");
      expect(await value.evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
    }
    await details.scrollIntoViewIfNeeded();
    await page.screenshot({ path: testInfo.outputPath("expanded-digests.png") });
    // Layout stress only: the normal preview uses short illustrative digests.
    // These synthetic 64-character strings do not claim backend SHA-256 validation.
    for (const value of (await details.locator("dd").all()).slice(0, 2)) {
      await value.evaluate(el => { el.textContent = "0123456789abcdef".repeat(4); });
      expect(await value.evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
      await expect(value).toHaveText("0123456789abcdef".repeat(4));
    }
    await details.scrollIntoViewIfNeeded();
    await page.screenshot({ path: testInfo.outputPath("expanded-digests-synthetic-64.png") });
    const ownership = page.locator(".task-list");
    for (const label of await ownership.locator("strong, small, em").all()) {
      await expect(label).toHaveCSS("white-space", "normal");
      expect(await label.evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
    }
    await ownership.scrollIntoViewIfNeeded();
    await page.screenshot({ path: testInfo.outputPath("ownership.png") });
    const enforcement = page.locator(".enforcement-details");
    await enforcement.locator("summary").click();
    await expect(enforcement.locator("p")).toHaveCount(4);
    await enforcement.scrollIntoViewIfNeeded();
    await page.screenshot({ path: testInfo.outputPath("enforcement.png") });
    await page.getByRole("heading", { name: "Attempt history", exact: true }).scrollIntoViewIfNeeded();
    await expect(page.getByText("No Apply attempts yet", { exact: true })).toBeVisible();
    expect(await page.locator(".mission-content > .content").evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
  });

  test(`review remains reachable with 150 percent text at ${width}`, async ({ page }, testInfo) => {
    await page.setViewportSize({ width, height: 844 });
    await completeOnboarding(page, { "pytxo-preview-candidate-check-v1": "passed" });
    await openReview(page);
    // Text-only enlargement exercises fixed-pixel typography independently of browser zoom.
    await page.evaluate(() => {
      const elements = [...document.querySelectorAll<HTMLElement>(".review-screen, .review-screen *")];
      if (!elements.length) throw new Error("Review screen must be present before enlarging text");
      const sizes = elements.map(el => Number.parseFloat(getComputedStyle(el).fontSize));
      elements.forEach((el, i) => { el.style.fontSize = `${sizes[i] * 1.5}px`; });
    });
    const content = page.locator(".mission-content > .content");
    expect(await content.evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
    if (width === 1280) {
      for (const header of await page.locator(".file-row > header").all()) {
        const kind = await header.locator(".kind").boundingBox();
        const labels = await header.locator("div").boundingBox();
        expect(kind!.x + kind!.width).toBeLessThanOrEqual(labels!.x);
      }
    }
    const apply = page.getByRole("button", { name: "Apply reviewed changes", exact: true });
    await apply.scrollIntoViewIfNeeded();
    await expect(apply).toBeInViewport();
    await page.screenshot({ path: testInfo.outputPath("larger-text-decision.png") });
    const diff = page.locator(".diff-side.after .text-content");
    await diff.scrollIntoViewIfNeeded();
    await expect(diff).toBeInViewport();
    await page.screenshot({ path: testInfo.outputPath("larger-text-content.png") });
    await apply.click();
    const dialog = page.getByRole("dialog", { name: "Apply exact reviewed package?" });
    await dialog.getByRole("button", { name: "Cancel", exact: true }).click();
    await expect(apply).toBeFocused();
  });
}

for (const theme of ["void", "light"]) {
  for (const viewport of [{ width: 1280, height: 800 }, { width: 960, height: 800 }, { width: 390, height: 844 }]) {
    test(`reference capture ${theme} ${viewport.width}`, async ({ page }, testInfo) => {
      await page.setViewportSize(viewport);
      await page.clock.setFixedTime(new Date("2026-09-10T01:00:00Z"));
      await completeOnboarding(page, {
        "pytxo-deck-theme": theme,
        "pytxo-preview-candidate-check-v1": "passed",
      });
      await page.goto("/#/work");
      await expect(page.getByRole("heading", { name: "What the agents are doing" })).toBeVisible();
      await page.screenshot({ path: testInfo.outputPath("work.png") });
      const workReview = await page.getByRole("button", { name: "Review changes", exact: true }).boundingBox();
      await openReview(page);
      const firstContent = await page.locator(".diff-side.after .text-content").boundingBox();
      const heading = await page.getByRole("heading", { name: "Prepared changes", exact: true }).boundingBox();
      await page.screenshot({ path: testInfo.outputPath("review.png") });
      const overflow = await page.locator(".mission-content > .content").evaluate(el => el.scrollWidth - el.clientWidth);
      await writeFile(testInfo.outputPath("metrics.json"), JSON.stringify({ viewport, theme, data: "browser fixture", workReview, preparedChanges: heading, firstContent, overflow }, null, 2));
      await page.getByRole("button", { name: "Apply reviewed changes", exact: true }).click();
      const dialog = page.getByRole("dialog", { name: "Apply exact reviewed package?" });
      await expect(dialog.getByRole("button", { name: "Cancel", exact: true })).toBeFocused();
      await page.screenshot({ path: testInfo.outputPath("confirmation.png") });
      await page.keyboard.press("Escape");
      await expect(dialog).toBeHidden();
    });
  }
}
