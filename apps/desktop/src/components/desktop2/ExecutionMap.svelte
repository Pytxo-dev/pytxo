<script lang="ts">
  import { onDestroy } from "svelte";
  import { agentState } from "../../lib/epistemic";
  import { executionTopology } from "../../lib/execution-topology";
  import { rememberWorkbenchSelection, workbenchSelection, type WorkbenchCamera } from "../../lib/workbench-selection";
  import type { DockPosition, DockReference } from "../../lib/dock-layout";
  import type { AgentDto, RoutingDisplaySummary, RunDto, RunReviewDto } from "../../lib/types";
  import RunLedger from "./RunLedger.svelte";
  import FleetBoard from "./FleetBoard.svelte";
  import type { DesktopBackend } from "../../lib/desktop-backend";

  let {
    taskDescriptions = {}, run, review, agents, selectedAgentId = null,
    routingSummary = null, routingLoading = false, routingIdentityUnknown = false,
    onSelect, onInspect, onDismissInspect = () => {}, backend = null, taskClis = {},
  }: {
    taskDescriptions?: Record<string, string>; run: RunDto; review: RunReviewDto | null; agents: AgentDto[];
    backend?: DesktopBackend | null; taskClis?: Record<string, string>;
    routingSummary?: RoutingDisplaySummary | null; routingLoading?: boolean; routingIdentityUnknown?: boolean;
    selectedAgentId?: string | null; onSelect: (id: string) => void;
    onInspect: (ref: DockReference, position?: DockPosition) => void; onDismissInspect?: () => void;
  } = $props();

  const MIN_SCALE = .55;
  const MAX_SCALE = 1.6;
  const FIT_PADDING = 48;
  // A small plan may fit larger than 100% so three nodes do not float in an empty canvas.
  const FIT_MAX_SCALE = 1.3;
  let viewportWidth = $state(0);
  let viewportHeight = $state(0);
  // Dock resizing changes the viewport frequently, but graph layout only
  // changes when the canvas crosses the compact orientation breakpoint.
  const canvasOrientation = $derived<"vertical" | "horizontal">(viewportWidth > 0 && viewportWidth < 520 ? "vertical" : "horizontal");
  const routed = $derived(run.routing_revision != null || routingIdentityUnknown);
  const topology = $derived(executionTopology(review?.plan ?? null, agents, run, canvasOrientation, routingSummary, routingIdentityUnknown));
  const nodes = $derived(topology.nodes);
  const edges = $derived(topology.edges);
  let viewport: HTMLDivElement | undefined = $state();
  let scene: HTMLDivElement | undefined = $state();
  let minimapViewport: SVGRectElement | undefined = $state();
  let scaleLabel = $state(100);
  let overviewMode = $state(false);
  let microOverviewMode = $state(false);
  let selectedId = $state<string | null>(null);
  // A run that put several CLIs to work opens on the fleet board; others keep the canvas.
  // A chosen view carries across runs while that view exists for the run.
  const runAgents = $derived(agents.filter(agent => agent.run_id === run.id));
  // Fleet is always reachable for recorded workers; it opens first when several agents share the run.
  const fleetAvailable = $derived(!routed && runAgents.length > 0);
  const fleetRun = $derived(fleetAvailable && new Set(runAgents.map(agent => agent.launcher?.id).filter(Boolean)).size > 1);
  let chosenView = $state<"fleet" | "canvas" | "list" | null>(null);
  const viewMode = $derived(chosenView && (chosenView !== "fleet" || fleetAvailable) ? chosenView : fleetRun ? "fleet" : "canvas");
  let runKey = "";
  let camera: WorkbenchCamera = { x: FIT_PADDING, y: FIT_PADDING, scale: 1 };
  let cameraMode: "fit" | "manual" = "fit";
  let gesture = $state<{ pointerId: number; x: number; y: number; originX: number; originY: number; started: boolean } | null>(null);
  let suppressNodeClick = false;
  let grid = $state<HTMLDivElement>();
  let transformFrame = 0;
  let inspectFrame = 0;
  let lastCompactLayout: boolean | null = null;
  let lastContentBounds = "";

  const selected = $derived(nodes.find(node => node.task.task_id === selectedId) ?? nodes.find(node => routed ? node.routingAttempt?.state === "running" : node.agent?.status === "running") ?? nodes[0]);
  const prerequisites = $derived(selected ? nodes.filter(node => selected.task.depends_on.includes(node.task.task_id)) : []);
  const dependents = $derived(selected ? nodes.filter(node => node.task.depends_on.includes(selected.task.task_id)) : []);
  const showMinimap = $derived(nodes.length > 8 || topology.width * (scaleLabel / 100) > viewportWidth || topology.height * (scaleLabel / 100) > viewportHeight);

  function label(value: string): string {
    const words = value.replaceAll("_", " ");
    return words.charAt(0).toUpperCase() + words.slice(1);
  }

  function clamp(value: number, minimum: number, maximum: number) { return Math.max(minimum, Math.min(maximum, value)); }
  function applyTransform() {
    if (scene) {
      scene.style.transform = `translate3d(${camera.x}px, ${camera.y}px, 0) scale(${camera.scale})`;
      scene.style.setProperty("--camera-inverse", String(1 / camera.scale));
    }
    // The dot grid travels with the scene so dragging reads as moving the canvas.
    if (grid) {
      const step = 24 * camera.scale;
      grid.style.backgroundSize = `${step}px ${step}px`;
      grid.style.backgroundPosition = `${camera.x}px ${camera.y}px`;
    }
    if (minimapViewport) {
      minimapViewport.setAttribute("x", String(-camera.x / camera.scale));
      minimapViewport.setAttribute("y", String(-camera.y / camera.scale));
      minimapViewport.setAttribute("width", String(viewportWidth / camera.scale));
      minimapViewport.setAttribute("height", String(viewportHeight / camera.scale));
    }
  }
  function scheduleTransform() {
    if (transformFrame) return;
    transformFrame = requestAnimationFrame(() => { transformFrame = 0; applyTransform(); });
  }
  function reflectCamera() {
    scaleLabel = Math.round(camera.scale * 100);
    overviewMode = camera.scale < MIN_SCALE;
    microOverviewMode = camera.scale < .28;
  }
  function rememberCamera() {
    reflectCamera();
    rememberWorkbenchSelection(run.id, run.domain_id, { camera: { ...camera, mode: cameraMode } });
  }
  function fit() {
    if (!viewportWidth || !viewportHeight) return;
    cameraMode = "fit";
    camera.scale = Math.max(Number.EPSILON, Math.min(Math.max(1, viewportWidth - FIT_PADDING * 2) / topology.width, Math.max(1, viewportHeight - FIT_PADDING * 2) / topology.height, FIT_MAX_SCALE));
    camera.x = (viewportWidth - topology.width * camera.scale) / 2;
    camera.y = (viewportHeight - topology.height * camera.scale) / 2;
    applyTransform();
    rememberCamera();
  }
  function zoomAt(nextScale: number, clientX: number, clientY: number) {
    cameraMode = "manual";
    // Fit may produce an overview below the manual zoom range. Zooming out
    // there must not unexpectedly zoom in; zooming in returns to 55%.
    const next = camera.scale < MIN_SCALE && nextScale <= camera.scale
      ? camera.scale : clamp(nextScale, MIN_SCALE, MAX_SCALE);
    const worldX = (clientX - camera.x) / camera.scale;
    const worldY = (clientY - camera.y) / camera.scale;
    camera.x = clientX - worldX * next;
    camera.y = clientY - worldY * next;
    camera.scale = next;
    applyTransform();
    rememberCamera();
  }
  function zoomBy(delta: number) { zoomAt(camera.scale + delta, viewportWidth / 2, viewportHeight / 2); }
  function resetZoom() { zoomAt(1, viewportWidth / 2, viewportHeight / 2); }
  function handleWheel(event: WheelEvent) {
    event.preventDefault();
    const bounds = viewport?.getBoundingClientRect();
    if (!bounds) return;
    if (event.ctrlKey || event.metaKey) {
      zoomAt(camera.scale * Math.exp(-event.deltaY * .0015), event.clientX - bounds.left, event.clientY - bounds.top);
    } else {
      cameraMode = "manual";
      camera.x -= event.deltaX;
      camera.y -= event.deltaY;
      scheduleTransform();
      rememberCamera();
    }
  }
  // Drags may start anywhere, including on a worker; a press that never moves stays a click.
  function beginPan(event: PointerEvent) {
    if (event.button !== 0 || (event.target as HTMLElement).closest("a, summary, .minimap button")) return;
    // Background presses grab at once; a press on a worker waits for real movement.
    const onWorker = !!(event.target as HTMLElement).closest("button");
    gesture = { pointerId: event.pointerId, x: event.clientX, y: event.clientY, originX: camera.x, originY: camera.y, started: !onWorker };
    if (!onWorker) { cameraMode = "manual"; viewport?.setPointerCapture(event.pointerId); }
  }
  function pan(event: PointerEvent) {
    if (!gesture || gesture.pointerId !== event.pointerId) return;
    if (!gesture.started) {
      if (Math.hypot(event.clientX - gesture.x, event.clientY - gesture.y) < 4) return;
      gesture.started = true;
      cameraMode = "manual";
      viewport?.setPointerCapture(event.pointerId);
    }
    camera.x = gesture.originX + event.clientX - gesture.x;
    camera.y = gesture.originY + event.clientY - gesture.y;
    scheduleTransform();
  }
  function endPan(event: PointerEvent) {
    if (!gesture || gesture.pointerId !== event.pointerId) return;
    if (viewport?.hasPointerCapture(event.pointerId)) viewport.releasePointerCapture(event.pointerId);
    const moved = gesture.started;
    gesture = null;
    if (!moved) return;
    suppressNodeClick = true;
    setTimeout(() => { suppressNodeClick = false; }, 0);
    rememberCamera();
  }
  function handleCanvasKey(event: KeyboardEvent) {
    if (event.key === "Escape") { event.preventDefault(); onDismissInspect(); return; }
    if (event.target !== viewport) return;
    navigateCamera(event);
  }
  function navigateCamera(event: KeyboardEvent) {
    if (["+", "="].includes(event.key)) { event.preventDefault(); zoomBy(.1); return; }
    if (event.key === "-") { event.preventDefault(); zoomBy(-.1); return; }
    if (event.key === "0") { event.preventDefault(); resetZoom(); return; }
    if (event.key.toLowerCase() === "f") { event.preventDefault(); fit(); return; }
    const distance = event.shiftKey ? 80 : 28;
    if (event.key === "ArrowLeft") camera.x += distance;
    else if (event.key === "ArrowRight") camera.x -= distance;
    else if (event.key === "ArrowUp") camera.y += distance;
    else if (event.key === "ArrowDown") camera.y -= distance;
    else return;
    event.preventDefault();
    cameraMode = "manual";
    applyTransform();
    rememberCamera();
  }
  function navigateMinimap(event: MouseEvent) {
    const svg = (event.currentTarget as HTMLButtonElement).querySelector("svg");
    const matrix = svg?.getScreenCTM();
    if (!matrix) return;
    // Keyboard activation centers the selected worker. Pointer activation maps
    // through the SVG viewBox, including its letterboxing, into scene space.
    const point = event.detail === 0 && selected
      ? { x: selected.sceneX, y: selected.sceneY }
      : new DOMPoint(event.clientX, event.clientY).matrixTransform(matrix.inverse());
    cameraMode = "manual";
    camera.x = viewportWidth / 2 - clamp(point.x, 0, topology.width) * camera.scale;
    camera.y = viewportHeight / 2 - clamp(point.y, 0, topology.height) * camera.scale;
    applyTransform();
    rememberCamera();
  }
  function handleMinimapKey(event: KeyboardEvent) {
    event.stopPropagation();
    navigateCamera(event);
  }
  function inspectAgent(agent: AgentDto) {
    selectedId = agent.task_id;
    rememberWorkbenchSelection(run.id, run.domain_id, { taskId: agent.task_id });
    onSelect(agent.id);
    cancelAnimationFrame(inspectFrame);
    inspectFrame = requestAnimationFrame(() => {
      inspectFrame = 0;
      onInspect({ kind: "agent", domainId: agent.domain_id, runId: agent.run_id, agentId: agent.id, title: taskDescriptions[agent.task_id] ?? agent.task_id }, "right");
    });
  }
  function choose(taskId: string) {
    const node = nodes.find(candidate => candidate.task.task_id === taskId);
    if (node?.agent) { inspectAgent(node.agent); return; }
    selectedId = taskId;
    rememberWorkbenchSelection(run.id, run.domain_id, { taskId });
  }
  function related(taskId: string) {
    return taskId === selected?.task.task_id || prerequisites.some(node => node.task.task_id === taskId) || dependents.some(node => node.task.task_id === taskId);
  }

  // Canvas/List replaces these DOM elements without remounting ExecutionMap.
  // Bind observation and input to the current viewport, not the first mount.
  $effect(() => {
    const element = viewport;
    if (!element) return;
    const observer = new ResizeObserver(([entry]) => {
      viewportWidth = entry.contentRect.width;
      viewportHeight = entry.contentRect.height;
      const compactLayout = entry.contentRect.width < 520;
      const layoutChanged = lastCompactLayout !== null && compactLayout !== lastCompactLayout;
      lastCompactLayout = compactLayout;
      const remembered = workbenchSelection(run.id, run.domain_id).camera;
      cameraMode = remembered?.mode ?? "fit";
      if (remembered && !layoutChanged && cameraMode === "manual") {
        camera = { ...remembered, scale: clamp(remembered.scale, Number.EPSILON, MAX_SCALE) };
        applyTransform();
        reflectCamera();
      } else queueMicrotask(fit);
    });
    observer.observe(element);
    element.addEventListener("wheel", handleWheel, { passive: false });
    return () => {
      observer.disconnect();
      element.removeEventListener("wheel", handleWheel);
      cancelAnimationFrame(transformFrame);
      transformFrame = 0;
    };
  });
  $effect(() => { if (scene || minimapViewport) applyTransform(); });
  // Review data can arrive after the viewport's first ResizeObserver event.
  // Refit new content only until the user takes control of the camera.
  $effect(() => {
    const bounds = `${topology.width}:${topology.height}`;
    if (bounds === lastContentBounds) return;
    lastContentBounds = bounds;
    if (cameraMode === "fit") queueMicrotask(() => {
      if (cameraMode === "fit") fit();
    });
  });
  $effect(() => {
    const nextKey = `${run.domain_id}\u0000${run.id}`;
    if (nextKey === runKey) return;
    runKey = nextKey;
    const remembered = workbenchSelection(run.id, run.domain_id);
    selectedId = remembered.taskId ?? null;
    cameraMode = remembered.camera?.mode ?? "fit";
    if (remembered.camera && cameraMode === "manual") {
      camera = { ...remembered.camera, scale: clamp(remembered.camera.scale, Number.EPSILON, MAX_SCALE) };
      queueMicrotask(() => { applyTransform(); reflectCamera(); });
    } else queueMicrotask(fit);
  });
  onDestroy(() => cancelAnimationFrame(inspectFrame));
