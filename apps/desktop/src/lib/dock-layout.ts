export type DockPosition = "right" | "bottom";
export type DockKind = "agent" | "evidence" | "files" | "diagram" | "terminal" | "preview";
export type DockReference = { kind: DockKind; domainId: string; runId: string; agentId?: string; sessionId?: string; title: string };
export type DockView = DockReference & { id: string; position: DockPosition; pinned: boolean };
export type DockLayout = {
  version: 1;
  views: DockView[];
  active: Record<DockPosition, string | null>;
  hidden: Record<DockPosition, boolean>;
  rightWidth: number;
  bottomHeight: number;
};
export const DOCK_STORAGE_KEY = "pytxo-mission-dock-v1";
export const dockId = (ref: DockReference) => JSON.stringify([ref.kind, ref.domainId, ref.runId, ref.agentId ?? ref.sessionId ?? null]);
export const defaultDockLayout = (): DockLayout => ({ version: 1, views: [], active: { right: null, bottom: null }, hidden: { right: false, bottom: false }, rightWidth: 440, bottomHeight: 260 });
export const clamp = (n: number, min: number, max: number) => Math.max(min, Math.min(max, n));
const cloneLayout = (layout: DockLayout): DockLayout => ({ ...layout, views: layout.views.map(v => ({ ...v })), active: { ...layout.active }, hidden: { ...layout.hidden } });

/** Stored layout is a set of observer references, never process/input authority. */
export function restoreDockLayout(raw: string | null): DockLayout {
  const fallback = defaultDockLayout();
  try {
    const data = JSON.parse(raw ?? "null");
    if (data?.version !== 1 || !Array.isArray(data.views)) return fallback;
    const seen = new Set<string>();
    for (const item of data.views.slice(0, 12)) {
      if (!item || !["agent", "evidence", "files", "diagram", "terminal", "preview"].includes(item.kind) ||
        !["right", "bottom"].includes(item.position) ||
        ![item.domainId, item.runId, item.title].every(v => typeof v === "string" && v.length > 0 && v.length <= 2048) ||
        (item.kind === "agent" && typeof item.agentId !== "string") ||
        (item.kind === "terminal" && (typeof item.sessionId !== "string" || item.sessionId.length > 80))) continue;
      const ref: DockReference = { kind: item.kind, domainId: item.domainId, runId: item.runId, title: item.title, ...(item.kind === "agent" ? { agentId: item.agentId } : {}), ...(item.kind === "terminal" ? { sessionId: item.sessionId } : {}) };
      const id = dockId(ref);
      if (seen.has(id)) continue;
      seen.add(id);
      fallback.views.push({ ...ref, id, position: item.position, pinned: item.pinned === true });
    }
    for (const pos of ["right", "bottom"] as const) {
      fallback.active[pos] = fallback.views.find(v => v.position === pos && v.id === data.active?.[pos])?.id ?? fallback.views.find(v => v.position === pos)?.id ?? null;
      fallback.hidden[pos] = data.hidden?.[pos] === true;
    }
    if (Number.isFinite(data.rightWidth)) fallback.rightWidth = clamp(data.rightWidth, 360, 900);
    if (Number.isFinite(data.bottomHeight)) fallback.bottomHeight = clamp(data.bottomHeight, 180, 700);
  } catch { /* A broken layout cannot prevent access to the mission. */ }
  return fallback;
}

export function openDockView(layout: DockLayout, ref: DockReference, position: DockPosition = "right"): DockLayout {
  const next = cloneLayout(layout);
  const id = dockId(ref);
  let view = next.views.find(v => v.id === id);
  if (!view) {
    // Follow selection replaces only the unpinned observer of the same kind.
    const preview = ref.kind === "terminal" ? null : next.views.find(v => !v.pinned && v.kind === ref.kind && v.position === position);
    if (preview) next.views.splice(next.views.indexOf(preview), 1);
    if (next.views.length >= 12) throw new Error("Twelve views are open. Close an unused view first.");
    view = { ...ref, id, position, pinned: ref.kind === "terminal" };
    next.views.push(view);
  }
  next.active[view.position] = id;
  next.hidden[view.position] = false;
  return next;
}

export function moveDockView(layout: DockLayout, id: string, position: DockPosition, beforeId?: string): DockLayout {
  const next = cloneLayout(layout);
  const index = next.views.findIndex(v => v.id === id);
  if (index < 0 || beforeId === id) return next;
  const [view] = next.views.splice(index, 1);
  const old = view.position;
  view.position = position;
  const before = next.views.findIndex(v => v.id === beforeId && v.position === position);
  next.views.splice(before < 0 ? next.views.length : before, 0, view);
  if (next.active[old] === id) next.active[old] = next.views.find(v => v.position === old)?.id ?? null;
  next.active[position] = id;
  next.hidden[position] = false;
  return next;
}

export function closeDockView(layout: DockLayout, id: string): DockLayout {
  const next = cloneLayout(layout);
  next.views = next.views.filter(v => v.id !== id);
  for (const pos of ["right", "bottom"] as const) {
    if (next.active[pos] === id) next.active[pos] = next.views.find(v => v.position === pos)?.id ?? null;
  }
  return next;
}
