import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";

let lastNotifiedCount = 0;

export async function notifyHitlIfNeeded(count: number, actions: string[]): Promise<void> {
  if (count <= 0 || count === lastNotifiedCount) return;
  lastNotifiedCount = count;

  let granted = await isPermissionGranted();
  if (!granted) {
    const perm = await requestPermission();
    granted = perm === "granted";
  }
  if (!granted) return;

  const preview = actions.slice(0, 2).join(", ") || "approval required";
  await sendNotification({
    title: `Pytxo: ${count} approval${count === 1 ? "" : "s"} pending`,
    body: preview,
  });
}

/** One system notification per finished run, so a fleet left running in the background reports back. */
export async function notifyRunFinished(title: string, body: string): Promise<void> {
  let granted = await isPermissionGranted();
  if (!granted) granted = (await requestPermission()) === "granted";
  if (granted) await sendNotification({ title, body });
}
