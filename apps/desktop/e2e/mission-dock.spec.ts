import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";
import { closeDockView, defaultDockLayout, dockId, moveDockView, openDockView, restoreDockLayout } from "../src/lib/dock-layout";

test("New run hides focused inspection without losing saved views", async ({ page }) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Checks & details", exact: true }).click();
  await page.getByLabel("Options for Checks & details", { exact: true }).click();
  await page.getByRole("button", { name: "Focus view", exact: true }).click();
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toBeVisible();
  await page.getByRole("button", { name: "New run from sidebar" }).click();
  await expect(page.getByRole("tablist", { name: /dock$/ })).toHaveCount(0);
  await expect(page.getByLabel("Task views", { exact: true })).toHaveCount(0);
  expect((await page.locator(".content").boundingBox())!.height).toBeGreaterThan(500);
  await page.getByRole("link", { name: "Work", exact: true }).click();
  await expect(page.getByRole("tab", { name: "Checks & details", exact: true })).toBeVisible();
  await page.setViewportSize({ width: 740, height: 800 });
  await page.getByRole("button", { name: "Checks & details", exact: true }).click();
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toBeVisible();
  await page.getByRole("button", { name: "New run from sidebar" }).click();
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toBeHidden();
  expect((await page.locator(".content").boundingBox())!.height).toBeGreaterThan(500);
  await page.getByRole("link", { name: "Work", exact: true }).click();
  await page.getByRole("button", { name: "Checks & details", exact: true }).click();
  await expect(page.getByRole("tab", { name: "Checks & details", exact: true })).toBeVisible();
});

test("dock identity includes domain and pinned observers survive selection", () => {
  const a = { kind: "agent" as const, domainId: "a", runId: "run", agentId: "worker", title: "Worker" };
  const b = { ...a, domainId: "b" };
  let layout = openDockView(defaultDockLayout(), a);
  layout.views[0].pinned = true;
  layout = openDockView(layout, b);
  expect(layout.views).toHaveLength(2);
  const original = JSON.stringify(layout);
  const moved = moveDockView(layout, dockId(a), "bottom");
  expect(JSON.stringify(layout)).toBe(original);
  expect(moved.active.bottom).toBe(dockId(a));
  expect(moved.active.right).toBe(dockId(b));
  expect(restoreDockLayout(JSON.stringify(moved))).toEqual(moved);
  expect(closeDockView(moved, dockId(a)).views[0].domainId).toBe("b");
  expect(restoreDockLayout('{"version":1,"views":[{"kind":"shell"}]}').views).toEqual([]);
});

