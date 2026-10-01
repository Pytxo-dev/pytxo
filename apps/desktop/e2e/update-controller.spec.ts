import { expect, test } from "@playwright/test";
import { get } from "svelte/store";
import { createUpdateController, type UpdatePackage, type UpdateState } from "../src/lib/update-controller";
import { completeOnboarding } from "./helpers";
import { browserUpdateReceipts, type UpdateReceipt } from "../src/lib/update-receipt";
import { isUpdateBusy, isUpdateCheckDisabled, shouldShowUpdateCheck, updateActionLabel, updatePhaseMessage } from "../src/lib/update-presentation";

function fixture() {
  const calls = { check: 0, download: 0, install: 0, relaunch: 0, close: 0, preflight: 0, handoff: 0, release: 0 };
  let handoffActive = false;
  let handoffRefused = false;
  const failures = { check: false, download: false, install: false, relaunch: false };
  const candidate: UpdatePackage = {
    version: "1.2.3", currentVersion: "1.2.2",
    async download(event) {
      calls.download++;
      event({ event: "Started", data: { contentLength: 100 } });
      event({ event: "Progress", data: { chunkLength: 40 } });
      event({ event: "Progress", data: { chunkLength: 60 } });
      event({ event: "Finished" });
      if (failures.download) throw new Error("signature invalid");
    },
    async install() { expect(handoffActive).toBe(true); calls.install++; if (failures.install) throw new Error("installer refused"); },
    async close() { calls.close++; },
  };
  let available = true;
  let receipt: UpdateReceipt | null = null;
  let blockAt = 0;
  let runningVersion = "1.2.2";
  let offline = false;
  let storageFailed = false;
  const receipts = { read: () => receipt, write: (value: UpdateReceipt) => { if (storageFailed) throw new Error("storage unavailable"); receipt = value; }, clear: () => { receipt = null; } };
  const nextLaunch = () => createUpdateController({
    supported: () => true, version: async () => runningVersion,
    check: async () => { if (offline) throw new Error("offline"); return null; },
    relaunch: async () => {}, beforeExit: async () => {}, beginHandoff: async () => async () => {}, receipts,
  });
  const controller = createUpdateController({
    supported: () => true, version: async () => "1.2.2",
    check: async options => { calls.check++; expect(options.allowDowngrades).toBe(false); expect(options.timeout).toBe(30_000); if (failures.check) throw new Error("network unavailable"); return available ? candidate : null; },
    relaunch: async () => { expect(handoffActive).toBe(true); calls.relaunch++; if (failures.relaunch) throw new Error("restart refused"); },
    beginHandoff: async () => {
      if (handoffRefused) throw new Error("Another process owns work");
      expect(handoffActive).toBe(false);
      handoffActive = true;
      calls.handoff++;
      return async () => { expect(handoffActive).toBe(true); handoffActive = false; calls.release++; };
    },
    beforeExit: async () => { calls.preflight++; if (calls.preflight === blockAt) throw new Error("Active work must finish"); },
    receipts,
  });
  return { controller, calls, failures, receipts, nextLaunch, refuseHandoff: (value: boolean) => { handoffRefused = value; }, failStorage: () => { storageFailed = true; }, block: (at: number) => { blockAt = at; }, launchVersion: (version: string, networkOffline = false) => { runningVersion = version; offline = networkOffline; }, noUpdate: () => { available = false; } };
}

test("update checks are shared, failures stay actionable, and replaced resources close", async () => {
  const f = fixture();
  await Promise.all([f.controller.check(), f.controller.check()]);
  expect(f.calls.check).toBe(1);
  expect(get(f.controller).currentVersion).toBe("1.2.2");
  f.failures.check = true;
  await f.controller.check();
  expect(get(f.controller)).toMatchObject({ phase: "available", version: "1.2.3", error: "Could not check for updates: network unavailable" });
  f.failures.check = false;
  f.noUpdate();
  await f.controller.check();
  expect(get(f.controller)).toMatchObject({ phase: "current", version: null, error: null });
  expect(f.calls.close).toBe(1);
});

