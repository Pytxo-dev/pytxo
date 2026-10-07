import { expect, test } from "@playwright/test";
import { clearOnboarding, completeOnboarding } from "./helpers";
import { restoreSelectedWork, SELECTED_WORK_KEY } from "../src/lib/mission-selection";
import type { RunDto } from "../src/lib/types";

const internalVocabulary = /\bmission\b|\bwave\b|workspace isolation|\bevidence\b|recorded launcher/i;

for (const viewport of [{ width: 1280, height: 800 }, { width: 860, height: 560 }]) {
  test(`first use explains the job without internal vocabulary at ${viewport.width}`, async ({ page }, info) => {
    await clearOnboarding(page);
    await page.setViewportSize(viewport);
    await page.goto("/");
    const welcome = page.locator(".setup-shell");
    await expect(welcome).toContainText("changes nothing until you Apply");
    expect(await welcome.innerText()).not.toMatch(internalVocabulary);
    await expect(page.getByRole("button", { name: "Get started" })).toBeInViewport({ ratio: 1 });
    await page.screenshot({ path: info.outputPath("welcome.png") });
  });

  test(`the requested change leads and its technical checks are one click away at ${viewport.width}`, async ({ page }, info) => {
    await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "long-mission" });
    await page.setViewportSize(viewport);
    await page.goto("/#/work");
    const work = page.getByRole("region", { name: "Work", exact: true });
    await expect(work.locator("h1")).not.toHaveText("Your coding task");
    await expect(work.locator(".work-summary")).toContainText("A decision for this run needs your attention.");
    // The task heading stays plain-language; the connected inspector deliberately
    // exposes an Evidence tab without promoting recorded output into proof.
    expect(await work.locator(".work-heading").innerText()).not.toMatch(internalVocabulary);
    expect(await work.innerText()).not.toContain("run-8f2c");
    // The task title leads, in a compact header that leaves the page to the work itself.
    expect(await work.locator("h1").evaluate(el => parseFloat(getComputedStyle(el).fontSize))).toBeGreaterThanOrEqual(18);
    if (viewport.width >= 1280) expect((await work.locator(".work-heading").boundingBox())!.height).toBeLessThanOrEqual(76);
    await expect(work.getByRole("button", { name: "Stop", exact: true })).toBeInViewport({ ratio: 1 });
    await expect(work.getByRole("button", { name: "Review changes", exact: true })).toBeInViewport({ ratio: 1 });
    await page.screenshot({ path: info.outputPath("active-task.png") });
    await work.getByRole("button", { name: "Details", exact: true }).click();
    const detail = page.locator(".dock-panel:visible");
    await expect(detail).toContainText("pkg-8f2c-immutable");
    await expect(detail).toContainText("Combined candidate verification has not been recorded.");
    await expect(detail).toContainText("Enforced");
    await page.screenshot({ path: info.outputPath("task-with-checks.png") });
  });
}

test("plan details disclose technical commands without hiding missing checks", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/flow");
  await page.getByLabel("What should Pytxo do?").fill("Fix src/parser.rs for empty input and test it");
  await page.locator(".composer-panel").getByRole("button", { name: "Build plan", exact: true }).click();
  await expect(page.getByRole("button", { name: "Run", exact: true })).toBeEnabled();
  await expect(page.getByText("Per-task verification commands", { exact: true })).toBeVisible();
  await expect(page.locator(".plan-technical dl")).toBeHidden();
  await page.getByText("Permissions and technical details", { exact: true }).click();
  await expect(page.getByText("Per-task verification commands", { exact: true })).toBeVisible();
  await expect(page.locator(".plan-contract")).toContainText("npm run check");
  await page.getByLabel("What should Pytxo do?").fill("Change the request after planning");
  await expect(page.getByText("Plan is stale", { exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Run", exact: true })).toBeDisabled();
});

test("a reopened task is resolved by both project and run, never by run ID alone", () => {
  const runs = [{ id: "same", domain_id: "a" }, { id: "same", domain_id: "b" }] as RunDto[];
  expect(restoreSelectedWork('{"runId":"same","domainId":"b"}', runs)).toBe(runs[1]);
  expect(restoreSelectedWork('{"runId":"same","domainId":"missing"}', runs)).toBeNull();
  expect(restoreSelectedWork('{"runId":"same"}', runs)).toBeNull();
  expect(restoreSelectedWork("broken", runs)).toBeNull();
});

test("reopening returns to the selected task without restoring execution authority", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-native-agent-ids-v1": "switch" });
  await page.goto("/#/work");
  await page.locator(".run-reference summary").click();
  await page.getByRole("tab", { name: "run-other", exact: true }).click();
  await expect(page.locator(".work-heading")).toContainText("Completed");
  expect(await page.evaluate(key => JSON.parse(localStorage.getItem(key)!), SELECTED_WORK_KEY)).toEqual({ runId: "run-other", domainId: "pytxo" });
  await page.reload();
  await expect(page.locator(".work-heading")).toContainText("Completed");
  await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeDisabled();
  await page.locator(".run-reference summary").click();
  await expect(page.getByRole("tab", { name: "run-other", exact: true })).toHaveAttribute("aria-selected", "true");
});