test("mission remains actionable while inspection moves, resizes, hides and restores", async ({ page }) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1600, height: 1000 });
  await page.goto("/#/work");
  await expect(page.getByRole("region", { name: "Work", exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Checks & details", exact: true }).click();
  await expect(page.getByRole("tab", { name: "Checks & details", exact: true })).toBeVisible();
  await expect(page.getByText("Combined candidate verification has not been recorded.", { exact: true })).toBeVisible();
  // The preview snapshot deliberately lacks package metadata while the exact
  // review response contains it: the detail must supersede that older snapshot.
  const packageRow = page.locator(".dock-panel:visible .candidate dl > div").filter({ has: page.locator("dt", { hasText: /^Package$/ }) });
  await expect(packageRow.locator("dd")).toHaveText("pkg-8f2c-immutable");
  const splitter = page.getByRole("separator", { name: "Resize right dock" });
  const before = Number(await splitter.getAttribute("aria-valuenow"));
  await splitter.focus();
  await page.keyboard.press("ArrowLeft");
  await expect(splitter).toHaveAttribute("aria-valuenow", String(before + 10));
  await page.getByLabel("Options for Checks & details").click();
  await page.getByRole("button", { name: "Move to bottom", exact: true }).click();
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toContainText("Checks & details");
  await page.getByRole("button", { name: "Hide bottom dock", exact: true }).click();
  await expect(page.getByRole("tab", { name: "Checks & details", exact: true })).toBeHidden();
  await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeVisible();
  await page.getByText("Layout · 1 views", { exact: true }).click();
  await page.getByRole("button", { name: "Show bottom dock", exact: true }).click();
  await page.reload();
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toContainText("Checks & details");
  await page.getByText("Layout · 1 views", { exact: true }).click();
  await page.getByRole("button", { name: "Reset layout", exact: true }).click();
  await expect(page.getByRole("tablist", { name: "right dock" })).toContainText("Checks & details");
  await page.screenshot({ path: "../../target/astra-mission-dock-20260912/browser-wide.png" });
  await page.setViewportSize({ width: 1000, height: 800 });
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toContainText("Checks & details");
  expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
  await page.screenshot({ path: "../../target/astra-mission-dock-20260912/browser-laptop.png" });
});

test("long mission and bottom output leave room for decisions, with full source details reachable", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "long-mission" });
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#/work");
  const brief = page.locator(".mission-outcome");
  await expect(brief).toBeVisible();
  await expect(brief).not.toHaveAttribute("open", "");
  await expect(page.locator(".work-heading h1")).toHaveText(/…$/);
  expect((await brief.boundingBox())!.height).toBeLessThan(80);
  await brief.locator("summary").focus(); await page.keyboard.press("Enter");
  await expect(brief.locator("p")).toBeVisible();
  expect((await brief.locator("p").textContent())!.length).toBeGreaterThan(800);
  await page.keyboard.press("Enter");
  await page.locator(".ledger").getByRole("button", { name: /architect/ }).click();
  await page.locator(".dock-panel:visible summary").filter({ hasText: "View options" }).click();
  await page.getByRole("button", { name: "Move to bottom", exact: true }).click();
  const panel = page.locator(".dock-panel:visible");
  const output = panel.getByRole("region", { name: "Recorded agent output", exact: true });
  await expect(output).toContainText("Browser fixture: recorded output example.");
  await expect(output).toBeInViewport({ ratio: 1 });
  expect((await output.boundingBox())!.height).toBeGreaterThanOrEqual(64);
  await expect(panel.getByText("Plain text · read only · claims", { exact: true })).toBeInViewport();
  await expect(output).not.toContainText("\x1b");
  await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeInViewport({ ratio: 1 });
  await panel.locator(".source summary").focus(); await page.keyboard.press("Enter");
  await expect(panel.locator(".source")).toContainText("Run run-8f2c");
  await expect(panel.locator(".source")).toContainText("Agent architect");
  await expect(panel.locator(".source")).toContainText("Independent checks appear in Checks & details");
  await panel.getByRole("button", { name: "Show raw event text", exact: true }).click();
  await expect(output).toContainText("\x1b");
  await panel.getByRole("button", { name: "Show plain text", exact: true }).click();
  await expect(output).not.toContainText("\x1b");
  await panel.locator(".source summary").focus();
  await page.keyboard.press("Enter");
  await page.getByRole("button", { name: "Checks & details", exact: true }).click();
  const bottomResize = page.getByRole("separator", { name: "Resize bottom dock" });
  await bottomResize.focus(); await page.keyboard.press("End"); await page.keyboard.press("Tab");
  await expect(page.getByRole("tablist", { name: "right dock" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeInViewport({ ratio: 1 });
  await expect(page.getByRole("button", { name: "Review changes", exact: true })).toBeInViewport({ ratio: 1 });
  await page.reload();
  await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeInViewport({ ratio: 1 });
});

test("prepared file dock reads immutable before/after content", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/history");
  await page.locator(".history .row", { hasText: "run-71ad" }).first().click();
  await page.getByRole("button", { name: "Review changes", exact: true }).click();
  await page.getByRole("button", { name: "Files", exact: true }).click();
  await page.locator(".dock-panel:visible .file").filter({ hasText: "crates/pytxo-signal/src/lib.rs" }).click();
  const frozen = page.getByRole("region", { name: "Frozen contents of crates/pytxo-signal/src/lib.rs", exact: true });
  await expect(frozen.getByRole("region", { name: "before content", exact: true })).toContainText("pub fn");
  await expect(frozen.getByRole("region", { name: "after content", exact: true })).toContainText("pub fn");
  await expect(frozen.getByRole("alert")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Apply reviewed changes", exact: true })).toBeVisible();
});

test("Review adapts to remaining workspace width beside the right dock", async ({ page }) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#/history");
  await page.locator(".history .row", { hasText: "run-71ad" }).first().click();
  await page.getByRole("button", { name: "Review changes", exact: true }).click();
  await page.getByRole("button", { name: "Checks & details", exact: true }).click();
  await expect(page.getByRole("tablist", { name: "right dock" })).toBeVisible();
  const files = page.locator(".files-panel");
  const grid = page.locator(".review-grid");
  expect((await files.boundingBox())!.width).toBeGreaterThan((await grid.boundingBox())!.width - 4);
  const filename = files.locator(".file-row strong").first();
  expect((await filename.boundingBox())!.width).toBeGreaterThan(200);
  await expect(page.getByRole("button", { name: "Apply reviewed changes", exact: true })).toBeInViewport();
  expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
});

