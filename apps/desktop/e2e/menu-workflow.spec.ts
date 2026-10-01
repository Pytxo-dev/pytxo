import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";

test.beforeEach(async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 800 });
  await completeOnboarding(page);
});

test("an unfinished mission survives visiting Setup, without retaining plan authority", async ({ page }) => {
  await page.goto("/#/flow");
  const outcome = "Fix src/parser.rs for empty input and add a regression test. Preserve the API.";
  await page.getByLabel("What should Pytxo do?").fill(outcome);
  await page.getByLabel("Agent CLI", { exact: true }).selectOption("claude");
  await page.getByRole("link", { name: "Setup", exact: true }).click();
  await page.getByRole("link", { name: "Work", exact: true }).click();
  await page.getByRole("button", { name: "Continue draft", exact: true }).click();
  await expect(page.getByLabel("What should Pytxo do?")).toHaveValue(outcome);
  await expect(page.getByLabel("Agent CLI", { exact: true })).toHaveValue("claude");
  await expect(page.getByText("Draft restored in this window. Build a fresh plan before running.")).toBeVisible();
  await expect(page.getByRole("button", { name: "Build plan", exact: true })).toBeEnabled();
  await expect(page.getByRole("button", { name: "Start run", exact: true })).toHaveCount(0);
});

test("Use in new work preserves the agent that was clicked", async ({ page }) => {
  await page.goto("/#/setup");
  await page.getByRole("button", { name: "Agents & permissions", exact: true }).click();
  await page.getByText("Additional agents", { exact: true }).click();
  await page.locator(".agent-row", { hasText: "Claude Code" }).getByRole("button", { name: "Use in new work" }).click();
  await expect(page.getByRole("heading", { name: "New work", exact: true })).toBeVisible();
  await expect(page.getByLabel("Agent CLI", { exact: true })).toHaveValue("claude");
});

test("dispatching a retained mission clears the sidebar draft", async ({ page }) => {
  await page.goto("/#/flow");
  await page.getByLabel("What should Pytxo do?").fill("Fix src/parser.rs for empty input and preserve the API.");
  await page.getByRole("link", { name: "History", exact: true }).click();
  await page.getByRole("button", { name: "Continue draft", exact: true }).click();
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  await page.getByRole("button", { name: "Run", exact: true }).click();
  await expect(page.getByRole("region", { name: "Work", exact: true }).getByRole("heading", { level: 1 })).toBeVisible();
  await expect(page.getByRole("button", { name: "Continue draft", exact: true })).toHaveCount(0);
  await page.getByRole("button", { name: "New work from sidebar", exact: true }).click();
  await expect(page.getByLabel("What should Pytxo do?")).toHaveValue("");
});

test("dispatch from the bottom of a plan reveals the live run overview", async ({ page }) => {
  await page.goto("/#/flow");
  await page.getByLabel("What should Pytxo do?").fill("Fix src/parser.rs for empty input and preserve the API.");
  await page.getByRole("button", { name: "Build plan", exact: true }).click();
  const run = page.getByRole("button", { name: "Run", exact: true });
  await run.scrollIntoViewIfNeeded();
  expect(await page.locator(".content").evaluate(element => element.scrollTop)).toBe(0);
  await run.click();
  await expect(page.getByRole("region", { name: "Work", exact: true }).getByRole("heading", { level: 1 })).toBeInViewport();
  await expect(page.locator(".run-bar")).toBeInViewport();
  await expect.poll(() => page.locator(".content").evaluate(element => element.scrollTop)).toBe(0);
});

test("a retained mission keeps its workspace when the shell switches folders", async ({ page }) => {
  await page.goto("/#/flow");
  await page.getByLabel("What should Pytxo do?").fill("Fix src/parser.rs in the original workspace.");
  const original = await page.getByLabel("Project", { exact: true }).inputValue();
  await page.getByTitle("Switch workspace", { exact: true }).click();
  await page.getByRole("dialog", { name: "Switch workspace", exact: true }).getByRole("option", { name: /signal-lab/ }).click();
  await expect(page.getByTitle("Switch workspace", { exact: true })).toContainText("signal-lab");
  await page.getByRole("button", { name: "Continue draft", exact: true }).click();
  await expect(page.getByLabel("Project", { exact: true })).toHaveValue(original);
  await expect(page.getByTitle("Switch workspace", { exact: true })).toContainText("pytxo");
  await expect(page.getByLabel("What should Pytxo do?")).toHaveValue("Fix src/parser.rs in the original workspace.");
});

