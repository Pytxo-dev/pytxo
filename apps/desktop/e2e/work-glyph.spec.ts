import { expect, test } from "@playwright/test";
import { workActivity } from "../src/lib/work-activity";
import type { RunDto, AgentDto, RoutingDisplaySummary } from "../src/lib/types";
import { completeOnboarding } from "./helpers";
const run = { id: "one", domain_id: "repo", status: "running", applied_at: null, recovery_state: null } as RunDto;
const agent = { run_id: "one", domain_id: "repo", status: "running" } as AgentDto;

test("History does not promote an applied timestamp over a newer error", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-legacy-retry-v1": "1", "pytxo-preview-newer-apply-error-v1": "1" });
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-8f2c"]').click();
  await expect(page.locator(".outcome-heading")).toContainText("Apply error recorded");
  await expect(page.locator(".recovery-notice")).toBeVisible();
});

test("expanding eight native-length run IDs preserves readable Work heading", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-many-runs-v1": "1" });
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#/work");
  await page.getByText("Runs", { exact: true }).click();
  await expect(page.getByRole("tablist", { name: "Runs in this snapshot" }).getByRole("tab")).toHaveCount(8);
  const heading = await page.locator(".work-heading h1").boundingBox();
  expect(heading!.width).toBeGreaterThan(280);
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({ path: "../../target/beta-regression-repair-20260919/work-expanded-runs.png" });
});

test("a successful retry settles historical rollback evidence but not unresolved recovery", () => {
  const applied = { ...run, status: "completed", apply_status: "applied", applied_at: "2026-09-19T21:00:00Z", recovery_state: "rolled_back",
    last_apply_error: { at: "2026-09-19T20:00:00Z", code: "interrupted_apply", message: "Rolled back", attempt_id: "old", rollback_confirmed: true } };
  expect(workActivity(applied, [], false, false, true).label).toBe("Apply recorded");
  for (const recovery_state of ["unprovable", "recovery_required"]) {
    expect(workActivity({ ...applied, recovery_state }, [], false, false, true).label).toBe("Needs attention");
  }
  for (const error of [
    { ...applied.last_apply_error, rollback_confirmed: false },
    { ...applied.last_apply_error, at: "2026-09-19T22:00:00Z" },
    { ...applied.last_apply_error, at: "unknown" },
  ]) expect(workActivity({ ...applied, last_apply_error: error }, [], false, false, true).label).toBe("Needs attention");
});
test("activity requires scoped running evidence and settles under uncertainty", () => {
  expect(workActivity(run, [agent], false, false, false).active).toBe(true);
  for (const state of ["stopped", "cancelled", "failed", "failed_startup", "completed", "starting", "new-unknown-state"]) expect(workActivity({ ...run, status: state }, [agent], false, false, true).active).toBe(false);
  expect(workActivity(run, [{ ...agent, domain_id: "other" }], false, false, false).active).toBe(false);
  expect(workActivity(run, [agent], true, false, false).active).toBe(false);
  expect(workActivity(run, [agent], false, true, false).label).toBe("Activity unknown");
  expect(workActivity({ ...run, status: "completed" }, [], false, false, true).label).toBe("Ready for review");
  expect(workActivity({ ...run, status: "completed" }, [], false, false, true).label).not.toMatch(/Apply/);
});

test("routed activity requires the current running attempt and its exact worker", () => {
  const routedRun = { ...run, routing_revision: "7" };
  const summary = {
    domain_id: "repo", run_id: "one", routing_revision: "7",
    tasks: [{ task_id: "ui", state: "active", current_attempt_id: "attempt-2", attempts: [
      { attempt_id: "attempt-1", agent_id: "stale", state: "failed" },
      { attempt_id: "attempt-2", agent_id: "current", state: "admitted" },
    ] }],
  } as RoutingDisplaySummary;
  const stale = { ...agent, id: "stale", task_id: "ui" };
  const current = { ...agent, id: "current", task_id: "ui" };
  expect(workActivity(routedRun, [stale], false, false, false, false, summary).active).toBe(false);
  expect(workActivity(routedRun, [current], false, false, false, false, summary).active).toBe(false);
  const running = structuredClone(summary);
  running.tasks[0].attempts[1].state = "running";
  expect(workActivity(routedRun, [current], false, false, false, false, running).active).toBe(true);
  expect(workActivity(routedRun, [{ ...current, task_id: "other" }], false, false, false, false, running).active).toBe(false);
  expect(workActivity(routedRun, [current, { ...current }], false, false, false, false, running).active).toBe(false);
  expect(workActivity(routedRun, [current], false, false, false, false, { ...running, routing_revision: "6" }).active).toBe(false);
});

