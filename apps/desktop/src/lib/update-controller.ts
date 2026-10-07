import { writable } from "svelte/store";
import type { DownloadEvent } from "@tauri-apps/plugin-updater";
import type { UpdateReceiptStore } from "./update-receipt";

export interface UpdatePackage {
  version: string;
  currentVersion: string;
  download(callback: (event: DownloadEvent) => void, options: { timeout: number }): Promise<void>;
  install(): Promise<void>;
  close(): Promise<void>;
}
export type UpdateState = {
  phase: "idle" | "checking" | "available" | "current" | "preflight" | "downloading" | "ready-to-install" | "installing" | "restart-required" | "restarting" | "unavailable";
  currentVersion: string | null;
  version: string | null;
  downloaded: number;
  total: number | null;
  error: string | null;
  notice: string | null;
};

/** One lifecycle per webview, shared by the banner and Settings. */
export function createUpdateController(adapter: {
  supported: () => boolean;
  version: () => Promise<string>;
  check: (options: { timeout: number; allowDowngrades: boolean }) => Promise<UpdatePackage | null>;
  relaunch: () => Promise<void>;
  beforeExit: () => Promise<void>;
  beginHandoff: () => Promise<() => Promise<void>>;
  receipts: UpdateReceiptStore;
}) {
  let state: UpdateState = { phase: "idle", currentVersion: null, version: null, downloaded: 0, total: null, error: null, notice: null };
  const store = writable(state);
  const set = (patch: Partial<UpdateState>) => { state = { ...state, ...patch }; store.set(state); };
  let candidate: UpdatePackage | null = null;
  let downloaded = false;
  let busy = false;
  let initialized = false;
  const message = (error: unknown) => error instanceof Error ? error.message : String(error);

  async function check() {
    // A verified download is owned by this updater resource until install
    // succeeds. Re-checking here would replace the resource and orphan bytes.
    if (busy || state.phase === "ready-to-install" || state.phase === "restart-required") return;
    if (!adapter.supported()) { set({ phase: "unavailable", error: null }); return; }
    busy = true;
    set({ phase: "checking", error: null });
    try {
      set({ currentVersion: await adapter.version() });
      if (!initialized) {
        try {
          const receipt = adapter.receipts.read();
          if (receipt) {
            if (receipt.toVersion === state.currentVersion) {
              set({ notice: `This launch is running the requested update, v${state.currentVersion}.` });
              adapter.receipts.clear();
            } else {
              set({ notice: `Update to v${receipt.toVersion} was requested. This launch is running v${state.currentVersion}; upgrade completion is not confirmed.` });
            }
          }
        } catch (error) {
          // Historical receipt recovery must not disable the signed update feed.
          // Installing still requires a successful receipt write before handoff.
          set({ notice: `Previous update completion could not be confirmed: ${message(error)}` });
        }
        initialized = true;
      }
      const found = await adapter.check({ timeout: 30_000, allowDowngrades: false });
      const previous = candidate;
      candidate = found;
      downloaded = false;
      set({ phase: found ? "available" : "current", version: found?.version ?? null, downloaded: 0, total: null });
      if (previous && previous !== found) await previous.close();
    } catch (error) {
      set({ phase: candidate ? "available" : "idle", error: `Could not check for updates: ${message(error)}` });
    } finally { busy = false; }
  }

  async function install() {
    if (busy || !candidate) return;
    busy = true;
    set({ error: null });
    const restartOnly = state.phase === "restart-required";
    let releaseHandoff: (() => Promise<void>) | null = null;
    try {
      set({ phase: "preflight" });
      await adapter.beforeExit();
      if (!restartOnly) {
        if (!downloaded) {
          set({ phase: "downloading", downloaded: 0, total: null });
          await candidate.download((event) => {
            if (event.event === "Started") set({ total: event.data.contentLength ?? null });
            if (event.event === "Progress") set({ downloaded: state.downloaded + event.data.chunkLength });
          }, { timeout: 120_000 });
          // Finished alone is not proof of a verified download.
          downloaded = true;
          // Keep Desktop open after verification. Installation can terminate
          // the Windows process before the secure UAC prompt is accepted, so
          // make that destructive handoff a separate, explicit action.
          set({ phase: "ready-to-install" });
          return;
        }
      }
      // Close admission after download, before checking activity again. Hold
      // native ownership through installation and restart (including retries).
      set({ phase: "preflight" });
      releaseHandoff = await adapter.beginHandoff();
      await adapter.beforeExit();
      if (!restartOnly) {
        adapter.receipts.write({ fromVersion: state.currentVersion ?? candidate.currentVersion, toVersion: candidate.version, requestedAt: new Date().toISOString() });
        set({ phase: "installing" });
        await candidate.install();
        // Windows may exit here. Only the next launch can confirm its version.
        set({ phase: "restart-required" });
      }
      set({ phase: "restarting" });
      if (!restartOnly) await adapter.beforeExit();
      await adapter.relaunch();
      set({ phase: "restart-required" });
    } catch (error) {
      const restarting = restartOnly || state.phase === "restarting";
      // Tauri updater 2.10.1 closes its byte resource only after install succeeds.
      // Preserve verified bytes after installer/preflight failure; downloading
      // again would orphan the SDK's previous resource. Failed downloads retry.
      if (state.phase === "downloading") downloaded = false;
      set({
        phase: restarting ? "restart-required" : downloaded ? "ready-to-install" : "available",
        error: `${restarting ? "Could not restart" : "Update failed"}: ${message(error)}`,
      });
    } finally {
      if (releaseHandoff) {
        try { await releaseHandoff(); }
        catch (error) { set({ error: `Could not release update handoff: ${message(error)}. Save work and restart Desktop.` }); }
      }
      busy = false;
    }
  }
  return { subscribe: store.subscribe, check, install };
}