test("workspace New work keeps context and preserves another workspace's draft", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-history-empty-v1": "1" });
  await page.goto("/#/flow");
  const original = await page.getByLabel("Project", { exact: true }).inputValue();
  const request = "Preserve this unfinished request in the original workspace.";
  await page.getByLabel("What should Pytxo do?").fill(request);
  await page.getByTitle("Switch workspace", { exact: true }).click();
  await page.getByRole("dialog", { name: "Switch workspace", exact: true }).getByRole("option", { name: /signal-lab/ }).click();
  await page.getByRole("button", { name: "New work", exact: true }).click();
  await expect(page.getByLabel("Project", { exact: true })).toHaveValue("signal-lab");
  await expect(page.getByLabel("What should Pytxo do?")).toHaveValue("");
  await expect(page.getByTitle("Switch workspace", { exact: true })).toContainText("signal-lab");
  await page.getByLabel("What should Pytxo do?").fill("A separate request for signal-lab.");
  await page.getByTitle("Switch workspace", { exact: true }).click();
  await page.getByRole("dialog", { name: "Switch workspace", exact: true }).getByRole("option", { name: /^pytxo/ }).click();
  await page.getByRole("button", { name: "New work", exact: true }).click();
  await expect(page.getByLabel("Project", { exact: true })).toHaveValue(original);
  await expect(page.getByLabel("What should Pytxo do?")).toHaveValue(request);
  await page.getByTitle("Switch workspace", { exact: true }).click();
  await page.getByRole("dialog", { name: "Switch workspace", exact: true }).getByRole("option", { name: /signal-lab/ }).click();
  await page.getByRole("button", { name: "New work", exact: true }).click();
  await expect(page.getByLabel("What should Pytxo do?")).toHaveValue("A separate request for signal-lab.");
});

test("sidebar recovers an older workspace draft after opening an empty request elsewhere", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-history-empty-v1": "1" });
  await page.goto("/#/flow");
  await page.getByLabel("What should Pytxo do?").fill("Keep the original workspace request.");
  await page.getByTitle("Switch workspace", { exact: true }).click();
  await page.getByRole("dialog", { name: "Switch workspace", exact: true }).getByRole("option", { name: /signal-lab/ }).click();
  await page.getByRole("button", { name: "New work", exact: true }).click();
  await expect(page.getByLabel("What should Pytxo do?")).toHaveValue("");
  await page.getByTitle("Switch workspace", { exact: true }).click();
  await page.getByRole("dialog", { name: "Switch workspace", exact: true }).getByRole("option", { name: /^pytxo/ }).click();
  await page.getByRole("button", { name: "Continue draft", exact: true }).click();
  await expect(page.getByLabel("What should Pytxo do?")).toHaveValue("Keep the original workspace request.");
});

for (const mode of ["hold", "click"]) {
  test(`Voice respects the ${mode} capture preference`, async ({ page }) => {
    await completeOnboarding(page, { "pytxo-desktop-voice-capture-v1": mode });
    await page.goto("/#/flow");
    await page.locator(".voice-disclosure > summary").click();
    const capture = page.getByTestId("voice-capture");
    if (mode === "hold") {
      await capture.focus();
      await page.keyboard.down("Space");
      await expect(page.getByRole("button", { name: "Pause", exact: true })).toBeVisible();
      await page.keyboard.up("Space");
    } else {
      await capture.click();
      await expect(page.getByRole("button", { name: "Pause", exact: true })).toBeVisible();
      await capture.click();
    }
    await expect(page.getByLabel("What should Pytxo do?")).not.toHaveValue("");
    await expect(page.getByRole("button", { name: "Pause", exact: true })).toHaveCount(0);
  });
}

test("keyboard hold capture cancels when focus moves away", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-desktop-voice-capture-v1": "hold" });
  await page.goto("/#/flow");
  await page.locator(".voice-disclosure > summary").click();
  await page.getByTestId("voice-capture").focus();
  await page.keyboard.down("Space");
  await expect(page.getByRole("button", { name: "Pause", exact: true })).toBeVisible();
  await page.keyboard.press("Tab");
  await page.keyboard.up("Space");
  await expect(page.getByText("Voice capture cancelled · no audio retained")).toBeVisible();
  await expect(page.getByRole("button", { name: "Pause", exact: true })).toHaveCount(0);
  await expect(page.getByLabel("What should Pytxo do?")).toHaveValue("");
});

