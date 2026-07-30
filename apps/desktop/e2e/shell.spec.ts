import { test, expect } from "@playwright/test";
import {
  clearOnboarding,
  completeOnboarding,
  expectRootZoom,
  openAppearance,
  SCALE_STORAGE_KEY,
  SETUP_STORAGE_KEY,
  setBrowserPreviewScale,
} from "./helpers";

test.describe("Pytxo Desktop shell", () => {
  test("setup wizard renders on fresh profile", async ({ page }) => {
    await clearOnboarding(page);
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Pytxo", exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Get started" })).toBeVisible();
    await expect(page.getByLabel("Setup progress")).toBeVisible();
  });

  test("setup skip-all reaches ready step", async ({ page }) => {
    await clearOnboarding(page);
    await page.goto("/");
    await page.getByRole("button", { name: "Get started" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await expect(page.getByRole("heading", { name: "Set your display" })).toBeVisible();
    await page.getByRole("button", { name: "Continue" }).click();
    await expect(page.getByRole("heading", { name: "You're ready" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Enter Pytxo Desktop" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Back" })).toBeVisible();
  });

  test("onboarding migration re-shows the wizard for an older completed profile", async ({ page }) => {
    // A profile that only carries the legacy boolean flag predates the
    // versioned onboarding gate and must see the refreshed flow once.
    await page.addInitScript(
      ({ setupKey }) => {
        localStorage.setItem(setupKey, "complete");
      },
      { setupKey: SETUP_STORAGE_KEY },
    );
    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Pytxo", exact: true })).toBeVisible();
  });

  test("onboarding can be replayed from Settings without losing the versioned flag afterwards", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/");
    await page.getByRole("link", { name: "Settings" }).click();
    await page.getByRole("button", { name: "Account & billing" }).click();
    await page.getByRole("button", { name: "Run onboarding again" }).click();
    await expect(page.getByRole("heading", { name: "Pytxo", exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Get started" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await page.getByRole("button", { name: "Continue" }).click();
    await page.getByRole("button", { name: "Enter Pytxo Desktop" }).click();
    await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible();
  });

  test("desktop 2 shell opens in operations and navigates primary routes", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/");
    await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Ops" })).toBeVisible();
    await expect(page.getByText(/^Updater:/)).toHaveCount(0);
    await page.getByRole("link", { name: "Flow" }).click();
    await expect(page.getByRole("heading", { name: "Flow" })).toBeVisible();
    await page.getByRole("link", { name: "Integrations" }).click();
    await expect(page.getByRole("heading", { name: "Integrations" })).toBeVisible();
  });

  test("legacy workspace tabs migrate to recents without deletion", async ({ page }) => {
    await completeOnboarding(page, {
      "pytxo-deck-tabs-v1": JSON.stringify({
        tabs: [
          {
            id: "tab-1",
            domainId: "/tmp/demo",
            label: "demo",
            projectId: null,
            selectedRunId: null,
            selectedAgentId: null,
            dispatchRepo: "",
          },
        ],
        activeTabId: "tab-1",
      }),
    });
    await page.goto("/");
    await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible();
    await expect(page.getByText("demo", { exact: true })).toBeVisible();
    const migration = await page.evaluate(() => ({
      legacy: localStorage.getItem("pytxo-deck-tabs-v1"),
      recents: localStorage.getItem("pytxo-desktop-recents-v2"),
    }));
    expect(migration.legacy).not.toBeNull();
    expect(migration.recents).toContain("demo");
  });

  test("legacy shell remains available for one rollback release", async ({ page }) => {
    await completeOnboarding(page, { desktop_shell_v1: "true" });
    await page.goto("/");
    await expect(page.getByRole("navigation", { name: "Open workspaces" })).toBeVisible();
  });

  test("sidebar collapse state persists across reload", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/");
    const toggle = page.getByRole("button", { name: "Collapse sidebar" });
    await expect(toggle).toBeVisible();
    await toggle.click();
    await expect(page.getByRole("button", { name: "Expand sidebar" })).toBeVisible();
    expect(await page.evaluate(() => localStorage.getItem("pytxo-desktop-sidebar-collapsed-v1"))).toBe("true");

    await page.reload();
    await expect(page.getByRole("button", { name: "Expand sidebar" })).toBeVisible();
    await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible();
  });

  test("command palette opens via shortcut, filters results, and navigates with the keyboard", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/");
    const dialog = page.getByRole("dialog", { name: "Command menu" });

    // Ctrl/Cmd+K is also a browser chrome shortcut (focus the address bar), so
    // dispatch the keydown directly at the window instead of relying on
    // Playwright's OS-level key press, which the browser may intercept first.
    await page.evaluate(() => window.dispatchEvent(new KeyboardEvent("keydown", { key: "k", ctrlKey: true, bubbles: true, cancelable: true })));
    await expect(dialog).toBeVisible();
    await expect(page.getByLabel("Command search")).toBeFocused();

    await page.keyboard.type("flow");
    await expect(dialog.getByRole("button", { name: /Flow/ })).toBeVisible();
    await page.keyboard.press("Enter");
    await expect(dialog).toBeHidden();
    await expect(page.getByRole("heading", { name: "Flow" })).toBeVisible();

    await page.getByRole("button", { name: "Open command palette" }).click();
    await expect(dialog).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(dialog).toBeHidden();
  });

  test("display scale preference applies and persists only the user zoom", async ({ page }) => {
    await completeOnboarding(page);
    await openAppearance(page);
    await setBrowserPreviewScale(page, "110%");
    await expect(page.getByRole("button", { name: "110%", exact: true })).toHaveClass(/active/);
    await expectRootZoom(page, 1.1);
    expect(await page.evaluate((key) => localStorage.getItem(key), SCALE_STORAGE_KEY)).toBe("1.1");

    await page.reload();
    await page.getByRole("button", { name: "Appearance" }).click();
    await expect(page.getByRole("button", { name: "110%", exact: true })).toHaveClass(/active/);
    await expectRootZoom(page, 1.1);

    await setBrowserPreviewScale(page, "100%");
    await expect(page.getByRole("button", { name: "100%", exact: true })).toHaveClass(/active/);
    await expectRootZoom(page, 1);
  });

  test("invalid persisted display scale is normalized to 100%", async ({ page }) => {
    await completeOnboarding(page, { [SCALE_STORAGE_KEY]: "1.25" });
    await page.goto("/");
    await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible();
    await expectRootZoom(page, 1);
    expect(await page.evaluate((key) => localStorage.getItem(key), SCALE_STORAGE_KEY)).toBe("1");
  });

  test("route persists across a cold reload at the root URL", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/");
    await page.getByRole("link", { name: "Runs" }).click();
    await expect(page.getByRole("heading", { name: "Runs" })).toBeVisible();

    await page.goto("/");
    await expect(page.getByRole("heading", { name: "Runs" })).toBeVisible();
  });

  test("workspaces show an honest empty state when a search matches nothing", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/workspaces");
    await expect(page.getByRole("heading", { name: "Workspaces" })).toBeVisible();
    await page.getByLabel("Search workspaces").fill("zzz-no-such-workspace-zzz");
    await expect(page.getByText("No workspaces match this filter")).toBeVisible();
  });

  test("sidebar reflects real account state instead of a fabricated identity", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/");
    await expect(page.getByText("Not signed in", { exact: true })).toBeVisible();
    await expect(page.getByText("Matt Bacon")).toHaveCount(0);
    await expect(page.getByText("MB", { exact: true })).toHaveCount(0);
  });

  test("text Flow requires preview before mock dispatch", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    await page.getByLabel("Flow outcome").fill("Make the desktop shell production ready");
    await expect(page.getByRole("button", { name: "Build plan", exact: true })).toBeEnabled();
    await page.getByRole("button", { name: /Build plan/ }).click();
    await expect(page.getByRole("heading", { name: "Review plan" })).toBeVisible();
    await page.getByRole("button", { name: "Run", exact: true }).click();
    await expect(page.getByRole("button", { name: "Running run-preview" })).toBeVisible();
  });

  test("Voice capture produces an editable Flow mission with the preview backend", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    await expect(page.getByLabel("Voice input device")).toBeEnabled();
    await page.getByRole("button", { name: "Start Voice" }).click();
    await expect(page.getByRole("button", { name: "Finish recording" })).toBeVisible();
    await page.getByRole("button", { name: "Pause" }).click();
    await expect(page.getByRole("button", { name: "Resume" })).toBeVisible();
    await page.getByRole("button", { name: "Resume" }).click();
    await page.getByRole("button", { name: "Finish recording" }).click();
    await expect(page.getByRole("button", { name: "Transcribing…" })).toBeVisible();
    await expect(page.getByLabel("Flow outcome")).toHaveValue("Create a production-ready Flow mission");
  });

  test("Voice supports press-and-hold capture", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    const voice = page.getByTestId("voice-capture");
    const box = await voice.boundingBox();
    expect(box).not.toBeNull();
    await page.mouse.move(box!.x + box!.width / 2, box!.y + box!.height / 2);
    await page.mouse.down();
    await page.waitForTimeout(400);
    await page.mouse.move(20, 20);
    await page.mouse.up();
    await expect(page.getByLabel("Flow outcome")).toHaveValue("Create a production-ready Flow mission");
  });

  test("Voice pointer cancellation releases capture without retaining audio", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    const voice = page.getByTestId("voice-capture");
    await voice.dispatchEvent("pointerdown", { button: 0, pointerId: 7 });
    await voice.dispatchEvent("pointercancel", { button: 0, pointerId: 7 });
    await expect(page.getByText("Voice capture cancelled · no audio retained")).toBeVisible();
  });

  test("leaving Flow cancels active microphone capture", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    await page.getByRole("button", { name: "Start Voice" }).click();
    await page.getByRole("link", { name: "Ops" }).click();
    await page.getByRole("link", { name: "Flow" }).click();
    await page.getByRole("button", { name: "Start Voice" }).click();
    await expect(page.getByRole("button", { name: "Finish recording" })).toBeVisible();
  });

  test("Voice transcription can be cancelled without retaining audio", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    await page.getByRole("button", { name: "Start Voice" }).click();
    await page.getByRole("button", { name: "Finish recording" }).click();
    await expect(page.getByRole("button", { name: "Cancel" })).toBeVisible();
    await page.getByRole("button", { name: "Cancel" }).click();
    await expect(page.getByText("Voice capture cancelled · no audio retained")).toBeVisible();
  });

  test("Voice exposes device loss and uncertain transcript correction", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    await page.getByLabel("Voice input device").selectOption("Disconnected microphone");
    await page.getByRole("button", { name: "Start Voice" }).click();
    await expect(page.getByText("Voice capture failed · choose another input device")).toBeVisible();

    await page.getByLabel("Voice input device").selectOption("Studio microphone");
    await page.getByRole("button", { name: "Start Voice" }).click();
    await page.getByRole("button", { name: "Finish recording" }).click();
    const correction = page.getByLabel("Correct uncertain segment 1");
    await expect(correction).toBeVisible();
    await correction.fill("Update the desktop approval flow");
    await expect(page.getByLabel("Flow outcome")).toHaveValue("Update the desktop approval flow");
  });

  test("current and legacy deep links route through the same shell", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/");
    await page.evaluate(() => window.dispatchEvent(new CustomEvent("pytxo-deep-link", { detail: "pytxo://flow" })));
    await expect(page.getByRole("heading", { name: "Flow" })).toBeVisible();
    await page.evaluate(() => window.dispatchEvent(new CustomEvent("pytxo-deep-link", { detail: "pytxo-deck://settings" })));
    await expect(page).toHaveURL(/#\/settings$/);
    await expect(page.getByRole("heading", { name: "Appearance" })).toBeVisible();
  });

  test("Ops focus and exact-run stop are keyboard-first but confirmation-gated", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    const mission = page.getByLabel("Flow outcome");
    await mission.focus();
    await mission.dispatchEvent("keydown", {
      key: "o",
      ctrlKey: true,
      shiftKey: true,
      bubbles: true,
      cancelable: true,
    });
    await expect(page.getByRole("heading", { name: "Flow" })).toBeVisible();

    await page.evaluate(() =>
      window.dispatchEvent(
        new KeyboardEvent("keydown", {
          key: "o",
          ctrlKey: true,
          shiftKey: true,
          repeat: true,
          bubbles: true,
          cancelable: true,
        }),
      ),
    );
    await expect(page.getByRole("heading", { name: "Flow" })).toBeVisible();

    await page.evaluate(() =>
      window.dispatchEvent(
        new KeyboardEvent("keydown", {
          key: "o",
          ctrlKey: true,
          shiftKey: true,
          bubbles: true,
          cancelable: true,
        }),
      ),
    );
    await expect(page.getByRole("heading", { name: "Ops" })).toBeVisible();
    await expect(page.getByLabel("Ops keyboard shortcuts")).toBeVisible();

    const activeRun = page.locator(".run-row").filter({ hasText: "run-8f2c" });
    await expect(activeRun).toBeFocused();
    await activeRun.dispatchEvent("keydown", {
      key: "Backspace",
      ctrlKey: true,
      shiftKey: true,
      bubbles: true,
      cancelable: true,
    });

    const dialog = page.getByRole("dialog", { name: "Stop active run?" });
    await expect(dialog).toBeVisible();
    await expect(dialog.getByText("pytxo", { exact: true }).first()).toBeVisible();
    await expect(dialog.getByText("run-8f2c", { exact: true })).toBeVisible();
    await expect(dialog.getByText("No changes are flushed and the sandbox is not deleted.")).toBeVisible();
    await expect(activeRun).toBeVisible();

    await dialog.getByRole("button", { name: "Keep running" }).click();
    await expect(dialog).toBeHidden();
    await expect(activeRun).toBeVisible();

    await page.getByRole("button", { name: "Review stop for run run-8f2c" }).click();
    await expect(dialog).toBeVisible();
    await dialog.getByRole("button", { name: "Stop run" }).click();
    await expect(dialog).toBeHidden();
    await expect(page.getByRole("status")).toContainText("Stop requested for run-8f2c in pytxo.");
    await expect(page.getByText("No active runs", { exact: true })).toBeVisible();
    await expect(page.getByLabel("Ops truth strip").getByRole("button", { name: /Running 0/ })).toBeVisible();
    await expect(page.getByLabel("Ops detail")).toContainText("0/0");
    await expect(page.getByLabel("Ops detail")).toContainText("No agents yet");
  });

  test("approval resolution, focus routes, and restart persistence work", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/approvals");
    await expect(page.getByRole("heading", { name: "Flush Blast Shield workspace" })).toBeVisible();
    await expect(page.getByLabel("Approval keyboard shortcuts")).toBeVisible();
    await expect(page.locator(".decision-meta").getByText("desktop", { exact: true })).toBeVisible();
    await expect(page.locator(".decision-meta").getByText("pytxo", { exact: true })).toBeVisible();
    await expect(page.locator(".decision-meta").getByText("blast.flush", { exact: true })).toBeVisible();
    await expect(page.getByText("Approving writes the isolated workspace changes into the repository. Denying discards them.")).toBeVisible();
    await expect(page.getByText("No secrets or permission escalation detected.")).toHaveCount(0);

    await page.keyboard.press("j");
    await expect(page.getByRole("heading", { name: "Allow network access" })).toBeVisible();
    await expect(page.locator(".decision-meta").getByText("net.egress", { exact: true })).toBeVisible();
    await page.keyboard.press("k");
    await expect(page.getByRole("heading", { name: "Flush Blast Shield workspace" })).toBeVisible();

    await page.getByRole("button", { name: "Review latest run" }).click();
    await expect(page.getByRole("heading", { name: "Run Review" })).toBeVisible();
    await page.goto("/#/approvals");
    await page.keyboard.press("Control+Enter");
    await expect(page.getByRole("status")).toContainText("Approved Flush Blast Shield workspace.");
    await expect(page.getByRole("heading", { name: "Allow network access" })).toBeVisible();
    await page.keyboard.press("Control+Backspace");
    await expect(page.getByRole("status")).toContainText("Denied Allow network access.");
    await expect(page.getByRole("heading", { name: "Inbox clear" })).toBeVisible();
    await page.getByRole("link", { name: "Spaces" }).click();
    for (const workspace of ["pytxo", "signal-lab"]) {
      const card = page.locator(".workspace-card").filter({
        has: page.getByRole("heading", { name: workspace, exact: true }),
      });
      await expect(
        card.locator(".workspace-stats span").filter({ hasText: "Approvals" }).locator("strong"),
      ).toHaveText("0");
    }

    await page.goto("/#/topology-focus");
    await expect(page.getByRole("heading", { name: "Topology Focus" })).toBeVisible();
    await page.goto("/#/run-review");
    await expect(page.getByRole("heading", { name: "Run Review" })).toBeVisible();
    await page.getByRole("link", { name: "Settings" }).click();
    await page.reload();
    await expect(page).toHaveURL(/#\/settings$/);
    await expect(page.getByRole("heading", { name: "Appearance" })).toBeVisible();
  });
});

