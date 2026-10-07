import { expect, test } from "@playwright/test";
import { completeOnboarding } from "./helpers";
import { previewAddress } from "../src/lib/local-preview";
import { defaultDockLayout, openDockView, restoreDockLayout } from "../src/lib/dock-layout";

test("preview address display rejects remote, credential and non-HTTP targets", () => {
  expect(previewAddress("http://localhost:5173")).toBe("http://localhost:5173/");
  expect(previewAddress("https://[::1]:8443/path")).toBe("https://[::1]:8443/path");
  for (const url of ["https://example.com", "http://localhost.evil.test", "http://user:pass@localhost", "http://localhost:0", "file:///C:/pytxo", "tauri://localhost", "blob:http://localhost/1234", "http://local\nhost/"]) expect(previewAddress(url)).toBeNull();
});

test("preview layout restores only an inert reference, never a URL or renderer", () => {
  const layout = openDockView(defaultDockLayout(), { kind: "preview", domainId: "a", runId: "workspace", title: "Local preview" });
  const raw = { ...layout, views: layout.views.map(view => ({ ...view, url: "http://localhost:5173/private", nativeId: "old", enabled: true })) };
  expect(restoreDockLayout(JSON.stringify(raw))).toEqual(layout);
});

test("local preview docks and restores without claiming browser execution", async ({ page }) => {
  await completeOnboarding(page);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Inspection tools", exact: true }).click();
  await page.getByRole("button", { name: "Preview", exact: true }).click();
  const panel = page.getByRole("region", { name: "Local preview", exact: true });
  await expect(panel).toContainText("Local previews require native Windows Desktop.");
  await panel.getByLabel("Local server URL").fill("https://example.com/");
  await expect(panel.getByRole("button", { name: "Open preview", exact: true })).toBeDisabled();
  await panel.getByLabel("Local server URL").fill("http://localhost:5173");
  await expect(panel).toContainText("Open http://localhost:5173/");
  await expect(panel.getByRole("button", { name: "Open preview", exact: true })).toBeDisabled();
  await page.getByLabel("Options for Local preview").click();
  await page.getByRole("button", { name: "Move to bottom", exact: true }).click();
  await page.reload();
  await expect(page.getByRole("tablist", { name: "bottom dock" })).toContainText("Local preview");
  await expect(panel.getByLabel("Local server URL")).toHaveValue("");
  await expect(page.getByRole("button", { name: "Stop", exact: true })).toBeInViewport();
});