test("worker summary follows recorded tasks and output opens only on request", async ({ page }) => {
  await completeOnboarding(page); await page.setViewportSize({ width: 1600, height: 1000 }); await page.goto("/#/work");
  const map = page.getByTestId("execution-map");
  await expect(map.locator("path.highlighted")).toHaveCount(2);
  await expect(page.getByRole("region", { name: "Recorded agent output" })).toHaveCount(0);
  await map.getByRole("button", { name: /^ui / }).click();
  const summary = page.locator(".dock-panel:visible:not(.output-panel)");
  await expect(summary).toContainText("Recorded scope");
  await expect(summary).toContainText("apps/desktop");
  await summary.getByRole("button", { name: "Open output", exact: true }).click();
  await expect(page.getByRole("region", { name: "Recorded agent output" })).toContainText("Browser fixture");
  await map.getByRole("button", { name: /^tests / }).focus(); await page.keyboard.press("Enter");
  await expect(map.locator(".task-node.chosen")).toContainText("tests");
  await expect(summary.locator("header strong").first()).toHaveText("tests");
  await expect(page.getByRole("tablist", { name: "bottom dock", exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeEnabled();
});

test("pending decisions, reduced motion and Stop keep the glyph settled", async ({ page }) => {
  await completeOnboarding(page); await page.goto("/#/work");
  const glyph = page.locator(".aperture-glyph");
  await expect(page.locator(".work-heading").getByText("Running", { exact: true })).toBeVisible();
  await expect(page.locator(".work-summary")).toHaveText("A decision for this run needs your attention.");
  await expect(glyph).toHaveAttribute("data-active", "false");
  await page.getByRole("button", { name: /Decision needed/ }).click();
  const inbox = page.getByRole("dialog", { name: "Approvals inbox" });
  await inbox.getByRole("button", { name: /Deny request/ }).click();
  await inbox.locator("button.primary").click();
  await page.getByRole("button", { name: "Close approvals" }).click();
  await expect(glyph).toHaveAttribute("data-active", "true");
  await page.emulateMedia({ reducedMotion: "reduce" });
  await expect(glyph).toHaveAttribute("data-active", "false");
  await expect(glyph.locator("svg")).not.toHaveClass(/concealed/);
  await page.emulateMedia({ reducedMotion: "no-preference" });
  await page.getByRole("button", { name: "Stop", exact: true }).click();
  await page.getByRole("button", { name: "Stop run", exact: true }).click();
  await expect(glyph).toHaveAttribute("data-active", "false");
  await expect(page.locator(".work-heading").getByText("Stopped", { exact: true })).toBeVisible();
});

for (const partial of [false, true]) test(`legacy applied retry keeps recovery honest (partial=${partial})`, async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-legacy-retry-v1": "1", "pytxo-preview-state-matrix-v1": partial ? "1" : "0" });
  await page.goto("/#/work");
  // Wait for review evidence, not only the fleet snapshot.
  await expect(page.getByTestId("execution-map")).toBeVisible();
  await expect(page.locator(".work-summary")).toHaveText(partial
    ? "Some project files may have changed. Check recovery before continuing."
    : "These changes have been saved to your project.");
  await expect(page.locator(".aperture-glyph")).toHaveAttribute("data-tone", partial ? "refuted" : "settled");
  if (partial) {
    await expect(page.getByText("Working tree may be partially modified", { exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Reconcile recovery state", exact: true })).toBeVisible();
    await expect(page.locator(".commit-rail")).toHaveCount(0);
    await expect(page.locator(".work-summary")).toContainText("Check recovery");
  } else {
    await expect(page.getByText("Working tree may be partially modified", { exact: true })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Review recovery", exact: true })).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Reconcile recovery state", exact: true })).toHaveCount(0);
  }
  await page.screenshot({ path: `../../target/beta-regression-repair-20260919/legacy-retry-partial-${partial}.png` });
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-8f2c"]').click();
  await expect(page.locator(".outcome-heading")).toContainText(partial ? "Outcome needs reconciliation" : "Apply recorded");
  await expect(page.locator(".recovery-notice")).toHaveCount(partial ? 1 : 0);
  await expect(page.locator(".attempt").filter({ hasText: "Rolled back" })).not.toHaveCount(0);
});
