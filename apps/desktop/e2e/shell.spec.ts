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
    await expect(page.getByRole("heading", { name: "Your coding agents. One clear place to work.", exact: true })).toBeVisible();
    await expect(page.getByRole("button", { name: "Get started" })).toBeVisible();
    await expect(page.getByLabel("Setup progress")).toBeVisible();
  });

  test("setup skip-all reaches ready step", async ({ page }) => {
    await clearOnboarding(page);
    await page.goto("/");
    await page.getByRole("button", { name: "Get started" }).click();
    await page.getByRole("button", { name: "Continue", exact: true }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await expect(page.getByRole("heading", { name: "Desktop setup complete" })).toBeVisible();
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
    await expect(page.getByRole("heading", { name: "Your coding agents. One clear place to work.", exact: true })).toBeVisible();
  });

  test("onboarding can be replayed from Settings without losing the versioned flag afterwards", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/");
    await page.getByRole("link", { name: "Setup" }).click();
    await page.getByRole("button", { name: "Account & billing" }).click();
    await page.getByRole("button", { name: "Run onboarding again" }).click();
    await expect(page.getByRole("heading", { name: "Your coding agents. One clear place to work.", exact: true })).toBeVisible();
    await page.getByRole("button", { name: "Get started" }).click();
    await page.getByRole("button", { name: "Continue", exact: true }).click();
    await page.getByRole("button", { name: "Skip for now" }).click();
    await page.getByRole("button", { name: "Enter Pytxo Desktop" }).click();
    await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible();
  });

  test("the shell opens in Work and navigates the three destinations", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/");
    await expect(page.getByRole("navigation", { name: "Primary" })).toBeVisible();
    await expect(page.getByRole("region", { name: "Work", exact: true })).toBeVisible();
    await expect(page.getByText(/^Updater:/)).toHaveCount(0);
    await page.getByRole("link", { name: "History" }).click();
    await expect(page.getByRole("heading", { level: 1, name: "History" })).toBeVisible();
    await page.getByRole("link", { name: "Setup" }).click();
    await expect(page.getByRole("heading", { level: 1, name: "Setup" })).toBeVisible();
  });

  test("Agents reports vendor-owned sessions without exposing account identifiers", async ({ page }) => {
    test.slow();
    await completeOnboarding(page);
    await page.goto("/#/integrations", { waitUntil: "domcontentloaded", timeout: 60_000 });

    await expect(page.getByRole("heading", { name: "Agents" })).toBeVisible();
    await expect(page.getByText("ChatGPT connected", { exact: true })).toBeVisible();
    await expect(page.getByText("Claude account connected", { exact: true })).toBeVisible();
    await expect(page.getByText("Cursor account connected", { exact: true })).toBeVisible();
    await expect(page.getByText("No OpenCode provider connected", { exact: true })).toBeVisible();
    await expect(page.getByText("credentials stay with each vendor CLI")).toBeVisible();
    await expect(page.getByText(/[\w.+-]+@[\w.-]+\.[A-Za-z]{2,}/)).toHaveCount(0);

    await page.getByRole("button", { name: "Recheck all" }).click();
    await expect(page.getByText("ChatGPT connected", { exact: true })).toBeVisible();

    await page.getByRole("button", { name: "Use in mission" }).first().click();
    await expect(page.getByRole("heading", { name: "New run" })).toBeVisible();
  });

  test("Providers distinguishes direct API keys from agent subscription sessions", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/settings");
    await page.getByRole("button", { name: "Providers" }).click();

    await expect(page.getByRole("heading", { name: "API providers" })).toBeVisible();
    await expect(page.getByText("DeepSeek", { exact: true })).toBeVisible();
    await expect(page.getByText("Metered DeepSeek API access. This is an API key, not a consumer login.")).toBeVisible();
    await expect(page.getByText("ChatGPT connects through Codex")).toBeVisible();
    await expect(page.getByText("Key values never enter Desktop.")).toBeVisible();
    await expect(page.getByRole("button", { name: "DEEPSEEK_API_KEY" })).toBeVisible();
    await expect(page.locator('input[type="password"]')).toHaveCount(0);

    await page.getByRole("button", { name: /Show all/ }).click();
    await expect(page.getByText("Google Gemini", { exact: true })).toBeVisible();
  });

  test("onboarding offers detected agents and a no-key guided example", async ({ page }) => {
    await clearOnboarding(page);
    await page.goto("/");
    await page.getByRole("button", { name: "Get started" }).click();

    await expect(page.getByRole("heading", { name: "Connect your coding agents" })).toBeVisible();
    await expect(page.getByText("ChatGPT connected", { exact: true })).toBeVisible();
    await expect(page.getByText("Use its existing account; Pytxo can open the tool's own sign-in if needed.")).toBeVisible();
    await page.getByRole("button", { name: "Continue" }).click();

    await expect(page.getByRole("heading", { name: "Choose your project" })).toBeVisible();
    await page.getByRole("button", { name: "Try the guided example" }).click();
    await expect(page.getByText("C:/Users/demo/Documents/Pytxo Examples/approval-risk-demo", { exact: true })).toBeVisible();
    await expect(page.getByText("Guided local Git example ready. Its baseline tests need no API key.")).toBeVisible();
    await page.getByRole("button", { name: "Continue" }).click();
    await expect(page.getByRole("heading", { name: "Desktop setup complete" })).toBeVisible();
  });

  test("Workspaces creates the guided example and takes it into a new run", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/workspaces");
    await page.getByRole("button", { name: "Try guided example" }).click();
    await expect(page.getByRole("heading", { name: "New run" })).toBeVisible();
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

    await page.keyboard.type("new");
    await expect(dialog.getByRole("option", { name: /New run/ })).toBeVisible();
    await page.keyboard.press("Enter");
    await expect(dialog).toBeHidden();
    await expect(page.getByRole("heading", { name: "New run" })).toBeVisible();

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
    await page.getByRole("link", { name: "History" }).click();
    await expect(page.getByRole("heading", { level: 1, name: "History" })).toBeVisible();

    await page.goto("/");
    await expect(page.getByRole("heading", { level: 1, name: "History" })).toBeVisible();
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
    await expect(page.getByRole("button", { name: "Open account and billing" })).toContainText("Local Core");
    await expect(page.getByText("Matt Bacon")).toHaveCount(0);
    await expect(page.getByText("MB", { exact: true })).toHaveCount(0);
    await page.getByRole("button", { name: "Open account and billing" }).click();
    await expect(page.getByText("Not signed in", { exact: true })).toBeVisible();
  });

  test("text Flow requires preview before mock dispatch", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    await page.getByLabel("What should Pytxo do?").fill("Make the desktop shell production ready");
    await expect(page.getByRole("button", { name: "Build plan", exact: true })).toBeEnabled();
    await page.getByRole("button", { name: /Build plan/ }).click();
    await expect(page.getByRole("heading", { name: "Review plan" })).toBeVisible();
    await page.getByRole("button", { name: "Run", exact: true }).click();
    await expect(page.getByRole("region", { name: "Work", exact: true })).toBeVisible();
  });

  test("Flow invalidates a ready plan after mission, workspace, or CLI edits", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    const outcome = page.getByLabel("What should Pytxo do?");
    const composerBuild = page.locator(".composer-panel").getByRole("button", { name: "Build plan", exact: true });
    const run = page.getByRole("button", { name: "Run", exact: true });

    await outcome.fill("Prepare an exact release-safe patch");
    await composerBuild.click();
    await expect(run).toBeEnabled();

    await outcome.fill("Prepare an exact release-safe patch with tests");
    await expect(page.getByText("Plan is stale")).toBeVisible();
    await expect(run).toBeDisabled();
    await composerBuild.click();
    await expect(run).toBeEnabled();

    await page.getByLabel("Project", { exact: true }).selectOption("signal-lab");
    await expect(run).toBeDisabled();
    await composerBuild.click();
    await expect(run).toBeEnabled();

    await page.getByLabel("Agent CLI").selectOption("claude");
    await expect(run).toBeDisabled();
    await composerBuild.click();
    await expect(run).toBeEnabled();
  });

  test("a failed replacement preview clears the old authority and keeps Run disabled", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    const outcome = page.getByLabel("What should Pytxo do?");
    const composerBuild = page.locator(".composer-panel").getByRole("button", { name: "Build plan", exact: true });

    await outcome.fill("Prepare a valid plan first");
    await composerBuild.click();
    await expect(page.getByRole("button", { name: "Run", exact: true })).toBeEnabled();

    await outcome.fill("Now preview the edited mission");
    await page.evaluate(() => localStorage.setItem("pytxo-preview-flow-error-v1", "1"));
    await composerBuild.click();
    await expect(page.getByText("Preview failed while checking the selected workspace.")).toBeVisible();
    await expect(page.getByText("The previous plan was cleared.")).toBeVisible();
    await expect(page.getByRole("button", { name: "Run", exact: true })).toBeDisabled();
  });

  test("Flow auto-selects the first ready detected CLI and explains unavailable choices", async ({ page }) => {
    await completeOnboarding(page, { "pytxo-preview-ade-state-v1": "codex-only" });
    await page.goto("/#/flow");

    await expect(page.getByLabel("Agent CLI")).toHaveValue("codex");
    await expect(page.getByText("OpenAI Codex: ready")).toBeVisible();
    await expect(page.getByText("Claude Code · not installed", { exact: true })).not.toBeVisible();
    await page.getByText("7 other CLIs unavailable", { exact: true }).click();
    await expect(page.getByText("Claude Code · not installed", { exact: true })).toBeVisible();
    await page.getByLabel("What should Pytxo do?").fill("Use the detected ready CLI");
    await expect(page.locator(".composer-panel").getByRole("button", { name: "Build plan" })).toBeEnabled();
  });

  test("Flow persists an explicit ready CLI choice", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    await page.getByLabel("Agent CLI").selectOption("claude");
    await page.reload();
    await expect(page.getByLabel("Agent CLI")).toHaveValue("claude");
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
    await expect(page.getByLabel("What should Pytxo do?")).toHaveValue("Create a production-ready Flow mission");
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
    await expect(page.getByLabel("What should Pytxo do?")).toHaveValue("Create a production-ready Flow mission");
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
    await page.getByRole("link", { name: "Work" }).click();
    await page.getByRole("button", { name: "New run from sidebar", exact: true }).click();
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
    await expect(page.getByLabel("What should Pytxo do?")).toHaveValue("Update the desktop approval flow");
  });

  test("current and legacy deep links route through the same shell", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/");
    await page.evaluate(() => window.dispatchEvent(new CustomEvent("pytxo-deep-link", { detail: "pytxo://flow" })));
    await expect(page.getByRole("heading", { name: "New run" })).toBeVisible();
    await page.evaluate(() => window.dispatchEvent(new CustomEvent("pytxo-deep-link", { detail: "pytxo-deck://settings" })));
    await expect(page).toHaveURL(/#\/setup$/);
    await expect(page.getByRole("heading", { name: "Appearance" })).toBeVisible();
  });

  test("Work focus and exact-run stop are keyboard-first but confirmation-gated", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/flow");
    const outcome = page.getByLabel("What should Pytxo do?");
    await outcome.focus();
    await outcome.dispatchEvent("keydown", {
      key: "o",
      ctrlKey: true,
      shiftKey: true,
      bubbles: true,
      cancelable: true,
    });
    // The chord is ignored inside a text field, so the composer stays put.
    await expect(page.getByRole("heading", { name: "New run" })).toBeVisible();

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
    await expect(page.getByRole("heading", { name: "New run" })).toBeVisible();

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
    await expect(page.getByRole("region", { name: "Work", exact: true })).toBeVisible();

    // This workspace has one visible run, so there is no redundant run switch.
    // Focus lands on the scoped Work heading and the exact run remains visible.
    await expect(page.locator(".work-heading > div")).toBeFocused();
    await expect(page.getByRole("region", { name: "Focused run" })).toContainText("run-8f2c");

    const stop = page.getByRole("button", { name: "Stop", exact: true });
    await stop.click();
    const dialog = page.locator("dialog.confirm-dialog");
    await expect(dialog).toBeVisible();
    await expect(dialog.getByRole("heading", { name: "Stop run-8f2c?" })).toBeVisible();
    await expect(dialog.getByText("pytxo", { exact: true })).toBeVisible();
    await expect(
      dialog.getByText(/Work already prepared for review is kept outside the repository until explicit Apply\./),
    ).toBeVisible();

    await dialog.getByRole("button", { name: "Keep running" }).click();
    await expect(dialog).toBeHidden();

    await stop.click();
    await expect(dialog).toBeVisible();
    await dialog.getByRole("button", { name: "Stop run" }).click();
    await expect(dialog).toBeHidden();
    await expect(page.locator('.work-feedback[role="status"]')).toHaveText(
      "Stop requested for run-8f2c in pytxo.",
    );
  });

  test("approval resolution, focus routes, and restart persistence work", async ({ page }) => {
    await completeOnboarding(page);
    await page.goto("/#/approvals");
    const inbox = page.getByRole("dialog", { name: "Approvals inbox" });
    await expect(inbox).toBeVisible();
    await expect(inbox.getByRole("heading", { name: "Apply reviewed workspace changes" })).toBeVisible();
    await expect(inbox.locator("dl").getByText("desktop", { exact: true })).toBeVisible();
    await expect(inbox.locator("dl").getByText("pytxo", { exact: true })).toBeVisible();
    await expect(inbox.locator("dl").getByText("blast.flush", { exact: true })).toBeVisible();
    await expect(inbox.getByText("Approving writes the isolated workspace changes into the repository. Denying discards them.")).toBeVisible();
    await expect(page.getByText("No secrets or permission escalation detected.")).toHaveCount(0);

    await page.keyboard.press("j");
    await expect(inbox.getByRole("heading", { name: "Allow network access" })).toBeVisible();
    await expect(inbox.locator("dl").getByText("net.egress", { exact: true })).toBeVisible();
    await page.keyboard.press("k");
    await expect(inbox.getByRole("heading", { name: "Apply reviewed workspace changes" })).toBeVisible();

    // Opening the run closes the overlay: the decision and the evidence are two
    // surfaces, and the operator is now on the evidence one.
    await inbox.getByRole("button", { name: "Review run" }).click();
    await expect(inbox).toBeHidden();
    await expect(page.getByRole("heading", { name: "Run Review" })).toBeVisible();

    await page.goto("/#/approvals");
    await page.keyboard.press("Control+Enter");
    await expect(inbox.getByRole("status")).toContainText("Approved Apply reviewed workspace changes.");
    await expect(inbox.getByRole("heading", { name: "Allow network access" })).toBeVisible();
    await page.keyboard.press("Control+Backspace");
    await expect(inbox.getByRole("status")).toContainText("Denied Allow network access.");
    await expect(inbox.getByText("Inbox clear").first()).toBeVisible();

    await inbox.getByRole("button", { name: "Close approvals" }).click();
    await page.goto("/#/workspaces");
    for (const workspace of ["pytxo", "signal-lab"]) {
      const row = page.locator(".workspace-table .table-row").filter({ hasText: workspace });
      // Compact rows include their column label; the count must still be zero.
      await expect(row.locator('[data-col="approvals"]')).toHaveText("Approvals0");
    }

    // The legacy 3D topology deep link now lands on Review, the surface that
    // actually answers what the run is about to do.
    await page.goto("/#/topology-focus");
    await expect(page.getByRole("heading", { name: "Run Review" })).toBeVisible();
    await page.getByRole("link", { name: "Setup" }).click();
    await page.reload();
    await expect(page).toHaveURL(/#\/setup/);
    // Both the destination and the section the operator last read survive the
    // restart, so a reload does not silently relocate them.
    await expect(page.getByRole("heading", { level: 1, name: "Setup" })).toBeVisible();
    await expect(page.getByRole("heading", { level: 2, name: "Workspaces" })).toBeVisible();
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

      test("Work renders without page-level overflow", async ({ page }) => {
        await completeOnboarding(page);
        await page.goto("/");
        await expect(page.getByRole("region", { name: "Work", exact: true })).toBeVisible();
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

        const stopButton = page.getByRole("button", { name: "Stop", exact: true });
        await stopButton.scrollIntoViewIfNeeded();
        await expect(stopButton).toBeVisible();
        await stopButton.click();
        const dialog = page.locator("dialog.confirm-dialog");
        await expect(dialog).toBeVisible();
        const internalOverflow = await Promise.all(
          [page.locator(".work-layout"), dialog, dialog.locator(".dialog-actions")].map(
            (locator) => locator.evaluate((element) => element.scrollWidth - element.clientWidth),
          ),
        );
        for (const value of internalOverflow) expect(value).toBeLessThanOrEqual(1);
        await expect(dialog.getByRole("button", { name: "Stop run" })).toBeVisible();
        await dialog.getByRole("button", { name: "Keep running" }).click();
      });

      test("a planned run keeps composer then plan controls reachable", async ({ page }) => {
        await completeOnboarding(page);
        await page.goto("/#/flow");
        await page.getByLabel("What should Pytxo do?").fill("Verify planned Flow controls at supported widths");

        const composer = page.locator(".composer-panel");
        const actions = page.locator(".composer-actions");
        const workspaceSelect = page.getByLabel("Project", { exact: true });
        const agentSelect = page.getByLabel("Agent CLI");
        const overflow = await actions.evaluate(
          (element) => element.scrollWidth - element.clientWidth,
        );
        expect(overflow).toBeLessThanOrEqual(1);

        const composerBox = await composer.boundingBox();
        const workspaceBox = await workspaceSelect.boundingBox();
        const agentBox = await agentSelect.boundingBox();
        const buildButtonBox = await page
          .getByRole("button", { name: "Build plan", exact: true })
          .boundingBox();
        expect(composerBox).not.toBeNull();
        expect(workspaceBox).not.toBeNull();
        expect(agentBox).not.toBeNull();
        expect(buildButtonBox).not.toBeNull();
        for (const box of [workspaceBox!, agentBox!, buildButtonBox!]) {
          expect(box.x).toBeGreaterThanOrEqual(composerBox!.x - 1);
          expect(box.x + box.width).toBeLessThanOrEqual(composerBox!.x + composerBox!.width + 1);
        }

        await page.getByRole("button", { name: "Build plan", exact: true }).click();
        await expect(page.getByRole("heading", { name: "Review plan" })).toBeVisible();
        const runBox = await page.getByRole("button", { name: "Run", exact: true }).boundingBox();
        const planBox = await page.locator(".plan-panel").boundingBox();
        expect(runBox).not.toBeNull();
        expect(planBox).not.toBeNull();
        expect(runBox!.x).toBeGreaterThanOrEqual(planBox!.x - 1);
        expect(runBox!.x + runBox!.width).toBeLessThanOrEqual(planBox!.x + planBox!.width + 1);
      });

      test("the approvals overlay keeps evidence and decision controls reachable", async ({ page }) => {
        await completeOnboarding(page);
        await page.goto("/#/approvals");
        const inbox = page.getByRole("dialog", { name: "Approvals inbox" });
        await expect(inbox.getByRole("heading", { name: "Apply reviewed workspace changes" })).toBeVisible();

        const horizontal = await page.evaluate(
          () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
        );
        expect(horizontal).toBeLessThanOrEqual(1);

        for (const region of [
          inbox,
          inbox.locator(".inbox-layout"),
          inbox.locator(".detail"),
          inbox.locator(".actions"),
        ]) {
          expect(
            await region.evaluate((element) => element.scrollWidth - element.clientWidth),
          ).toBeLessThanOrEqual(1);
        }

        const approve = inbox.getByRole("button", { name: /Approve and apply/ });
        await approve.scrollIntoViewIfNeeded();
        await expect(approve).toBeVisible();
        await expect(inbox.getByRole("button", { name: "Review run" })).toBeVisible();
      });

      test("Setup bottom content is reachable via internal scroll", async ({ page }) => {
        await completeOnboarding(page);
        await page.goto("/#/setup");
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
