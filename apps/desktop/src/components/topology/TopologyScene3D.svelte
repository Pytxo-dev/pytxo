<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import * as THREE from "three";
  import { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
  import type { StructuralGraphDto } from "../../lib/types";
  import type { DeckTheme } from "../../lib/theme";
  import { isLightDeck } from "../../lib/theme";
  import { forceLayout3d } from "../../lib/force-layout";

  let {
    structural = null as StructuralGraphDto | null,
    loading = false,
    selectedNodeId = $bindable(null as string | null),
    deckTheme = "void" as DeckTheme,
  }: {
    structural?: StructuralGraphDto | null;
    loading?: boolean;
    selectedNodeId?: string | null;
    deckTheme?: DeckTheme;
  } = $props();

  let containerEl: HTMLDivElement | undefined = $state();
  let hoverLabel = $state<string | null>(null);

  let renderer: THREE.WebGLRenderer | null = null;
  let scene: THREE.Scene | null = null;
  let camera: THREE.PerspectiveCamera | null = null;
  let controls: OrbitControls | null = null;
  let frameId = 0;
  let nodeMeshes = new Map<string, THREE.Mesh>();
  let resizeObserver: ResizeObserver | null = null;

  const lightTheme = $derived(isLightDeck(deckTheme));

  function token(name: string, fallback: string): string {
    if (typeof getComputedStyle === "undefined") return fallback;
    const v = getComputedStyle(document.documentElement).getPropertyValue(name).trim();
    return v || fallback;
  }

  function hexColor(cssVar: string, fallback: number): number {
    const raw = token(cssVar, "");
    if (!raw) return fallback;
    try {
      return new THREE.Color(raw).getHex();
    } catch {
      return fallback;
    }
  }

  function clearSceneNodes() {
    if (!scene) return;
    for (const mesh of nodeMeshes.values()) {
      scene.remove(mesh);
      mesh.geometry.dispose();
      (mesh.material as THREE.Material).dispose();
    }
    nodeMeshes.clear();
    const toRemove = scene.children.filter((c: THREE.Object3D) => c.userData.kind === "edge");
    for (const obj of toRemove) {
      scene.remove(obj);
      if (obj instanceof THREE.Line) {
        obj.geometry.dispose();
        (obj.material as THREE.Material).dispose();
      }
    }
  }

  function rebuildGraph() {
    if (!scene) return;
    clearSceneNodes();

    const graph = structural;
    if (!graph || graph.nodes.length === 0) return;

    const layout = forceLayout3d(graph.nodes, graph.edges);
    const teal = hexColor("--brand-teal", 0x2dd4bf);
    const violet = hexColor("--brand-violet", 0xa78bfa);
    const gold = hexColor("--brand-gold", 0xfbbf24);

    for (const node of graph.nodes) {
      const pos = layout.get(node.id);
      if (!pos) continue;
      const isSymbol = node.label.includes("::") || node.id.includes("::");
      const size = isSymbol ? 0.14 : node.edited ? 0.35 : 0.22;
      const color = isSymbol ? violet : node.edited ? gold : teal;
      const geom = new THREE.SphereGeometry(size, 16, 16);
      const mat = new THREE.MeshStandardMaterial({
        color,
        emissive: isSymbol ? violet : node.edited ? gold : violet,
        emissiveIntensity: isSymbol ? 0.28 : node.edited ? 0.35 : 0.12,
      });
      const mesh = new THREE.Mesh(geom, mat);
      mesh.position.set(pos.x, pos.y, pos.z);
      mesh.userData = { nodeId: node.id, label: node.label };
      scene.add(mesh);
      nodeMeshes.set(node.id, mesh);
    }

    const edgeMat = new THREE.LineBasicMaterial({
      color: hexColor("--brand-violet", 0xa78bfa),
      transparent: true,
      opacity: 0.45,
    });
    for (const edge of graph.edges) {
      const a = layout.get(edge.from);
      const b = layout.get(edge.to);
      if (!a || !b) continue;
      const geom = new THREE.BufferGeometry().setFromPoints([
        new THREE.Vector3(a.x, a.y, a.z),
        new THREE.Vector3(b.x, b.y, b.z),
      ]);
      const line = new THREE.Line(geom, edgeMat);
      line.userData = { kind: "edge" };
      scene.add(line);
    }

    highlightSelection();
  }

  function highlightSelection() {
    for (const [id, mesh] of nodeMeshes) {
      const mat = mesh.material as THREE.MeshStandardMaterial;
      const isSel = id === selectedNodeId;
      mat.emissiveIntensity = isSel ? 0.65 : 0.12;
      mesh.scale.setScalar(isSel ? 1.25 : 1);
    }
  }

  function applySceneTheme() {
    if (!scene) return;
    const bg = lightTheme ? token("--void", "#f4f6f8") : token("--void", "#020205");
    scene.background = new THREE.Color(bg);
  }

  function resize() {
    if (!containerEl || !renderer || !camera) return;
    const w = containerEl.clientWidth;
    const h = containerEl.clientHeight;
    if (w < 1 || h < 1) return;
    camera.aspect = w / h;
    camera.updateProjectionMatrix();
    renderer.setSize(w, h, false);
  }

  function pickNode(ev: PointerEvent): string | null {
    if (!camera || !containerEl) return null;
    const rect = containerEl.getBoundingClientRect();
    const x = ((ev.clientX - rect.left) / rect.width) * 2 - 1;
    const y = -((ev.clientY - rect.top) / rect.height) * 2 + 1;
    const raycaster = new THREE.Raycaster();
    raycaster.setFromCamera(new THREE.Vector2(x, y), camera);
    const hits = raycaster.intersectObjects([...nodeMeshes.values()]);
    if (hits.length > 0) {
      return hits[0].object.userData.nodeId as string;
    }
    return null;
  }

  function onPointerDown(ev: PointerEvent) {
    const id = pickNode(ev);
    if (id) {
      selectedNodeId = selectedNodeId === id ? null : id;
      highlightSelection();
    }
  }

  function onPointerMove(ev: PointerEvent) {
    const id = pickNode(ev);
    if (id) {
      const mesh = nodeMeshes.get(id);
      hoverLabel = (mesh?.userData.label as string) ?? id;
    } else {
      hoverLabel = null;
    }
  }

  function animate() {
    frameId = requestAnimationFrame(animate);
    controls?.update();
    renderer?.render(scene!, camera!);
  }

  onMount(() => {
    if (!containerEl) return;

    scene = new THREE.Scene();
    applySceneTheme();

    camera = new THREE.PerspectiveCamera(50, 1, 0.1, 100);
    camera.position.set(0, 0, 10);

    renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true });
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    containerEl.appendChild(renderer.domElement);

    const ambient = new THREE.AmbientLight(0xffffff, 0.65);
    const dir = new THREE.DirectionalLight(0xffffff, 0.85);
    dir.position.set(4, 6, 8);
    scene.add(ambient, dir);

    controls = new OrbitControls(camera, renderer.domElement);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.minDistance = 3;
    controls.maxDistance = 24;

    resizeObserver = new ResizeObserver(resize);
    resizeObserver.observe(containerEl);
    containerEl.addEventListener("pointerdown", onPointerDown);
    containerEl.addEventListener("pointermove", onPointerMove);
    resize();
    rebuildGraph();
    animate();
  });

  onDestroy(() => {
    cancelAnimationFrame(frameId);
    resizeObserver?.disconnect();
    containerEl?.removeEventListener("pointerdown", onPointerDown);
    containerEl?.removeEventListener("pointermove", onPointerMove);
    clearSceneNodes();
    controls?.dispose();
    renderer?.dispose();
  });

  $effect(() => {
    structural;
    rebuildGraph();
  });

  $effect(() => {
    selectedNodeId;
    highlightSelection();
  });

  $effect(() => {
    deckTheme;
    applySceneTheme();
  });