test("narrow arrangement persists without overwriting wide placement or geometry", async ({ page }) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1600, height: 1000 });
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Checks & details", exact: true }).click();
  await page.getByRole("button", { name: "Files", exact: true }).click();
  const wide = page.getByRole("separator", { name: "Resize right dock" });
  const width = await wide.getAttribute("aria-valuenow");
  await page.setViewportSize({ width: 1000, height: 800 });
  await page.getByLabel("Options for Files", { exact: true }).click();
  await page.getByRole("button", { name: "Move tab left", exact: true }).click();
  const bottom = page.getByRole("tablist", { name: "bottom dock" });
  await expect(bottom.getByRole("tab").first()).toHaveText("Files");
  const height = page.getByRole("separator", { name: "Resize bottom dock" });
  await height.focus(); await page.keyboard.press("Shift+ArrowUp"); await page.keyboard.press("Tab");
  const savedHeight = await height.getAttribute("aria-valuenow");
  await page.reload();
  await expect(bottom.getByRole("tab").first()).toHaveText("Files");
  await expect(height).toHaveAttribute("aria-valuenow", savedHeight!);
  await page.getByRole("button", { name: "Hide bottom dock", exact: true }).click();
  await page.reload(); await expect(bottom).toBeHidden();
  await page.setViewportSize({ width: 1600, height: 1000 });
  await expect(wide).toHaveAttribute("aria-valuenow", width!);
  await expect(page.getByRole("tablist", { name: "right dock" }).getByRole("tab").first()).toHaveText("Checks & details");
  await expect(page.getByRole("tab", { name: "Files", exact: true })).toHaveAttribute("aria-selected", "true");
});

test("hiding a focused inspection returns to the mission without reopening its original dock", async ({ page }) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1600, height: 1000 });
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Checks & details", exact: true }).click();
  await page.getByLabel("Options for Checks & details", { exact: true }).click();
  await page.getByRole("button", { name: "Focus view", exact: true }).click();
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toBeVisible();
  const focusedResize = page.getByRole("separator", { name: "Resize bottom dock" });
  const focusedBefore = await page.locator(".dock-panel:visible").boundingBox();
  await focusedResize.focus(); await page.keyboard.press("ArrowDown"); await page.keyboard.press("Tab");
  const focusedAfter = await page.locator(".dock-panel:visible").boundingBox();
  expect(focusedBefore!.height - focusedAfter!.height).toBeCloseTo(10, 0);
  await page.getByRole("button", { name: "Hide bottom dock", exact: true }).click();
  await expect(page.getByRole("tablist", { name: /dock$/ })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeInViewport();
});