test("decision overlays wait for preview hiding and inline details do not blank it", async ({ page }) => {
  await completeOnboarding(page, { "pytxo-preview-flow-history-v1": "long-mission" });
  await page.setViewportSize({ width: 1600, height: 1000 });
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Inspection tools", exact: true }).click();
  await expect(page.getByRole("region", { name: "Work", exact: true })).toBeVisible();
  // IPC contract fixture, not evidence of a native renderer or its isolation.
  await page.evaluate(() => {
    const fixture = { calls: [] as { command: string; args: Record<string, unknown> }[], hold: false, failHides: false, releases: [] as (() => void)[] };
    const info = { id: "preview-fixture", url: "http://localhost:9418/", storage_verified: true, failure: null, blocked_requests: 0 };
    Object.assign(window, { previewFixture: fixture, __TAURI_INTERNALS__: { invoke: async (command: string, args: Record<string, unknown> = {}) => {
      fixture.calls.push({ command, args });
      if (command === "workspace_terminal_list") return [];
      if (command === "local_preview_sync" && !args.bounds && fixture.failHides) throw new Error("Preview no longer responds");
      if (command === "local_preview_sync" && !args.bounds && fixture.hold) await new Promise<void>(resolve => fixture.releases.push(resolve));
      return info;
    } } });
  });
  await page.getByRole("button", { name: "Preview", exact: true }).click();
  const panel = page.getByRole("region", { name: "Local preview", exact: true });
  await panel.getByLabel("Local server URL").fill("http://localhost:9418");
  await panel.getByRole("button", { name: "Open preview", exact: true }).click();
  await expect(panel.getByRole("button", { name: "Pause preview" })).toBeVisible();
  await page.locator(".mission-outcome summary").click();
  await expect.poll(() => page.evaluate(() => (window as any).previewFixture.calls.filter((c: any) => c.command === "local_preview_sync").at(-1)?.args.bounds !== null)).toBe(true);
  await page.evaluate(() => { (window as any).previewFixture.hold = true; });
  await page.getByRole("button", { name: "Stop", exact: true }).click();
  await expect.poll(() => page.evaluate(() => (window as any).previewFixture.releases.length)).toBeGreaterThan(0);
  await expect(page.getByRole("dialog")).toBeHidden();
  await page.evaluate(() => { const f = (window as any).previewFixture; f.hold = false; f.releases.splice(0).forEach((release: () => void) => release()); });
  await expect(page.getByRole("dialog")).toBeVisible();
  await expect.poll(() => page.evaluate(() => (window as any).previewFixture.calls.filter((c: any) => c.command === "local_preview_sync").at(-1)?.args.bounds)).toBeNull();
  await page.keyboard.press("Escape");
  await expect(page.getByRole("dialog")).toBeHidden();
  await expect.poll(() => page.evaluate(() => (window as any).previewFixture.calls.filter((c: any) => c.command === "local_preview_sync").at(-1)?.args.bounds !== null)).toBe(true);
  await page.setViewportSize({ width: 1280, height: 800 });
  await page.getByLabel("Options for Local preview").click();
  await page.getByRole("button", { name: "Move to bottom", exact: true }).click();
  await expect(panel.locator(".preview-viewport")).toBeInViewport({ ratio: 1 });
  expect((await panel.locator(".preview-viewport").boundingBox())!.height).toBeGreaterThanOrEqual(80);
  // A dead preview is closed rather than silently disabling Stop.
  await page.evaluate(() => { (window as any).previewFixture.failHides = true; });
  await page.getByRole("button", { name: "Stop", exact: true }).click();
  await expect(page.getByRole("dialog")).toBeVisible();
  expect(await page.evaluate(() => (window as any).previewFixture.calls.some((c: any) => c.command === "local_preview_close"))).toBe(true);
});