test("settings search finds controls and recovers from no results", async ({ page }) => {
  await page.goto("/#/setup");
  for (const [query, section] of [[" scale ", "Appearance"], ["TRAY", "General"], ["Whisper", "Voice"], ["updates", "General"], ["api key", "Providers"]]) {
    await page.getByLabel("Search settings").fill(query);
    const match = page.getByRole("navigation", { name: "Settings sections" }).getByRole("button", { name: section, exact: true });
    await expect(match).toBeVisible();
    await match.click();
    await expect(page.locator(".section-title")).toHaveText(section);
  }
  await page.getByLabel("Search settings").fill("no-such-setting");
  await expect(page.getByText("No settings found", { exact: false })).toBeVisible();
  await page.getByRole("button", { name: "Clear settings search" }).click();
  await expect(page.getByRole("navigation", { name: "Settings sections" }).getByRole("button")).toHaveCount(9);
  await expect(page.getByLabel("Search settings")).toBeFocused();
});

test("switching settings sections returns the content pane to the top", async ({ page }) => {
  await page.goto("/#/setup");
  await page.getByRole("button", { name: "Agents & permissions", exact: true }).click();
  await page.getByText("Additional agents", { exact: true }).click();
  await page.getByText("Change default permissions", { exact: true }).click();
  await page.getByRole("button", { name: "Supernova Full host privileges", exact: true }).scrollIntoViewIfNeeded();
  expect(await page.locator(".settings-main").evaluate(element => element.scrollTop)).toBeGreaterThan(100);
  await page.getByRole("button", { name: "Appearance", exact: true }).click();
  await expect.poll(() => page.locator(".settings-main").evaluate(element => element.scrollTop)).toBe(0);
});

test("the command palette scopes Stop to the workspace and opens confirmation", async ({ page }) => {
  await page.goto("/#/work");
  await page.getByTitle("Switch workspace", { exact: true }).click();
  await page.getByRole("dialog", { name: "Switch workspace", exact: true }).getByRole("option", { name: /signal-lab/ }).click();
  await page.getByRole("button", { name: "Open command palette" }).click();
  await page.getByLabel("Command search").fill("stop");
  await expect(page.getByRole("option", { name: /Stop active run/ })).toBeDisabled();
  await page.keyboard.press("Escape");
  await page.getByTitle("Switch workspace", { exact: true }).click();
  await page.getByRole("option", { name: /^pytxo/ }).click();
  await page.getByRole("button", { name: "Open command palette" }).click();
  await page.getByLabel("Command search").fill("stop");
  await page.keyboard.press("Enter");
  await expect(page.getByRole("heading", { name: "Stop run-8f2c?", exact: true })).toBeVisible();
  await expect(page.getByRole("button", { name: "Keep running" })).toBeFocused();
  await page.getByRole("button", { name: "Keep running" }).click();
  await expect(page.locator(".work-heading")).toContainText("Running");
});

test("workspace menus support keyboard selection, Escape and modal focus", async ({ page }) => {
  await page.goto("/#/work");
  const trigger = page.getByTitle("Switch workspace", { exact: true });
  await trigger.focus();
  await page.keyboard.press("ArrowDown");
  await expect(page.getByLabel("Find workspace")).toBeFocused();
  await page.getByLabel("Find workspace").fill("signal");
  await page.keyboard.press("ArrowDown");
  await expect(page.getByRole("dialog", { name: "Switch workspace", exact: true }).getByRole("option", { name: /signal-lab/ })).toBeFocused();
  await page.keyboard.press("Enter");
  await expect(trigger).toContainText("signal-lab");
  await trigger.click();
  await page.keyboard.press("Escape");
  await expect(trigger).toBeFocused();
  await trigger.click();
  await page.getByRole("button", { name: "Workspace settings", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: "Workspace settings · signal-lab" });
  await expect(dialog).toBeVisible();
  await expect(dialog.getByRole("button", { name: "Close", exact: true })).toBeFocused();
  await page.keyboard.press("Shift+Tab");
  expect(await page.evaluate(() => !!document.activeElement?.closest("dialog[open]"))).toBe(true);
  await page.keyboard.press("a");
  await expect(page.getByRole("dialog")).toHaveCount(1);
  await page.keyboard.press("Escape");
  await expect(dialog).toBeHidden();
  await expect(trigger).toBeFocused();
});