test("mission actions and inspection stay reachable at 200 percent text zoom", async ({ page }) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Checks & details", exact: true }).click();
  // Browser CSS-zoom simulation; this is not Windows display-scaling evidence.
  await page.evaluate(() => document.documentElement.style.zoom = "2");
  await expect(page.getByRole("complementary", { name: "Primary sidebar" })).toHaveClass(/collapsed/);
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth - document.documentElement.clientWidth)).toBeLessThanOrEqual(1);
  const compactResize = page.getByRole("separator", { name: "Resize bottom dock" });
  const panelBefore = await page.locator(".dock-panel:visible").boundingBox();
  const sizeBefore = Number(await compactResize.getAttribute("aria-valuenow"));
  await compactResize.focus(); await page.keyboard.press("ArrowDown"); await page.keyboard.press("Tab");
  const panelAfter = await page.locator(".dock-panel:visible").boundingBox();
  expect(panelBefore!.height - panelAfter!.height).toBeCloseTo(20, 0);
  await expect(compactResize).toHaveAttribute("aria-valuenow", String(sizeBefore - 10));
  const rail = await compactResize.boundingBox();
  await page.mouse.move(rail!.x + rail!.width / 2, rail!.y + rail!.height / 2);
  await page.mouse.down(); await page.mouse.move(rail!.x + rail!.width / 2, rail!.y + rail!.height / 2 - 20, { steps: 5 }); await page.mouse.up();
  const dragged = await page.locator(".dock-panel:visible").boundingBox();
  expect(dragged!.height - panelAfter!.height).toBeCloseTo(20, 0);
  const stop = page.getByRole("button", { name: "Stop", exact: true });
  await stop.scrollIntoViewIfNeeded();
  await page.screenshot({ path: "../../target/astra-mission-dock-20260912/browser-200-percent-inspection.png" });
  await expect(stop).toBeInViewport({ ratio: 1 });
  await stop.click();
  await expect(page.getByRole("button", { name: "Keep running", exact: true })).toBeInViewport();
  await page.getByRole("button", { name: "Keep running", exact: true }).click();
  await page.getByRole("button", { name: "Hide bottom dock", exact: true }).click();
  await expect(stop).toBeVisible();
  await page.screenshot({ path: "../../target/astra-mission-dock-20260912/browser-200-percent.png" });
});

test("named layouts and focus preserve mission warnings, and Escape cancels keyboard resizing", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-state-matrix-v1": "1" });
  await page.setViewportSize({ width: 1600, height: 1000 });
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Checks & details", exact: true }).click();
  const separator = page.getByRole("separator", { name: "Resize right dock" });
  const original = await separator.getAttribute("aria-valuenow");
  await separator.focus(); await page.keyboard.press("Shift+ArrowLeft"); await page.keyboard.press("Escape");
  await expect(separator).toHaveAttribute("aria-valuenow", original!);
  await page.getByText("Layout · 1 views", { exact: true }).click();
  await page.getByRole("button", { name: "Save current layout", exact: true }).click();
  await page.getByLabel("Layout name", { exact: true }).fill("Review desk");
  await page.getByRole("button", { name: "Save layout", exact: true }).click();
  await page.getByLabel("Options for Checks & details").click();
  await page.getByRole("button", { name: "Focus view", exact: true }).click();
  await expect(page.locator(".work").getByText("Working tree may be partially modified", { exact: true })).toBeVisible();
  await page.locator(".workspace-tools").getByRole("button", { name: "Return to task", exact: true }).click();
  await page.getByRole("button", { name: "Hide right dock", exact: true }).click();
  await expect(page.locator(".work").getByText("Working tree may be partially modified", { exact: true })).toBeVisible();
  await page.getByText("Layout · 1 views", { exact: true }).click();
  await page.getByRole("button", { name: "Restore Review desk", exact: true }).click();
  await expect(page.getByRole("tablist", { name: "right dock" })).toBeVisible();
  await page.setViewportSize({ width: 720, height: 700 });
  await expect(page.locator(".workspace-tools").getByRole("button", { name: "Return to task", exact: true })).toBeVisible();
  await page.locator(".workspace-tools").getByRole("button", { name: "Return to task", exact: true }).click();
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toBeHidden();
  await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeVisible();
});

