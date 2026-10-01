import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";
import { defaultDockLayout, openDockView } from "../src/lib/dock-layout";

test("run-scoped views stay saved but do not leak into another run or global screens", async ({ page }) => {
  const layout = openDockView(defaultDockLayout(), { kind: "files", domainId: "pytxo", runId: "previous-run", title: "Earlier files" }, "right");
  await completeOnboarding(page, { "pytxo-mission-dock-v1": JSON.stringify(layout) });
  await page.setViewportSize({ width: 1600, height: 1000 });
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Inspection tools", exact: true }).click();
  const files = page.getByRole("button", { name: "Files", description: "Click to open, or drag to the right or bottom edge", exact: true });
  await expect(files).toBeEnabled();
  await expect(page.getByRole("tab", { name: "Earlier files", exact: true })).toHaveCount(0);
  const from = (await files.boundingBox())!;
  await page.mouse.move(from.x + 20, from.y + 15); await page.mouse.down();
  await page.mouse.move(from.x + 30, from.y + 40, { steps: 5 });
  const to = (await page.locator(".drop-bottom").boundingBox())!;
  await page.mouse.move(to.x + to.width / 2, to.y + to.height / 2, { steps: 10 });
  await page.mouse.up();
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toContainText("Files");
  const savedViews = await page.evaluate(() => JSON.parse(localStorage.getItem("pytxo-mission-dock-v1") ?? "{}").views);
  expect(savedViews.map((view: { title: string }) => view.title)).toEqual(expect.arrayContaining(["Earlier files", "Files"]));
  await page.getByRole("link", { name: "Setup", exact: true }).click();
  await expect(page.getByLabel("Task views", { exact: true })).toHaveCount(0);
  await expect(page.getByRole("tablist", { name: /dock$/ })).toHaveCount(0);
  await page.getByRole("link", { name: "Work", exact: true }).click();
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toContainText("Files");
  await expect(page.getByRole("tab", { name: "Earlier files", exact: true })).toHaveCount(0);
});

test("workspace success notice can be dismissed without undoing or repeating trust", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/work");
  await expect(page.getByRole("region", { name: "Work", exact: true })).toBeVisible();
  await page.evaluate(() => {
    const calls: string[] = [];
    Object.assign(window, { noticeCalls: calls, __TAURI_INTERNALS__: { invoke: async (command: string) => {
      calls.push(command);
      if (command === "domain_is_trusted") return false;
      if (command === "set_domain_permission") return "orbit";
    } } });
  });
  await page.getByRole("link", { name: "Setup", exact: true }).click();
  await page.getByRole("button", { name: "Workspaces", exact: true }).click();
  await page.getByRole("button", { name: "Add workspace", exact: true }).click();
  await expect(page.getByRole("status").filter({ hasText: "Trusted as Orbit." })).toBeVisible();
  const calls = await page.evaluate(() => (window as any).noticeCalls.filter((name: string) => name === "set_domain_permission").length);
  await page.getByRole("button", { name: "Dismiss workspace message" }).click();
  await expect(page.getByText("Trusted as Orbit.", { exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).noticeCalls.filter((name: string) => name === "set_domain_permission").length)).toBe(calls);
});

test("workspace folder controls create, cancel, add and remove without a trust grant", async ({ page }, testInfo) => {
  await completeOnboarding(page);
  await page.goto("/#/work");
  await expect(page.getByRole("region", { name: "Work", exact: true })).toBeVisible();
  // UI contract fixture installed after backend selection; Rust tests cover real metadata writes.
  await page.evaluate(() => {
    const fixture = { calls: [] as string[], pick: null as string | null, fail: false, roots: [{ label: "primary", path: "C:/fixture/primary", primary: true, read_only: false }] };
    Object.assign(window, { folderFixture: fixture, __TAURI_INTERNALS__: { invoke: async (command: string, args: Record<string, unknown> = {}) => {
      fixture.calls.push(command);
      if (command === "domain_is_trusted") return true;
      if (command === "get_domain_permission") return "orbit";
      if (command === "pick_workspace_folder") return fixture.pick;
      if (command === "project_create_cmd") { fixture.roots.push({ label: "shared", path: String(args.path), primary: false, read_only: false }); return { id: "ui-fixture-project", manifest_path: "ui-fixture.toml" }; }
      if (command === "project_roots_cmd") return fixture.roots;
      if (command === "project_add_root_cmd") { if (fixture.fail) throw { message: "Folder is already attached." }; fixture.roots.push({ label: "docs", path: String(args.path), primary: false, read_only: false }); return fixture.roots; }
      if (command === "project_remove_root_cmd") { fixture.roots = fixture.roots.filter(root => root.label !== args.label); return fixture.roots; }
    } } });
  });
  await page.getByTitle("Switch workspace", { exact: true }).click();
  await page.getByRole("button", { name: "Workspace settings", exact: true }).click();
  const dialog = page.getByRole("dialog", { name: /^Workspace settings/ });
  const add = dialog.getByRole("button", { name: "Add folder", exact: true });
  await add.click();
  await expect(add).toBeEnabled();
  expect(await page.evaluate(() => (window as any).folderFixture.calls.includes("project_create_cmd"))).toBe(false);
  await page.evaluate(() => { (window as any).folderFixture.pick = "C:/fixture/shared"; });
  await add.click();
  await expect(dialog.getByText("C:/fixture/shared", { exact: true })).toBeVisible();
  await page.evaluate(() => { (window as any).folderFixture.fail = true; });
  await add.click();
  await expect(dialog.getByText("Folder is already attached.", { exact: true })).toBeVisible();
  await page.evaluate(() => { (window as any).folderFixture.fail = false; (window as any).folderFixture.pick = "C:/fixture/docs"; });
  await add.click();
  await expect(dialog.getByText("C:/fixture/docs", { exact: true })).toBeVisible();
  await dialog.getByRole("button", { name: "Remove shared", exact: true }).click();
  await expect(dialog.getByText("C:/fixture/shared", { exact: true })).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).folderFixture.calls.includes("set_domain_permission"))).toBe(false);
  await page.screenshot({ path: testInfo.outputPath("folder-management-ui-fixture.png") });
});