test("signature failure after Finished never installs and retry downloads again", async () => {
  const f = fixture();
  await f.controller.check();
  f.failures.download = true;
  await f.controller.install();
  expect(f.calls.install).toBe(0);
  expect(get(f.controller)).toMatchObject({ phase: "available", downloaded: 100, error: "Update failed: signature invalid" });
  f.failures.download = false;
  await Promise.all([f.controller.install(), f.controller.install()]);
  expect(f.calls.download).toBe(2);
  expect(f.calls.install).toBe(0);
  expect(get(f.controller).phase).toBe("ready-to-install");
  await f.controller.install();
  expect(f.calls.install).toBe(1);
  expect(get(f.controller).currentVersion).toBe("1.2.2");
});

test("verified download keeps Desktop open and blocks resource replacement", async () => {
  const f = fixture();
  await f.controller.check();
  await f.controller.install();
  expect(get(f.controller)).toMatchObject({ phase: "ready-to-install", downloaded: 100 });
  expect(f.calls).toMatchObject({ download: 1, install: 0, handoff: 0, relaunch: 0 });
  await f.controller.check();
  expect(f.calls.check).toBe(1);
  expect(f.calls.close).toBe(0);
});

test("ready-to-install presentation explains the Windows handoff", () => {
  const ready: UpdateState = {
    phase: "ready-to-install", currentVersion: "1.2.2", version: "1.2.3",
    downloaded: 100, total: 100, error: null, notice: null,
  };
  expect(isUpdateBusy(ready)).toBe(false);
  expect(isUpdateCheckDisabled(ready)).toBe(true);
  expect(shouldShowUpdateCheck(ready, false)).toBe(false);
  expect(shouldShowUpdateCheck(ready, true)).toBe(false);
  expect(updateActionLabel(ready)).toBe("Install and restart");
  expect(updatePhaseMessage(ready)).toBe(
    "v1.2.3 is downloaded and verified. Installing closes Pytxo, then Windows asks for administrator permission.",
  );
});

test("update checks disappear once the lifecycle has a concrete next action", () => {
  const idle: UpdateState = {
    phase: "idle", currentVersion: "1.2.2", version: null,
    downloaded: 0, total: null, error: null, notice: null,
  };
  expect(shouldShowUpdateCheck(idle, false)).toBe(true);
  expect(shouldShowUpdateCheck(idle, true)).toBe(false);
  expect(shouldShowUpdateCheck({ ...idle, error: "offline" }, true)).toBe(true);
  expect(shouldShowUpdateCheck({ ...idle, phase: "available", version: "1.2.3" }, false)).toBe(false);
});

test("exclusive handoff refusal cannot install and failed install releases admission", async () => {
  const f = fixture();
  await f.controller.check();
  await f.controller.install();
  f.refuseHandoff(true);
  await f.controller.install();
  expect(f.calls.install).toBe(0);
  expect(f.calls.handoff).toBe(0);
  expect(f.receipts.read()).toBeNull();
  f.refuseHandoff(false);
  f.failures.install = true;
  await f.controller.install();
  expect(f.calls.handoff).toBe(1);
  expect(f.calls.release).toBe(1);
  f.failures.install = false;
  await f.controller.install();
  expect(f.calls.handoff).toBe(2);
  expect(f.calls.release).toBe(2);
  expect(f.calls.download).toBe(1);
});

test("installer and restart failures retry the correct stage", async () => {
  const f = fixture();
  await f.controller.check();
  await f.controller.install();
  f.failures.install = true;
  await f.controller.install();
  expect(f.calls.relaunch).toBe(0);
  f.failures.install = false;
  f.failures.relaunch = true;
  await f.controller.install();
  expect(f.calls.download).toBe(1);
  expect(get(f.controller)).toMatchObject({ phase: "restart-required", error: "Could not restart: restart refused" });
  await f.controller.check();
  expect(f.calls.check).toBe(1);
  f.failures.relaunch = false;
  await f.controller.install();
  expect(f.calls.install).toBe(2);
  expect(f.calls.relaunch).toBe(2);
  expect(get(f.controller).currentVersion).toBe("1.2.2");
});

