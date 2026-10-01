<script lang="ts">
  import { flushSync, onMount, tick, untrack, type Snippet } from "svelte";
  import { evidenceMotion } from "../../lib/evidence-motion";
  import { uiPrefs } from "../../lib/ui-prefs.svelte";
  import type { DesktopBackend, DesktopSnapshot } from "../../lib/desktop-backend";
  import type { RunDto } from "../../lib/types";
  import { DOCK_STORAGE_KEY, clamp, closeDockView, defaultDockLayout, dockId, moveDockView, openDockView, restoreDockLayout, type DockPosition, type DockReference, type DockView } from "../../lib/dock-layout";
  import DockInspection from "./DockInspection.svelte";
  import LocalPreview from "./LocalPreview.svelte";
  import { previewMenu, withPreviewsHidden } from "../../lib/preview-overlay";
  import { workspaceTerminal, hasNativeTerminal, type TerminalInfo } from "../../lib/workspace-terminal";
  type DockSurface = "run" | "review" | "other";
  let { children, backend, snapshot, activeDomainId, run, onReview, onOpenApprovals, previewAllowed = true, surface = "run" }: {
    children: Snippet; backend: DesktopBackend; snapshot: DesktopSnapshot; activeDomainId: string | null; run: RunDto | null;
    onReview: (runId: string, domainId: string) => void; onOpenApprovals: () => void; previewAllowed?: boolean; surface?: DockSurface;
  } = $props();
  let toolsExpanded = $state(false);
  let layout = $state(defaultDockLayout());
  let loaded = $state(false);
  let host: HTMLDivElement | undefined = $state();
  let grid: HTMLDivElement | undefined = $state();
  let gridHeight = $state(600);
  let width = $state(1200);
  let narrow = $state(false);
  let short = $state(false);
  let compact = $state(false);
  let focusId = $state<string | null>(null);
  let focusHeight = $state<number | null>(null);
  let geometryDomain = $state<string | null>(null);
  let narrowActive = $state<string | null>(null);
  let narrowOrder = $state<string[]>([]);
  let narrowHidden = $state(false);
  let narrowHeight = $state(260);
  let dragging = $state<string | null>(null);
  let dragTarget = $state<DockPosition | null>(null);
  let pointerDrag: { reference: DockReference; id: string; x: number; y: number; target: HTMLElement; pointer: number; started: boolean } | null = null;
  let suppressDragClick = false;
  let reviewContext = $state<string | null>(null);
  let reviewRevealedIds = $state<string[]>([]);
  let inspectionReady = $state<string[]>([]);
  let inspectionFrame = 0;

  function beginDrag(event: PointerEvent, reference: DockReference) {
    if (event.button !== 0) return;
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    pointerDrag = { reference, id: dockId(reference), x: event.clientX, y: event.clientY, target, pointer: event.pointerId, started: false };
  }
  function trackDrag(event: PointerEvent) {
    const gesture = pointerDrag;
    if (!gesture || gesture.pointer !== event.pointerId) return;
    if (!gesture.started && Math.hypot(event.clientX - gesture.x, event.clientY - gesture.y) < 6) return;
    if (!gesture.started) {
      gesture.started = true;
      // A device may deliver only one move before release. Mount destinations
      // before hit-testing that move rather than requiring a second event.
      flushSync(() => { dragging = gesture.id; });
    }
    const hit = document.elementsFromPoint(event.clientX, event.clientY).find(element => element instanceof HTMLElement && element.dataset.dockPosition) as HTMLElement | undefined;
    dragTarget = hit?.dataset.dockPosition as DockPosition ?? null;
    dropBefore = hit?.dataset.dockBefore ?? null;
  }
  async function finishDrag(cancel = false) {
    const gesture = pointerDrag;
    if (!gesture) return;
    const position = dragTarget;
    const before = dropBefore ?? undefined;
    pointerDrag = null;
    if (gesture.target.hasPointerCapture(gesture.pointer)) gesture.target.releasePointerCapture(gesture.pointer);
    suppressDragClick = gesture.started;
    setTimeout(() => { suppressDragClick = false; }, 0);
    try {
      if (gesture.started && position && !cancel) await withPreviewsHidden(() => {
        if (!layout.views.some(view => view.id === gesture.id)) open(gesture.reference, position);
        focusId = null;
        move(gesture.id, position, before);
      });
      if (gesture.started) message = position && !cancel
        ? `${gesture.reference.title} docked ${position === "right" ? "right" : "below"}.`
        : `${gesture.reference.title} stayed in place.`;
    } finally { dragging = null; dragTarget = null; dropBefore = null; }
  }
  function consumeDragClick(event: MouseEvent) { if (suppressDragClick) { event.preventDefault(); event.stopPropagation(); } }
  function runReference(kind: "evidence" | "files" | "diagram"): DockReference | null {
    return run ? { kind, runId: run.id, domainId: run.domain_id, title: kind === "evidence" ? "Checks & details" : kind === "files" ? "Files" : "Dependencies" } : null;
  }
  let resizing = $state(false);
  let keyboardResizing = $state(false);
  let message = $state("");
  let sessions = $state<TerminalInfo[]>([]);
  let creating = $state(false);
  let createDialog: HTMLDialogElement | undefined = $state();
  let createDomain = $state<string | null>(null);
  let saveDialog: HTMLDialogElement | undefined = $state();
  let layoutName = $state("");
  let savedLayouts = $state<{ name: string; data: string }[]>([]);
  let dropBefore = $state<string | null>(null);
  let keyboardResize: { position: DockPosition; original: number } | null = null;
  const positions: DockPosition[] = ["right", "bottom"];
  const rightMax = $derived(Math.max(360, width - 600));
  const bottomMin = $derived(compact ? 80 : short ? 120 : 180);
  const rightWidth = $derived(clamp(layout.rightWidth, 360, rightMax));
  const contextualTools = $derived(surface === "run" || surface === "review");
  function belongsToCurrentContext(view: DockView) {
    if (!contextualTools) return false;
    if (view.pinned) return true;
    if (view.runId === "workspace") return view.domainId === activeDomainId;
    return !!run && view.domainId === run.domain_id && view.runId === run.id;
  }
  const presentedViews = $derived(layout.views.filter(belongsToCurrentContext));
  const surfaceViews = $derived(
    surface === "review"
      ? presentedViews.filter((view) => view.pinned || reviewRevealedIds.includes(view.id))
      : presentedViews,
  );
  const effectiveFocusId = $derived(
    focusId && surfaceViews.some((view) => view.id === focusId) ? focusId : null,
  );
  // Reserve the mission action row even when a right dock wraps its heading.
  const bottomMax = $derived(Math.max(bottomMin, gridHeight - (compact || effectiveFocusId ? 122 : surface === "run" ? 470 : 422)));
  // A long command strip plus two docks can consume the canvas even when the
  // overall window is above the short-height breakpoint. Share one inspection
  // surface while output is open until there is room for both axes.
  const outputNeedsSharedDock = $derived(surface === "run" && gridHeight < 800 && surfaceViews.some(view => view.kind === "agent-output" && !hiddenView(view)));
  const stacked = $derived(narrow || short || outputNeedsSharedDock || effectiveFocusId !== null);
  const bottomHeight = $derived(clamp(effectiveFocusId ? (focusHeight ?? (compact ? bottomMax : gridHeight - 240)) : narrow ? narrowHeight : short ? Math.min(layout.bottomHeight, 150) : layout.bottomHeight, bottomMin, bottomMax));
  const rightOpen = $derived(!stacked && !layout.hidden.right && surfaceViews.some(v => v.position === "right"));
  const bottomOpen = $derived(narrow ? surfaceViews.length > 0 && !narrowHidden : surfaceViews.some(v => stacked ? !hiddenView(v) : v.position === "bottom" && !layout.hidden.bottom));
  const visibleNarrowId = $derived(surfaceViews.find(v => v.id === narrowActive && !hiddenView(v))?.id ?? surfaceViews.find(v => v.id === layout.active.right && !hiddenView(v))?.id ?? surfaceViews.find(v => v.id === layout.active.bottom && !hiddenView(v))?.id ?? surfaceViews.find(v => !hiddenView(v))?.id ?? null);
  const activeRightView = $derived(surfaceViews.find(view => view.position === "right" && isVisible(view)) ?? null);
  // A worker summary is a contextual focus panel, not workspace geometry. Keep
  // it over the canvas so first selection does not relayout every graph node.
  const summaryOverlay = $derived(surface === "run" && rightOpen && activeRightView?.kind === "agent");
  let resize: { position: DockPosition; start: number; original: number; target: HTMLElement; pointer: number; scale: number } | null = null;

  onMount(() => {
    const previewError = (event: Event) => { message = (event as CustomEvent<string>).detail; };
    window.addEventListener("pytxo-preview-error", previewError);
    try { layout = restoreDockLayout(localStorage.getItem(DOCK_STORAGE_KEY)); } catch { message = "Layout storage unavailable. Changes last for this window."; }
    loaded = true;
    const observer = new ResizeObserver(entries => {
      for (const entry of entries) {
        if (entry.target === grid) {
          gridHeight = entry.contentRect.height;
          short = window.innerHeight < 780 || entry.contentRect.height < 660;
        }
        else {
          width = entry.contentRect.width;
          narrow = window.innerWidth < 1080 || width < 966;
          compact = window.innerWidth < 760 || width < 702;
        }
      }
    });
    if (host) observer.observe(host);
    if (grid) observer.observe(grid);
    return () => { observer.disconnect(); window.removeEventListener("pytxo-preview-error", previewError); };
  });
  $effect(() => {
    const shouldPoll = loaded && (toolsExpanded || surfaceViews.some(view => view.kind === "terminal" && isVisible(view)));
    if (!shouldPoll) return;
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    const refreshSessions = async () => {
      try {
        const value = await workspaceTerminal.list();
        if (!disposed) sessions = value;
      } catch (cause) {
        if (!disposed) message = String(cause);
      } finally {
        if (!disposed) timer = setTimeout(refreshSessions, 2000);
      }
    };
    void refreshSessions();
    return () => { disposed = true; clearTimeout(timer); };
  });
  $effect(() => {
    if (!loaded) return;
    try { localStorage.setItem(DOCK_STORAGE_KEY, JSON.stringify(layout)); }
    catch { untrack(() => message = "Layout could not be saved. Your open views remain available."); }
  });
  $effect(() => {
    const nextContext = surface === "review" && run ? `${run.domain_id}\u0000${run.id}` : null;
    if (nextContext === reviewContext) return;
    untrack(() => {
      reviewContext = nextContext;
      reviewRevealedIds = [];
      if (nextContext) {
        focusId = null;
        focusHeight = null;
      }
    });
  });
  // References stay scoped and visible across workspace switches; geometry is local
  // to the selected workspace. Switching never rebinds an observer or process.
  $effect(() => {
    const domain = activeDomainId;
    if (!loaded || domain === geometryDomain) return;
    untrack(() => {
      try {
        if (geometryDomain) localStorage.setItem(`${DOCK_STORAGE_KEY}:geometry:${geometryDomain}`, JSON.stringify({ rightWidth: layout.rightWidth, bottomHeight: layout.bottomHeight }));
        const saved = JSON.parse(localStorage.getItem(`${DOCK_STORAGE_KEY}:geometry:${domain}`) ?? "null");
        if (geometryDomain || saved) {
          layout.rightWidth = Number.isFinite(saved?.rightWidth) ? clamp(saved.rightWidth, 360, 900) : 440;
          layout.bottomHeight = Number.isFinite(saved?.bottomHeight) ? clamp(saved.bottomHeight, 180, 700) : 240;
        }
      } catch { message = "Workspace layout unavailable; using the current arrangement."; }
      try { restoreNarrow(JSON.parse(localStorage.getItem(`${DOCK_STORAGE_KEY}:narrow:${domain ?? "window"}`) ?? "null")); }
      catch { restoreNarrow(null); }
      geometryDomain = domain;
    });
  });
  $effect(() => {
    const data = { active: narrowActive, order: narrowOrder, hidden: narrowHidden, height: narrowHeight };
    if (loaded) {
      try { localStorage.setItem(`${DOCK_STORAGE_KEY}:narrow:${geometryDomain ?? "window"}`, JSON.stringify(data)); } catch { /* Primary save reports storage errors. */ }
    }
  });
  function restoreNarrow(data: unknown) {
    const saved = data as { active?: unknown; order?: unknown; hidden?: unknown; height?: unknown } | null;
    narrowActive = typeof saved?.active === "string" ? saved.active : null;
    narrowOrder = Array.isArray(saved?.order) ? saved.order.filter((id): id is string => typeof id === "string" && id.length <= 5000).slice(0, 12) : [];
    narrowHidden = saved?.hidden === true;
    narrowHeight = typeof saved?.height === "number" && Number.isFinite(saved.height) ? clamp(saved.height, 80, 700) : 260;
  }
  function hiddenView(view: DockView) { return narrow ? narrowHidden : layout.hidden[view.position]; }
  function hidePanel(pos: DockPosition) {
    // Preserve the presented group before leaving Focus view; clearing focus
    // changes the derived placement immediately and could reopen its old dock.
    const wasStacked = stacked;
    focusId = null;
    if (narrow) narrowHidden = true;
    else if (wasStacked) { layout.hidden.right = true; layout.hidden.bottom = true; }
    else layout.hidden[pos] = true;
  }
  $effect(() => {
    const geometry = { rightWidth: layout.rightWidth, bottomHeight: layout.bottomHeight };
    if (loaded && geometryDomain) {
      try { localStorage.setItem(`${DOCK_STORAGE_KEY}:geometry:${geometryDomain}`, JSON.stringify(geometry)); } catch { /* The primary save reports storage errors. */ }
    }
  });

  export function leaveFocus() {
    closeMenus();
    focusId = null;
    focusHeight = null;
    if (compact) narrowHidden = true;
  }
  export function open(ref: DockReference, position: DockPosition = "right") {
    try {
      layout = openDockView(layout, ref, position);
      const retained = new Set(layout.views.map(view => view.id));
      inspectionReady = inspectionReady.filter(id => retained.has(id));
      const id = dockId(ref); revealForReview(id); narrowActive = id; narrowHidden = false; message = "";
    }
    catch (cause) { message = cause instanceof Error ? cause.message : String(cause); }
  }
  function revealForReview(id: string) {
    if (surface === "review" && !reviewRevealedIds.includes(id)) reviewRevealedIds = [...reviewRevealedIds, id];
  }
  function dismissibleMenu(menu: HTMLDetailsElement) {
    const key = (event: KeyboardEvent) => {
      if (event.key !== "Escape" || !menu.open) return;
      event.preventDefault(); event.stopPropagation();
      menu.open = false;
      menu.querySelector<HTMLElement>("summary")?.focus();
    };
    menu.addEventListener("keydown", key);
    return { destroy() { menu.removeEventListener("keydown", key); } };
  }
  function closeMenus() { host?.querySelectorAll<HTMLDetailsElement>("details[open]").forEach(menu => menu.open = false); }
  function loadSavedLayouts() {
    try {
      const data = JSON.parse(localStorage.getItem(`${DOCK_STORAGE_KEY}:saved:${activeDomainId ?? "window"}`) ?? "[]");
      savedLayouts = Array.isArray(data) ? data.filter(v => typeof v?.name === "string" && v.name.length <= 60 && typeof v?.data === "string").slice(0, 8) : [];
    } catch { savedLayouts = []; }
  }
  function saveLayout() {
    const name = layoutName.trim().slice(0, 60);
    if (!name) return;
    loadSavedLayouts();
    const next = [...savedLayouts.filter(v => v.name !== name), { name, data: JSON.stringify({ ...layout, narrow: { active: narrowActive, order: narrowOrder, hidden: narrowHidden, height: narrowHeight } }) }];
    if (next.length > 8) { message = "Eight layouts are saved. Reuse an existing name to replace one."; return; }
    try { localStorage.setItem(`${DOCK_STORAGE_KEY}:saved:${activeDomainId ?? "window"}`, JSON.stringify(next)); savedLayouts = next; saveDialog?.close(); message = `Saved layout “${name}”. Shell sessions and input permissions are not saved.`; }
    catch { message = "Layout could not be saved."; }
  }
  function restoreSaved(data: string) { layout = restoreDockLayout(data); try { restoreNarrow(JSON.parse(data)?.narrow); } catch { restoreNarrow(null); } focusId = null; if (surface === "review") reviewRevealedIds = layout.views.filter(belongsToCurrentContext).map(view => view.id); closeMenus(); message = "Layout restored. Existing shell sessions remain in Sessions; none were started or ended."; }
  function openSession(session: TerminalInfo) {
    open({ kind: "terminal", domainId: session.domain_id, runId: "workspace", sessionId: session.id, title: `Your terminal ${session.id.slice(0, 6)}` }, "bottom"); closeMenus();
  }
  async function createTerminal() {
    if (!createDomain || creating) return;
    creating = true;
    try {
      if (layout.views.length >= 12) throw new Error("Close an unused view before creating a terminal.");
      const session = await workspaceTerminal.create(createDomain);
      sessions = [...sessions, session]; openSession(session); createDialog?.close();
    } catch (e) { message = String(e); } finally { creating = false; }
  }
  function openRun(kind: "evidence" | "files" | "diagram") {
    if (!run) return;
    open({ kind, runId: run.id, domainId: run.domain_id, title: kind === "evidence" ? "Checks & details" : kind === "files" ? "Files" : "Dependencies" });
  }
  function isVisible(view: DockView) {
    if (!belongsToCurrentContext(view) || hiddenView(view)) return false;
    if (effectiveFocusId) return view.id === effectiveFocusId;
    if (stacked) return view.id === visibleNarrowId;
    const peers = surfaceViews.filter(candidate => candidate.position === view.position);
    const active = peers.some(candidate => candidate.id === layout.active[view.position])
      ? layout.active[view.position]
      : peers[0]?.id;
    return view.id === active;
  }
  // Opening a dock changes the canvas geometry. Mount its detail tree on the
  // following paint so selection feedback and inspection layout never compete
  // for one main-thread frame. Hidden saved views stay dormant until selected.
  $effect(() => {
    const pending = layout.views
      .filter(view => view.kind !== "terminal" && view.kind !== "preview" && isVisible(view) && !inspectionReady.includes(view.id))
      .map(view => view.id);
    if (!pending.length) return;
    inspectionFrame = requestAnimationFrame(() => {
      inspectionFrame = 0;
      inspectionReady = [...new Set([...inspectionReady, ...pending])];
    });
    return () => { cancelAnimationFrame(inspectionFrame); inspectionFrame = 0; };
  });
  function select(view: DockView) { revealForReview(view.id); if (focusId) focusId = view.id; if (!narrow) { layout.active[view.position] = view.id; layout.hidden[view.position] = false; } narrowHidden = false; narrowActive = view.id; }
  function move(id: string, pos: DockPosition, before?: string) {
    revealForReview(id);
    if (narrow && pos === "bottom") {
      if (id !== before) { const order = tabsFor("bottom").map(v => v.id).filter(v => v !== id); const at = before ? order.indexOf(before) : -1; order.splice(at < 0 ? order.length : at, 0, id); narrowOrder = order; }
    } else layout = moveDockView(layout, id, pos, before);
    narrowActive = id; narrowHidden = false; closeMenus();
    void revealRunCommands();
  }
  export function dismiss(position: DockPosition = "right") {
    hidePanel(position);
  }
  async function revealRunCommands() {
    await tick();
    // Repositioning a dock can retain a scroll anchor halfway through the
    // command row. Preserve access to Stop/Review without moving keyboard focus.
    if (surface === "run") host?.querySelector<HTMLElement>(".run-bar")?.scrollIntoView({ block: "nearest", inline: "nearest", behavior: "instant" });
  }
  function close(id: string) { if (focusId === id) focusId = null; reviewRevealedIds = reviewRevealedIds.filter(viewId => viewId !== id); inspectionReady = inspectionReady.filter(viewId => viewId !== id); layout = closeDockView(layout, id); }
  async function drop(event: DragEvent, pos: DockPosition, before?: string) {
    event.preventDefault(); event.stopPropagation();
    const id = dragging;
    if (id) await withPreviewsHidden(() => move(id, pos, before));
    dragging = null; dropBefore = null;
  }
  function tabsFor(pos: DockPosition) {
    const views = surfaceViews.filter(v => stacked ? pos === "bottom" && !hiddenView(v) : v.position === pos);
    if (!narrow) return views;
    const rank = (id: string) => { const index = narrowOrder.indexOf(id); return index < 0 ? 12 + layout.views.findIndex(v => v.id === id) : index; };
    return views.sort((a, b) => rank(a.id) - rank(b.id));
  }
  function tabKey(event: KeyboardEvent, view: DockView, pos: DockPosition) {
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    const tabs = tabsFor(pos);
    const index = tabs.findIndex(v => v.id === view.id);
    const next = event.key === "ArrowRight" ? (index + 1) % tabs.length : event.key === "ArrowLeft" ? (index + tabs.length - 1) % tabs.length : event.key === "Home" ? 0 : event.key === "End" ? tabs.length - 1 : -1;
    if (next < 0) return;
    event.preventDefault(); select(tabs[next]);
    (event.currentTarget as HTMLElement).closest('[role="tablist"]')?.querySelectorAll<HTMLButtonElement>('[role="tab"]')[next]?.focus();
  }
  function size(pos: DockPosition) { return pos === "right" ? rightWidth : bottomHeight; }
  function setSize(pos: DockPosition, value: number) { if (pos === "right") layout.rightWidth = clamp(value, 360, rightMax); else if (focusId) focusHeight = clamp(value, bottomMin, bottomMax); else if (narrow) narrowHeight = clamp(value, bottomMin, bottomMax); else layout.bottomHeight = clamp(value, bottomMin, bottomMax); void revealRunCommands(); }
  async function startResize(event: PointerEvent, pos: DockPosition) {
    if (event.button !== 0) return;
    keyboardResize = null;
    const target = event.currentTarget as HTMLElement;
    target.setPointerCapture(event.pointerId);
    resizing = true;
    const bounds = target.getBoundingClientRect();
    const scale = pos === "right" ? bounds.width / target.offsetWidth : bounds.height / target.offsetHeight;
    await withPreviewsHidden(() => {
      if (resizing && target.hasPointerCapture(event.pointerId)) resize = { position: pos, start: pos === "right" ? event.clientX : event.clientY, original: size(pos), target, pointer: event.pointerId, scale: scale > 0 ? scale : 1 };
    });
    target.focus();
  }
  function resizeMove(event: PointerEvent) { if (resize) setSize(resize.position, resize.original + (resize.start - (resize.position === "right" ? event.clientX : event.clientY)) / resize.scale); }
  function endResize(cancel = false) { resizing = false; if (!resize) return; if (cancel) setSize(resize.position, resize.original); if (resize.target.hasPointerCapture(resize.pointer)) resize.target.releasePointerCapture(resize.pointer); resize = null; }
  function resizeKey(event: KeyboardEvent, pos: DockPosition) {
    if (event.key === "Escape") { endResize(true); keyboardResizing = false; if (keyboardResize) { setSize(keyboardResize.position, keyboardResize.original); keyboardResize = null; } return; }
    if (!["ArrowLeft", "ArrowRight", "ArrowUp", "ArrowDown", "Home", "End"].includes(event.key)) return;
    event.preventDefault();
    keyboardResizing = true;
    keyboardResize ??= { position: pos, original: size(pos) };
    const step = event.shiftKey ? 50 : 10;
    setSize(pos, event.key === "Home" ? (pos === "right" ? 360 : bottomMin) : event.key === "End" ? (pos === "right" ? rightMax : bottomMax) : size(pos) + (["ArrowLeft", "ArrowUp"].includes(event.key) ? step : -step));
  }
  function resetLayout() {
    closeMenus();
    focusId = null;
    restoreNarrow(null);
    const views = layout.views.map(view => ({ ...view, position: view.kind === "agent-output" || view.kind === "terminal" ? "bottom" as const : "right" as const }));
    layout = { ...defaultDockLayout(), views, active: { right: views.find(view => view.position === "right")?.id ?? null, bottom: views.find(view => view.position === "bottom")?.id ?? null } };
    message = "Layout reset. Open views and their source identities were preserved.";
  }
  function panelShown(pos: DockPosition) {
    return narrow
      ? surfaceViews.length > 0 && !narrowHidden
      : !layout.hidden[pos] && surfaceViews.some(view => view.position === pos);
  }
  function togglePanel(pos: DockPosition) {
    if (panelShown(pos)) {
      hidePanel(pos);
      return;
    }
    if (surface === "review") {
      const ids = presentedViews
        .filter(view => narrow || view.position === pos)
        .map(view => view.id);
      reviewRevealedIds = [...new Set([...reviewRevealedIds, ...ids])];
    }
    if (narrow) narrowHidden = false;
    else layout.hidden[pos] = false;
  }
