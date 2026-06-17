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
