import { expect, test, type Page } from "@playwright/test";
import { completeOnboarding } from "./helpers";

// Stable software rendering for local Windows browser capture (not native acceptance).
test.use({ launchOptions: { args: ["--disable-gpu"] } });

async function open(page: Page, state = "ready", checks = "passed") {
  await completeOnboarding(page, {
    "pytxo-preview-review-substantial-v1": "true",
    "pytxo-preview-review-state-v1": state,
    "pytxo-preview-candidate-check-v1": checks,
    "pytxo-preview-flow-history-v1": "ready",
  });
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-71ad"]').click();
  await page.locator(".outcome-action").click();
  await expect(page.locator(".diff-side.after .text-content")).toContainText("preserves_public_shape_35");
}
async function capture(page: Page, name: string) {
  const size = page.viewportSize()!;
  await page.setViewportSize({ ...size, width: size.width + 1 });
  await page.setViewportSize(size);
  await page.screenshot({ path: `../../target/review-spatial-correction-20260917/${name}.png`, animations: "disabled" });
}
for (const state of ["ready", "missing", "stale"]) {
  test(`default candidate map: ${state}`, async ({ page }) => {
    await page.setViewportSize({ width: 1600, height: 1000 });
    await open(page, state === "missing" ? "ready" : state, state === "missing" ? "missing" : "passed");
    await expect(page.getByText("Focus on code", { exact: true })).toBeVisible();
    await expect(page.locator(".convergence path")).toHaveCount(3);
    await expect(page.locator(".identity-peek")).toContainText("pkg-71ad");
    await expect(page.locator(".candidate-context")).toBeVisible();
    if (state !== "ready") await expect(page.getByRole("button", { name: "Apply reviewed changes", exact: true })).toBeDisabled();
    await capture(page, `after-${state}-wide`);
    const url = page.url();
    await page.locator(".map-caption button").click();
    expect(page.url()).toBe(url);
    await expect(page.locator(".decision-bar")).toBeInViewport();
    await page.getByRole("button", { name: "Compare crates/pytxo-signal/tests/skeleton.rs", exact: true }).click();
    await expect(page.locator(".comparison-context strong")).toHaveText("skeleton.rs");
    await expect(page.locator(".convergence path.selected")).toHaveCount(1);
    await page.getByRole("button", { name: /more · Browse all files/ }).click();
    await expect(page.locator(".file-navigation button").first()).toBeFocused();
    await page.locator(".destination-node summary").click();
    await expect(page.locator(".destination-node code")).toHaveText("C:/dev/signal-lab");
    await page.locator(".verification-node").click();
    await expect(page.locator(".candidate-checks")).toBeFocused();
    await expect(page.locator(".technical-evidence")).toHaveAttribute("open", "");
  });
}
test("focus preference persists and available width governs the default", async ({ page }) => {
  await page.setViewportSize({ width: 1600, height: 1000 });
  await open(page);
  await page.getByText("Focus on code", { exact: true }).click();
  await expect(page.locator(".candidate-context")).toBeHidden();
  await capture(page, "after-code-focused");
  await page.reload();
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-71ad"]').click();
  await page.getByRole("button", { name: "Review prepared changes", exact: true }).click();
  await expect(page.getByText("Show candidate map", { exact: true })).toBeVisible();
  await expect(page.locator(".candidate-context")).toBeHidden();
  await page.getByText("Show candidate map", { exact: true }).click();
  await expect(page.locator(".candidate-context")).toBeVisible();
  await page.reload();
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-71ad"]').click();
  await page.getByRole("button", { name: "Review prepared changes", exact: true }).click();
  await expect(page.getByText("Focus on code", { exact: true })).toBeVisible();
});
test("narrow layout uses the readable list alternative", async ({ page }) => {
  await page.setViewportSize({ width: 860, height: 900 });
  await open(page);
  await expect(page.locator(".candidate-context")).toBeHidden();
  await capture(page, "after-narrow");
  await page.getByText("Show candidate map", { exact: true }).click();
  await expect(page.locator(".candidate-context")).toBeVisible();
  await expect(page.locator(".convergence")).toBeHidden();
  await capture(page, "after-narrow-map");
  expect(await page.locator(".review-screen").evaluate(el => el.scrollWidth - el.clientWidth)).toBeLessThanOrEqual(1);
});
test("dock occupancy collapses the automatic map without changing preference", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await open(page);
  await expect(page.locator(".candidate-context")).toBeVisible();
  await page.getByRole("button", { name: "Inspection tools", exact: true }).click();
  await page.getByRole("button", { name: "Files", exact: true }).click();
  await expect(page.getByRole("tablist", { name: "right dock" })).toBeVisible();
  await expect(page.locator(".candidate-context")).toBeHidden();
  expect(await page.evaluate(() => localStorage.getItem("pytxo-review-composition-v1"))).toBeNull();
});