test("History detail follows the filtered result and review identifies its workspace", async ({ page }) => {
  await page.goto("/#/history");
  await page.getByLabel("Search work history").fill("run-71ad");
  await expect(page.locator(".history .row")).toHaveCount(1);
  await page.getByRole("button", { name: "Review prepared changes", exact: true }).click();
  await expect(page.getByTitle("Switch workspace", { exact: true })).toContainText("signal-lab");
  await page.getByRole("link", { name: "History", exact: true }).click();
  await page.getByLabel("Search work history").fill("missing-run");
  await expect(page.getByRole("button", { name: "Review prepared changes", exact: true })).toHaveCount(0);
});

for (const theme of ["void", "light"]) {
  test(`all Setup sections and menus remain readable in ${theme}`, async ({ page }, testInfo) => {
    await completeOnboarding(page, { "pytxo-deck-theme": theme });
    await page.goto("/#/setup");
    for (const section of ["General", "Appearance", "Keyboard", "Workspaces", "Agents & permissions", "Providers", "Voice", "Privacy", "Account & billing"]) {
      await page.getByRole("navigation", { name: "Settings sections" }).getByRole("button", { name: section, exact: true }).click();
      await expect(page.locator(".section-title")).toHaveText(section);
      await page.screenshot({ path: testInfo.outputPath(`${section.replace(/\W+/g, "-")}.png`) });
      if (section === "Workspaces") {
        const table = page.getByRole("table");
        const edge = (await table.boundingBox())!;
        for (const header of await table.getByRole("columnheader").all()) {
          expect(await header.evaluate(element => element.scrollWidth - element.clientWidth)).toBeLessThanOrEqual(1);
        }
        for (const action of await table.getByRole("button", { name: "Settings", exact: true }).all()) {
          const box = (await action.boundingBox())!;
          expect(box.x + box.width).toBeLessThanOrEqual(edge.x + edge.width);
        }
      }
      if (section === "Agents & permissions") {
        await page.getByText("Additional agents", { exact: true }).click();
        const connect = page.getByRole("button", { name: "Connect an OpenCode provider", exact: true });
        await connect.scrollIntoViewIfNeeded();
        expect(await connect.evaluate(element => element.scrollHeight - element.clientHeight)).toBeLessThanOrEqual(1);
        await page.screenshot({ path: testInfo.outputPath("agent-actions.png") });
      }
      if (section === "Voice") {
        await page.getByRole("button", { name: "Review consent", exact: true }).click();
        const consent = page.getByRole("dialog", { name: "Cloud transcription consent" });
        await expect(consent.getByRole("heading", { name: "Cloud transcription fallback", exact: true })).toBeVisible();
        // Dialogs use an opaque surface; settings cards intentionally use 70% opacity.
        await expect(consent).toHaveCSS("background-color", theme === "light" ? "rgb(255, 255, 255)" : "rgb(16, 16, 18)");
        await expect(consent.getByRole("button", { name: "Cancel", exact: true })).toBeFocused();
        await page.keyboard.press("Shift+Tab");
        await expect(consent.getByRole("button", { name: "Consent for next session", exact: true })).toBeFocused();
        await page.screenshot({ path: testInfo.outputPath("voice-consent.png") });
        await page.keyboard.press("Escape");
        await expect(consent).toBeHidden();
        await expect(page.getByRole("button", { name: "Review consent", exact: true })).toBeFocused();
      }
    }
    await page.getByRole("button", { name: "Open command palette" }).click();
    await page.getByLabel("Command search").fill("theme");
    await expect(page.getByRole("option", { name: /Appearance/ })).toBeVisible();
    await page.screenshot({ path: testInfo.outputPath("command-menu.png") });
    await page.keyboard.press("Enter");
    await expect(page.locator(".section-title")).toHaveText("Appearance");
    await page.getByTitle("Switch workspace", { exact: true }).click();
    await page.screenshot({ path: testInfo.outputPath("workspace-menu.png") });
    await page.getByRole("button", { name: "Workspace settings", exact: true }).click();
    await expect(page.getByRole("dialog", { name: /Workspace settings/ })).toBeVisible();
    await page.screenshot({ path: testInfo.outputPath("workspace-settings.png") });
    await page.keyboard.press("Escape");
    await page.setViewportSize({ width: 700, height: 800 });
    await expect(page.getByRole("link", { name: "Work", exact: true })).toBeVisible();
    await expect(page.getByRole("link", { name: "Setup", exact: true })).toBeVisible();
    await page.screenshot({ path: testInfo.outputPath("narrow-settings.png") });
    expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBeLessThanOrEqual(1);
  });
}