</script>

<section class="execution-map" aria-label="Recorded worker canvas" data-testid="execution-map">
  <header class="map-toolbar">
    <div><strong>Recorded workers</strong><span>{nodes.length} {nodes.length === 1 ? "task" : "tasks"} · {topology.waves.length} {topology.waves.length === 1 ? "step" : "steps"}</span></div>
    <div class="view-toggle" role="group" aria-label="Worker view">
      {#if fleetAvailable}<button aria-pressed={viewMode === "fleet"} class:active={viewMode === "fleet"} onclick={() => chosenView = "fleet"}>Fleet</button>{/if}
      <button aria-pressed={viewMode === "canvas"} class:active={viewMode === "canvas"} onclick={() => chosenView = "canvas"}>Canvas</button>
      <button aria-pressed={viewMode === "list"} class:active={viewMode === "list"} onclick={() => chosenView = "list"}>List</button>
    </div>
    {#if viewMode === "canvas"}<div class="camera-tools" role="group" aria-label="Canvas zoom"><button aria-label="Zoom out" onclick={() => zoomBy(-.1)}>−</button><button aria-label="Reset zoom to 100 percent" onclick={resetZoom}>{scaleLabel}%</button><button aria-label="Zoom in" onclick={() => zoomBy(.1)}>+</button><button onclick={fit}>Fit</button></div>{/if}
  </header>

  <div class="map-body">
    {#if viewMode === "fleet"}
      <FleetBoard {run} {review} {agents} {taskDescriptions} {taskClis} {backend} onInspect={inspectAgent} />
    {:else if viewMode === "canvas"}
      <!-- Direct scene transforms keep pointer frames outside Svelte's render path. -->
      <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
      <div class="canvas-viewport" class:panning={!!gesture?.started} bind:this={viewport} tabindex="0" role="application" aria-label="Worker canvas. Drag or use arrow keys to pan. Control plus wheel, plus, or minus zooms. F fits all tasks." onpointerdown={beginPan} onpointermove={pan} onpointerup={endPan} onpointercancel={endPan} onkeydown={handleCanvasKey} onclickcapture={event => { if (suppressNodeClick) { event.preventDefault(); event.stopPropagation(); } }}>
        <div class="canvas-grid" aria-hidden="true" bind:this={grid}></div>
        {#if nodes.length}
          <div class="scene" class:overview={overviewMode} class:micro-overview={microOverviewMode} bind:this={scene} style={`width:${topology.width}px;height:${topology.height}px`}>
            <svg class="flowlines" viewBox={`0 0 ${topology.width} ${topology.height}`} aria-hidden="true">
              {#each edges as edge}{@const handedOff = edge.source.agent?.status === "completed" && edge.source.agent.exit_code === 0}<path class:highlighted={edge.source.task.task_id === selected?.task.task_id || edge.target.task.task_id === selected?.task.task_id} class:flowing={handedOff && ["running", "starting", "pending"].includes(edge.target.agent?.status ?? "")} class:upstream={!routed && !handedOff} d={topology.orientation === "vertical" ? `M ${edge.source.sceneX} ${edge.source.sceneY + 50} C ${edge.source.sceneX} ${(edge.source.sceneY + edge.target.sceneY) / 2}, ${edge.target.sceneX} ${(edge.source.sceneY + edge.target.sceneY) / 2}, ${edge.target.sceneX} ${edge.target.sceneY - 50}` : `M ${edge.source.sceneX + 112} ${edge.source.sceneY} C ${(edge.source.sceneX + edge.target.sceneX) / 2} ${edge.source.sceneY}, ${(edge.source.sceneX + edge.target.sceneX) / 2} ${edge.target.sceneY}, ${edge.target.sceneX - 112} ${edge.target.sceneY}`} />{/each}
            </svg>
            {#each nodes as node (node.task.task_id)}
              {@const state = node.agent ? agentState(node.agent) : null}
              {@const attemptLabel = node.routingAttempt ? `Attempt ${node.routingAttempt.ordinal} · ${label(node.routingAttempt.role)} · ${label(node.routingAttempt.state)}` : null}
              {@const mainLabel = routingIdentityUnknown
                ? "Routing identity unavailable"
                : attemptLabel ?? (routed
                  ? routingLoading ? "Routing record loading" : routingSummary ? "No worker attempt recorded" : "Routing record unavailable"
                  : node.agent?.status === "completed" && node.agent.exit_code === 0 ? "Completed · checks separate" : state?.label ?? "Worker not recorded")}
              {@const running = routed ? node.routingAttempt?.state === "running" : node.agent?.status === "running"}
              <article class="task-node" class:running={running} class:chosen={node.task.task_id === selected?.task.task_id} class:related={!!selected && related(node.task.task_id)} data-state={node.routingAttempt?.state === "passed" ? "completed" : node.routingAttempt?.state ?? node.agent?.status ?? "unreported"} style={`left:${node.sceneX}px;top:${node.sceneY}px`}>
                <div class="node-kicker"><span>{routed && node.routingAttempt ? `Attempt ${node.routingAttempt.ordinal} of ${node.attemptCount} · ${label(node.routingAttempt.role)}` : running ? "Active worker" : node.agent?.launcher?.display_name ?? `Step ${node.task.wave + 1}`}</span>{#if running}<i class="worker-pulse" aria-hidden="true">. : + * = x</i>{/if}</div>
                <button aria-pressed={node.task.task_id === selected?.task.task_id} aria-label={`${node.task.task_id} ${mainLabel}${routed && node.routingAttempt && !node.agent ? ", worker not recorded" : ""}`} title={state?.detail ?? "No exact worker record for this task attempt"} onclick={() => choose(node.task.task_id)}>
                  <span class="node-overview" aria-hidden="true">
                    <span class="overview-meta"><i></i>S{node.task.wave + 1}</span>
                    <strong>{node.task.task_id}</strong>
                    <span>{routed ? mainLabel : node.agent?.launcher?.display_name ?? state?.label ?? "Unassigned"}</span>
                  </span>
                  <span class="node-copy"><strong title={taskDescriptions[node.task.task_id]}>{taskDescriptions[node.task.task_id] ?? node.task.task_id}</strong><span>{mainLabel}</span>{#if routed && node.routingAttempt && !node.agent}<span>Worker not recorded</span>{/if}</span>
                </button>
                <small>{node.task.depends_on.length ? `Requires ${node.task.depends_on.join(", ")}` : "No task prerequisites"}</small>
              </article>
            {/each}
          </div>
          {#if selected}<section class="relationship-strip" aria-label="Selected task relationships"><strong>{selected.task.task_id}</strong><span>Needs {selected.task.depends_on.length ? selected.task.depends_on.join(", ") : "none"}</span><span>Unblocks {dependents.length ? dependents.map(node => node.task.task_id).join(", ") : "none"}</span></section>{/if}
          {#if showMinimap}<aside class="minimap" aria-label="Canvas minimap"><button aria-label="Navigate canvas minimap. Arrow keys pan; Enter centers selected worker; F fits." title="Click to navigate · Arrow keys to pan · Enter to center selected worker · F to fit" onclick={navigateMinimap} onkeydown={handleMinimapKey}><svg viewBox={`0 0 ${topology.width} ${topology.height}`} aria-hidden="true">{#each edges as edge}<line x1={edge.source.sceneX} y1={edge.source.sceneY} x2={edge.target.sceneX} y2={edge.target.sceneY} />{/each}{#each nodes as node}<rect class:selected={node.task.task_id === selected?.task.task_id} x={node.sceneX - 38} y={node.sceneY - 14} width="76" height="28" rx="3" />{/each}<rect class="minimap-viewport" bind:this={minimapViewport} /></svg></button></aside>{/if}
        {:else}<div class="canvas-empty"><strong>Task relationships unavailable</strong><span>Recorded workers remain available in List.</span></div>{/if}
      </div>
    {:else}
      <div class="list-view"><RunLedger {agents} {taskDescriptions} plan={review?.plan ?? null} agentReceipts={review?.enforcement?.agents ?? null} {selectedAgentId} inlineInspector={false} onSelect={(agentId) => { const agent = agents.find(candidate => candidate.id === agentId); if (agent) inspectAgent(agent); }} /></div>
    {/if}
  </div>

</section>

<style>
  .execution-map{display:flex;flex:1;min-width:0;min-height:0;flex-direction:column;overflow:hidden;border:1px solid var(--pytxo-line);border-radius:7px;background:var(--pytxo-work-canvas);font-family:var(--pytxo-font-ui)}
  .map-toolbar{display:flex;min-height:44px;flex:0 0 auto;align-items:center;gap:14px;padding:6px 10px;border-bottom:1px solid var(--pytxo-line);background:color-mix(in srgb,var(--pytxo-surface-panel) 88%,transparent)}.map-toolbar>div:first-child{display:flex;min-width:0;align-items:baseline;gap:9px;margin-right:auto}.map-toolbar strong{font-size:13px}.map-toolbar span{color:var(--pytxo-text-muted);font:11px var(--pytxo-font-ui)}
  .view-toggle,.camera-tools{display:flex;align-items:center;border:1px solid var(--pytxo-line);border-radius:4px;overflow:hidden}.view-toggle button,.camera-tools button{min-width:34px;height:30px;padding:0 9px;border:0;border-left:1px solid var(--pytxo-line);background:transparent;color:var(--pytxo-text-muted);font:11px var(--pytxo-font-ui);cursor:pointer}.view-toggle button:first-child,.camera-tools button:first-child{border-left:0}.view-toggle button.active,.camera-tools button:hover{background:var(--pytxo-surface-active);color:var(--pytxo-text-strong)}button:focus-visible,.canvas-viewport:focus-visible{outline:2px solid var(--pytxo-accent);outline-offset:-2px}
  .map-body{position:relative;flex:1;min-height:0;overflow:hidden}.canvas-viewport{position:absolute;inset:0;overflow:hidden;touch-action:none;cursor:grab;outline:0}.canvas-viewport.panning{cursor:grabbing}.canvas-grid{position:absolute;inset:0;background-image:radial-gradient(circle,color-mix(in srgb,var(--pytxo-text-muted) 62%,transparent) 1.1px,transparent 1.5px);background-size:24px 24px;pointer-events:none}.canvas-viewport.panning .task-node button{cursor:grabbing}
  .scene{position:absolute;left:0;top:0;transform-origin:0 0;will-change:transform;contain:layout paint style}.flowlines{position:absolute;inset:0;width:100%;height:100%;overflow:visible}.flowlines path{fill:none;stroke:color-mix(in srgb,var(--pytxo-text-muted) 54%,transparent);stroke-width:1.3;vector-effect:non-scaling-stroke}.flowlines path.highlighted{stroke:var(--pytxo-activity);stroke-width:2.2}
  .task-node{position:absolute;width:224px;min-height:100px;transform:translate(-50%,-50%);border:1px solid var(--pytxo-line);border-radius:5px;background:color-mix(in srgb,var(--pytxo-work-node) 96%,transparent);box-shadow:0 12px 32px color-mix(in srgb,var(--pytxo-surface-shell) 62%,transparent)}.task-node.related:not(.chosen){border-color:color-mix(in srgb,var(--pytxo-activity) 36%,var(--pytxo-line))}.task-node.chosen{border-color:var(--pytxo-text-soft);box-shadow:0 0 0 2px color-mix(in srgb,var(--pytxo-text-strong) 12%,transparent),0 16px 36px color-mix(in srgb,var(--pytxo-surface-shell) 72%,transparent)}
  .node-kicker{display:flex;min-height:27px;align-items:center;justify-content:space-between;gap:8px;padding:4px 9px;border-bottom:1px solid var(--pytxo-line-soft)}.node-kicker span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--pytxo-text-muted);font:10px "IBM Plex Mono",monospace}.worker-pulse{width:54px;overflow:hidden;color:var(--pytxo-activity);font:10px "IBM Plex Mono",monospace;white-space:nowrap;animation:worker-pulse .8s steps(6,end) infinite}
  .task-node button{display:grid;width:100%;gap:5px;padding:8px 9px;border:0;background:transparent;color:var(--pytxo-text-strong);text-align:left;cursor:pointer}.node-copy{display:grid;min-width:0;gap:5px}.node-copy>strong{display:-webkit-box;overflow:hidden;font-size:13px;line-height:1.35;-webkit-box-orient:vertical;-webkit-line-clamp:2;line-clamp:2}.node-copy>span{color:var(--pytxo-text-soft);font-size:11px}.node-overview{display:none}.task-node small{display:block;padding:0 9px 8px;color:var(--pytxo-text-muted);font:10px/1.35 "IBM Plex Mono",monospace;overflow-wrap:anywhere}
  .scene.overview .task-node{height:100px;min-height:100px;box-shadow:0 8px 22px color-mix(in srgb,var(--pytxo-surface-shell) 54%,transparent)}.scene.overview .node-kicker,.scene.overview .node-copy,.scene.overview .task-node>small{display:none}.scene.overview .task-node button{height:100%;padding:13px 16px}.scene.overview .node-overview{display:grid;min-width:0;gap:7px}.scene.overview .node-overview>strong{overflow:hidden;font-size:clamp(25px,calc(9px * var(--camera-inverse,1)),45px);line-height:1.05;text-overflow:ellipsis;white-space:nowrap}.scene.overview .node-overview>span:last-child{overflow:hidden;color:var(--pytxo-text-soft);font-size:clamp(17px,calc(7px * var(--camera-inverse,1)),34px);line-height:1.1;text-overflow:ellipsis;white-space:nowrap}.scene.overview .overview-meta{display:flex;align-items:center;gap:8px;color:var(--pytxo-text-muted);font:clamp(15px,calc(6px * var(--camera-inverse,1)),30px) "IBM Plex Mono",monospace;letter-spacing:.05em}.scene.overview .overview-meta i{width:clamp(11px,calc(5px * var(--camera-inverse,1)),22px);height:clamp(11px,calc(5px * var(--camera-inverse,1)),22px);flex:0 0 auto;border-radius:50%;background:var(--pytxo-text-muted)}.scene.overview .task-node[data-state="running"] .overview-meta i{background:var(--pytxo-activity)}.scene.overview .task-node[data-state="completed"] .overview-meta i{background:var(--pytxo-success)}.scene.overview .task-node[data-state="failed"] .overview-meta i{background:var(--pytxo-danger,#d98994)}
  .scene.micro-overview .task-node button{display:flex;align-items:center;padding:0 14px}.scene.micro-overview .node-overview{display:block;width:100%}.scene.micro-overview .node-overview>strong{display:block}.scene.micro-overview .node-overview>span:last-child{display:none}.scene.micro-overview .overview-meta{position:absolute;top:8px;left:8px;font-size:0}.scene.micro-overview .overview-meta i{display:block}
  .relationship-strip{position:absolute;left:10px;bottom:10px;display:flex;max-width:calc(100% - 168px);align-items:center;gap:10px;padding:7px 9px;border:1px solid var(--pytxo-line);border-radius:4px;background:color-mix(in srgb,var(--pytxo-surface-panel) 92%,transparent);font-size:10px;box-shadow:0 8px 22px #0004}.relationship-strip strong{color:var(--pytxo-text-strong)}.relationship-strip span{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;color:var(--pytxo-text-muted)}
  .minimap{position:absolute;right:10px;bottom:10px;width:140px;height:84px;padding:5px;border:1px solid var(--pytxo-line);border-radius:4px;background:color-mix(in srgb,var(--pytxo-surface-panel) 92%,transparent);box-shadow:0 8px 22px #0004}.minimap svg{width:100%;height:100%}.minimap line{stroke:var(--pytxo-line);stroke-width:3}.minimap rect{fill:var(--pytxo-text-muted);opacity:.48}.minimap rect.selected{fill:var(--pytxo-activity);opacity:1}
  .canvas-empty{position:absolute;inset:0;display:grid;place-content:center;gap:6px;color:var(--pytxo-text-muted);text-align:center}.canvas-empty strong{color:var(--pytxo-text-soft);font-size:13px}.canvas-empty span{font-size:11px}.list-view{position:absolute;inset:0;overflow:auto;padding:10px}.list-view :global(.ledger){min-height:100%;border-radius:4px}
  @keyframes worker-pulse{from{width:8px}to{width:54px}}
  /* Live instrument: a handed-off result flows along its edge into the running task; edges still waiting on their input stay dashed. An active worker carries an activity rail (activity, not progress). */
  .flowlines path.upstream{stroke-dasharray:3 6;opacity:.7}.flowlines path.flowing{stroke:var(--pytxo-activity);stroke-width:2;stroke-dasharray:7 7;animation:handoff 1.1s linear infinite}
  @keyframes handoff{to{stroke-dashoffset:-14}}
  .task-node.running::after{content:"";position:absolute;top:-1px;left:-1px;right:-1px;height:2px;border-radius:4px 4px 0 0;background:var(--pytxo-aperture-horizontal);background-size:200% 100%;animation:activity-rail 2.4s linear infinite;pointer-events:none}
  @keyframes activity-rail{to{background-position:-200% 0}}
  .minimap button{display:block;width:100%;height:100%;padding:0;border:0;background:transparent;cursor:crosshair}.minimap .minimap-viewport{fill:transparent;stroke:var(--pytxo-text-strong);stroke-width:1;vector-effect:non-scaling-stroke;opacity:.8;pointer-events:none}
  @media(max-width:1100px){.relationship-strip{max-width:calc(100% - 138px)}.minimap{width:110px;height:66px}}
  @media(max-width:650px){.map-toolbar{flex-wrap:wrap}.map-toolbar>div:first-child{flex-basis:100%}.camera-tools{margin-left:auto}.relationship-strip{display:none}.map-body{min-height:150px}}
  @media(prefers-reduced-motion:reduce){.worker-pulse{width:auto;animation:none;font-size:0}.worker-pulse::after{content:"●";font-size:10px}.flowlines path.flowing,.task-node.running::after{animation:none}}
  :global([data-force-reduced-motion]) .flowlines path.flowing,:global([data-force-reduced-motion]) .task-node.running::after{animation:none}
</style>
