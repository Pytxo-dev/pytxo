import { invoke } from "@tauri-apps/api/core";
import { normalizeIpcError } from "./ipc";

export type PreviewInfo = { id: string; url: string; storage_verified: boolean; failure: string | null; blocked_requests: number };
export type PreviewBounds = { x: number; y: number; width: number; height: number; viewport_width: number; viewport_height: number };
export function previewAddress(input: string): string | null {
  if (input.length > 8192 || /[\u0000-\u001f\u007f]/.test(input)) return null;
  try {
    const url = new URL(input.trim());
    return ["http:", "https:"].includes(url.protocol) && ["localhost", "127.0.0.1", "[::1]"].includes(url.hostname) && !url.username && !url.password && url.port !== "0" ? url.href : null;
  } catch { return null; }
}
async function call<T>(command: string, args: Record<string, unknown>): Promise<T> {
  try { return await invoke<T>(command, args); } catch (error) { throw normalizeIpcError(error); }
}
export const localPreview = {
  open: (url: string) => call<PreviewInfo>("local_preview_open", { url }),
  sync: (id: string, bounds: PreviewBounds | null, revision: number) => call<PreviewInfo>("local_preview_sync", { id, bounds, revision }),
  close: (id: string) => call<void>("local_preview_close", { id }),
};