test("browser Setup exposes updates in General without invoking a native installer", async ({ page }, info) => {
  await completeOnboarding(page);
  await page.goto("/#/setup");
  await page.getByRole("button", { name: "General", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Desktop updates" })).toBeVisible();
  await expect(page.getByText("Updates are available in the installed Desktop app.")).toBeVisible();
  await expect(page.getByRole("button", { name: "Check for updates" })).toBeDisabled();
  await expect(page.getByRole("button", { name: "Check for updates" })).toBeInViewport();
  expect((await page.locator(".updater-settings").boundingBox())!.height).toBeLessThan(160);
  await page.screenshot({ path: info.outputPath("general-updates.png") });
});

test("active work is checked before download, after download, and on restart retry", async () => {
  const f = fixture();
  await f.controller.check();
  f.block(1);
  await f.controller.install();
  expect(f.calls.download).toBe(0);
  expect(f.receipts.read()).toBeNull();
  f.block(3);
  await f.controller.install();
  expect(f.calls.download).toBe(1);
  expect(f.calls.install).toBe(0);
  expect(f.receipts.read()).toBeNull();
  f.block(0);
  f.failures.relaunch = true;
  await f.controller.install();
  expect(get(f.controller).phase).toBe("restart-required");
  const installs = f.calls.install;
  const restarts = f.calls.relaunch;
  f.block(f.calls.preflight + 1);
  await f.controller.install();
  expect(f.calls.relaunch).toBe(restarts);
  expect(f.calls.install).toBe(installs);
  expect(get(f.controller)).toMatchObject({ phase: "restart-required", error: "Could not restart: Active work must finish" });
});

test("next launch confirms running version independently of update-feed availability", async () => {
  const f = fixture();
  await f.controller.check();
  await f.controller.install();
  expect(f.receipts.read()).toBeNull();
  await f.controller.install();
  expect(f.receipts.read()).toMatchObject({ fromVersion: "1.2.2", toVersion: "1.2.3" });
  const oldLaunch = f.nextLaunch();
  await oldLaunch.check();
  expect(get(oldLaunch).notice).toContain("upgrade completion is not confirmed");
  expect(f.receipts.read()).not.toBeNull();
  f.launchVersion("1.2.3", true);
  const updatedLaunch = f.nextLaunch();
  await updatedLaunch.check();
  expect(get(updatedLaunch).notice).toBe("This launch is running the requested update, v1.2.3.");
  expect(get(updatedLaunch).error).toContain("offline");
  expect(f.receipts.read()).toBeNull();
});

test("receipt storage persists only the requested versions and rejects malformed records", () => {
  const values = new Map<string, string>();
  const storage = { getItem: (key: string) => values.get(key) ?? null, setItem: (key: string, value: string) => { values.set(key, value); }, removeItem: (key: string) => { values.delete(key); } };
  const receipts = browserUpdateReceipts(storage);
  const receipt = { fromVersion: "1.2.2", toVersion: "1.2.3", requestedAt: "2026-09-19T00:00:00Z" };
  receipts.write(receipt);
  expect(browserUpdateReceipts(storage).read()).toEqual(receipt);
  values.set([...values.keys()][0]!, "{}");
  expect(() => receipts.read()).toThrow("unreadable");
  receipts.clear();
  expect(receipts.read()).toBeNull();
});

test("unwritable receipt prevents installer handoff", async () => {
  const f = fixture();
  await f.controller.check();
  await f.controller.install();
  f.failStorage();
  await f.controller.install();
  expect(f.calls.install).toBe(0);
  expect(f.calls.relaunch).toBe(0);
  expect(get(f.controller).error).toContain("storage unavailable");
});

test("a malformed prior receipt does not permanently block fresh update checks", async () => {
  for (const raw of ["{", "{}"]) {
    const f = fixture();
    const storage = { getItem: () => raw, setItem: () => {}, removeItem: () => {} };
    f.receipts.read = () => browserUpdateReceipts(storage).read();
    await f.controller.check();
    expect(f.calls.check).toBe(1);
    expect(get(f.controller)).toMatchObject({ phase: "available", version: "1.2.3", error: null });
    expect(get(f.controller).notice).toContain("Previous update completion could not be confirmed");
    await f.controller.check();
    expect(f.calls.check).toBe(2);
    expect(f.calls.install).toBe(0);
  }
});