test("terminal UI routes only explicitly enabled input and preserves sessions when views close", async ({ page }) => {
  // UI contract fixture only. Actual ConPTY execution is covered by native Rust
  // tests; this mock cannot be used as evidence that a native shell ran.
  const ref = { kind: "terminal" as const, domainId: "pytxo", runId: "workspace", sessionId: "test-user-session", title: "Your terminal fixture" };
  await completeOnboarding(page, { "pytxo-mission-dock-v1": JSON.stringify(openDockView(defaultDockLayout(), ref, "bottom")) });
  await page.goto("/#/work");
  await expect(page.getByRole("region", { name: "Work", exact: true })).toBeVisible();
  // Install only after the app has chosen the browser backend. No native bridge
  // or production fixture branch is added to the product for this test.
  await page.evaluate(() => {
    const fixture = { calls: [] as { command: string; args: Record<string, unknown> }[], state: "running" };
    const session = () => ({ id: "test-user-session", domain_id: "pytxo", cwd: "pytxo", owner: "You", state: fixture.state, exit_code: fixture.state === "ended" ? 0 : null });
    Object.assign(window, { terminalFixture: fixture, __TAURI_INTERNALS__: { invoke: async (command: string, args: Record<string, unknown> = {}) => {
      fixture.calls.push({ command, args });
      if (command === "workspace_terminal_list") return [session()];
      if (command === "workspace_terminal_read") { const output = "UI contract fixture — no real shell\r\n"; const bytes = new TextEncoder().encode(output); const after = Number(args.after); return { session: session(), start: after, next: bytes.length, gap: false, error: null, data_base64: btoa(String.fromCharCode(...bytes.slice(after))) }; }
      if (command === "workspace_terminal_end") fixture.state = "ended";
    } } });
  });
  await expect(page.getByRole("button", { name: "Enable input", exact: true })).toBeEnabled();
  expect(await page.evaluate(() => (window as any).terminalFixture.calls.filter((c: any) => c.command === "workspace_terminal_input"))).toHaveLength(0);
  await page.getByRole("button", { name: "Enable input", exact: true }).click();
  await page.keyboard.type("abc");
  await page.keyboard.press("Control+k");
  await expect(page.getByRole("dialog", { name: /Command/ })).toHaveCount(0);
  await expect.poll(() => page.evaluate(() => (window as any).terminalFixture.calls.filter((c: any) => c.command === "workspace_terminal_input").length)).toBeGreaterThan(0);
  const inputs = await page.evaluate(() => (window as any).terminalFixture.calls.filter((c: any) => c.command === "workspace_terminal_input"));
  expect(inputs.every((c: any) => c.args.sessionId === "test-user-session" && c.args.domainId === "pytxo")).toBe(true);
  await page.getByRole("button", { name: "Close Your terminal fixture view", exact: true }).click();
  await page.getByText("Sessions · 1", { exact: true }).click();
  await page.getByRole("button", { name: /You · pytxo · running/ }).click();
  await expect(page.getByRole("button", { name: "Enable input", exact: true })).toBeEnabled();
  expect(await page.evaluate(() => (window as any).terminalFixture.calls.filter((c: any) => c.command === "workspace_terminal_end"))).toHaveLength(0);
  await page.getByRole("button", { name: "End session", exact: true }).click();
  const endBounds = await page.getByRole("dialog", { name: "End your terminal session", exact: true }).boundingBox();
  const viewport = page.viewportSize()!;
  expect(Math.abs(endBounds!.x + endBounds!.width / 2 - viewport.width / 2)).toBeLessThan(2);
  expect(Math.abs(endBounds!.y + endBounds!.height / 2 - viewport.height / 2)).toBeLessThan(2);
  await page.getByRole("button", { name: "Keep session", exact: true }).click();
  expect(await page.evaluate(() => (window as any).terminalFixture.calls.filter((c: any) => c.command === "workspace_terminal_end"))).toHaveLength(0);
  await page.getByRole("button", { name: "End session", exact: true }).click();
  await page.getByRole("button", { name: "End this session", exact: true }).click();
  await expect(page.getByRole("button", { name: "Enable input", exact: true })).toBeDisabled();
});
