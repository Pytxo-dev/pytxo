import { tick } from "svelte";

const observers = new Set<() => Promise<void>>();
let suspensions = 0;
export const previewsSuspended = () => suspensions > 0;
export function registerPreviewSuspension(hide: () => Promise<void>) {
  observers.add(hide);
  return () => { observers.delete(hide); };
}
/** Native children must acknowledge hiding before a decision can paint. */
export async function withPreviewsHidden(action: () => void) {
  suspensions++;
  try {
    await Promise.all([...observers].map(hide => hide()));
    action();
    await tick();
  } catch (cause) {
    window.dispatchEvent(new CustomEvent("pytxo-preview-error", { detail: cause instanceof Error ? cause.message : String(cause) }));
  } finally { suspensions--; }
}

/** Used only for overlay menus, never ordinary inline disclosures. */
export function previewMenu(node: HTMLDetailsElement) {
  let pending = false;
  const click = (event: MouseEvent) => {
    if (!(event.target instanceof Element) || !event.target.closest("summary") || !observers.size) return;
    event.preventDefault();
    if (pending) return;
    pending = true;
    void withPreviewsHidden(() => { if (node.isConnected) node.open = !node.open; })
      .finally(() => { pending = false; });
  };
  node.addEventListener("click", click);
  return { destroy() { node.removeEventListener("click", click); } };
}