</script>

<svelte:window onpointermove={trackDrag} onpointerup={event => { trackDrag(event); void finishDrag(); }} onpointercancel={() => void finishDrag(true)} onkeydown={event => { if (event.key === "Escape" && pointerDrag) { event.preventDefault(); void finishDrag(true); } }} onblur={() => void finishDrag(true)} />

<div class="dock-workspace" class:compact class:drag-active={!!dragging} class:inspection-focused={!!effectiveFocusId || (compact && bottomOpen)} bind:this={host}>
  {#if contextualTools}<div class="workspace-tools" class:tools-folded={!toolsExpanded} aria-label="Task views">
    {#if contextualTools}<button class="tools-toggle" aria-expanded={toolsExpanded} onclick={() => toolsExpanded = !toolsExpanded}>Inspection tools</button>{/if}
    <span>Inspect</span>
    {#if focusId || (compact && bottomOpen)}<button onclick={() => { focusId = null; if (compact) narrowHidden = true; }}>Return to task</button>{/if}
    {#each ["evidence", "files", "diagram"] as kind}
      {@const reference = runReference(kind as "evidence" | "files" | "diagram")}
      <button class="draggable-view" class:drag-source={!!reference && dragging === dockId(reference)} aria-label={reference?.title ?? (kind === "diagram" ? "Dependencies" : kind === "files" ? "Files" : "Checks & details")} disabled={!reference} title="Click to open, or drag to the right or bottom edge" onpointerdown={event => reference && beginDrag(event, reference)} onclick={event => { consumeDragClick(event); if (!event.defaultPrevented) openRun(kind as "evidence" | "files" | "diagram"); }}>{kind === "evidence" ? "Checks & details" : kind === "files" ? "Files" : "Dependencies"}</button>
    {/each}
    <button class="draggable-view" aria-label="Preview" title="Click to open, or drag to the right or bottom edge" disabled={!activeDomainId} onpointerdown={event => activeDomainId && beginDrag(event, { kind: "preview", domainId: activeDomainId, runId: "workspace", title: "Local preview" })} onclick={event => { consumeDragClick(event); if (!event.defaultPrevented && activeDomainId) open({ kind: "preview", domainId: activeDomainId, runId: "workspace", title: "Local preview" }); }}>Preview</button>
    <details use:previewMenu use:dismissibleMenu><summary>Sessions · {sessions.filter(s => s.state !== "ended").length}</summary><div class="menu">
      <button disabled={!activeDomainId || !hasNativeTerminal()} onclick={() => withPreviewsHidden(() => { createDomain = activeDomainId; closeMenus(); createDialog?.showModal(); })}>New workspace terminal</button>
      {#if !hasNativeTerminal()}<span class="menu-note">Interactive terminals need native Desktop.</span>{/if}
      {#each sessions as session (session.id)}<button onclick={() => openSession(session)}>You · {session.cwd.split(/[\\/]/).pop()} · {session.state} · {session.id.slice(0, 6)}</button>{/each}
      {#if !sessions.length}<span class="menu-note">No user sessions in this app process.</span>{/if}
    </div></details>
    <details use:previewMenu use:dismissibleMenu class="layout-menu" ontoggle={e => { if ((e.currentTarget as HTMLDetailsElement).open) loadSavedLayouts(); }}><summary>Layout · {presentedViews.length} views</summary><div class="menu layout-options">
      <div class="layout-options-body">
        <p class="menu-heading">Panels</p>
        {#each (narrow ? ["bottom"] as const : positions) as pos}<button aria-pressed={panelShown(pos)} onclick={() => togglePanel(pos)}>{panelShown(pos) ? "Hide" : "Show"} {pos} dock</button>{/each}
        <p class="menu-note">Drag a view or tab to the right or bottom edge. Drag dividers to resize.</p>
        <p class="menu-heading">Saved layouts <span>{savedLayouts.length}</span></p>
        {#each savedLayouts as saved (saved.name)}<button onclick={() => restoreSaved(saved.data)}>Restore {saved.name}</button>{:else}<p class="menu-note">No saved layouts for this workspace.</p>{/each}
        {#if layout.views.length}<p class="menu-heading">Open views</p>{/if}
        {#each presentedViews as view (view.id)}<button onclick={() => select(view)}>{view.title} · {view.domainId.split(/[\\/]/).pop()}</button>{/each}
      </div>
      <div class="layout-options-footer">
        <button onclick={resetLayout}>Reset layout</button>
        <button onclick={() => withPreviewsHidden(() => { layoutName = ""; closeMenus(); saveDialog?.showModal(); })}>Save current layout</button>
      </div>
    </div></details>
  </div>{/if}
  {#if contextualTools && message}<p class="layout-status" role="status">{message}</p>{/if}
  <div class="dock-grid" bind:this={grid} class:right-open={rightOpen} class:bottom-open={bottomOpen} class:summary-overlay={summaryOverlay} class:dragging={!!dragging}
    style:--right-size={`${rightWidth}px`} style:--bottom-size={`${bottomHeight}px`}>
    {#if dragging}
      <div class="drop-zones" aria-hidden="true">
        {#if !narrow}<div data-dock-position="right" class="drop-zone drop-right" class:targeted={dragTarget === "right"}>Dock right</div>{/if}
        <div data-dock-position="bottom" class="drop-zone drop-bottom" class:targeted={dragTarget === "bottom"}>Dock below</div>
      </div>
    {/if}
    <div class="mission-content">{@render children()}</div>
    {#each positions as pos}
      {@const shown = pos === "right" ? rightOpen : bottomOpen}
      {#if shown}
        <!-- An adjustable ARIA separator is keyboard interactive (window splitter pattern). -->
        <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
        <div class="resize" tabindex="0" class:right={pos === "right"} class:bottom={pos === "bottom"} role="separator" aria-label={`Resize ${pos} dock`} aria-orientation={pos === "right" ? "vertical" : "horizontal"}
          aria-valuenow={Math.round(size(pos))} aria-valuemin={pos === "right" ? 360 : bottomMin} aria-valuemax={Math.round(pos === "right" ? rightMax : bottomMax)}
          onpointerdown={e => startResize(e, pos)} onpointermove={resizeMove} onpointerup={() => endResize()} onpointercancel={() => endResize(true)} onkeydown={e => resizeKey(e, pos)} onblur={() => { keyboardResize = null; keyboardResizing = false; }}></div>
      {/if}
      {#if shown || dragging}
        <div class="dock-tabs" data-dock-position={pos} tabindex="-1" class:right={pos === "right"} class:bottom={pos === "bottom"} class:drop-only={!shown} style:grid-area={pos === "right" ? "right-tabs" : "bottom-tabs"} role="tablist" aria-label={`${pos} dock`} ondragover={e => e.preventDefault()} ondrop={e => drop(e, pos)}>
          {#each tabsFor(pos) as view (view.id)}
            <div class="tab-wrap" class:drop-before={!!dragging && dropBefore === view.id} draggable="false" ondragstart={e => { dragging = view.id; e.dataTransfer?.setData("text/plain", view.id); }} ondragend={() => { dragging = null; dropBefore = null; }} role="presentation" ondragover={e => { e.preventDefault(); dropBefore = view.id; }} ondrop={e => drop(e, pos, view.id)}>
              <button role="tab" draggable="false" data-dock-position={pos} data-dock-before={view.id} onpointerdown={event => beginDrag(event, view)} aria-selected={isVisible(view)} tabindex={isVisible(view) ? 0 : -1} class:active={isVisible(view)} class:drag-source={dragging === view.id} title={`${view.title} · ${view.domainId} · ${view.runId} · Drag to move`} onclick={event => { consumeDragClick(event); if (!event.defaultPrevented) select(view); }} ondblclick={() => view.pinned = true} onkeydown={e => tabKey(e, view, pos)}>{view.pinned ? "• " : ""}{view.title}</button>
              <button class="close-view" aria-label={`Close ${view.title} view`} onclick={() => close(view.id)}>×</button>
            </div>
          {/each}
          {#if !shown}<span class="drop-label">Move view to {pos}</span>{/if}
          <button class="hide-dock" aria-label={`Hide ${pos} dock`} title="Hide panel; keep views" onclick={() => hidePanel(pos)}>▥</button>
        </div>
      {/if}
    {/each}
    <!-- One keyed list preserves view instances when their grid area changes. -->
    {#each layout.views as view (view.id)}
      <div class="dock-panel" role={view.kind === "agent" ? "complementary" : undefined} aria-label={view.kind === "agent" ? "Selected worker details" : undefined} use:evidenceMotion={{ visible: isVisible(view), position: stacked ? "bottom" : view.position, reducedMotion: uiPrefs.reducedMotion, suspended: view.kind === "preview" || !!dragging || resizing || keyboardResizing }} class:summary-panel={view.kind === "agent"} class:right-panel={!stacked && view.position === "right"} class:output-panel={view.kind === "agent-output" || view.kind === "preview"} hidden={!isVisible(view)} style:grid-area={stacked || view.position === "bottom" ? "bottom-panel" : "right-panel"}>
        <div class="panel-tools">
          {#if view.domainId !== activeDomainId}<strong>Other project</strong>{/if}
          <button aria-pressed={view.pinned} onclick={() => view.pinned = !view.pinned}>{view.pinned ? "Unpin view" : "Pin view"}</button>
          <details use:previewMenu use:dismissibleMenu><summary aria-label={`Options for ${view.title}`}>View options</summary><div class="menu">
            <button onclick={() => { focusHeight = null; focusId = focusId === view.id ? null : view.id; narrowActive = view.id; }}>{focusId === view.id ? "Return to task" : "Focus view"}</button>
            <button onclick={() => move(view.id, "right")}>Move to right</button><button onclick={() => move(view.id, "bottom")}>Move to bottom</button>
            <button onclick={() => { const pos = narrow ? "bottom" : view.position; const peers = tabsFor(pos); const i = peers.findIndex(v => v.id === view.id); if (i > 0) move(view.id, pos, peers[i - 1].id); }}>Move tab left</button>
            <button onclick={() => { const pos = narrow ? "bottom" : view.position; const peers = tabsFor(pos); const i = peers.findIndex(v => v.id === view.id); if (i < peers.length - 1) move(view.id, pos, peers[i + 2]?.id); }}>Move tab right</button>
            <button onclick={() => close(view.id)}>Close view</button>
          </div></details>
        </div>
        {#if view.kind === "terminal" && view.sessionId}
          {#await import("./WorkspaceTerminal.svelte")}
            <p class="menu-note" role="status">Loading terminal view…</p>
          {:then module}
            <module.default sessionId={view.sessionId} domainId={view.domainId} visible={isVisible(view)} />
          {:catch}
            <p class="menu-note" role="alert">Terminal view could not load. Your session remains available in Sessions.</p>
          {/await}
        {:else if view.kind === "preview"}
          <LocalPreview visible={isVisible(view) && view.domainId === activeDomainId && previewAllowed && !dragging && !resizing && !keyboardResizing} />
        {:else if inspectionReady.includes(view.id)}
          <DockInspection {view} {backend} {snapshot} visible={isVisible(view)} {onReview} {onOpenApprovals} onOpenOutput={(reference) => open(reference, "bottom")} />
        {:else}
          <p class="menu-note" role="status">Opening details…</p>
        {/if}
      </div>
    {/each}
  </div>
</div>
<dialog bind:this={createDialog} aria-label="Create your workspace terminal">
  <h2>Open your workspace terminal</h2><p>{createDomain}</p>
  <p>This starts your own interactive shell in this folder. Its edits are direct and outside Run Apply protection. Agent output remains read only.</p>
  {#if message}<p role="alert">{message}</p>{/if}
  <button disabled={creating} onclick={() => createDialog?.close()}>Cancel</button><button disabled={creating} onclick={createTerminal}>{creating ? "Opening…" : "Create terminal"}</button>
</dialog>
<dialog bind:this={saveDialog} aria-label="Save workspace layout"><form onsubmit={e => { e.preventDefault(); saveLayout(); }}>
  <h2>Save workspace layout</h2><label>Layout name<input bind:value={layoutName} maxlength="60" required /></label><p>Stores view references and sizing for this workspace. It cannot restart processes or restore input permission.</p>
  <button type="button" onclick={() => saveDialog?.close()}>Cancel</button><button type="submit">Save layout</button>
</form></dialog>

<style>
  .mission-content { container-type: inline-size; container-name: mission; }
  .dock-grid { position: relative; }
  .draggable-view, [role="tab"] { touch-action: none; }
  .draggable-view, [role="tab"], .mission-content, .drop-zone { transition: transform var(--pytxo-motion-fast) var(--pytxo-motion-ease), background-color var(--pytxo-motion-fast) var(--pytxo-motion-ease), border-color var(--pytxo-motion-fast) var(--pytxo-motion-ease), color var(--pytxo-motion-fast) var(--pytxo-motion-ease), opacity var(--pytxo-motion-fast) var(--pytxo-motion-ease); }
  .draggable-view::before { content: "⠿"; margin-right: 5px; opacity: .5; }
  .draggable-view:hover, [role="tab"]:hover { cursor: grab; }
  .drag-active { cursor: grabbing; user-select: none; }
  .drag-source { transform: translateY(-1px); border-color: color-mix(in srgb, var(--pytxo-accent) 52%, var(--pytxo-line)); background: var(--pytxo-surface-active); color: var(--pytxo-text-strong); box-shadow: 0 4px 14px color-mix(in srgb, var(--pytxo-surface-shell) 74%, transparent); cursor: grabbing !important; }
  .dock-grid.dragging .mission-content { transform: scale(.995); opacity: .82; }
  .drop-zones { position: absolute; inset: 0; z-index: 50; pointer-events: none; }
  .drop-zone { position: absolute; display: grid; place-items: center; pointer-events: auto; border: 2px dashed var(--pytxo-accent); border-radius: 10px; background: color-mix(in srgb, var(--pytxo-surface-panel) 88%, var(--pytxo-accent)); color: var(--pytxo-text-strong); font-size: 14px; font-weight: 600; }
  .drop-right { right: 12px; top: 60px; bottom: 32%; width: 28%; }
  .drop-bottom { left: 12px; right: 12px; bottom: 12px; height: 26%; }
  .drop-zone.targeted { transform: scale(1.012); border-style: solid; background: color-mix(in srgb, var(--pytxo-surface-panel) 70%, var(--pytxo-accent)); }
  .dock-workspace{position:relative;display:flex;flex:1;min-height:0;min-width:0;flex-direction:column;overflow:hidden}.workspace-tools{display:flex;flex-wrap:wrap;align-items:center;gap:6px;min-height:38px;padding:4px 16px;border-bottom:1px solid var(--pytxo-line-soft);font-size:12px}.workspace-tools>span{margin-right:auto;color:var(--pytxo-text-muted)}
  button,summary{min-height:28px;padding:4px 9px;border:1px solid transparent;border-radius:4px;background:transparent;color:var(--pytxo-text-soft);font:inherit;cursor:pointer}button:hover,summary:hover{background:var(--pytxo-surface-active)}button:disabled{opacity:.45;cursor:not-allowed}button:focus-visible,summary:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:-2px}
  details{position:relative}summary{list-style:none}.menu{position:absolute;right:0;top:100%;z-index:30;display:flex;min-width:200px;max-width:320px;max-height:50vh;overflow:auto;flex-direction:column;padding:6px;border:1px solid var(--pytxo-line);border-radius:6px;background:var(--pytxo-surface-raised);box-shadow:0 8px 24px #0005}.menu button{text-align:left;overflow-wrap:anywhere}.layout-status{margin:0;padding:6px 16px;font-size:12px;color:var(--pytxo-text-muted)}
  .workspace-tools > button, .workspace-tools > details > summary { min-height: 40px; display: flex; align-items: center; font-size: 13px; }
  .menu-note{padding:8px;line-height:1.5;color:var(--pytxo-text-muted)}dialog{position:fixed;inset:0;margin:auto;max-height:calc(100vh - 32px);overflow:auto;max-width:min(500px,90vw);padding:24px;border:1px solid var(--pytxo-line);border-radius:8px;background:var(--pytxo-surface-raised);color:var(--pytxo-text-strong)}dialog::backdrop{background:#0009}dialog h2{font-size:18px}dialog p{font-size:13px;line-height:1.6;overflow-wrap:anywhere}dialog button{border-color:var(--pytxo-line);margin-right:8px}
  .dock-grid{display:grid;flex:1;min-height:0;min-width:0;grid-template-columns:minmax(0,1fr) 0 0;grid-template-rows:0 minmax(0,1fr) 0 0 0;grid-template-areas:"mission right-resize right-tabs" "mission right-resize right-panel" "bottom-resize right-resize right-panel" "bottom-tabs right-resize right-panel" "bottom-panel right-resize right-panel"}
  .dock-grid.right-open{grid-template-columns:minmax(0,1fr) 6px var(--right-size);grid-template-rows:36px minmax(0,1fr) 0 0 0}.dock-grid.bottom-open{grid-template-rows:0 minmax(0,1fr) 6px 36px var(--bottom-size)}.dock-grid.right-open.bottom-open{grid-template-rows:36px minmax(0,1fr) 6px 36px var(--bottom-size)}
  .dock-grid.right-open.summary-overlay{grid-template-columns:minmax(0,1fr) 0 0;grid-template-rows:0 minmax(0,1fr) 0 0 0}
  .summary-overlay>.resize.right{position:absolute;z-index:7;top:0;right:var(--right-size);bottom:0;width:6px}
  .summary-overlay>.dock-tabs.right{position:absolute;z-index:8;top:0;right:0;width:var(--right-size);height:36px;border-left:1px solid var(--pytxo-line)}
  .summary-overlay>.dock-panel.summary-panel.right-panel{position:absolute;z-index:6;top:36px;right:0;bottom:0;width:var(--right-size);border-left:1px solid var(--pytxo-line);box-shadow:-18px 0 36px color-mix(in srgb,var(--pytxo-surface-shell) 70%,transparent)}
  .dock-grid.right-open.summary-overlay.bottom-open{grid-template-rows:0 minmax(0,1fr) 6px 36px var(--bottom-size)}
  .summary-overlay.bottom-open>.resize.right,.summary-overlay.bottom-open>.dock-panel.summary-panel.right-panel{bottom:calc(var(--bottom-size) + 42px)}
  .compact .dock-grid.bottom-open{grid-template-rows:0 minmax(80px,1fr) 6px 36px var(--bottom-size)}
  .mission-content{grid-area:mission;min-width:0;min-height:0;display:flex;flex-direction:column;overflow:hidden}.mission-content :global(.content){flex:1;min-height:0;overflow:auto}.mission-content :global(.content.work-content){overflow:hidden}
  .inspection-focused .mission-content :global(.ledger),.inspection-focused .mission-content :global(.mission-stages),.inspection-focused .mission-content :global(.mission-outcome),.inspection-focused .mission-content :global(.run-bar dl){display:none}
  .resize{padding:0;border:0;border-radius:0;touch-action:none;background:var(--pytxo-line-soft)}.resize.right{grid-area:right-resize;cursor:col-resize}.resize.bottom{grid-area:bottom-resize;cursor:row-resize}.resize:hover,.resize:focus-visible{background:var(--pytxo-accent)}
  .dock-tabs{position:relative;z-index:4;display:flex;align-items:stretch;min-width:0;overflow:auto;background:var(--pytxo-surface-raised);border-bottom:1px solid var(--pytxo-line);font-size:12px}.tab-wrap{display:flex;flex-shrink:0;max-width:220px}.tab-wrap>[role="tab"]{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;border-radius:0}.tab-wrap>[role="tab"].active{border-bottom:2px solid var(--pytxo-accent);color:var(--pytxo-text-strong);background:var(--pytxo-surface-panel)}.close-view{padding-inline:5px}.hide-dock{margin-left:auto;min-width:30px}.dock-panel{min-height:0;min-width:0;overflow:auto;background:var(--pytxo-surface-panel)}.dock-panel[hidden]{display:none}.panel-tools{display:flex;flex-wrap:wrap;align-items:center;justify-content:flex-end;gap:4px;padding:4px 8px;border-bottom:1px solid var(--pytxo-line-soft);font-size:11px}.panel-tools strong{margin-right:auto;color:var(--state-unknown)}
  .dragging .dock-tabs{outline:1px dashed var(--pytxo-accent);outline-offset:-2px}.drop-only{position:absolute;z-index:40;right:12px;bottom:12px;min-height:42px;padding:8px;border:1px dashed var(--pytxo-accent)}.drop-only[aria-label="right dock"]{bottom:90px}.drop-label{align-self:center;padding:0 10px}
  .output-panel,.summary-panel{display:flex;flex-direction:column}.output-panel .panel-tools,.summary-panel .panel-tools{flex-shrink:0}
  .tab-wrap{position:relative}.tab-wrap.drop-before::before{content:"";position:absolute;inset-block:3px;left:0;width:2px;background:var(--pytxo-accent);z-index:1}dialog label{display:flex;flex-direction:column;gap:8px;font-size:13px}dialog input{min-height:36px;padding:6px;border:1px solid var(--pytxo-line);border-radius:4px;background:var(--pytxo-surface-panel);color:var(--pytxo-text-strong);font:inherit}
  @media(max-width:760px){.workspace-tools{padding-inline:8px;gap:2px}.workspace-tools>span{display:none}.dock-tabs{font-size:11px}}
  .menu.layout-options { width: min(300px, calc(100vw - 32px)); max-height: min(560px, calc(100dvh - 160px)); overflow: hidden; padding: 0; border-radius: 8px; }
  .layout-options-body { min-height: 0; overflow: auto; overscroll-behavior: contain; padding: 6px; }
  .layout-options button { display: flex; align-items: center; width: 100%; min-height: 40px; font-size: 13px; }
  .menu-heading { display: flex; justify-content: space-between; margin: 10px 10px 4px; color: var(--pytxo-text-muted); font-size: 11px; font-weight: 600; }
  .layout-options .menu-note { margin: 0; font-size: 12px; }
  .layout-options-footer { flex: none; display: flex; padding: 6px; border-top: 1px solid var(--pytxo-line); }
  .layout-options-footer button:first-child { flex: none; }
  .layout-options-footer button { white-space: nowrap; justify-content: center; width: auto; flex: 1; }
  .workspace-tools.tools-folded { position: absolute; top: 8px; right: 12px; z-index: 3; min-height: 28px; padding: 0; border: 0; justify-content: flex-end; }
  .workspace-tools.tools-folded>:not(.tools-toggle) { display: none; }
  .workspace-tools>.tools-toggle { display: inline-flex; align-items: center; gap: 7px; min-height: 28px; font-size: 11px; color: var(--pytxo-text-muted); }
  .workspace-tools>.tools-toggle:hover { color: var(--foreground); }
  /* Disclosure chevron so the toggle reads as a control, not an orphan label. */
  .workspace-tools>.tools-toggle::after { content: ""; width: 5px; height: 5px; border-right: 1.5px solid currentColor; border-bottom: 1.5px solid currentColor; transform: translateY(-2px) rotate(45deg); transition: transform 150ms ease; }
  .workspace-tools>.tools-toggle[aria-expanded="true"]::after { transform: translateY(1px) rotate(-135deg); }
  @media (prefers-reduced-motion: reduce) { .workspace-tools>.tools-toggle::after { transition: none; } }
</style>