test("failed preview retries only after the old renderer closes", async ({ page }) => {
  await completeOnboarding(page);
  await page.goto("/#/work");
  await page.getByRole("button", { name: "Inspection tools", exact: true }).click();
  // IPC lifecycle contract only; real load failure/recovery is checked natively.
  await page.evaluate(() => {
    const fixture = { opens: 0, failClose: true, failed: false, calls: [] as string[], holdNextSync: false, rejectSync: null as null | (() => void), failNextHide: false, holdNextClose: false, releaseClose: null as null | (() => void) };
    Object.assign(window, { retryFixture: fixture, __TAURI_INTERNALS__: { invoke: async (command: string, args: Record<string, unknown> = {}) => {
      fixture.calls.push(command);
      if (command === "workspace_terminal_list") return [];
      if (command === "local_preview_sync" && !args.bounds && fixture.failNextHide) { fixture.failNextHide = false; throw new Error("Hide failed"); }
      if (command === "local_preview_close" && fixture.holdNextClose) {
        fixture.holdNextClose = false;
        await new Promise<void>(resolve => { fixture.releaseClose = resolve; });
      }
      if (command === "local_preview_sync" && fixture.holdNextSync) {
        fixture.holdNextSync = false;
        await new Promise<void>((_, reject) => { fixture.rejectSync = () => reject(new Error("Old preview has closed")); });
      }
      if (command === "local_preview_open") { fixture.opens++; fixture.failed = false; }
      if (command === "local_preview_close" && fixture.failClose) throw new Error("Renderer close failed");
      return { id: `preview-${fixture.opens}`, url: "http://localhost:9418/", storage_verified: true,
        failure: fixture.failed ? "The local page did not finish loading." : null, blocked_requests: 0 };
    } } });
  });
  await page.getByRole("button", { name: "Preview", exact: true }).click();
  const panel = page.getByRole("region", { name: "Local preview", exact: true });
  await panel.getByLabel("Local server URL").fill("http://localhost:9418");
  await panel.getByRole("button", { name: "Open preview", exact: true }).click();
  await expect(panel.getByRole("button", { name: "Pause preview" })).toBeVisible();
  await page.evaluate(() => { (window as any).retryFixture.failed = true; });
  await expect(panel.getByRole("button", { name: "Retry preview", exact: true })).toBeVisible();
  await expect(panel.getByRole("button", { name: "Resume preview" })).toHaveCount(0);
  await panel.getByRole("button", { name: "Retry preview", exact: true }).click();
  await expect(panel.getByRole("alert")).toContainText("Renderer close failed");
  expect(await page.evaluate(() => (window as any).retryFixture.opens)).toBe(1);
  await page.evaluate(() => { (window as any).retryFixture.failClose = false; });
  await panel.getByRole("button", { name: "Retry preview", exact: true }).click();
  await expect(panel.getByRole("button", { name: "Pause preview" })).toBeVisible();
  await expect(panel.getByRole("alert")).toHaveCount(0);
  expect(await page.evaluate(() => (window as any).retryFixture.opens)).toBe(2);
  const lifecycle = await page.evaluate(() => (window as any).retryFixture.calls.filter((c: string) => c !== "local_preview_sync" && c !== "workspace_terminal_list"));
  expect(lifecycle).toEqual(["local_preview_open", "local_preview_close", "local_preview_close", "local_preview_open"]);
  // A late failure from the old renderer must not pause or contaminate its replacement.
  await page.evaluate(() => { (window as any).retryFixture.failed = true; });
  await expect(panel.getByRole("button", { name: "Retry preview", exact: true })).toBeVisible();
  await page.evaluate(() => { (window as any).retryFixture.holdNextSync = true; });
  await expect.poll(() => page.evaluate(() => !!(window as any).retryFixture.rejectSync)).toBe(true);
  await panel.getByRole("button", { name: "Retry preview", exact: true }).click();
  await expect.poll(() => page.evaluate(() => (window as any).retryFixture.opens)).toBe(3);
  await page.evaluate(() => { (window as any).retryFixture.rejectSync(); });
  await expect(panel.getByRole("button", { name: "Pause preview" })).toBeVisible();
  await expect(panel.getByRole("alert")).toHaveCount(0);
  // Background hide recovery can finish after an explicit retry has replaced the view.
  await page.evaluate(() => { (window as any).retryFixture.failed = true; });
  await expect(panel.getByRole("button", { name: "Retry preview", exact: true })).toBeVisible();
  await page.evaluate(() => {
    const f = (window as any).retryFixture; f.failNextHide = true; f.holdNextClose = true;
    document.dispatchEvent(new PointerEvent("pointerdown"));
  });
  await expect.poll(() => page.evaluate(() => !!(window as any).retryFixture.releaseClose)).toBe(true);
  await panel.getByRole("button", { name: "Retry preview", exact: true }).click();
  await expect.poll(() => page.evaluate(() => (window as any).retryFixture.opens)).toBe(4);
  await page.evaluate(() => { (window as any).retryFixture.releaseClose(); });
  await expect(panel.getByRole("button", { name: "Pause preview" })).toBeVisible();
  await expect(panel.getByRole("alert")).toHaveCount(0);
  // Explicit close is serialized: input cannot start Retry or another Close until it settles.
  await page.evaluate(() => { const f = (window as any).retryFixture; f.releaseClose = null; f.holdNextClose = true; });
  await panel.getByRole("button", { name: "Close preview", exact: true }).click();
  await expect(panel.getByRole("button", { name: "Closing…", exact: true })).toBeDisabled();
  await expect(panel.getByRole("button", { name: "Pause preview" })).toBeDisabled();
  await page.evaluate(() => { (window as any).retryFixture.releaseClose(); });
  await expect(panel.getByRole("button", { name: "Open preview", exact: true })).toBeEnabled();
  await expect(panel.getByRole("alert")).toHaveCount(0);
});
