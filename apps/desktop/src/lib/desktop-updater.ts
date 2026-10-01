import { isTauri } from "@tauri-apps/api/core";
import { getVersion } from "@tauri-apps/api/app";
import { check } from "@tauri-apps/plugin-updater";
import { relaunch } from "@tauri-apps/plugin-process";
import { createUpdateController } from "./update-controller";
import { updatePreflight, beginUpdateHandoff } from "./ipc";
import { browserUpdateReceipts } from "./update-receipt";

export const desktopUpdater = createUpdateController({
  supported: isTauri, version: getVersion, check, relaunch, beforeExit: updatePreflight,
  beginHandoff: beginUpdateHandoff,
  receipts: {
    read: () => browserUpdateReceipts(localStorage).read(),
    write: receipt => browserUpdateReceipts(localStorage).write(receipt),
    clear: () => browserUpdateReceipts(localStorage).clear(),
  },
});
