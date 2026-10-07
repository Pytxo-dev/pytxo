import { expect, test } from "@playwright/test";
import { completeOnboarding, rootOverflow } from "./helpers";

const EXACT = { "pytxo-review-comparison-v1": "exact" };

const REVIEW_STATE_KEY = "pytxo-preview-review-state-v1";
const APPLY_OUTCOME_KEY = "pytxo-preview-apply-outcome-v1";
const AGENT_VERIFICATION_KEY = "pytxo-preview-agent-verification-v1";
const CANDIDATE_CHECK_KEY = "pytxo-preview-candidate-check-v1";

/**
 * Review is reached from History by selecting a run and opening its package.
 * The retired `#/missions` and `#/runs` hashes resolve to History, so deep links
 * written by older builds still land somewhere truthful.
 */
async function openHistory(
  page: import("@playwright/test").Page,
  state: string = "ready",
) {
  await completeOnboarding(page, { ...EXACT,
    [REVIEW_STATE_KEY]: state,
    [CANDIDATE_CHECK_KEY]: "passed",
  });
  await page.goto("/#/history");
  await expect(page.getByRole("heading", { level: 1, name: "History" })).toBeVisible();
}

async function openReview(page: import("@playwright/test").Page, runId: string) {
  await page.locator(`.history .row[data-run-id="${runId}"]`).click();
  await page.locator(".outcome-action").click();
  await expect(page.locator("#run-review-title")).toBeVisible();
  await page.locator(".technical-evidence > summary").click();
}