test("pointer dragging opens hidden docks, moves tabs and cancels without changing layout", async ({ page }, testInfo) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1600, height: 1000 });
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Inspection tools", exact: true }).click();
  const evidence = page.getByRole("button", { name: "Checks & details", exact: true });
  await expect(evidence).toBeEnabled();
  const start = (await evidence.boundingBox())!;
  await page.mouse.move(start.x + 20, start.y + 15);
  await page.mouse.down();
  await page.mouse.move(start.x + 30, start.y + 40, { steps: 5 });
  await expect(page.locator(".drop-bottom")).toBeVisible();
  await expect(evidence).toHaveClass(/drag-source/);
  const target = (await page.locator(".drop-bottom").boundingBox())!;
  await page.mouse.move(target.x + target.width / 2, target.y + target.height / 2, { steps: 10 });
  await expect(page.locator(".drop-bottom")).toHaveClass(/targeted/);
  await page.mouse.up();
  await expect(page.locator(".layout-status")).toContainText("Checks & details docked below.");
  const tab = page.getByRole("tab", { name: "Checks & details", exact: true });
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toContainText("Checks & details");
  const from = (await tab.boundingBox())!;
  await page.mouse.move(from.x + 20, from.y + 15); await page.mouse.down();
  await page.mouse.move(from.x + 30, from.y - 30, { steps: 5 });
  const right = (await page.locator(".drop-right").boundingBox())!;
  await page.mouse.move(right.x + right.width / 2, right.y + right.height / 2, { steps: 10 });
  await page.mouse.up();
  await expect(page.getByRole("tablist", { name: "right dock" })).toContainText("Checks & details");
  const saved = await page.evaluate(() => localStorage.getItem("pytxo-mission-dock-v1"));
  const fresh = (await tab.boundingBox())!;
  await page.mouse.move(fresh.x + 20, fresh.y + 15); await page.mouse.down();
  await page.mouse.move(fresh.x - 40, fresh.y + 60, { steps: 5 });
  await page.keyboard.press("Escape"); await page.mouse.up();
  await expect(page.locator(".drop-zones")).toHaveCount(0);
  await expect(page.locator(".layout-status")).toContainText("Checks & details stayed in place.");
  expect(await page.evaluate(() => localStorage.getItem("pytxo-mission-dock-v1"))).toBe(saved);
  await page.screenshot({ path: testInfo.outputPath("dragged-right-dock.png") });
});

test("review evidence follows the files without the tall-column blank area", async ({ page }, testInfo) => {
  await completeOnboarding(page, { "pytxo-preview-review-state-v1": "ready" });
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto("/#/history");
  await page.locator('.history .row[data-run-id="run-71ad"]').click();
  await page.getByRole("button", { name: "Review prepared changes", exact: true }).click();
  await page.locator(".technical-evidence > summary").click();
  const disclosure = (await page.locator(".technical-evidence > summary").boundingBox())!;
  const evidence = (await page.locator(".review-support").boundingBox())!;
  expect(evidence.y - (disclosure.y + disclosure.height)).toBeGreaterThanOrEqual(0);
  expect(evidence.y - (disclosure.y + disclosure.height)).toBeLessThan(25);
  await page.locator(".review-support").scrollIntoViewIfNeeded();
  await page.screenshot({ path: testInfo.outputPath("review-evidence-flow.png") });
  await page.setViewportSize({ width: 1024, height: 768 });
  expect(await page.evaluate(() => document.documentElement.scrollWidth - innerWidth)).toBeLessThanOrEqual(1);
});

test("profile emblems load and voice control reserves room for its label and arrow", async ({ page }, testInfo) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#/setup");
  await page.getByRole("button", { name: "Agents & permissions", exact: true }).click();
  const icons = page.locator(".profile-emblem");
  await expect(icons).toHaveCount(4);
  expect(await icons.evaluateAll(nodes => nodes.every(node => node.tagName.toLowerCase() === "svg" && node.getAttribute("aria-hidden") === "true" && node.getBoundingClientRect().width >= 24))).toBe(true);
  await icons.first().scrollIntoViewIfNeeded();
  await page.screenshot({ path: testInfo.outputPath("permission-emblems.png") });
  await page.goto("/#/flow");
  await page.locator(".voice-disclosure > summary").click();
  const selector = page.locator(".voice-device");
  await expect(selector).toBeVisible();
  const size = (await selector.boundingBox())!;
  expect(size.width).toBeGreaterThanOrEqual(140);
  expect(size.height).toBeGreaterThanOrEqual(40);
  const padding = await selector.evaluate(node => parseFloat(getComputedStyle(node).paddingRight));
  expect(padding).toBeGreaterThanOrEqual(30);
  await page.screenshot({ path: testInfo.outputPath("voice-selector.png") });
});