const VIEWPORTS = [
  { name: "960x640", width: 960, height: 640 },
  { name: "1024x576", width: 1024, height: 576 },
  { name: "1280x720", width: 1280, height: 720 },
  { name: "1280x800", width: 1280, height: 800 },
  { name: "1600x900", width: 1600, height: 900 },
];

test.describe("Pytxo Desktop shell viewport coverage", () => {
  for (const viewport of VIEWPORTS) {
    test.describe(`at ${viewport.name}`, () => {
      test.use({ viewport: { width: viewport.width, height: viewport.height } });

      test("Operations renders without page-level overflow", async ({ page }) => {
        await completeOnboarding(page);
        await page.goto("/");
        await expect(page.getByRole("heading", { name: "Ops" })).toBeVisible();
        // `#app` is intentionally `overflow: hidden` so inner panes scroll
        // instead of the page (see app.css); scrollHeight ignores `overflow`
        // entirely, so the meaningful check is the actual root scrolling
        // container, not the shell div's intrinsic content height.
        const overflow = await page.evaluate(() => ({
          horizontal: document.documentElement.scrollWidth - document.documentElement.clientWidth,
          vertical: document.documentElement.scrollHeight - document.documentElement.clientHeight,
        }));
        expect(overflow.horizontal).toBeLessThanOrEqual(1);
        expect(overflow.vertical).toBeLessThanOrEqual(1);

        await expect(page.getByLabel("Ops keyboard shortcuts")).toBeVisible();
        const stopButton = page.getByRole("button", { name: "Review stop for run run-8f2c" });
        await stopButton.scrollIntoViewIfNeeded();
        await expect(stopButton).toBeVisible();
        await stopButton.click();
        const dialog = page.getByRole("dialog", { name: "Stop active run?" });
        await expect(dialog).toBeVisible();
        const internalOverflow = await Promise.all(
          [page.locator(".ops-truth"), page.locator(".workspace-grid"), dialog, dialog.locator("footer")].map(
            (locator) => locator.evaluate((element) => element.scrollWidth - element.clientWidth),
          ),
        );
        for (const value of internalOverflow) expect(value).toBeLessThanOrEqual(1);
        await expect(dialog.getByRole("button", { name: "Stop run" })).toBeVisible();
        await dialog.getByRole("button", { name: "Keep running" }).click();
      });

      test("planned Flow keeps composer controls reachable", async ({ page }) => {
        await completeOnboarding(page);
        await page.goto("/#/flow");
        await page.getByLabel("Flow outcome").fill("Verify planned Flow controls at supported widths");
        await page.getByRole("button", { name: "Build plan", exact: true }).click();
        await expect(page.getByRole("heading", { name: "Review plan" })).toBeVisible();

        const composer = page.locator(".composer-panel");
        const actions = page.locator(".composer-actions");
        const overflow = await actions.evaluate(
          (element) => element.scrollWidth - element.clientWidth,
        );
        expect(overflow).toBeLessThanOrEqual(1);

        const composerBox = await composer.boundingBox();
        const buildButtonBox = await page
          .getByRole("button", { name: "Build plan", exact: true })
          .boundingBox();
        expect(composerBox).not.toBeNull();
        expect(buildButtonBox).not.toBeNull();
        expect(buildButtonBox!.x + buildButtonBox!.width).toBeLessThanOrEqual(
          composerBox!.x + composerBox!.width + 1,
        );
      });

      test("Approvals keeps evidence and decision controls reachable", async ({ page }) => {
        await completeOnboarding(page);
        await page.goto("/#/approvals");
        await expect(page.getByRole("heading", { name: "Flush Blast Shield workspace" })).toBeVisible();

        const layout = page.locator(".approval-layout");
        const decision = page.locator(".decision-detail");
        const actions = page.locator(".decision-actions");
        const horizontal = await page.evaluate(
          () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
        );
        const layoutOverflow = await layout.evaluate(
          (element) => element.scrollWidth - element.clientWidth,
        );
        const decisionOverflow = await decision.evaluate(
          (element) => element.scrollWidth - element.clientWidth,
        );
        const actionsOverflow = await actions.evaluate(
          (element) => element.scrollWidth - element.clientWidth,
        );
        expect(horizontal).toBeLessThanOrEqual(1);
        expect(layoutOverflow).toBeLessThanOrEqual(1);
        expect(decisionOverflow).toBeLessThanOrEqual(1);
        expect(actionsOverflow).toBeLessThanOrEqual(1);

        const approve = page.getByRole("button", { name: "Approve & flush" });
        await approve.scrollIntoViewIfNeeded();
        await expect(approve).toBeVisible();
        await expect(page.getByRole("button", { name: "Review latest run" })).toBeVisible();
      });

      test("Settings bottom content is reachable via internal scroll", async ({ page }) => {
        await completeOnboarding(page);
        await page.goto("/#/settings");
        await page.getByRole("button", { name: "Account & billing" }).click();
        const onboardingRow = page.getByRole("button", { name: "Run onboarding again" });
        await onboardingRow.scrollIntoViewIfNeeded();
        await expect(onboardingRow).toBeVisible();
        const horizontal = await page.evaluate(
          () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
        );
        expect(horizontal).toBeLessThanOrEqual(1);
      });
    });
  }
});