test.describe("Review depth", () => {
  test("another client refresh invalidates an open confirmation and requires fresh review", async ({ page, context }) => {
    await openHistory(page);
    await openReview(page, "run-71ad");
    await page.getByRole("button", { name: "Apply reviewed changes", exact: true }).click();
    const dialog = page.getByRole("dialog", { name: /^Apply these \d+ reviewed files?\?$/ });
    await expect(dialog).toContainText("pkg-71ad-immutable");
    const other = await context.newPage();
    await other.goto("/");
    await other.evaluate(() => localStorage.setItem("pytxo-preview-review-revision:run-71ad", "-B"));
    await expect(dialog).not.toBeVisible();
    await expect(page.getByText("The changes were updated. Review them again before Apply.")).toBeVisible();
    expect(await page.evaluate(() => localStorage.getItem("pytxo-preview-apply-count-v1"))).toBeNull();
    await page.getByRole("button", { name: "Apply reviewed changes", exact: true }).click();
    await expect(dialog).toContainText("pkg-71ad-immutable-B");
    await dialog.getByRole("button", { name: "Apply exact package", exact: true }).click();
    await expect(page.locator(".review-status")).toContainText("Applied");
    expect(await page.evaluate(() => localStorage.getItem("pytxo-preview-apply-count-v1"))).toBe("1");
    await other.close();
  });

  test("a missed refresh notification still sends the old displayed identity and refuses Apply", async ({ page }) => {
    await openHistory(page);
    await openReview(page, "run-71ad");
    await page.getByRole("button", { name: "Apply reviewed changes", exact: true }).click();
    // Same-document storage mutation intentionally emits no storage event.
    await page.evaluate(() => localStorage.setItem("pytxo-preview-review-revision:run-71ad", "-B"));
    await page.getByRole("button", { name: "Apply exact package", exact: true }).click();
    await expect(page.locator(".action-error")).toContainText("stale_review");
    expect(await page.evaluate(() => localStorage.getItem("pytxo-preview-apply-count-v1"))).toBeNull();
    await page.getByRole("button", { name: "Apply reviewed changes", exact: true }).click();
    await expect(page.getByRole("dialog")).toContainText("pkg-71ad-immutable-B");
  });

  test("opening a pinned run replaces the current review package and evidence", async ({ page }, testInfo) => {
    await completeOnboarding(page, { ...EXACT, [REVIEW_STATE_KEY]: "ready" });
    await page.setViewportSize({ width: 1600, height: 1000 });
    await page.goto("/#/work");
    await page.getByRole("button", { name: "Inspection tools", exact: true }).click();
    await page.getByRole("button", { name: "Files", description: "Click to open, or drag to the right or bottom edge", exact: true }).click();
    await page.getByRole("button", { name: "Pin view", exact: true }).click();
    const dock = page.getByRole("region", { name: "Files inspection", exact: true });
    await expect(dock.getByRole("button", { name: "Open prepared review" })).toBeVisible();

    await page.getByRole("link", { name: "History", exact: true }).click();
    await openReview(page, "run-71ad");
    await page.getByText("Run details", { exact: true }).click();
    await expect(page.locator(".run-details code")).toHaveText("run-71ad");
    await expect(page.locator(".package-identity")).toContainText("pkg-71ad-immutable");
    await expect(page.locator(".plan-panel .task-list strong")).toHaveText(["signal-core", "contract-tests"]);

    await dock.getByRole("button", { name: "Open prepared review" }).click();
    await page.getByText("Run details", { exact: true }).click();
    await expect(page.locator(".run-details code")).toHaveText("run-8f2c");
    await expect(page.locator(".package-identity")).toContainText("pkg-8f2c-immutable");
    await expect(page.locator(".package-identity")).not.toContainText("pkg-71ad-immutable");
    await expect(page.locator(".plan-panel .task-list strong")).toHaveText(["plan", "ui", "tests"]);
    await page.screenshot({ path: testInfo.outputPath("switched-review.png") });
  });

  test("retired Runs and Run Review deep links resolve to History and Review", async ({ page }) => {
    await completeOnboarding(page, EXACT);
    await page.goto("/#/runs");
    await expect(page.getByRole("heading", { level: 1, name: "History" })).toBeVisible();

    await page.goto("/#/missions");
    await expect(page.getByRole("heading", { level: 1, name: "History" })).toBeVisible();

    await page.goto("/#/run-review");
    await expect(page.locator("#run-review-title")).toBeVisible();
  });

  test("completed runs open immutable exact add modify delete review", async ({ page }) => {
    await openHistory(page);
    await openReview(page, "run-71ad");

    await expect(page.getByText("71ad8f2c4d90b6c6", { exact: true })).toBeVisible();
    await page.getByText("Change set ID", { exact: true }).click();
    await expect(page.locator(".package-identity code")).toBeVisible();
    await expect(page.getByText(/Aug 1, 2026, .* UTC/)).toBeVisible();
    await expect(page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/src/lib.rs", exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/tests/skeleton.rs", exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/src/legacy.rs", exact: true })).toBeVisible();
    await expect(page.locator(".file-row.modify")).toHaveCount(1);
    await expect(page.locator(".file-row.add")).toHaveCount(2);
    await expect(page.locator(".file-row.delete")).toHaveCount(1);
    await page.locator(".file-content .digest-details summary").click();
    await expect(page.locator(".file-content .digest-details")).toContainText("signal-core · run-71ad:agent-0");
    await expect(page.locator(".diff-side.before .text-content").first()).toContainText("source.to_owned()");
    await expect(page.locator(".diff-side.after .text-content").first()).toContainText("structural_skeleton()");
    await expect(page.locator(".exact-diff")).toHaveCount(1);
    await page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/tests/skeleton.rs" }).click();
    await expect(page.locator(".diff-side.after .text-content")).toContainText("keeps_public_shape");
    await page.getByRole("button", { name: "Inspect exact content for crates/pytxo-signal/src/legacy.rs" }).click();
    await expect(page.locator(".diff-side.before .text-content")).toContainText("legacy_raw_context");
    await expect(page.getByRole("button", { name: "Inspect exact content for assets/signal-mark.bin", exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Inspect exact content for assets/signal-mark.bin" }).click();
    await expect(page.getByText("Binary file · first bytes in hex", { exact: true })).toBeVisible();
    await expect(page.locator(".binary-content")).toContainText("00 ff 50 4e 47");
    await expect(page.locator(".binary-content")).not.toContainText("de ad be ef");
    await page.getByRole("button", { name: "Show all loaded binary bytes" }).click();
    await expect(page.locator(".binary-content")).toContainText("de ad be ef");
  });

  test("loads exact content only for the selected file with bounded side concurrency", async ({ page }) => {
    await openHistory(page);
    await openReview(page, "run-71ad");
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
    await openHistory(page);
    await openReview(page, "run-71ad");
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

  // Plan, watch, and read-the-package are three surfaces, not three tabs. A
  // dispatched run therefore lands on Work rather than on a pane switcher.
  test("a dispatched run lands on Work and its package is reached from History", async ({ page }) => {
    await completeOnboarding(page, EXACT);
    await page.goto("/#/flow");
    await page.getByLabel("What should Pytxo do?").fill("Exercise the dispatch path");
    await page.getByRole("button", { name: "Build plan", exact: true }).click();
    await page.getByRole("button", { name: "Run", exact: true }).click();
    await expect(page.getByRole("region", { name: "Work", exact: true })).toBeVisible();
    await expect(page.getByRole("tablist", { name: "Run panes" })).toHaveCount(0);

    await page.getByRole("link", { name: "History" }).click();
    await openReview(page, "run-71ad");
  });

  test("stale review refreshes before Apply and discard is confirmation-gated", async ({ page }) => {
    await openHistory(page, "stale");
    await openReview(page, "run-71ad");
    await expect(page.getByRole("button", { name: "Refresh review" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Apply reviewed changes", exact: true })).toBeDisabled();
    await page.getByRole("button", { name: "Refresh review" }).click();
    await expect(page.getByText("Ready to Apply")).toBeVisible();

    const apply = page.getByRole("button", { name: "Apply reviewed changes" });
    await apply.click();
    const applyDialog = page.getByRole("dialog", { name: /^Apply these \d+ reviewed files?\?$/ });
    await expect(applyDialog).toBeVisible();
    await expect(page.getByRole("button", { name: "Cancel" })).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(applyDialog).toBeHidden();
    await expect(apply).toBeFocused();

    await page.getByRole("button", { name: "Discard review" }).click();
    await expect(page.getByRole("dialog", { name: "Discard these changes?" })).toBeVisible();
    await page.getByRole("button", { name: "Keep review" }).click();
    await expect(page.getByRole("dialog", { name: "Discard these changes?" })).toBeHidden();
  });

  test("retry prevents double Apply while request is in flight", async ({ page }) => {
    await openHistory(page, "recovered");
    await openReview(page, "run-71ad");
    const retry = page.getByRole("button", { name: "Retry Apply" });
    await retry.click();
    await page.getByRole("button", { name: "Apply exact package" }).dblclick();
    const appliedTitle = page.getByText("Applied successfully", { exact: true });
    const appliedDetail = page.getByText(
      "The reviewed files were written to your project.",
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
    await openHistory(page, "recovered");
    await openReview(page, "run-71ad");

    await expect(page.getByText("attempt-preview-1", { exact: false })).toBeVisible();
    await expect(page.getByText("attempt-preview-0", { exact: false })).toBeVisible();
    await expect(page.getByText("rolled back", { exact: true })).toHaveCount(2);
    await expect(page.locator(".attempt code", { hasText: "apply_failed" })).toHaveCount(2);
  });

  test("a failed Apply refreshes the review and the run list immediately", async ({ page }) => {
    await completeOnboarding(page, { ...EXACT,
      [REVIEW_STATE_KEY]: "ready",
      [APPLY_OUTCOME_KEY]: "stale",
      [CANDIDATE_CHECK_KEY]: "passed",
    });
    await page.goto("/#/history");
    await expect(page.getByRole("heading", { level: 1, name: "History" })).toBeVisible();
    await openReview(page, "run-71ad");

    await page.getByRole("button", { name: "Apply reviewed changes" }).click();
    await page.getByRole("button", { name: "Apply exact package" }).click();
    await expect(page.locator(".review-status")).toContainText("Review is stale");
    // The stale status explains the refusal; the raw runner error is not repeated.
    await expect(page.locator(".action-error")).toHaveCount(0);
    await page.getByRole("link", { name: "History", exact: true }).click();
    // History reports the refreshed apply state without needing a reload.
    await expect(
      page.locator('.history .row[data-run-id="run-71ad"]'),
    ).toContainText("Outcome needs reconciliation");
  });

  test("recovery-required refuses Apply and offers guarded reconciliation", async ({ page }) => {
    await openHistory(page, "recovery_required");
    await openReview(page, "run-71ad");
    await expect(page.getByRole("button", { name: "Apply reviewed changes", exact: true })).toBeDisabled();
    await expect(page.getByText(/Apply is blocked until recovery is reconciled/).first()).toBeVisible();
    await page.getByRole("button", { name: "Recover" }).click();
    await expect(page.locator(".review-status")).toContainText("Recovered and ready");
    await expect(page.getByRole("button", { name: "Retry Apply" })).toBeEnabled();
  });

  test("history and review actions are keyboard operable", async ({ page }) => {
    await openHistory(page);
    await page.locator('.history .row[data-run-id="run-71ad"]').click();
    const review = page.getByRole("button", { name: "Review prepared changes", exact: true });
    await review.focus();
    await page.keyboard.press("Enter");
    await expect(page.locator("#run-review-title")).toBeVisible();
    await expect(page.getByRole("button", { name: "Back to Work", exact: true })).toBeFocused();
  });

  test("disabled Apply exposes a persistent agent-verification reason", async ({ page }) => {
    await completeOnboarding(page, { ...EXACT,
      [REVIEW_STATE_KEY]: "ready",
      [AGENT_VERIFICATION_KEY]: "missing",
      [CANDIDATE_CHECK_KEY]: "passed",
    });
    await page.goto("/#/history");
    await expect(page.getByRole("heading", { level: 1, name: "History" })).toBeVisible();
    await openReview(page, "run-71ad");

    const apply = page.getByRole("button", { name: "Apply reviewed changes" });
    await expect(apply).toBeDisabled();
    await expect(page.locator(".boundary-reason")).toBeVisible();
    await expect(page.locator(".boundary-reason")).toHaveText("Every agent must finish successfully before Apply.");
    await expect(apply).toHaveAttribute("aria-describedby", "apply-disabled-reason");
  });

  test("failed agent verification keeps the same visible Apply refusal", async ({ page }) => {
    await completeOnboarding(page, { ...EXACT,
      [REVIEW_STATE_KEY]: "ready",
      [AGENT_VERIFICATION_KEY]: "failed",
      [CANDIDATE_CHECK_KEY]: "passed",
    });
    await page.goto("/#/history");
    await expect(page.getByRole("heading", { level: 1, name: "History" })).toBeVisible();
    await openReview(page, "run-71ad");

    await expect(page.getByRole("button", { name: "Apply reviewed changes" })).toBeDisabled();
    await expect(page.locator(".boundary-reason")).toBeVisible();
    await expect(page.locator(".boundary-reason")).toHaveText("Every agent must finish successfully before Apply.");
  });

  test("discard confirmation traps focus, closes on Escape, and restores its trigger", async ({ page }) => {
    await openHistory(page);
    await openReview(page, "run-71ad");
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
    await expect(page.getByRole("dialog", { name: "Discard these changes?" })).toBeHidden();
    await expect(trigger).toBeFocused();
  });

  test("reduced motion removes review and loading animation", async ({ page }) => {
    await page.emulateMedia({ reducedMotion: "reduce" });
    await openHistory(page, "preparing");
    await openReview(page, "run-71ad");
    const busy = page.getByLabel("Preparing review");
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
      await openHistory(page);
      await openReview(page, "run-71ad");
      await expect(page.getByRole("button", { name: "Apply reviewed changes" })).toBeInViewport();
      await expect(page.getByRole("button", { name: "Discard review" })).toBeInViewport();
      expect((await rootOverflow(page)).horizontal).toBeLessThanOrEqual(1);
    });
  }
});
