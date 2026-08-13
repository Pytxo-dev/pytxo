import { expect, test } from "@playwright/test";
import { completeOnboarding, rootOverflow } from "./helpers";

const REVIEW_STATE_KEY = "pytxo-preview-review-state-v1";
const APPLY_OUTCOME_KEY = "pytxo-preview-apply-outcome-v1";
const AGENT_VERIFICATION_KEY = "pytxo-preview-agent-verification-v1";

async function openMissions(
  page: import("@playwright/test").Page,
  state: string = "ready",
) {
  await completeOnboarding(page, { [REVIEW_STATE_KEY]: state });
  await page.goto("/#/missions");
  await expect(page.getByRole("heading", { name: "Missions", exact: true })).toBeVisible();
}

test.describe("mission-centric Flow", () => {
  test("old Runs and Run Review deep links open Missions and Review", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/runs");
    await expect(page.getByRole("heading", { name: "Mission history" })).toBeVisible();

    await page.goto("/#/run-review");
    await expect(page.getByRole("heading", { name: "Run Review" })).toBeVisible();
  });

  test("completed missions open immutable exact add modify delete review", async ({ page }) => {
    await openMissions(page);
    await page.getByRole("button", { name: /Review run-71ad/ }).click();

    await expect(page.getByRole("heading", { name: "Run Review" })).toBeVisible();
    await expect(page.getByText("71ad8f2c4d90b6c6", { exact: true })).toBeVisible();
    await expect(page.getByText("pkg-71ad-immutable", { exact: true })).toBeVisible();
    await expect(page.getByText("Prepared Aug 1, 2026", { exact: false })).toBeVisible();
    await expect(page.getByText("crates/pytxo-signal/src/lib.rs", { exact: true })).toBeVisible();
    await expect(page.getByText("crates/pytxo-signal/tests/skeleton.rs", { exact: true })).toBeVisible();
    await expect(page.getByText("crates/pytxo-signal/src/legacy.rs", { exact: true })).toBeVisible();
    await expect(page.getByText("Modified", { exact: true })).toBeVisible();
    await expect(page.getByText("Added", { exact: true })).toHaveCount(2);
    await expect(page.getByText("Deleted", { exact: true })).toBeVisible();
    await expect(page.getByText("signal-core · run-71ad:agent-0", { exact: true }).first()).toBeVisible();
    await expect(page.locator(".diff-side.before .text-content").first()).toContainText("source.to_owned()");
    await expect(page.locator(".diff-side.after .text-content").first()).toContainText("structural_skeleton()");
    await expect(page.locator(".exact-diff")).toHaveCount(1);
    await page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/tests/skeleton.rs" }).click();
    await expect(page.locator(".diff-side.after .text-content")).toContainText("keeps_public_shape");
    await page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/src/legacy.rs" }).click();
    await expect(page.locator(".diff-side.before .text-content")).toContainText("legacy_raw_context");
    await expect(page.getByText("assets/signal-mark.bin", { exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Inspect exact content for assets/signal-mark.bin" }).click();
    await expect(page.getByText("Exact binary bytes · hexadecimal preview", { exact: true })).toBeVisible();
    await expect(page.locator(".binary-content")).toContainText("00 ff 50 4e 47");
    await expect(page.locator(".binary-content")).not.toContainText("de ad be ef");
    await page.getByRole("button", { name: "Show all loaded binary bytes" }).click();
    await expect(page.locator(".binary-content")).toContainText("de ad be ef");
  });

  test("loads exact content only for the selected file with bounded side concurrency", async ({ page }) => {
    await openMissions(page);
    await page.getByRole("button", { name: /Review run-71ad/ }).click();
    await expect(page.locator(".diff-side")).toHaveCount(2);
    await expect
      .poll(() => page.evaluate(() => JSON.parse(localStorage.getItem("pytxo-preview-review-content-requests-v1") ?? "[]")))
      .toEqual([
        { path: "crates/pytxo-signal/src/lib.rs", side: "before" },
        { path: "crates/pytxo-signal/src/lib.rs", side: "after" },
      ]);

    await page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/tests/skeleton.rs" }).click();
    await expect(page.locator(".diff-side")).toHaveCount(1);
    await expect
      .poll(() => page.evaluate(() => JSON.parse(localStorage.getItem("pytxo-preview-review-content-requests-v1") ?? "[]")))
      .toEqual([
        { path: "crates/pytxo-signal/src/lib.rs", side: "before" },
        { path: "crates/pytxo-signal/src/lib.rs", side: "after" },
        { path: "crates/pytxo-signal/tests/skeleton.rs", side: "after" },
      ]);
  });

  test("caps delayed exact reads globally and cancels queued stale selections", async ({ page }) => {
    await openMissions(page);
    await page.getByRole("button", { name: /Review run-71ad/ }).click();
    await page.evaluate(() => {
      localStorage.setItem("pytxo-preview-review-content-delay-v1", "250");
      localStorage.setItem("pytxo-preview-review-content-requests-v1", "[]");
      localStorage.setItem("pytxo-preview-review-content-active-v1", "0");
      localStorage.setItem("pytxo-preview-review-content-max-active-v1", "0");
    });

    await page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/tests/skeleton.rs" }).click();
    await page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/src/lib.rs" }).click();
    await page.getByRole("button", { name: "Inspect exact content for assets/signal-mark.bin" }).click();

    await expect(page.locator(".binary-content")).toContainText("00 ff 50 4e 47");
    const telemetry = await page.evaluate(() => ({
      maxActive: Number(localStorage.getItem("pytxo-preview-review-content-max-active-v1")),
      requests: JSON.parse(localStorage.getItem("pytxo-preview-review-content-requests-v1") ?? "[]") as Array<{ path: string; side: string }>,
    }));
    expect(telemetry.maxActive).toBe(2);
    expect(telemetry.requests).toEqual(
      expect.arrayContaining([
        { path: "crates/pytxo-signal/tests/skeleton.rs", side: "after" },
        { path: "crates/pytxo-signal/src/lib.rs", side: "before" },
        { path: "assets/signal-mark.bin", side: "after" },
      ]),
    );
    expect(telemetry.requests.at(-1)).toEqual({
      path: "assets/signal-mark.bin",
      side: "after",
    });
    expect(
      telemetry.requests.filter(
        ({ path, side }) =>
          path === "crates/pytxo-signal/tests/skeleton.rs" && side === "after",
      ),
    ).toHaveLength(1);
    await expect(page.locator(".exact-diff")).toHaveCount(1);
    await expect(page.locator(".diff-side")).toHaveAttribute(
      "aria-label",
      "after content for assets/signal-mark.bin",
    );
  });

  test("mission panes expose Plan Live and Review", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/missions");
    await page.getByRole("button", { name: /Review run-8f2c/ }).click();
    const plan = page.getByRole("button", { name: "Plan", exact: true });
    const live = page.getByRole("button", { name: "Live", exact: true });
    const review = page.getByRole("button", { name: "Review", exact: true });
    await expect(live).toHaveClass(/active/);
    await plan.click();
    await expect(page.getByRole("heading", { name: "Mission", exact: true })).toBeVisible();
    await review.click();
    await expect(page.getByRole("heading", { name: "Run Review" })).toBeVisible();
  });

  test("stale review refreshes before Apply and discard is confirmation-gated", async ({ page }) => {
    await openMissions(page, "stale");
    await page.getByRole("button", { name: /Review run-71ad/ }).click();
    await expect(page.getByRole("button", { name: "Refresh review" })).toBeVisible();
    await expect(page.getByRole("button", { name: /Apply/ })).toBeDisabled();
    await page.getByRole("button", { name: "Refresh review" }).click();
    await expect(page.getByText("Ready to Apply")).toBeVisible();

    await page.getByRole("button", { name: "Discard review" }).click();
    await expect(page.getByRole("dialog", { name: "Discard prepared review?" })).toBeVisible();
    await page.getByRole("button", { name: "Keep review" }).click();
    await expect(page.getByRole("dialog", { name: "Discard prepared review?" })).toBeHidden();
  });

  test("retry prevents double Apply while request is in flight", async ({ page }) => {
    await openMissions(page, "recovered");
    await page.getByRole("button", { name: /Review run-71ad/ }).click();
    const retry = page.getByRole("button", { name: "Retry Apply" });
    await retry.dblclick();
    const appliedTitle = page.getByText("Applied successfully", { exact: true });
    const appliedDetail = page.getByText(
      "The reviewed package was applied to the primary checkout.",
      { exact: true },
    );
    await expect(appliedTitle).toBeVisible();
    await expect(appliedTitle).toHaveCount(1);
    await expect(appliedDetail).toBeVisible();
    await expect(appliedDetail).toHaveCount(1);
    await expect(page.locator(".action-explanation")).toHaveCount(0);
    await expect(page.getByRole("button", { name: "Apply reviewed changes" })).not.toHaveAttribute(
      "aria-describedby",
      /.+/,
    );
    await expect(page.getByText("Reviewed package applied.", { exact: true })).toHaveCount(0);
    await expect
      .poll(() => page.evaluate(() => localStorage.getItem("pytxo-preview-apply-count-v1")))
      .toBe("1");
  });

  test("renders durable history for multiple Apply attempts", async ({ page }) => {
    await openMissions(page, "recovered");
    await page.getByRole("button", { name: /Review run-71ad/ }).click();

    await expect(page.getByText("attempt-preview-1", { exact: false })).toBeVisible();
    await expect(page.getByText("attempt-preview-0", { exact: false })).toBeVisible();
    await expect(page.getByText("rolled back", { exact: true })).toHaveCount(2);
    await expect(page.locator(".attempt code", { hasText: "apply_failed" })).toHaveCount(2);
  });

  test("failed Apply refreshes the review and mission snapshot immediately", async ({ page }) => {
    await completeOnboarding(page, {
      [REVIEW_STATE_KEY]: "ready",
      [APPLY_OUTCOME_KEY]: "stale",
    });
    await page.goto("/#/missions");
    await expect(page.getByRole("heading", { name: "Missions", exact: true })).toBeVisible();
    await page.getByRole("button", { name: /Review run-71ad/ }).click();

    await page.getByRole("button", { name: "Apply reviewed changes" }).click();
    await expect(page.getByText("Review is stale")).toBeVisible();
    await page.getByRole("button", { name: "Back to missions" }).click();
    await expect(page.locator(".mission-row.completed .contract")).toHaveText("stale");
  });

  test("recovery-required refuses Apply and offers guarded reconciliation", async ({ page }) => {
    await openMissions(page, "recovery_required");
    await page.getByRole("button", { name: /Review run-71ad/ }).click();
    await expect(page.getByRole("button", { name: /Apply/ })).toBeDisabled();
    await expect(page.getByText(/Apply is blocked until recovery is reconciled/).first()).toBeVisible();
    await page.getByRole("button", { name: "Reconcile recovery" }).click();
    await expect(page.getByText("Recovered and ready")).toBeVisible();
    await expect(page.getByRole("button", { name: "Retry Apply" })).toBeEnabled();
  });

  test("mission history and review actions are keyboard operable", async ({ page }) => {
    await openMissions(page);
    const review = page.getByRole("button", { name: /Review run-71ad/ });
    await review.focus();
    await page.keyboard.press("Enter");
    await expect(page.getByRole("heading", { name: "Run Review" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Back to missions" })).toBeFocused();
  });

  test("disabled Apply exposes a persistent agent-verification reason", async ({ page }) => {
    await completeOnboarding(page, {
      [REVIEW_STATE_KEY]: "ready",
      [AGENT_VERIFICATION_KEY]: "missing",
    });
    await page.goto("/#/missions");
    await expect(page.getByRole("heading", { name: "Missions", exact: true })).toBeVisible();
    await page.getByRole("button", { name: /Review run-71ad/ }).click();

    const apply = page.getByRole("button", { name: "Apply reviewed changes" });
    await expect(apply).toBeDisabled();
    await expect(page.getByText("Every agent must exit successfully before Apply.")).toBeVisible();
    await expect(apply).toHaveAttribute("aria-describedby", "apply-disabled-reason");
  });

  test("failed agent verification keeps the same visible Apply refusal", async ({ page }) => {
    await completeOnboarding(page, {
      [REVIEW_STATE_KEY]: "ready",
      [AGENT_VERIFICATION_KEY]: "failed",
    });
    await page.goto("/#/missions");
    await expect(page.getByRole("heading", { name: "Missions", exact: true })).toBeVisible();
    await page.getByRole("button", { name: /Review run-71ad/ }).click();

    await expect(page.getByRole("button", { name: "Apply reviewed changes" })).toBeDisabled();
    await expect(page.getByText("Every agent must exit successfully before Apply.")).toBeVisible();
  });

  test("discard confirmation traps focus, closes on Escape, and restores its trigger", async ({ page }) => {
    await openMissions(page);
    await page.getByRole("button", { name: /Review run-71ad/ }).click();
    const trigger = page.getByRole("button", { name: "Discard review" });
    await trigger.focus();
    await page.keyboard.press("Enter");

    const keep = page.getByRole("button", { name: "Keep review" });
    const discard = page.getByRole("button", { name: "Discard permanently" });
    await expect(keep).toBeFocused();
    await page.keyboard.press("Shift+Tab");
    await expect(discard).toBeFocused();
    await page.keyboard.press("Tab");
    await expect(keep).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(page.getByRole("dialog", { name: "Discard prepared review?" })).toBeHidden();
    await expect(trigger).toBeFocused();
  });

  test("reduced motion removes review and loading animation", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await openMissions(page, "preparing");
    await page.getByRole("button", { name: /Review run-71ad/ }).click();
    const busy = page.getByLabel("Preparing immutable review");
    await expect(busy).toBeVisible();
    await expect(busy).toHaveCSS("animation-name", "none");
  });

  for (const viewport of [
    { width: 1600, height: 1000 },
    { width: 1280, height: 800 },
    { width: 960, height: 640 },
  ]) {
    test(`keeps mission review controls readable at ${viewport.width}x${viewport.height}`, async ({
      page,
    }) => {
      await page.setViewportSize(viewport);
      await openMissions(page);
      await page.getByRole("button", { name: /Review run-71ad/ }).click();
      await expect(page.getByRole("button", { name: "Apply reviewed changes" })).toBeInViewport();
      await expect(page.getByRole("button", { name: "Discard review" })).toBeInViewport();
      expect((await rootOverflow(page)).horizontal).toBeLessThanOrEqual(1);
    });
  }
});