</script>

<section class="topology-3d glass-panel">
  <header class="topology-3d__header">
    <h2 class="panel-title">Structural topology</h2>
    <div class="legend">
      <span class="legend__item"><i class="dot dot--file"></i> File</span>
      <span class="legend__item"><i class="dot dot--edited"></i> Edited</span>
      <span class="legend__item"><i class="dot dot--symbol"></i> Symbol</span>
    </div>
    {#if hoverLabel || selectedNodeId}
      <span class="selection">{hoverLabel ?? selectedNodeId}</span>
    {/if}
  </header>
  <div class="topology-3d__canvas" bind:this={containerEl}>
    {#if loading}
      <div class="loading">Building graph…</div>
    {:else if !structural || structural.nodes.length === 0}
      <div class="empty">Select a workspace folder to preview import topology.</div>
    {/if}
  </div>
</section>

<style>
  .topology-3d {
    display: flex;
    flex-direction: column;
    flex: 1;
    min-width: 0;
    min-height: 55vh;
    padding: 0.85rem;
    margin: 0 0.5rem;
    border-radius: 16px;
    box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
  }
  .topology-3d__header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    flex-wrap: wrap;
    margin-bottom: 0.5rem;
  }
  .legend {
    display: flex;
    gap: 0.65rem;
    font-size: 0.7rem;
    color: var(--muted-foreground);
  }
  .legend__item {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    display: inline-block;
  }
  .dot--file {
    background: var(--brand-teal);
  }
  .dot--edited {
    background: var(--brand-gold);
  }
  .dot--symbol {
    background: var(--brand-violet);
  }
  .selection {
    font-size: 0.72rem;
    color: var(--brand-violet);
    font-family: ui-monospace, monospace;
    margin-left: auto;
  }
  .topology-3d__canvas {
    flex: 1;
    min-height: 280px;
    border-radius: 12px;
    overflow: hidden;
    background: var(--void-elevated);
    position: relative;
    box-shadow: inset 0 0 0 1px color-mix(in oklab, var(--foreground) 6%, transparent);
  }
  .topology-3d__canvas :global(canvas) {
    display: block;
    width: 100% !important;
    height: 100% !important;
  }
  .empty,
  .loading {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.85rem;
    color: var(--muted-foreground);
    pointer-events: none;
    text-wrap: balance;
    padding: 1rem;
    text-align: center;
  }
</style>
